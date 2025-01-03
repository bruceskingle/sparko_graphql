use std::collections::HashSet;
use std::fmt::Display;
use std::rc::Rc;
use std::{collections::HashMap, error::Error};
use std::io::Write;

use inflections::case::to_snake_case;

use crate::utils::to_pascal_case;
use crate::{error::GraphQLError, parsed_model, Output};

#[derive(Debug)]
pub enum TypeDefinition {
//     Enum(Rc<Enum>),
//     Union(Rc<Union>),
    Object(Rc<Object>),
    Interface(Rc<Interface>),
//     Scalar(Rc<TypeDef>),
}

impl TypeDefinition {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            // DefinedType::Enum(content) => content.print(out),
            // DefinedType::Union(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            TypeDefinition::Interface(content) => content.print(out),
            // DefinedType::Scalar(content) => content.print(out),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            // TypeDefinition::Enum(_) => "enum",
            // TypeDefinition::Union(union) => "union",
            TypeDefinition::Object(object) => "object",
            TypeDefinition::Interface(interface) => "interface",
            // TypeDefinition::Scalar(type_def) => "scalar",
        }
    }

    // fn validate2(out: &mut Output, name: &String, defined_types: HashMap<String, TypeDefinition>, parsed_types: HashMap<String, parsed_model::TypeDefinition>, loop_detect: HashSet<String>) {
    //     if loop_detect.contains(name) {
    //         out.error(GraphQLError::);
    //     }
    // }

    // pub fn validate(out: &mut Output, name: &String, defined_types: HashMap<String, TypeDefinition>, parsed_types: HashMap<String, parsed_model::TypeDefinition>) {
    //     Self::validate2(out, name, defined_types, parsed_types, HashSet::new())
    // }

    // pub fn new(parsed: parsed_model::TypeDefinition) -> TypeDefinition {
    //     match parsed {
    //         parsed_model::TypeDefinition::Enum(_) => todo!(),
    //         parsed_model::TypeDefinition::Union(union) => todo!(),
    //         parsed_model::TypeDefinition::Object(object) => {

    //         },
    //         parsed_model::TypeDefinition::Interface(interface) => todo!(),
    //         parsed_model::TypeDefinition::Scalar(type_def) => todo!(),
    //     }
    // }

    pub fn generate(&self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        match self {
            TypeDefinition::Object(object_model) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "#[serde(rename = \"{}\")]", &object_model.parsed.name)?;
                writeln!(out, "pub struct {} {{", to_pascal_case(&object_model.parsed.name))?;
        
                for (name, field) in &object_model.fields {
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
                    writeln!(out, "    {}_: {},", to_snake_case(&field.name), field.rust_type())?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
            TypeDefinition::Interface(interface_model) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "pub enum {} {{", to_pascal_case(&interface_model.parsed.name))?;

                for object in &interface_model.implemented_by {
                    
                    writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
                }

                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
        }

        // for (name, scalar) in &self.scalars {
        //     writeln!(out, "type {} = {}; // HERE", to_pascal_case(&scalar.name), &scalar.rust_type)?;
        // }

        // for (name, enum_model) in &self.enums {
        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "#[serde(rename = \"{}\")]", &enum_model.name)?;
        //     writeln!(out, "pub enum {} {{", to_pascal_case(&enum_model.name))?;
        //     for v in &enum_model.variants {
        //         writeln!(out, "    #[serde(rename = \"{}\")]", v)?;
        //         writeln!(out, "    {},", to_pascal_case(v))?;
        //     }
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }

        // for (_name, interface_model) in &self.interfaces {
        //     
        // }

        // for (_name, object_model) in &self.objects {


            
        // }

        // for (_name, union_model) in &self.unions {
        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "pub enum {} {{", to_pascal_case(&union_model.name))?;
    
        //     for object_name in &union_model.implements {
                
        //         writeln!(out, "    {}({}),", to_pascal_case(&object_name), to_pascal_case(&object_name))?;
        //     }
    
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Type {
    Int,
    Float,
    String,
    Boolean,
    ID,
    Enum(Rc<parsed_model::Enum>),
    Union(Rc<parsed_model::Union>),
    Object(Rc<parsed_model::Object>),
    Interface(Rc<parsed_model::Interface>),
    Scalar(Rc<parsed_model::TypeDef>),
}



impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Type::DefinedType(defined_type) => write!(f, "{}", defined_type),
            Type::Int => write!(f, "Int"),
            Type::Float => write!(f, "Float"),
            Type::String => write!(f, "String"),
            Type::Boolean => write!(f, "Boolean"),
            Type::ID => write!(f, "ID"),
            Type::Enum(content) => write!(f, "Enum {}", content.name),
            Type::Union(content) => write!(f, "Union {}", content.name),
            Type::Object(content) => write!(f, "Object {}", content.name),
            Type::Interface(content) => write!(f, "Interface {}", content.name),
            Type::Scalar(content) => write!(f, "Scalar {}", content.name),
        }
    }
}

impl Type {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            // Type::DefinedType(content) => {content.print(out); Ok(0 as usize)},
            Type::Int => out.write(b"Int"),
            Type::Float => out.write(b"Float"),
            Type::String => out.write(b"String"),
            Type::Boolean => out.write(b"Boolean"),
            Type::ID => out.write(b"ID"),
            Type::Enum(content) => {content.print(out); Ok(0 as usize)},
            Type::Union(content) => {content.print(out); Ok(0 as usize)},
            Type::Object(content) => {content.print(out); Ok(0 as usize)},
            Type::Interface(content) => {content.print(out); Ok(0 as usize)},
            Type::Scalar(content) => {content.print(out); Ok(0 as usize)},
        }?;
        Ok(())
    }

    // pub fn position(&self) -> &graphql_parser::Pos {
    //     match self {
    //         Type::Enum(content) => &content.position,
    //         Type::Union(content) => &content.position,
    //         Type::Object(content) => &content.position,
    //         Type::Interface(content) => &content.position,
    //         Type::Scalar(content) => &content.position,
    //     }
    // }

    fn new(out: &mut Output, field: &parsed_model::Field, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Self {
        match &field.ty {
            parsed_model::Type::DefinedType(name) => {
                if let Some(def) = named_types.get(name) {
                    match def {
                        parsed_model::TypeDefinition::Enum(content) => Type::Enum(content.clone()),
                        parsed_model::TypeDefinition::Union(content) =>Type::Union(content.clone()),
                        parsed_model::TypeDefinition::Object(content) =>Type::Object(content.clone()),
                        parsed_model::TypeDefinition::Interface(content) =>Type::Interface(content.clone()),
                        parsed_model::TypeDefinition::Scalar(content) =>Type::Scalar(content.clone()),
                    }
                }
                else {
                    out.error(GraphQLError::UndefinedTypeError(field.position.clone(), format!("Missing type {}", name)));
                    Type::String // TODO: is this what we want to do?
                }
            },
            parsed_model::Type::Int => Type::Int,
            parsed_model::Type::Float => Type::Float,
            parsed_model::Type::String => Type::String,
            parsed_model::Type::Boolean => Type::Boolean,
            parsed_model::Type::ID => Type::ID,
        }
     }
     
    fn rust_type(&self, multiple: bool, nonnull: bool) -> String {
        let base_type =  match self {
            // Type::DefinedType(defined_type) => write!(f, "{}", defined_type),
            Type::Int => "i32".to_string(),
            Type::Float => "f64".to_string(),
            Type::String => "String".to_string(),
            Type::Boolean => "bool".to_string(),
            Type::ID => "String".to_string(),
            Type::Enum(content) => to_pascal_case(&content.name),
            Type::Union(content) => to_pascal_case(&content.name),
            Type::Object(content) => to_pascal_case(&content.name),
            Type::Interface(content) => to_pascal_case(&content.name),
            Type::Scalar(content) => to_pascal_case(&content.name),
        };

        let type2 = if multiple {
            format!("Vec<{}>", base_type)
        }
        else {
            base_type
        };

        if nonnull {
            type2
        }
        else {
            format!("Option<{}>", type2)
        }
    }
     
}


