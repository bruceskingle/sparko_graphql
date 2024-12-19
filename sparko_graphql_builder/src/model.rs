use std::collections::HashMap;
use std::error::Error;
use std::io::Write;

use inflections::case::to_snake_case;

use crate::error::GraphQLError;
use crate::Output;

fn to_pascal_case(s: &str) -> String {
    let mut string = String::with_capacity(s.len());

    enum Phase {
        Leadin,
        Before,
        FirstUpper,
        Upper(char),
        Lower
    }

    let mut phase = Phase::Leadin;
    let mut underscores = 0;
    let mut push = |char| string.push(char);

    for char in s.chars() {
        

        match phase {
            Phase::Leadin => {
                if char == '_' {
                    underscores += 1;
                }
                else {
                    char.to_uppercase().for_each(&mut push);
                    phase = Phase::FirstUpper;
                }
            },
            Phase::Before => {
                if char.is_alphanumeric() {
                    char.to_uppercase().for_each(&mut push);
                    phase = Phase::FirstUpper;
                }
                // else skip
            },
            Phase::FirstUpper => {
                if char.is_uppercase() {
                    phase = Phase::Upper(char);
                }
                else {
                    if char.is_alphanumeric() {
                        char.to_lowercase().for_each(&mut push);
                    }
                    phase = Phase::Lower;
                }
            },
            Phase::Upper(pending) => {

                if char.is_uppercase() {
                    pending.to_lowercase().for_each(&mut push);
                    phase = Phase::Upper(char);
                }
                else {
                    if char.is_alphanumeric() {
                        pending.to_uppercase().for_each(&mut push);
                        char.to_lowercase().for_each(&mut push);
                        phase = Phase::Lower;
                    }
                    else {
                        pending.to_lowercase().for_each(&mut push);
                        phase = Phase::Before;
                    }
                }
            },
            Phase::Lower => {
                if char.is_lowercase() {
                    push(char);
                }
                else {
                    if char.is_alphanumeric() {
                        char.to_uppercase().for_each(&mut push);
                        phase = Phase::FirstUpper;
                    }
                    else {
                        phase = Phase::Before;
                    }
                }
            },
        }
    }

    match phase {
        Phase::Leadin => string = format!("underscore{}", underscores),
        Phase::Upper(pending) => pending.to_lowercase().for_each(&mut push),
        _ => {},
    }

    string
}

#[derive(Debug)]
struct FieldModel {
    position: graphql_parser::Pos,
    name: String,
    ty: String,
}

#[derive(Debug)]
struct EnumModel {
    position: graphql_parser::Pos,
    name: String,
    variants: Vec<String>,
}

impl EnumModel {
    pub fn new(out: &mut Output, enum_type: &graphql_parser::schema::EnumType<'_, String>) -> Result<EnumModel, Box<dyn Error>> {
        let mut model = EnumModel {
            position: enum_type.position,
            name: enum_type.name.clone(),
            variants: Vec::new(),
        };

        for v in &enum_type.values {
            writeln!(out, "//  variant {}", v.name)?;
    
            model.variants.push(v.name.clone());
            
        }
        Ok(model)
    }
}

#[derive(Debug)]
struct UnionModel {
    position: graphql_parser::Pos,
    name: String,
    implements: Vec<String>,
}

impl UnionModel {
    fn new(union_type: graphql_parser::schema::UnionType<'_, String>) -> UnionModel {
        UnionModel {
            position: union_type.position,
            name: union_type.name,
            implements: union_type.types,
        }
    }
}


#[derive(Debug)]
pub struct ObjectModel {
    position: graphql_parser::Pos,
    name: String,
    field_names: Vec<String>,
    fields: HashMap<String, FieldModel>,
    implements: Vec<String>,
    fully_implements: Vec<String>,
}

impl ObjectModel {
    pub fn new(out: &mut Output, object_type: graphql_parser::schema::ObjectType<'_, String>) -> Result<ObjectModel, Box<dyn Error>> {
        let mut field_names = Vec::new();
        let mut fields = HashMap::new();

        for f in object_type.fields {
            writeln!(out, "//  field {} type {}", f.name,  visit_type(&f.field_type, false)?)?;
    
            field_names.push(f.name.clone());
            fields.insert(f.name.clone(), FieldModel {
                position: f.position,
                name: f.name,
                ty: visit_type(&f.field_type, false)?,
            });
            
        }
        Ok(ObjectModel {
            position: object_type.position,
            name: object_type.name,
            field_names,
            fields,
            implements: object_type.implements_interfaces,
            fully_implements: Vec::new(),
        })
    }
}




