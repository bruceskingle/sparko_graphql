use std::fmt::Display;
use std::io::Write;
use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::to_snake_case;
use crate::utils::to_pascal_case;
use crate::BuildError;
use crate::BuildWarning;
use crate::Error;
use crate::ErrorCollector;
use crate::Output;

#[derive(Debug)]
pub enum Status {
    Unchecked,
    Invalid,
    Validated,
    Generated,
}

#[derive(Debug)]
pub struct SchemaDefinition {
    pub position: Pos,
    pub query: Option<String>,
    pub mutation: Option<String>,
    pub subscription: Option<String>,
}

impl SchemaDefinition {
    fn new(_err: &mut ErrorCollector, schema_definition: graphql_parser::schema::SchemaDefinition<'_, String>) -> SchemaDefinition {
        SchemaDefinition {
            position: schema_definition.position,
            query: schema_definition.query,
            mutation: schema_definition.mutation,
            subscription: schema_definition.subscription,
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub status: Status,
    pub defined_types: IndexMap<String, TypeDefinition>,
    pub schema_definition: Option<SchemaDefinition>,
}

impl Schema {

    pub fn new(err: &mut ErrorCollector,
        ast: graphql_parser::schema::Document<'_, String>) -> Result<Schema, Error> {
        let mut model = Schema {
            status: Status::Unchecked,
            defined_types: IndexMap::new(),
            schema_definition: None,
        };

        for def in ast.definitions {
            match def {
                graphql_parser::schema::Definition::SchemaDefinition(schema_definition) => {
                    if let Some(existing) = &model.schema_definition {
                        err.error(BuildError::DuplicateName(schema_definition.position, existing.position, "Schema".to_string()));
                    }
                    else {
                        model.schema_definition = Some(SchemaDefinition::new(err, schema_definition));
                    }
                },
                graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                    if let Ok(type_definition) = TypeDefinition::new(err, type_definition) {
                        let position = type_definition.position().clone();
                        if let Some(existing) = model.defined_types.insert(type_definition.name().clone(), type_definition) {
                            err.error(BuildError::DuplicateName(existing.position().clone(), position, existing.name().clone()));
                        }
                    }
                },
                graphql_parser::schema::Definition::TypeExtension(type_extension) => {
                    err.warning(BuildWarning::UnsupportedFeature(match type_extension {
                        graphql_parser::schema::TypeExtension::Scalar(value) => value.position,
                        graphql_parser::schema::TypeExtension::Object(value) => value.position,
                        graphql_parser::schema::TypeExtension::Interface(value) => value.position,
                        graphql_parser::schema::TypeExtension::Union(value) => value.position,
                        graphql_parser::schema::TypeExtension::Enum(value) => value.position,
                        graphql_parser::schema::TypeExtension::InputObject(value) => value.position,
                    }, "TypeExtension"));
                },
                graphql_parser::schema::Definition::DirectiveDefinition(directive_definition) => {
                    err.warning(BuildWarning::UnsupportedFeature(directive_definition.position, "DirectiveDefinition"));
                },
            }
        }

        Ok(model)
    }

    pub fn validate(&mut self, err: &mut ErrorCollector) -> Result<(), Error> {
        for defined_type in self.defined_types.values_mut() {
            defined_type.vaidate(err, &self)?;
        }
        self.status = Status::Validated;

        Ok(())
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "named_types {{")?;
            {
                let mut out = out.indent();

                for (name, ty) in &self.defined_types {
                    write!(out, "{}\t", name)?;
                    ty.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub enum TypeDefinition {
    Scalar(Scalar),
    Object(Object),
    // Interface(Interface),
    // Union(Union),
    // Enum(Enum),
    // InputObject(InputObject),
}

impl TypeDefinition {
    fn new(err: &mut ErrorCollector, type_definition: graphql_parser::schema::TypeDefinition<'_, String>) -> Result<TypeDefinition, Error> {
        match type_definition {
            graphql_parser::schema::TypeDefinition::Scalar(ast) => Ok(TypeDefinition::Scalar(Scalar::new(ast))),
            graphql_parser::schema::TypeDefinition::Object(ast) => 
                // err.fail(BuildError::Unimplemented(ast.position, "Object")),
                Ok(TypeDefinition::Object(Object::from_object(err, ast))),
            graphql_parser::schema::TypeDefinition::Interface(ast) => 
                err.fail(BuildError::Unimplemented(ast.position, "Interface")),
                // TypeDefinition::Interface(Interface::new(err, ast)),
            graphql_parser::schema::TypeDefinition::Union(ast) => 
                err.fail(BuildError::Unimplemented(ast.position, "Union")),
                // TypeDefinition::Union(Union::new(ast)),
            graphql_parser::schema::TypeDefinition::Enum(ast) => 
                err.fail(BuildError::Unimplemented(ast.position, "Enum")),
                // TypeDefinition::Enum(Enum::new(err, ast)),
            graphql_parser::schema::TypeDefinition::InputObject(ast) => 
                err.fail(BuildError::Unimplemented(ast.position, "InputObject")),
                // TypeDefinition::Object(Object::from_input_object(err, ast)),
        }
    }

    pub fn name(&self) -> &String {
        match self {
            TypeDefinition::Scalar(content) => &content.name,
            TypeDefinition::Object(content) => &content.name,
        }
    }

    pub fn position(&self) -> &Pos {
        match self {
            TypeDefinition::Scalar(content) => &content.position,
            TypeDefinition::Object(content) => &content.position,
        }
    }

    pub fn vaidate(&mut self, err: &mut ErrorCollector, schema: &Schema) -> Result<(), Error> {
        match self {
            TypeDefinition::Scalar(content) => content.validate(err, schema),
            TypeDefinition::Object(content) => content.validate(err, schema),
            // TypeDefinition::Interface(content) => content.validate(err),
            // TypeDefinition::Union(content) => content.validate(err),
            // TypeDefinition::Enum(content) => content.validate(err),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            TypeDefinition::Scalar(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            // TypeDefinition::Interface(content) => content.print(out),
            // TypeDefinition::Union(content) => content.print(out),
            // TypeDefinition::Enum(content) => content.print(out),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Scalar(_) => "scalar",
            TypeDefinition::Object(_) => "object",
            // TypeDefinition::Interface(_) => "interface",
            // TypeDefinition::Union(_) => "union",
            // TypeDefinition::Enum(_) => "enum",
        }
    }
}

// This is the definition of a scalar, a typedef in C terms.
#[derive(Debug)]
pub struct Scalar {
    pub name: String,
    pub position: Pos,
    pub rust_name: String,
    pub rust_type: String,
}

impl Scalar {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Scalar {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "rust_name: {}", self.rust_name)?;
            writeln!(out, "rust_type: {}", self.rust_type)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(scalar_type: graphql_parser::schema::ScalarType<'_, String>) -> Self {
        let rust_name = to_pascal_case(&scalar_type.name);
        let rust_type = String::from("serde_json::Value");

        Scalar {
            position: scalar_type.position,
            name: scalar_type.name,
            rust_name,
            rust_type,
        }
    }
    
    fn validate(&self, _err: &mut ErrorCollector<'_>, _schema: &Schema) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Object {
    pub status: Status,
    pub name: String,
    pub position: Pos,
    pub fields: IndexMap<String, Field>,
    pub implements: Vec<String>,
    pub is_input: bool,
    // pub fully_implements: Vec<String>,
}

impl Object {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Object {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "fields {{")?;
            {
                let mut out = out.indent();

                for field in self.fields.values() {
                    field.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "implements {{")?;
            {
                let mut out = out.indent();

                for name in &self.implements {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }

    pub fn from_object(_err: &mut ErrorCollector, object_type: graphql_parser::schema::ObjectType<'_, String>) -> Object {
        let mut fields = IndexMap::new();

        for f in object_type.fields {
            fields.insert(f.name.clone(), Field::from_field(f));
            
        }
        Object {
            status: Status::Unchecked,
            position: object_type.position,
            name: object_type.name,
            fields,
            implements: object_type.implements_interfaces,
            is_input: false,
        }
    }
    
    fn from_input_object(_err: &mut ErrorCollector<'_>, object_type: graphql_parser::schema::InputObjectType<'_, String>) -> Object {
        let mut fields = IndexMap::new();

        for f in object_type.fields {
            fields.insert(f.name.clone(), Field::from_input_value(f));
            
        }
        Object {
            status: Status::Unchecked,
            position: object_type.position,
            name: object_type.name,
            fields,
            implements: Vec::new(),
            is_input: true,
        }
    }
    
    fn validate(&mut self, err: &mut ErrorCollector<'_>, schema: &Schema) -> Result<(), Error> {
        for field in self.fields.values_mut() {
            field.validate(err, schema)?
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Field {
    pub position: Pos,
    pub name: String,
    pub ty: Type,
}

impl Field {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "ty:        {}", self.ty)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }

    fn do_new(position: Pos, name: String, field_type: graphql_parser::schema::Type<'_, String>) -> Field {
        let ty = Type::new(field_type, &position);
        Field {
            position,
            name,
            ty,
        }
        
     }

     pub fn from_input_value(field: graphql_parser::schema::InputValue<'_, String>) -> Self {
         Self::do_new(field.position, field.name, field.value_type)
     }

     pub fn from_field(field: graphql_parser::schema::Field<'_, String>) -> Self {
         Self::do_new(field.position, field.name, field.field_type)
     }
    
    fn from_variable(variable: graphql_parser::query::VariableDefinition<'_, String>) -> Field {
        Self::do_new(variable.position, variable.name, variable.var_type)
    }
    
    fn validate(&mut self, err: &mut ErrorCollector<'_>, schema: &Schema) -> Result<(), Error> {
        self.ty.validate(err, schema)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltinType {
    Int,
    Float,
    String,
    Boolean,
    ID,
}

impl BuiltinType {
    pub fn from_str(v: &str) -> Option<BuiltinType> {
        match v {
            "Boolean" => Some(BuiltinType::Boolean),
            "Float" => Some(BuiltinType::Float),
            "ID" => Some(BuiltinType::ID),
            "Int" => Some(BuiltinType::Int),
            "String" => Some(BuiltinType::String),

            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuiltinType::Int =>"Int",
            BuiltinType::Float =>"Float",
            BuiltinType::String =>"String",
            BuiltinType::Boolean =>"Boolean",
            BuiltinType::ID =>"ID",
        }
    }

    pub fn rust_type(&self) -> &'static str {
        match self {
            BuiltinType::Int =>"i32",
            BuiltinType::Float =>"f64",
            BuiltinType::String =>"String",
            BuiltinType::Boolean =>"bool",
            BuiltinType::ID =>"String",
        }
    }
}

impl Display for BuiltinType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug)]
pub enum ScalarType {
    DefinedType{name: String, position: Pos},
    BuiltinType(BuiltinType),
}

impl ScalarType {
    fn new(v: &str, position: &Pos) -> ScalarType {
        if let Some(builtin_type) = BuiltinType::from_str(v) {
            ScalarType::BuiltinType(builtin_type)
        }
        else {
            ScalarType::DefinedType{ name: v.to_string(), position: position.clone()}
        }
    }
    
    fn validate(&self, err: &mut ErrorCollector<'_>, schema: &Schema) -> Result<(), Error> {
        match self {
            ScalarType::DefinedType { name, position } => todo!(),
            ScalarType::BuiltinType(builtin_type) => Ok(()),
        }
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::DefinedType{name, position} => write!(f, "{} (referenced at {})", name, position),
            ScalarType::BuiltinType(builtin_type) => builtin_type.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum Type {
    Scalar(ScalarType),
    Required(Box<Type>),
    Array(Box<Type>),
}

impl Type {
    fn new(parsed: graphql_parser::schema::Type<'_, String>, position: &Pos) -> Type {
        match parsed {
            graphql_parser::query::Type::NamedType(name) => Type::Scalar(ScalarType::new(&name, position)),
            graphql_parser::query::Type::ListType(wrapped) => Type::Array(Box::new(Type::new(*wrapped, position))),
            graphql_parser::query::Type::NonNullType(wrapped) => Type::Required(Box::new(Type::new(*wrapped, position))),
        }
    }
    
    fn validate(&mut self, err: &mut ErrorCollector<'_>, schema: &Schema) -> Result<(), Error> {
        match self {
            Type::Scalar(scalar_type) => scalar_type.validate(err, schema),
            Type::Required(wrapped) => wrapped.validate(err, schema),
            Type::Array(wrapped) => wrapped.validate(err, schema),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Scalar(scalar_type) => scalar_type.fmt(f),
            Type::Required(wrapped) => write!(f, "{}!", wrapped),
            Type::Array(wrapped) => write!(f, "[{}]", wrapped),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::BaseErrorCollector;

    use super::*;

    fn test_schema(schema: &str) -> BaseErrorCollector {

        let mut base = BaseErrorCollector::new_test();
        let mut err = ErrorCollector::new(&mut base);
        let builder = crate::builder("test");
        let _ = builder.do_build_schema(&mut err, schema);
        
        base
    }

    // fn test_query(schema: &str, query: &str) -> BaseErrorCollector {

    //     let mut base = BaseErrorCollector::new();
    //     let mut err = ErrorCollector::new(&mut base);
    //     let builder = crate::builder("test");
    //     let schema = builder.do_build_schema(&mut err, schema).unwrap();
    //     let _q = builder.do_build_query(&mut err, &schema, query, "test_query");
        
    //     base
    // }

    #[test]
    fn test_builtin_types() {
        let out = test_schema(r#"
type Query {
    int: Int,
    float: Float,
    string: String,
    boolean: Boolean,
    id: ID,
}
"#);
        out.expect_ok();        
        // if let Some(BuildError>::MissingInterfaceError(..)) = out.expect_one_error() {
        //     return;
        // }
        // println!("{}", out);
        // panic!("Expected missing interface");
    }

    #[test]
    fn test_builtin_required_types() {
        let out = test_schema(r#"
type Query {
    int: Int!,
    float: Float!,
    string: String!,
    boolean: Boolean!,
    id: ID!,
}
"#);
        out.expect_ok();   
    }

    #[test]
    fn test_builtin_multiple_types() {
        let out = test_schema(r#"
type Query {
    int:[Int],
    float: [Float],
    string: [String],
    boolean: [Boolean],
    id: [ID],
}
"#);
        out.expect_ok();   
    }

    #[test]
    fn test_builtin_required_multiple_types() {
        let out = test_schema(r#"
    type Query {
    int:[Int]!,
    float: [Float]!,
    string: [String]!,
    boolean: [Boolean]!,
    id: [ID]!,
    }
    "#);
        out.expect_ok();   
    }

    #[test]
    fn test_builtin_required_multiple_required_types() {
        let out = test_schema(r#"
    type Query {
    int:[Int!]!,
    float: [Float!]!,
    string: [String!]!,
    boolean: [Boolean!]!,
    id: [ID!]!,
    }
    "#);
        out.expect_ok();   
    }

    #[test]
    fn test_builtin_required_multiple_required_types2() {
        let out = test_schema(r#"
    type Query {
    int:[[Int!]!],
    float: [[[Float!]!]!]!,
    string: [[[String]]!],
    boolean: [Boolean!]!,
    id: [ID!]!,
    }
    "#);
        out.expect_ok();   
    }
}

#[derive(Debug)]
pub struct ExecutableDocument {
    pub status: Status,
    pub name: String,
    pub fragments: IndexMap<String, FragmentDefinition>,
    pub queries: Vec<GenericOperation>,
    pub mutations: Vec<GenericOperation>
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector,
        definitions: Vec<graphql_parser::query::Definition<'_, String>>, model_name: &str,
        schema: &Schema) -> ExecutableDocument {

        let name = to_snake_case(model_name);
        //     if model_name.ends_with(".graphql") {
        //     let end = model_name.len() - 8;
        //     &model_name[..end]
        // }
        // else {
        //     &model_name
        // });

        let mut fragments = IndexMap::new();
        let mut queries = Vec::new();
        let mut mutations = Vec::new();

        for def in definitions {
            match def {
                graphql_parser::query::Definition::Operation(operation_definition) => {
                    match operation_definition {
                        graphql_parser::query::OperationDefinition::SelectionSet(selection_set) => {
                            for item in selection_set.items {
                                match item {

                                    graphql_parser::query::Selection::OptionalField(_field) => {
                                        unimplemented!()
                                    },

                                    graphql_parser::query::Selection::Field(_field) => {
                                        unimplemented!()
                                        
                                    },
                                    graphql_parser::query::Selection::FragmentSpread(_fragment_spread) => {
                                        unimplemented!()
                                        
                                    },
                                    graphql_parser::query::Selection::InlineFragment(_inline_fragment) => {
                                        unimplemented!()
                                        
                                    },
                                }
                            }
                        },
                        graphql_parser::query::OperationDefinition::Query(query) => {
                            err.error(BuildError::Unimplemented(query.position, "Query"));
                            // if let  Ok(query) = GenericOperation::from_query(err, query, schema) {
                            //     queries.push(query);
                            // }
                        },
                        graphql_parser::query::OperationDefinition::Mutation(mutation) => {
                            err.error(BuildError::Unimplemented(mutation.position, "Mutation"));
                            // if let Ok(mutation) = GenericOperation::from_mutation(err, mutation, schema) {
                            //     mutations.push(mutation);
                            // }
                        },
                        graphql_parser::query::OperationDefinition::Subscription(subscription) => {
                            err.error(BuildError::Unimplemented(subscription.position, "Subscription"));
                            
                        },
                    }
                },
                graphql_parser::query::Definition::Fragment(fragment_definition) => {
                    err.error(BuildError::Unimplemented(fragment_definition.position, "FragmentDefinition"));
                    // if let Ok(fragment) = FragmentDefinition::new(err, fragment_definition, schema) {
                    //     fragments.insert(fragment.name.clone(), fragment);
                    // }
                },
            }
        }

        ExecutableDocument {
            status: Status::Unchecked,
            name,
            fragments,
            queries,
            mutations,
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "ExecutableDocument {{")?;
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

    pub fn validate(&mut self, _err: &mut ErrorCollector) -> Result<(), Error> {
        self.status = Status::Validated;

        Ok(())
    }
    
    pub fn generate(&mut self, out: &mut Output, schema: &Schema) -> Result<(), Error> {
        writeln!(out, "pub mod {} {{", self.name)?;

        {
            let mut out = out.indent();
            for query in &self.queries {
                query.generate(&mut out, schema)?;
            }

            for mutation in &self.mutations {
                mutation.generate(&mut out, schema)?;
            }
        }
        writeln!(out, "}} // End of executable_document {}", self.name)?;
        self.status = Status::Generated;
        Ok(())
    }
}



#[derive(Debug)]
pub struct FragmentDefinition {}

#[derive(Debug)]
pub struct GenericOperation {}
impl GenericOperation {
    fn print(&self, out: &mut Output<'_>) -> std::io::Result<()>{
        todo!()
    }
    
    fn generate(&self, out: &mut Output<'_>, schema: &Schema) -> std::io::Result<()>{
        todo!()
    }
} 