#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub multiple: bool,
    pub nonnull: bool,
    pub ty: Type,
}

impl Field {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "ty:        {}", self.ty)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(out: &mut Output, field: &parsed_model::Field, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Field {
        Field {
            name: field.name.clone(),
            multiple: field.multiple,
            nonnull: field.nonnull,
            ty: Type::new(out, field, named_types),
        }
    }

    pub fn rust_type(&self) -> String {
        self.ty.rust_type(self.multiple, self.nonnull)
    }
}

#[derive(Debug)]
pub struct Object {
    pub parsed: Rc<parsed_model::Object>,
    pub fully_implements: Vec<Rc<parsed_model::Interface>>,
    pub fields: HashMap<String, Field>,
}

impl Object {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Object {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "fully_implements {{")?;
            {
                let mut out = out.indent();

                for object in &self.fully_implements {
                    object.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    fn new(out: &mut Output, parsed: Rc<parsed_model::Object>, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Result<TypeDefinition, Box<dyn Error>> {
        let mut fully_implements = Vec::new();

        for interface_name in &parsed.implements {
            if let Some(defined_type) = named_types.get(interface_name) {
                if let parsed_model::TypeDefinition::Interface(interface) = defined_type {
                    let mut ok = true;
                    for field_name in &interface.field_names {
                        if ! parsed.fields.contains_key(field_name) {
                            ok = false;
                            break;
                        }
                    }
                    if ok {
                        fully_implements.push(interface.clone());
                    }
                }
                else {
                    out.error(GraphQLError::TypeMismatchError(parsed.position, format!("Expected interface {} but found {}", parsed.name, defined_type.type_name())));
                }
            }
            else {
                out.error(GraphQLError::MissingInterfaceError(parsed.position, format!("Interface {} Not Found", parsed.name)))
            };
        }

        let mut fields = HashMap::new();

        for (name, field) in &parsed.fields {
            fields.insert(name.clone(), Field::new(out, field, &named_types));
        }

        Ok(TypeDefinition::Object(Rc::new(Object {
            parsed,
            fully_implements,
            fields,
        })))
    }
}

#[derive(Debug)]
pub struct Interface {
    pub parsed: Rc<parsed_model::Interface>,
    pub implemented_by: Vec<Rc<parsed_model::Object>>,
    pub fields: HashMap<String, Field>,
}

impl Interface {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Interface {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "implemented_by {{")?;
            {
                let mut out = out.indent();

                for object in &self.implemented_by {
                    object.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn new( out: &mut Output, parsed: Rc<parsed_model::Interface>, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Result<TypeDefinition, Box<dyn Error>> {
        let mut implemented_by = Vec::new();

        for (name, defined_type) in named_types {
            if let parsed_model::TypeDefinition::Object(object_model) = defined_type {
                for implements in &object_model.fully_implements {
                    if implements == &parsed.name {
                        implemented_by.push(object_model.clone());
                        break;
                    }
                }
            }
        } 

        // for interface_name in &parsed.implements {
        //     if let Some(interface) = interfaces.get(interface_name) {
        //         let mut ok = true;
        //         for field_name in &interface.field_names {
        //             if ! parsed.fields.contains_key(field_name) {
        //                 ok = false;
        //                 break;
        //             }
        //         }
        //         if ok {
        //             fully_implements.push(*interface);
        //         }
        //     }
        //     else {
        //         out.error(GraphQLError::MissingInterfaceError(parsed.position, format!("Interface {} Not Found", parsed.name)))
        //     };

            
        // }



        let mut fields = HashMap::new();

        for (name, field) in &parsed.fields {
            fields.insert(name.clone(), Field::new(out, field, named_types));
        }

        Ok(TypeDefinition::Interface(Rc::new(Interface {
            parsed,
            implemented_by,
            fields,
        })))
    }
}

#[derive(Debug)]
pub struct Schema {
    parsed: Rc<parsed_model::Schema>,
    defined_types: HashMap<String, TypeDefinition>,

    // pub enums: Vec<&'p parsed_model::Enum>,

    // pub objects: HashMap<String, Rc<parsed_model::Object>>,
    // pub interfaces: HashMap<String, Rc<parsed_model::Interface>>,
    // pub scalars: HashMap<String, Rc<parsed_model::TypeDef>>,
    // pub enums: HashMap<String, Rc<parsed_model::Enum>>,
    // pub unions: HashMap<String, Rc<parsed_model::Union>>,

    query: Rc<Object>,
    mutation: Option<Rc<Object>>,
    subscription: Option<Rc<Object>>,
    // queries: Vec<Query>,
}

impl Schema {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "named_types {{")?;
            {
                let mut out = out.indent();

                for (name, ty) in &self.defined_types {
                    write!(out, "{}\t", name)?;
                    ty.print(&mut out);
                }
            }
            writeln!(out, "}}")?;

            // writeln!(out, "objects {{")?;
            // {
            //     let mut out = out.indent();

            //     for object in self.objects.values() {
            //         object.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            // writeln!(out, "interfaces {{")?;
            // {
            //     let mut out = out.indent();

            //     for interface in self.interfaces.values() {
            //         interface.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            // writeln!(out, "scalars {{")?;
            // {
            //     let mut out = out.indent();

            //     for object in self.scalars.values() {
            //         object.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            // writeln!(out, "enums {{")?;
            // {
            //     let mut out = out.indent();

            //     for object in self.enums.values() {
            //         object.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            // writeln!(out, "unions {{")?;
            // {
            //     let mut out = out.indent();

            //     for object in self.unions.values() {
            //         object.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }


    fn get_object(out: &mut Output, name: &Option<String>, position: graphql_parser::Pos, defined_types: &HashMap<String, TypeDefinition>, missing_error: Option<String>) -> Option<Rc<Object>> {
        if let Some(name) = name {
            if let Some(type_definition) = &defined_types.get(name) {
                if let TypeDefinition::Object(object) = type_definition {
                    Some(object.clone())
                }
                else {
                    out.error(GraphQLError::TypeMismatchError(position, format!("Expected object \"{}\" but found {}", name, type_definition.type_name())));
                    None
                }
            }
            else {
                out.error(GraphQLError::MissingObjectError(position, format!("Expected object \"{}\" not found", name)));
                None
            }
        }
        else {
            if let Some(message) = missing_error { 
                out.error(GraphQLError::MissingObjectError(position, message));
            }
            None
        }
    }

    pub fn new(parsed: Rc<parsed_model::Schema>, out: &mut Output) -> Result<Schema, Box<dyn Error>> {
        let mut defined_types: HashMap<String, TypeDefinition> = HashMap::new();
        
        
        for(name, ty) in &parsed.named_types {
            defined_types.insert(name.clone(), match ty {
                parsed_model::TypeDefinition::Enum(_) => todo!(),
                parsed_model::TypeDefinition::Union(union) => todo!(),
                parsed_model::TypeDefinition::Object(object) => {
                    Object::new(out, object.clone(), &parsed.named_types)?
                },
                parsed_model::TypeDefinition::Interface(interface) => {
                    Interface::new(out, interface.clone(), &parsed.named_types)?
                },
                parsed_model::TypeDefinition::Scalar(type_def) => todo!(),
            });
        }




        // let mut parsed_objects = HashMap::new();
        // let mut parsed_interfaces = HashMap::new();
        // let mut parsed_scalars = HashMap::new();
        // let mut parsed_enums = HashMap::new();
        // let mut parsed_unions = HashMap::new();

        // for(name, ty) in &parsed.named_types {
        //     match ty {
        //         parsed_model::TypeDefinition::Enum(value) => {parsed_enums.insert(value.name.clone(), value.clone());},
        //         parsed_model::TypeDefinition::Union(value) => {parsed_unions.insert(value.name.clone(), value.clone());},
        //         parsed_model::TypeDefinition::Object(value) => {parsed_objects.insert(value.name.clone(), value.clone());},
        //         parsed_model::TypeDefinition::Interface(value) => {parsed_interfaces.insert(value.name.clone(), value.clone());},
        //         parsed_model::TypeDefinition::Scalar(value) => {parsed_scalars.insert(value.name.clone(), value.clone());},
        //     }
        // }

        // let mut objects = HashMap::new();

        // for (name, value) in &parsed_objects {
        //     objects.insert(name, Object::new(value.clone(), &parsed_interfaces, out)?);
        // }

        // let mut interfaces = HashMap::new();

        // for (name, value) in &parsed_interfaces {
        //     interfaces.insert(name, Interface::new(value.clone(), &parsed_objects, out)?);
        // }

        let query = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.query, schema_definition.position, &defined_types, Some(format!("Schema declartion is present but no query is given")))
        }
        else {
            Self::get_object(out, &Some("Query".to_string()), graphql_parser::Pos { line: 0, column: 0 }, &defined_types, Some(format!("Default query \"Query\" not found")))
        };

        let mutation = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.mutation, schema_definition.position, &defined_types, None)
        }
        else {
            Self::get_object(out, &Some("Mutation".to_string()), graphql_parser::Pos { line: 0, column: 0 }, &defined_types, None)
        };
        
        let subscription = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.subscription, schema_definition.position, &defined_types, None)
        }
        else {
            Self::get_object(out, &Some("Subscription".to_string()), graphql_parser::Pos { line: 0, column: 0 }, &defined_types, None)
        };
        