#[derive(Debug)]
pub struct InterfaceModel {
    position: graphql_parser::Pos,
    name: String,
    field_names: Vec<String>,
    fields: HashMap<String, FieldModel>,
    implements: Vec<String>,
    implemented_by: Vec<String>,
}

impl InterfaceModel {
    pub fn new(out: &mut Output, interface_type: graphql_parser::schema::InterfaceType<'_, String>) -> Result<InterfaceModel, Box<dyn Error>> {
        let mut field_names = Vec::new();
        let mut fields = HashMap::new();

        for f in interface_type.fields {
            writeln!(out, "//  field {} type {}", f.name,  visit_type(&f.field_type, false)?)?;
    
            field_names.push(f.name.clone());
            fields.insert(f.name.clone(), FieldModel {
                position: f.position,
                name: f.name,
                ty: visit_type(&f.field_type, false)?,
            });
            
        }
        Ok(InterfaceModel {
            position: interface_type.position,
            name: interface_type.name,
            field_names,
            fields,
            implements: interface_type.implements_interfaces,
            implemented_by: Vec::new(),
        })
    }
}

pub struct GraphQlModel {
    // named_types: HashMap<String, Type>,
    names: HashMap<String, graphql_parser::Pos>,
    objects: HashMap<String, ObjectModel>,
    interfaces: HashMap<String, InterfaceModel>,
    scalars: Vec<FieldModel>,
    enums: Vec<EnumModel>,
    unions: HashMap<String, UnionModel>,
    query: Option<String>,
    mutation: Option<String>,
    subscription: Option<String>,
    schema_position: Option<graphql_parser::Pos>,
}

impl GraphQlModel {
    fn name_defined(&mut self, out: &mut Output, name: &String, position: graphql_parser::Pos) {
        if let Some(existing) = self.names.insert(name.clone(), position.clone()) {
            out.error(GraphQLError::DuplicateName(existing, position, name.clone()));
        }
    }