        // let query = if let Some(schema_definition) = &parsed.schema_definition {
        //     if let Some(query_name) = &schema_definition.query {
        //         if let Some(query_type) = &defined_types.get(query_name) {
        //             if let TypeDefinition::Object(query) = query_type {
        //                 Some(query.clone())
        //             }
        //             else {
        //                 out.error(GraphQLError::TypeMismatchError(schema_definition.position, format!("Declared query \"{}\" expected object but found {}", query_name, query_type.type_name())));
        //                 None
        //             }
        //         }
        //         else {
        //             out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared query \"{}\" not found", query_name)));
        //             None
        //         }
        //     }
        //     else {
        //         out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Schema declartion is present but no query is given")));
        //         None
        //     }
        // }
        // else {
        //     if let Some(query_type) = defined_types.get(&"Query".to_string()) {
        //         if let TypeDefinition::Object(query) = query_type {
        //             Some(query.clone())
        //         }
        //         else {
        //             out.error(GraphQLError::TypeMismatchError(graphql_parser::Pos { line: 0, column: 0 }, format!("Default query \"Query\" expected object but found {}", query_type.type_name())));
        //             None
        //         }
        //     }
        //     else {
        //         out.error(GraphQLError::NoQueryDefinition);
        //         None
        //     }
        // };

        // let mutation = if let Some(schema_definition) = &parsed.schema_definition {
        //     if let Some(mutation_name) = &schema_definition.mutation {
        //         if let Some(mutation) = parsed_objects.get(mutation_name) {
        //             Some(mutation.clone())
        //         }
        //         else {
        //             out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared mutation \"{}\" not found", mutation_name)));
        //             None
        //         }
        //     }
        //     else {
        //         None
        //     }
        // }
        // else {
        //     if let Some(mutation) = parsed_objects.get(&"Mutation".to_string()) {
        //         Some(mutation.clone())
        //     }
        //     else {
        //         None
        //     }
        // };
        
        // let subscription = if let Some(schema_definition) = &parsed.schema_definition {
        //     if let Some(subscription_name) = &schema_definition.subscription {
        //         if let Some(subscription) = parsed_objects.get(subscription_name) {
        //             Some(subscription.clone())
        //         }
        //         else {
        //             out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared subscription \"{}\" not found", subscription_name)));
        //             None
        //         }
        //     }
        //     else {
        //         None
        //     }
        // }
        // else {
        //     if let Some(subscription) = parsed_objects.get(&"Subscription".to_string()) {
        //         Some(subscription.clone())
        //     }
        //     else {
        //         None
        //     }
        // };