    pub fn new(out: &mut Output,
        definitions: Vec<graphql_parser::schema::Definition<'_, String>>) -> Result<GraphQlModel, Box<dyn Error>> {
        let mut model = GraphQlModel {
            names: HashMap::new(),
            objects: HashMap::new(),
            interfaces: HashMap::new(),
            scalars: Vec::new(),
            enums: Vec::new(),
            unions: HashMap::new(),
            query: None,
            mutation: None,
            subscription: None,
            schema_position: None,
        };

        for def in definitions {
            match def {
                graphql_parser::schema::Definition::SchemaDefinition(schema_definition) => {
                    writeln!(out, "//Schema Definition")?;

                    model.query = schema_definition.query;
                    model.mutation = schema_definition.mutation;
                    model.subscription = schema_definition.subscription;
                    model.schema_position = Some(schema_definition.position);

                    if let Some(query) = & model.query {
                        writeln!(out, "//query {}", query)?;
                    }
                    for directive in schema_definition.directives {
                        writeln!(out, "//Directive {}", directive.name)?;
                
                        for arg in directive.arguments {
                            writeln!(out, "//  Arg {} {}", arg.0, arg.1)?;
                        }
                    }
                },
                graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                    writeln!(out, "//Type Definition")?;
                    match type_definition {
                        graphql_parser::schema::TypeDefinition::Scalar(scalar_type) => {
                            writeln!(out, "//scalar_type {}", scalar_type)?;
    
                            model.name_defined(out, &scalar_type.name, scalar_type.position.clone());
                            model.scalars.push(FieldModel {
                                position: scalar_type.position,
                                name: scalar_type.name,
                                ty: String::from("serde_json::Value"),
                            });
                        },
                        graphql_parser::schema::TypeDefinition::Object(ast) => {
                            writeln!(out, "/* object_type {} */", ast.name)?;

                            if ast.name == "APICallType" {
                                println!("Parse APICallType");
                            }
    
                            model.name_defined(out, &ast.name, ast.position.clone());
                            model.objects.insert(ast.name.clone(), ObjectModel::new(out, ast)?);
                        },
                        graphql_parser::schema::TypeDefinition::Interface(ast) => {
                            writeln!(out, "/* interface_type {} */", ast)?;

                            model.name_defined(out, &ast.name, ast.position.clone());
                            model.interfaces.insert(ast.name.clone(), InterfaceModel::new(out, ast)?);
                        },
                        graphql_parser::schema::TypeDefinition::Union(ast) => {
                            writeln!(out, "/* union_type {} */", ast)?;

                            model.name_defined(out, &ast.name, ast.position.clone());
                            model.unions.insert(ast.name.clone(), UnionModel::new(ast));
                        },
                        graphql_parser::schema::TypeDefinition::Enum(ast) => {
                            writeln!(out, "/* enum_type {} */", &ast)?;

                            model.name_defined(out, &ast.name, ast.position.clone());
                            model.enums.push(EnumModel::new(out, &ast)?);
                        },
                        graphql_parser::schema::TypeDefinition::InputObject(input_object_type) => {
                            writeln!(out, "/* input_object_type {} */", input_object_type)?;
    
                            // Self::named_type(out, model, input_object_type.name.clone(), Type::InputObject(ObjectModel::from_input_object(out, input_object_type)?));
                        },
                    }
                },
                graphql_parser::schema::Definition::TypeExtension(type_extension) => {
                    writeln!(out, "//Type Extension {}", type_extension)?;
                },
                graphql_parser::schema::Definition::DirectiveDefinition(directive_definition) => {
                    writeln!(out, "//Directive Definition {}", directive_definition)?;
                },
            }
        }
        Ok(model)
    }

    pub fn validate(&mut self, out: &mut Output) -> Result<(), Box<dyn Error>> {

        // Validate objects
        for (name, object_model) in &mut self.objects {

            if name == "APICallType" {
                println!("Validate APICallType");
            }
            for interface_name in &object_model.implements {
                if let Some(interface) = self.interfaces.get(interface_name) {
                    let mut ok = true;
                    for field_name in &interface.field_names {
                        if ! object_model.fields.contains_key(field_name) {
                            ok = false;
                            break;
                        }
                    }
                    if ok {
                        object_model.fully_implements.push(interface_name.clone());
                    }
                }
                else {
                    out.error(GraphQLError::MissingInterfaceError(object_model.position, format!("Interface {} Not Found", name)))
                };

                
            }
        }

        // Validate interfaces
        for (interface_name, interface_model) in &mut self.interfaces {
            for (object_name, object_model) in &mut self.objects {
                for implements in &object_model.fully_implements {
                    if implements == interface_name {
                        interface_model.implemented_by.push(object_name.clone());
                        break;
                    }
                }
            }        
        }

        if let Some(query) = &self.query {
            if let None = self.objects.get(query) {
                out.error(GraphQLError::MissingObjectError(self.schema_position.unwrap(), format!("No query object \"{}\" defined", query)));
            }
        }
        else {
            if let Some(query) = self.objects.get("Query") {
                self.query = Some(query.name.clone());
            }
        }

        if let Some(mutation) = &self.mutation {
            if let None = self.objects.get(mutation) {
                out.error(GraphQLError::MissingObjectError(self.schema_position.unwrap(), format!("No mutation object \"{}\" defined", mutation)));
            }
        }
        else {
            if let Some(mutation) = self.objects.get("Mutation") {
                self.mutation = Some(mutation.name.clone());
            }
        }

        if let Some(subscription) = &self.subscription {
            if let None = self.objects.get(subscription) {
                out.error(GraphQLError::MissingObjectError(self.schema_position.unwrap(), format!("No subscription object \"{}\" defined", subscription)));
            }
        }
        else {
            if let Some(subscription) = self.objects.get("Subscription") {
                self.subscription = Some(subscription.name.clone());
            }
        }

        if self.query == None {
            out.error(GraphQLError::NoQueryDefinition);
        }

        Ok(())
    }

    pub fn generate(&mut self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        

        for scalar in &self.scalars {
            writeln!(out, "type {} = {}; // HERE", to_pascal_case(&scalar.name), &scalar.ty)?;
        }

        for enum_model in &self.enums {
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "#[serde(rename = \"{}\")]", &enum_model.name)?;
            writeln!(out, "enum {} {{", to_pascal_case(&enum_model.name))?;
            for v in &enum_model.variants {
                writeln!(out, "    #[serde(rename = \"{}\")]", v)?;
                writeln!(out, "    {},", to_pascal_case(v))?;
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;
        }

        for (_name, interface_model) in &self.interfaces {
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "enum {} {{", to_pascal_case(&interface_model.name))?;

            for object_name in &interface_model.implemented_by {
                
                writeln!(out, "    {}({}),", to_pascal_case(&object_name), to_pascal_case(&object_name))?;
            }

            writeln!(out, "}}")?;
            writeln!(out, "")?;
        }

        for (_name, object_model) in &self.objects {

            if _name == "APICallType" {
                println!("Generate APICallType");
            }
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "#[serde(rename = \"{}\")]", &object_model.name)?;
            writeln!(out, "struct {} {{", to_pascal_case(&object_model.name))?;
    
            for name in &object_model.field_names {
                if let Some(field) = object_model.fields.get(name) {
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
                    writeln!(out, "    {}_: {},", to_snake_case(&field.name), field.ty)?;
                }
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;
        }

        for (_name, union_model) in &self.unions {
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "enum {} {{", to_pascal_case(&union_model.name))?;
    
            for object_name in &union_model.implements {
                
                writeln!(out, "    {}({}),", to_pascal_case(&object_name), to_pascal_case(&object_name))?;
            }
    
            writeln!(out, "}}")?;
            writeln!(out, "")?;
        }
        Ok(())
    }
}