        // let query = if let Some(query_name) = &parsed.query {
        //     if let Some(query) = parsed.objects.get(query_name) {
        //         query
        //     }
        //     else {
        //         out.error(GraphQLError::NoQueryDefinition(parsed.position.unwrap(), format!("No query object \"{}\" defined", query)));
        //     }
        // }
        // else {
        //     if let Some(query) = parsed.objects.get("Query") {
        //         query
        //     }
        //     else {
        //         out.error(GraphQLError::NoQueryDefinition(parsed.position.unwrap(), format!("No query object \"{}\" defined", query)));
        //     }
            
        // };

        // return Ok(Schema {
        //     parsed,
        //     query,
        // });

        Ok(Schema {
            parsed,
            defined_types,
            // objects: parsed_objects,
            // interfaces: parsed_interfaces,
            // scalars: parsed_scalars,
            // unions: parsed_unions,
            // enums: parsed_enums,
            query: query.ok_or(Box::new(GraphQLError::ValidationError))?,
            mutation,
            subscription,
        })
        
    }


    pub fn generate(&self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        for (name, defined_type) in &self.defined_types {
            defined_type.generate(out)?;
        }

        // for (name, scalar) in &self.scalars {
        //     writeln!(out, "type {} = {}; // HERE", to_pascal_case(&scalar.name), &scalar.rust_type)?;
        // }

        // for (name, enum_model) in &self.enums {
        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "#[serde(rename = \"{}\")]", &enum_model.name)?;
        //     writeln!(out, "pub enum {} {{", to_pascal_case(&enum_model.name))?;
        //     for v in &enum_model.variants {
        //         writeln!(out, "    #[serde(rename = \"{}\")]", v)?;
        //         writeln!(out, "    {},", to_pascal_case(v))?;
        //     }
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }

        // for (_name, interface_model) in &self.interfaces {
        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "pub enum {} {{", to_pascal_case(&interface_model.name))?;

        //     for object_name in &interface_model.implemented_by {
                
        //         writeln!(out, "    {}({}),", to_pascal_case(&object_name), to_pascal_case(&object_name))?;
        //     }

        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }

        // for (_name, object_model) in &self.objects {


        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "#[serde(rename = \"{}\")]", &object_model.name)?;
        //     writeln!(out, "pub struct {} {{", to_pascal_case(&object_model.name))?;
    
        //     for name in &object_model.field_names {
        //         if let Some(field) = object_model.fields.get(name) {
        //             writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
        //             writeln!(out, "    {}_: XX{},", to_snake_case(&field.name), field.ty)?;
        //         }
        //     }
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }

        // for (_name, union_model) in &self.unions {
        //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //     writeln!(out, "pub enum {} {{", to_pascal_case(&union_model.name))?;
    
        //     for object_name in &union_model.implements {
                
        //         writeln!(out, "    {}({}),", to_pascal_case(&object_name), to_pascal_case(&object_name))?;
        //     }
    
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;
        // }
        Ok(())
    }
    
    // pub fn new(parsed: parsed_model::Schema<'p>, out: &mut Output) -> Result<Schema<'p>, Box<dyn Error>> {

    //     Ok(Schema {
    //         parsed,
    //         query: || -> Option<&'p parsed_model::Object> {
    //             if let Some(schema_definition) = &parsed.schema_definition {
    //                 if let Some(query_name) = &schema_definition.query {
    //                     if let Some(query) = parsed.objects.get(query_name) {
    //                         Some(query)
    //                     }
    //                     else {
    //                         out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared query \"{}\" not found", query_name)));
    //                         None
    //                     }
    //                 }
    //                 else {
    //                     out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Schema declartion is present but no query is given")));
    //                     None
    //                 }
    //             }
    //             else {
    //                 if let Some(query) = parsed.objects.get("Query") {
    //                     Some(query)
    //                 }
    //                 else {
    //                     out.error(GraphQLError::NoQueryDefinition);
    //                     None
    //                 }
    //             }
                
    //         }().ok_or(Box::new(GraphQLError::ValidationError))?,
    //         mutation: || -> Option<&'p parsed_model::Object> {
    //             if let Some(schema_definition) = &parsed.schema_definition {
    //                 if let Some(mutation_name) = &schema_definition.mutation {
    //                     if let Some(mutation) = parsed.objects.get(mutation_name) {
    //                         Some(mutation)
    //                     }
    //                     else {
    //                         out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared mutation \"{}\" not found", mutation_name)));
    //                         None
    //                     }
    //                 }
    //                 else {
    //                     None
    //                 }
    //             }
    //             else {
    //                 parsed.objects.get("Mutation")
    //             }
    //         }(),
    //         subscription: || -> Option<&'p parsed_model::Object> {
    //             if let Some(schema_definition) = &parsed.schema_definition {
    //                 if let Some(subscription_name) = &schema_definition.subscription {
    //                     if let Some(subscription) = parsed.objects.get(subscription_name) {
    //                         Some(subscription)
    //                     }
    //                     else {
    //                         out.error(GraphQLError::MissingObjectError(schema_definition.position, format!("Declared subscription \"{}\" not found", subscription_name)));
    //                         None
    //                     }
    //                 }
    //                 else {
    //                     None
    //                 }
    //             }
    //             else {
    //                 parsed.objects.get("Subscription")
    //             }
    //         }(),
    //     })
        
    // }
    
    // pub fn generate(&self, out: &mut Output) -> Result<(), Box<dyn Error>> {
    //     self.parsed.generate(out);
    //     // if !self.variables.is_empty() {

    //     //     writeln!(out, "struct {} {{", &to_pascal_case(&format!("{}Variables", &self.name)))?;
    //     //     {
    //     //         let mut out = out.indent();

    //     //         for variable in &mut self.variables {
    //     //             writeln!(out, "{}_: {},", &to_snake_case(&variable.name), &to_pascal_case(&variable.type_name))?;
    //     //         }
    //     //     }
    //     //     writeln!(out, "}}")?;
    //     // }
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct VariableDefinition {
    parsed: Rc<parsed_model::VariableDefinition>,
    // queries: Vec<Query>,
}