fn visit_type(field_type: &graphql_parser::query::Type<'_, String>, non_null: bool) -> Result<String, Box<dyn Error>> {
   
   Ok( match field_type {
        graphql_parser::query::Type::NamedType(v) => {
            let t = match v as &str {
                "Boolean" => String::from("bool"),
                "Date" => String::from("time::Date"),
                "DateTime" => String::from("time::OffsetDateTime"),
                "Float" => String::from("f64"),
                "ID" => String::from("String"),
                "Int" => String::from("i32"),
                "String" => String::from("String"),

                _ => to_pascal_case(v),
            };

            if non_null {
                t
            }
            else {
                format!("Option<{}>", t)
            }
        },
        graphql_parser::query::Type::ListType(t) => {
            format!("Vec<{}>", visit_type(&*t, non_null)?)
        },
        graphql_parser::query::Type::NonNullType(t) => {
            visit_type(&*t, true)?
        },
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pascal_case() {

        assert_eq!(to_pascal_case("HTTPServer"), "HttpServer");
        assert_eq!(to_pascal_case("HTTP"), "Http");
        assert_eq!(to_pascal_case("être"), "Être");
        assert_eq!(to_pascal_case("d'être"), "Dêtre");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("HelloWorld"), "HelloWorld");
        assert_eq!(to_pascal_case("_hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("___"), "underscore3");
    }

    // #[test]
    // #[should_panic]
    // fn test_any_panic() {
    //     divide_non_zero_result(1, 0);
    // }

    // #[test]
    // #[should_panic(expected = "Divide result is zero")]
    // fn test_specific_panic() {
    //     divide_non_zero_result(1, 10);
    // }
}

#[derive(Debug)]

pub enum Value {
    Variable,
    Int(Option<i64>),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Enum,
    List,
    Object,
}

impl Value {
    pub fn new(out: &mut Output, value: graphql_parser::query::Value<'_, String> ) -> Result<Value, GraphQLError> {
        match value {
            graphql_parser::query::Value::Variable(_) => todo!(),
            graphql_parser::query::Value::Int(number) => Ok(Value::Int(number.as_i64())),
            graphql_parser::query::Value::Float(_) => todo!(),
            graphql_parser::query::Value::String(string) => Ok(Value::String(string)),
            graphql_parser::query::Value::Boolean(_) => todo!(),
            graphql_parser::query::Value::Null => todo!(),
            graphql_parser::query::Value::Enum(_) => todo!(),
            graphql_parser::query::Value::List(vec) => todo!(),
            graphql_parser::query::Value::Object(btree_map) => todo!(),
        }
    }
}


#[derive(Debug)]
pub struct Argument {
    name: String,
    value: Value,
}

impl Argument {
    pub fn new(out: &mut Output, name: String, value: graphql_parser::query::Value<'_, String> ) -> Result<Argument, GraphQLError> {
        Ok(Argument {
            name,
            value: Value::new(out, value)?,
        })
    }
}

#[derive(Debug)]
pub struct VariableDefinition {
    position: graphql_parser::Pos,
    name: String,
    // pub var_type: Type<'a, T>,
    // pub default_value: Option<Value<'a, T>>,
}

impl VariableDefinition {
    pub fn new(out: &mut Output, variable: graphql_parser::query::VariableDefinition<'_, String> ) -> Result<VariableDefinition, GraphQLError> {
        Ok(VariableDefinition {
            position: variable.position,
            name: variable.name,
        })
    }
}


#[derive(Debug)]
pub struct SelectionField {
    position: graphql_parser::Pos,
    name: String,
    arguments: Vec<Argument>,
    selections: Vec<SelectionModel>,
}

impl SelectionField {
    pub fn new(out: &mut Output, field: graphql_parser::query::Field<'_, String> ) -> Result<SelectionField, GraphQLError> {
        Ok(SelectionField {
            position: field.position,
            name: field.name,
            arguments: build_arguments(out, field.arguments)?,
            selections: build_selections(out, field.selection_set)?,
        })
    }
}

#[derive(Debug)]
enum SelectionModel {
    Field(SelectionField),
    // FragmentSpread(FragmentSpread),
    // InlineFragment(InlineFragment),
} 

impl SelectionModel {
    pub fn new(out: &mut Output, selection: graphql_parser::query::Selection<'_, String> ) -> Result<SelectionModel, GraphQLError> {
        Ok(match selection {
            graphql_parser::query::Selection::Field(field) => SelectionModel::Field(SelectionField::new(out, field)?),
            graphql_parser::query::Selection::FragmentSpread(fragment_spread) => todo!(),
            graphql_parser::query::Selection::InlineFragment(inline_fragment) => todo!(),
        })
    }
}


#[derive(Debug)]
pub struct QueryModel {
    position: graphql_parser::Pos,
    name: String,
    selections: Vec<SelectionModel>,
    variables: Vec<VariableDefinition>,
    // fields: HashMap<String, FieldModel>,
    // implements: Vec<String>,
    // fully_implements: Vec<String>,
}

impl QueryModel {
    pub fn new(out: &mut Output, query: graphql_parser::query::Query<'_, String> ) -> Result<QueryModel, GraphQLError> {
        let name = match query.name {
            Some(name) => name,
            None => {
                return Err(GraphQLError::UnsupportedError(query.position, String::from("Anonymous query")))
            },
        };

        

        Ok(QueryModel {
            position: query.position,
            name,
            selections: build_selections(out, query.selection_set)?,
            variables: build_variables(out, query.variable_definitions)?,
        })
    }
}

fn build_variables(out: &mut Output, variable_definitions: Vec<graphql_parser::query::VariableDefinition<'_, String>>) -> Result<Vec<VariableDefinition>, GraphQLError> {
    let mut variables = Vec::new();

    for variable in variable_definitions {
        variables.push(VariableDefinition::new(out, variable)?);
    }

    Ok(variables)
}

fn build_selections(out: &mut Output, selection_set: graphql_parser::query::SelectionSet<'_, String>) -> Result<Vec<SelectionModel>, GraphQLError> {

    let mut selections = Vec::new();

    for selection in selection_set.items {
        selections.push(SelectionModel::new(out, selection)?);
    }

    Ok(selections)
}



fn build_arguments(out: &mut Output, arguments: Vec<(String, graphql_parser::query::Value<'_, String>)>) -> Result<Vec<Argument>, GraphQLError> {
    let mut result = Vec::new();

    for (name, value) in arguments {
        result.push(Argument::new(out, name, value)?);
    }

    Ok(result)
}

pub struct GraphQLQuerySet {

}

impl GraphQLQuerySet {
    pub fn new(out: &mut Output,
        definitions: Vec<graphql_parser::query::Definition<'_, String>>) -> Result<GraphQLQuerySet, Box<dyn Error>> {
        let mut model = GraphQLQuerySet {
        };

        for def in definitions {
            match def {
                graphql_parser::query::Definition::Operation(operation_definition) => {

                    writeln!(out, "//Operation Definition")?;
                    match operation_definition {
                        graphql_parser::query::OperationDefinition::SelectionSet(selection_set) => {
                            writeln!(out, "//  SelectionSet")?;
                            for item in selection_set.items {
                                match item {
                                    graphql_parser::query::Selection::Field(field) => {
                                        writeln!(out, "//    Field {}", &field.name)?;
                                        
                                    },
                                    graphql_parser::query::Selection::FragmentSpread(fragment_spread) => {
                                        writeln!(out, "//    FragmentSpread \n/*{}*/", &fragment_spread)?;
                                        
                                    },
                                    graphql_parser::query::Selection::InlineFragment(inline_fragment) => {
                                        writeln!(out, "//    InlineFragment \n/*{}*/", &inline_fragment)?;
                                        
                                    },
                                }
                            }
                        },
                        graphql_parser::query::OperationDefinition::Query(query) => {
                            writeln!(out, "//  Query \n/*{}*/", &query)?;

                            let query_model = QueryModel::new(out, query)?;

                            writeln!(out, "//    query_model \n/*{:?}*/", &query_model)?;

                        },
                        graphql_parser::query::OperationDefinition::Mutation(mutation) => {
                            writeln!(out, "//  Mutation \n/*{}*/", &mutation)?;
                            
                        },
                        graphql_parser::query::OperationDefinition::Subscription(subscription) => {
                            writeln!(out, "//  Subscription \n/*{}*/", &subscription)?;
                            
                        },
                    }
                },
                graphql_parser::query::Definition::Fragment(fragment_definition) => {
                    writeln!(out, "//Fragment Definition")?;
                },
            }
        }

        Ok(model)
    }
}