impl VariableDefinition {
    pub fn new(parsed: Rc<parsed_model::VariableDefinition>, out: &mut Output) -> Result<VariableDefinition, Box<dyn Error>> {


        Ok(VariableDefinition {
            parsed,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        // writeln!(out, "Operations {{")?;
        // {
        //     let mut out = out.indent();

        //     writeln!(out, "variables {{")?;
        //     {
        //         let mut out = out.indent();

        //         for item in self.variables {
        //             item.print(&mut out)?;
        //         }
        //     }
        //     writeln!(out, "}}")?;

        //     writeln!(out, "selections {{")?;
        //     {
        //         let mut out = out.indent();

        //         for item in self.selections {
        //             item.print(&mut out)?;
        //         }
        //     }
        //     writeln!(out, "}}")?;
        // }
        // writeln!(out, "}}")
        writeln!(out, "VariableDefinition is empty")
    }
    
    pub fn generate(&mut self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        // if !self.variables.is_empty() {

        //     writeln!(out, "struct {} {{", &to_pascal_case(&format!("{}Variables", &self.name)))?;
        //     {
        //         let mut out = out.indent();

        //         for variable in &mut self.variables {
        //             writeln!(out, "{}_: {},", &to_snake_case(&variable.name), &to_pascal_case(&variable.type_name))?;
        //         }
        //     }
        //     writeln!(out, "}}")?;
        // }
        Ok(())
    }
}


#[derive(Debug)]
pub enum Selection {
    Field(SelectionField),
    // FragmentSpread(FragmentSpread),
    // InlineFragment(InlineFragment),
} 

impl Selection {
    pub fn new(selection: &parsed_model::Selection, schema: &Rc<Schema>, context: &Rc<Object>, out: &mut Output) -> Result<Selection, Box<dyn Error>> {
        Ok(match selection {
            parsed_model::Selection::Field(selection_field) => Selection::Field(SelectionField::new(selection_field.clone(), schema, out)?),
            // graphql_parser::query::Selection::OptionalField(field) => 
            //     Selection::Field(SelectionField::new(out, *field, true)?),
            // graphql_parser::query::Selection::Field(field) => Selection::Field(SelectionField::new(out, field, false)?),
            // graphql_parser::query::Selection::FragmentSpread(fragment_spread) => todo!(),
            // graphql_parser::query::Selection::InlineFragment(inline_fragment) => todo!(),
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::Field(selection_field) => selection_field.print(out),
        }
    }

    pub fn position(&self) -> graphql_parser::Pos {
        match self {
            Selection::Field(selection_field) => selection_field.parsed.position,
        }
    }
}



#[derive(Debug)]
pub struct SelectionField {
    parsed: Rc<parsed_model::SelectionField>,
    // queries: Vec<Query>,
}

impl SelectionField {
    pub fn new(parsed: Rc<parsed_model::SelectionField>, schema: &Rc<Schema>, out: &mut Output) -> Result<SelectionField, Box<dyn Error>> {


        Ok(SelectionField {
            parsed,
        })
    }



    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionField {{")?;
        {
            let mut out = out.indent();

            out.start_of_line = false;
            write!(out, "parsed:    ")?;
            self.parsed.print(&mut out)?;
            // writeln!(out, "name:      {}", self.name)?;
            // writeln!(out, "optional:  {}", self.optional)?;
            // writeln!(out, "arguments {{")?;
            // {
            //     let mut out = out.indent();

            //     for argument in &self.arguments {
            //         argument.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            // writeln!(out, "selections {{")?;
            // {
            //     let mut out = out.indent();

            //     for selection in &self.selections {
            //         selection.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    pub fn generate(&mut self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        // if !self.variables.is_empty() {

        //     writeln!(out, "struct {} {{", &to_pascal_case(&format!("{}Variables", &self.name)))?;
        //     {
        //         let mut out = out.indent();

        //         for variable in &mut self.variables {
        //             writeln!(out, "{}_: {},", &to_snake_case(&variable.name), &to_pascal_case(&variable.type_name))?;
        //         }
        //     }
        //     writeln!(out, "}}")?;
        // }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Query {
    // parsed: &parsed_model::Query,
    // queries: Vec<Query>,
    variables: HashMap<String, VariableDefinition>,
    selections: Vec<Selection>,
}

impl Query {
    pub fn new(parsed: parsed_model::Query, schema:  &Rc<Schema>, out: &mut Output) ->  Result<Query, Box<dyn Error>> {
        let mut variables = HashMap::new();

        for variable in &parsed.variables {
            if let Some(existing) = variables.insert(variable.name.clone(), VariableDefinition::new(variable.clone(), out)?) {
                out.error(GraphQLError::DuplicateName(existing.parsed.position, variable.position, variable.name.clone()));
            }
        }

        let mut selections = Vec::new();
        for selection in &parsed.selections {
            selections.push(Selection::new(selection, schema, &schema.query, out)?);
        }

        Ok(Query {
            // parsed,
            variables,
            selections,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Query {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "variables {{")?;
            {
                let mut out = out.indent();

                for item in self.variables.values() {
                    item.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "selections {{")?;
            {
                let mut out = out.indent();

                for item in &self.selections {
                    item.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    pub fn generate(&self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        // if !self.variables.is_empty() {

        //     writeln!(out, "struct {} {{", &to_pascal_case(&format!("{}Variables", &self.name)))?;
        //     {
        //         let mut out = out.indent();

        //         for variable in &mut self.variables {
        //             writeln!(out, "{}_: {},", &to_snake_case(&variable.name), &to_pascal_case(&variable.type_name))?;
        //         }
        //     }
        //     writeln!(out, "}}")?;
        // }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Operations {
    queries: Vec<Query>,
}

impl Operations {
    pub fn new(parsed: parsed_model::Operations, out: &mut Output, schema: &Rc<Schema>) ->  Result<Operations, Box<dyn Error>>{
        let mut queries = Vec::new();

        for query in parsed.queries {
            queries.push(Query::new(query, schema, out)?);
        }
        Ok(Operations {
            queries,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Operations {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "queries {{")?;
            {
                let mut out = out.indent();

                for query in &self.queries {
                    query.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn generate(&self, out: &mut Output) -> Result<(), Box<dyn Error>> {
        for query in &self.queries {
            query.generate(out)?;
        }
        Ok(())
    }
}