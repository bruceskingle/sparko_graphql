use std::fmt::Display;
use std::io::Write;
use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::to_snake_case;

// use crate::utils::to_pascal_case;
use crate::BuildError;
use crate::BuildWarning;
use crate::Error;
use crate::ErrorCollector;
use crate::Output;
use crate::validated_model;

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

    pub fn name(&self) -> &str {
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
pub enum ScalarType<'a> {
    DefinedType{name: &'a str, position: &'a Pos},
    BuiltinType(BuiltinType),
}

impl<'a> ScalarType<'a> {
    fn new(v: &'a str, position: &'a Pos) -> ScalarType<'a> {
        if let Some(builtin_type) = BuiltinType::from_str(v) {
            ScalarType::BuiltinType(builtin_type)
        }
        else {
            ScalarType::DefinedType{ name: v, position: position}
        }
    }
}

impl Display for ScalarType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::DefinedType{name, position} => write!(f, "{} (defined at {})", name, position),
            ScalarType::BuiltinType(builtin_type) => builtin_type.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum Type<'a> {
    Scalar(ScalarType<'a>),
    Required(Box<Type<'a>>),
    Array(Box<Type<'a>>),
}

impl<'a> Type<'a> {
    fn new(parsed: &'a graphql_parser::schema::Type<'_, String>, position: &'a Pos) -> Type<'a> {
        match parsed {
            graphql_parser::query::Type::NamedType(name) => Type::Scalar(ScalarType::new(&name, position)),
            graphql_parser::query::Type::ListType(wrapped) => Type::Array(Box::new(Type::new(wrapped, position))),
            graphql_parser::query::Type::NonNullType(wrapped) => Type::Required(Box::new(Type::new(wrapped, position))),
        }
    }
}

impl Display for Type<'_> {
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
    use crate::tests::test_schema;

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
pub enum TypeDefinition<'a> {
    Scalar(Scalar<'a>),
    Object(Object<'a>),
    Interface(Interface<'a>),
    Union(Union<'a>),
    Enum(Enum<'a>),
}

impl<'a> TypeDefinition<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            TypeDefinition::Scalar(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            TypeDefinition::Interface(content) => content.print(out),
            TypeDefinition::Union(content) => content.print(out),
            TypeDefinition::Enum(content) => content.print(out),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            TypeDefinition::Scalar(content) => content.name,
            TypeDefinition::Object(content) => content.name,
            TypeDefinition::Interface(content) => content.name,
            TypeDefinition::Union(content) => content.name,
            TypeDefinition::Enum(content) => content.name,
        }
    }

    pub fn position(&self) -> &Pos {
        match self {
            TypeDefinition::Scalar(content) => content.position,
            TypeDefinition::Object(content) => content.position,
            TypeDefinition::Interface(content) => content.position,
            TypeDefinition::Union(content) => content.position,
            TypeDefinition::Enum(content) => content.position,
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Scalar(_) => "scalar",
            TypeDefinition::Object(_) => "object",
            TypeDefinition::Interface(_) => "interface",
            TypeDefinition::Union(_) => "union",
            TypeDefinition::Enum(_) => "enum",
        }
    }
}

#[derive(Debug)]
pub struct Scalar<'a> {
    pub position: &'a Pos,
    pub name: &'a str,
}

impl<'a> Scalar<'a> {
    fn new(scalar_type: &'a graphql_parser::schema::ScalarType<'_, String>) -> (&'a str, Scalar<'a>) {
        (&scalar_type.name, Scalar {
            position: &scalar_type.position,
            name: &scalar_type.name,
        })
    }
    
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "TypeDef {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub struct Object<'a> {
    pub position: &'a Pos,
    pub name: &'a str,
    pub fields: IndexMap<&'a str, Field<'a>>,
    pub implements: Option<&'a Vec<String>>,
    pub is_input: bool,
    // pub fully_implements: Vec<String>,
}

impl<'a> Object<'a> {
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

            if let Some(implements) = self.implements {
                writeln!(out, "implements {{")?;
                {
                    let mut out = out.indent();

                    for name in implements {
                        writeln!(out, "{}", name)?;
                    }
                }
                writeln!(out, "}}")?;
            }
        }
        writeln!(out, "}}")

        
    }

    pub fn from_object(_err: &mut ErrorCollector, object_type: &'a graphql_parser::schema::ObjectType<'a, String>) -> (&'a str, Self) {
        let mut fields: IndexMap<&str, Field<'_>> = IndexMap::new();

        for f in &object_type.fields {
            fields.insert(&f.name, Field::from_field(&f));
            
        }
        (&object_type.name, Object {
            position: &object_type.position,
            name: &object_type.name,
            fields,
            implements: Some(&object_type.implements_interfaces),
            is_input: false,
        })
    }
    
    fn from_input_object(_err: &mut ErrorCollector<'_>, object_type: &'a graphql_parser::schema::InputObjectType<'a, String>) -> (&'a str, Self) {
        let mut field_names = Vec::new();
        let mut fields: IndexMap<&str, Field<'_>> = IndexMap::new();

        for f in &object_type.fields {
            field_names.push(f.name.clone());
            fields.insert(&f.name, Field::from_input_value(&f));
            
        }
        (&object_type.name, Object {
            position: &object_type.position,
            name: &object_type.name,
            fields,
            implements: None,
            is_input: true,
        })
    }
}


#[derive(Debug)]
pub struct Interface<'a> {
    pub position: &'a Pos,
    pub name: &'a str,
    pub fields: IndexMap<String, Field<'a>>,
    pub implements: &'a Vec<String>,
}

impl<'a> Interface<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Interface {{")?;
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

                for name in self.implements {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    pub fn new(_out: &mut ErrorCollector, interface_type: &'a graphql_parser::schema::InterfaceType<'a, String>) -> (&'a str, Self) {
        let mut fields = IndexMap::new();

        for f in &interface_type.fields {
            fields.insert(f.name.clone(), Field::from_field(&f));
            
        }
        (&interface_type.name, Interface {
            position: &interface_type.position,
            name: &interface_type.name,
            fields,
            implements: &interface_type.implements_interfaces,
        })
    }
}

#[derive(Debug)]
pub struct Union<'a> {
    pub position: &'a Pos,
    pub name: &'a str,
    pub types: &'a Vec<String>,
}

impl<'a> Union<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Union {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "implements {{")?;
            {
                let mut out = out.indent();

                for name in self.types {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    fn new(_err: &mut ErrorCollector, union_type: &'a graphql_parser::schema::UnionType<'_, String>) -> (&'a str, Self) {
        ( &union_type.name, Union {
            position: &union_type.position,
            name: &union_type.name,
            types: &union_type.types,
        })
    }
}

#[derive(Debug)]
pub struct Enum<'a> {
    pub position: &'a Pos,
    pub name: &'a str,
    pub variants: &'a Vec<graphql_parser::schema::EnumValue<'a, std::string::String, >>,
}

impl<'a> Enum<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Enum {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "variants {{")?;
            {
                let mut out = out.indent();

                for variant in self.variants {
                    writeln!(out, "{}", variant.name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn new(_err: &mut ErrorCollector, enum_type: &'a graphql_parser::schema::EnumType<'a, String>) -> (&'a str, Self) {
        (&enum_type.name, Enum {
            position: &enum_type.position,
            name: &enum_type.name,
            variants: &enum_type.values,
        })
    }
}

#[derive(Debug)]
pub struct Field<'a> {
    pub position:&'a Pos,
    pub name: &'a str,
    pub ty: Type<'a>,
    // pub nonnull: bool,
    // pub multiple: bool,
}

impl<'a> Field<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "ty:        {}", self.ty)?;
            // writeln!(out, "nonnull:   {}", self.nonnull)?;
            // writeln!(out, "multiple:  {}", self.multiple)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }

    fn do_new(position: &'a Pos, name: &'a str, field_type: &'a graphql_parser::schema::Type<'a, String>) -> Field<'a> {
        let ty = Type::new(field_type, &position);
        Field {
            position,
            name,
            ty,
        }
        
     }

     pub fn from_input_value(field: &'a graphql_parser::schema::InputValue<'a, String>) -> Self {
         Self::do_new(&field.position, &field.name, &field.value_type)
     }

     pub fn from_field(field: &'a graphql_parser::schema::Field<'a, String>) -> Self {
         Self::do_new(&field.position, &field.name, &field.field_type)
     }
    
    fn from_variable(variable: &'a graphql_parser::query::VariableDefinition<'a, String>) -> Field<'a> {
        Self::do_new(&variable.position, &variable.name, &variable.var_type)
    }
}

#[derive(Debug)]
pub struct SchemaDefinition<'a> {
    pub position: &'a Pos,
    pub query: &'a Option<String>,
    pub mutation: &'a Option<String>,
    pub subscription: &'a Option<String>,
}

impl<'a> SchemaDefinition<'a> {
    fn new(_err: &mut ErrorCollector, schema_definition: &'a graphql_parser::schema::SchemaDefinition<'a, String>) -> SchemaDefinition<'a> {
        SchemaDefinition {
            position: &schema_definition.position,
            query: &schema_definition.query,
            mutation: &schema_definition.mutation,
            subscription: &schema_definition.subscription,
        }
    }
}

#[derive(Debug)]
pub struct Schema<'a> {
    pub named_types: IndexMap<&'a str, TypeDefinition<'a>>,
    pub schema_definition: Option<SchemaDefinition<'a>>,
}

impl Schema<'_> {

    // fn name_defined(&mut self, out: &mut ErrorCollector, name: &String, position: Pos) {
    //     if let Some(existing) = self.names.insert(name.clone(), position.clone()) {
    //         out.error(BuildError>::DuplicateName(existing, position, name.clone()));
    //     }
    // }



    // fn new_global(&mut self, err: &mut ErrorCollector, name: String, value: TypeDefinition) {
    //     let position = value.position().clone();

    //     if let Some(existing) = self.named_types.insert(name.clone(), value) {
    //         err.error(BuildError::DuplicateName(existing.position().clone(), position, name));
    //     }
    // }

    pub fn new<'a>(err: &mut ErrorCollector, ast: &'a graphql_parser::schema::Document<'a, String>) -> Result<Schema<'a>, Error> {
        let mut model = Schema {
            named_types: IndexMap::new(),
            schema_definition: None,
        };

        for def in &ast.definitions {
            match def {
                graphql_parser::schema::Definition::SchemaDefinition(schema_definition) => {
                    if let Some(existing) = &model.schema_definition {
                        err.error(BuildError::DuplicateName(schema_definition.position, existing.position.clone(), "Schema".to_string()));
                    }
                    else {
                        model.schema_definition = Some(SchemaDefinition::new(err, schema_definition));
                    }
                },
                graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                    let (name, value) = match type_definition {
                        graphql_parser::schema::TypeDefinition::Scalar(scalar_type) => {
                            let (name, content) = Scalar::new(scalar_type);
                            (name, TypeDefinition::Scalar(content))
                        },
                        graphql_parser::schema::TypeDefinition::Object(ast) => {
                            // err.fail(BuildError::Unimplemented(ast.position.clone(), "Object"))
                            let (name, content) = Object::from_object(err, ast);
                            (name, TypeDefinition::Object(content))
                        },
                        graphql_parser::schema::TypeDefinition::Interface(ast) => {
                            // err.fail(BuildError::Unimplemented(ast.position.clone(), "Interface"))
                            let (name, content) = Interface::new(err, ast);
                            (name, TypeDefinition::Interface(content))
                        },
                        graphql_parser::schema::TypeDefinition::Union(ast) => {
                            let (name, content) = Union::new(err, ast);
                            (name, TypeDefinition::Union(content))
                        },
                        graphql_parser::schema::TypeDefinition::Enum(ast) => {
                            let (name, content) = Enum::new(err, ast);
                            (name, TypeDefinition::Enum(content))
                        },
                        graphql_parser::schema::TypeDefinition::InputObject(ast) => {
                            let (name, content) = Object::from_input_object(err, ast);
                            (name, TypeDefinition::Object(content))
                        },
                    };
                    let position = value.position().clone();

                    if let Some(existing) = model.named_types.insert(name, value) {
                        err.error(BuildError::DuplicateName(existing.position().clone(), position, existing.name().to_string()));
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

        err.ok(model)
        // Ok(model)
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "parsed_model::Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "named_types {{")?;
            {
                let mut out = out.indent();

                for (name, ty) in &self.named_types {
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
pub enum Selection<'a> {
    Field(SelectionField<'a>),
    Fragment(FragmentSpread<'a>),
    // InlineFragment(InlineFragment),
} 

impl<'a> Selection<'a>{
    pub fn new(err: &mut ErrorCollector, selection: &'a graphql_parser::query::Selection<'a, String> ) -> Selection<'a> {
        match selection {
            graphql_parser::query::Selection::OptionalField(field) => 
                Selection::Field(SelectionField::new(err, field, true)),
            graphql_parser::query::Selection::Field(field) => Selection::Field(SelectionField::new(err, field, false)),
            graphql_parser::query::Selection::FragmentSpread(fragment) => Selection::Fragment(FragmentSpread::new(err, &fragment)),
            graphql_parser::query::Selection::InlineFragment(_inline_fragment) => todo!(),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::Field(selection_field) => selection_field.print(out),
            Selection::Fragment(selection_fragment) => selection_fragment.print(out),
        }
    }

    // pub fn position(&self) -> Pos {
    //     match self {
    //         Selection::Field(selection_field) => selection_field.position,
    //     }
    // }
}
#[derive(Debug)]
pub struct SelectionField<'a> {
    pub name: &'a str,
    pub alias: &'a Option<String>,
    pub position: &'a Pos,
    pub optional: bool,
    pub arguments: Vec<Argument<'a>>,
    pub selections: Vec<Selection<'a>>,
}

impl<'a> SelectionField<'a> {
    pub fn new(out: &mut ErrorCollector, field: &'a graphql_parser::query::Field<'a, String>, optional: bool) -> Self {
        SelectionField {
            position: &field.position,
            name: &field.name,
            alias: &field.alias,
            optional,
            arguments: build_arguments(out, &field.arguments),
            selections: build_selections(out, &field.selection_set),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionField {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "optional:  {}", self.optional)?;
            writeln!(out, "arguments {{")?;
            {
                let mut out = out.indent();

                for argument in &self.arguments {
                    argument.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "selections {{")?;
            {
                let mut out = out.indent();

                for selection in &self.selections {
                    selection.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub enum OperationType {
    Query,
    Mutation
}

impl Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "Query"),
            OperationType::Mutation => write!(f, "Mutation"),
        }
    }
}

impl OperationType {
    pub fn to_lower_case(&self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
        }
    }


    pub fn to_upper_case(&self) -> &'static str {
        match self {
            OperationType::Query => "QUERY",
            OperationType::Mutation => "MUTATION",
        }
    }
}

#[derive(Debug)]
pub struct GenericOperation<'a> {
    pub name: &'a str,
    pub position: &'a Pos,
    pub operation: OperationType,
    pub selections: Vec<Selection<'a>>,
    pub variables: Vec<Field<'a>>,
}

impl<'a> GenericOperation<'a> {
    pub fn from_query(err: &mut ErrorCollector, query: &'a graphql_parser::query::Query<'a, String>, schema: &validated_model::Schema ) -> Result<GenericOperation<'a>, Error> {
        Self::new(err, OperationType::Query, &query.name, &query.position, &query.selection_set, &query.variable_definitions, schema)
    }

    pub fn from_mutation(err: &mut ErrorCollector, query: &'a graphql_parser::query::Mutation<'a, String>, schema: &validated_model::Schema ) -> Result<GenericOperation<'a>, Error> {
        Self::new(err, OperationType::Query, &query.name, &query.position, &query.selection_set, &query.variable_definitions, schema)
    }

    pub fn new(err: &mut ErrorCollector, operation: OperationType, name: &'a Option<String>, position: &'a Pos, selection_set: &'a graphql_parser::query::SelectionSet<'a, String>, 
        variable_definitions: &'a Vec<graphql_parser::query::VariableDefinition<'a, String>>, _schema: &validated_model::Schema ) -> Result<GenericOperation<'a>, Error> {
        let name = match name {
            Some(name) => name,
            None => {
                return err.fail(BuildError::UnsupportedError(position.clone(), String::from("Anonymous query")))
            },
        };

        let selections = build_selections(err, selection_set);
        let variables = build_variables(err, variable_definitions);

        if selections.is_empty() {
            return err.fail(BuildError::InvalidQueryError(position.clone(), format!("Query {} has no selection set", name)));
        }

        // for selection in selections {
        //    let query_object = if let SelectionModel::Field(selection_field) = selection {

        //         let query_object = if let Some(query_schema) = schema.query {
        //             if let Some(query_schema_object) = schema.objects.get(&query_schema) {
        //                 query_schema_object.fields.get(&selection_field.name)
        //             } else {None}
        //         } else {None};

        //         if let Some(query_object) = query_object {
        //             query_object
        //         }
        //         else {
        //             return Err(BuildError>::MissingObjectError(selection.position(), format!("Query {} has mising selection \"{}\"", name, selection_field.name)));
        //         }
        //     }
        //     else {
        //         return Err(BuildError>::InvalidQueryError(selection.position(), format!("Query {} has invalid selection set, expected a field name", name)));
        //     };
        // }
        

        Ok(GenericOperation {
            name,
            position,
            operation,
            selections,
            variables,
            // query_object,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{} {{", &self.operation)?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "selections {{")?;
            {
                let mut out = out.indent();

                for selection in &self.selections {
                    selection.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "variables {{")?;
            {
                let mut out = out.indent();

                for variable in &self.variables {
                    variable.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}



fn build_variables<'a>(_out: &mut ErrorCollector, variable_definitions: &'a Vec<graphql_parser::query::VariableDefinition<'a, String>>) -> Vec<Field<'a>> {
    let mut variables = Vec::new();

    for variable in variable_definitions {
        variables.push(Field::from_variable(variable));
    }

    variables
}

fn build_selections<'a>(out: &mut ErrorCollector, selection_set: &'a graphql_parser::query::SelectionSet<'a, String>) -> Vec<Selection<'a>> {

    let mut selections = Vec::new();

    for selection in &selection_set.items {
        selections.push(Selection::new(out, selection));
    }

    selections
}



fn build_arguments<'a>(out: &mut ErrorCollector, arguments: &'a Vec<(String, graphql_parser::query::Value<'a, String>)>) -> Vec<Argument<'a>> {
    let mut result = Vec::new();

    for (name, value) in arguments {
        result.push(Argument::new(out, name, value));
    }

    result
}

#[derive(Debug)]
pub enum Value<'a> {
    Variable(&'a str),
    Int(i64),
    // Float(f64),
    String(&'a str),
    // Boolean(bool),
    // Null,
    // Enum,
    // List,
    // Object,
}

impl<'a> Value<'a> {
    pub fn new(_out: &mut ErrorCollector, value: &'a graphql_parser::query::Value<'a, String> ) -> Self {
        match value {
            graphql_parser::query::Value::Variable(name) => Value::Variable(name),
            graphql_parser::query::Value::Int(number) => Value::Int(number.as_i64().unwrap()),
            graphql_parser::query::Value::Float(_) => todo!(),
            graphql_parser::query::Value::String(string) => Value::String(string),
            graphql_parser::query::Value::Boolean(_) => todo!(),
            graphql_parser::query::Value::Null => todo!(),
            graphql_parser::query::Value::Enum(_) => todo!(),
            graphql_parser::query::Value::List(_) => todo!(),
            graphql_parser::query::Value::Object(_) => todo!(),
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Variable(name) => write!(f, "${}", name),
            Value::Int(number) => write!(f, "{}", number),
            // Value::Float(_) => todo!(),
            Value::String(string) => write!(f, "\"{}\"", string),
            // Value::Boolean(_) => todo!(),
            // Value::Null => todo!(),
            // Value::Enum => todo!(),
            // Value::List => todo!(),
            // Value::Object => todo!(),
        }
    }
}


#[derive(Debug)]
pub struct Argument<'a> {
    pub name: &'a str,
    pub value: Value<'a>,
}

impl<'a> Argument<'a> {
    pub fn new(out: &mut ErrorCollector, name: &'a str, value: &'a graphql_parser::query::Value<'a, String> ) -> Self {
        Argument {
            name,
            value: Value::new(out, value),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Argument {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "value:     {}", self.value)?;
        }
        writeln!(out, "}}")
    }
    
    pub(crate) fn generate_query(&self, out: &mut Output<'_>, fields: validated_model::FieldMapRef<'a>, schema: &validated_model::Schema, arg: bool) -> Result<(), Error> {
        writeln!(out, "{}: {}", self.name, self.value)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FragmentDefinition<'a> {
    pub position: &'a Pos,
    pub name: &'a String,
    pub selections: Vec<Selection<'a>>,
    pub type_condition: &'a str,
}

impl<'a> FragmentDefinition<'a>{
    pub fn new(err: &mut ErrorCollector, fragment_definition: &'a graphql_parser::query::FragmentDefinition<'a, String>, _schema: &validated_model::Schema ) -> Result<Self, Error> {

        let selections = build_selections(err, &fragment_definition.selection_set);

        if selections.is_empty() {
            return err.fail(BuildError::InvalidQueryError(fragment_definition.position, format!("FragmentDefinition {} has no selection set", &fragment_definition.name)));
        }

        let type_condition = match &fragment_definition.type_condition {
            graphql_parser::query::TypeCondition::On(value) => value,
        };


        // for selection in selections {
        //    let query_object = if let SelectionModel::Field(selection_field) = selection {

        //         let query_object = if let Some(query_schema) = schema.query {
        //             if let Some(query_schema_object) = schema.objects.get(&query_schema) {
        //                 query_schema_object.fields.get(&selection_field.name)
        //             } else {None}
        //         } else {None};

        //         if let Some(query_object) = query_object {
        //             query_object
        //         }
        //         else {
        //             return Err(BuildError>::MissingObjectError(selection.position(), format!("Query {} has mising selection \"{}\"", name, selection_field.name)));
        //         }
        //     }
        //     else {
        //         return Err(BuildError>::InvalidQueryError(selection.position(), format!("Query {} has invalid selection set, expected a field name", name)));
        //     };
        // }
        

        Ok(FragmentDefinition {
            position: &fragment_definition.position,
            name: &fragment_definition.name,
            selections,
            type_condition,
            // query_object,
        })
    }
    
    // pub fn validate(&mut self, _out: &mut ErrorCollector) -> Result<(), Error> {
    //     Ok(())
    // }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Query {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:        {}", self.position)?;
            writeln!(out, "name:            {}", self.name)?;
            writeln!(out, "type_condition:  {}", self.type_condition)?;
            writeln!(out, "selections {{")?;
            {
                let mut out = out.indent();

                for selection in &self.selections {
                    selection.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub struct FragmentSpread<'a> {
    pub name: &'a str,
    pub position: &'a Pos,
}

impl<'a> FragmentSpread<'a> {
    pub fn new(out: &mut ErrorCollector, fragment: &'a graphql_parser::query::FragmentSpread<'_, String>) -> Self {
        FragmentSpread {
            position: &fragment.position,
            name: &fragment.fragment_name,
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "FragmentSpread {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub struct ExecutableDocument<'a> {
    pub name: String,
    pub fragments: IndexMap<String, FragmentDefinition<'a>>,
    pub queries: Vec<GenericOperation<'a>>,
    pub mutations: Vec<GenericOperation<'a>>
}

impl<'a> ExecutableDocument<'a> {
    pub fn new(err: &mut ErrorCollector,
        definitions: &'a Vec<graphql_parser::query::Definition<'a, String>>, model_name: &str,
        schema: &'a validated_model::Schema) -> Self {

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
                            for item in &selection_set.items {
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
                            if let  Ok(query) = GenericOperation::from_query(err, &query, schema) {
                                queries.push(query);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Mutation(mutation) => {
                            if let Ok(mutation) = GenericOperation::from_mutation(err, &mutation, schema) {
                                mutations.push(mutation);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Subscription(_subscription) => {
                            unimplemented!()
                            
                        },
                    }
                },
                graphql_parser::query::Definition::Fragment(fragment_definition) => {
                    if let Ok(fragment) = FragmentDefinition::new(err, &fragment_definition, schema) {
                        fragments.insert(fragment.name.clone(), fragment);
                    }
                },
            }
        }

        ExecutableDocument {
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
}


// Unordered --------------------------------------------------------------------------------------------------------------------------------------------


// #[derive(Debug)]
// pub struct DefinedType {
//     pub name: String,
//     pub nonnull: bool,
//     pub multiple: bool,
// }

// impl DefinedType {

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "Field {{")?;
//         {
//             let mut out = out.indent();

//             writeln!(out, "name:      {}", self.name)?;
//             writeln!(out, "}}")?;
//         }
//         writeln!(out, "}}")

        
//     }
// }

// #[derive(Debug)]
// pub struct BuiltinType {
//     pub nonnull: bool,
//     pub multiple: bool,
// }

// impl BuiltinType {

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "Field {{")?;
//         {
//             let mut out = out.indent();

//             writeln!(out, "nonnull:   {}", self.nonnull)?;
//             writeln!(out, "multiple:  {}", self.multiple)?;
//             writeln!(out, "}}")?;
//         }
//         writeln!(out, "}}")

        
//     }
// }








// #[derive(Debug)]
// struct ObjectReferenceList<'a> {
//     schema: &'a Schema,
//     names: Vec<String>,
// }

// impl<'a> IntoIterator for &'a ObjectReferenceList<'a> {
//     type Item = &'a Object;
//     type IntoIter = ObjectReferenceListItertor<'a>;
    
//     fn into_iter(self) -> Self::IntoIter {
//         ObjectReferenceListItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// struct ObjectReferenceListItertor<'a> {
//     list: &'a ObjectReferenceList<'a>,
//     // iter: std::slice::Iter<'a, String>,
//     index: usize,
// }

// impl<'a> Iterator for ObjectReferenceListItertor<'a> {
//     type Item = &'a Object;

//     fn next(&mut self) -> Option<Self::Item> {        
//         match self.list.names.get(self.index) {
//             Some(name) => {
//                 self.index += 1;
//                 match self.list.schema.named_types.get(name) {
//                     Some(type_definition) => {
//                         if let TypeDefinition::Object(object) = type_definition {
//                             Some(object)
//                         }
//                         else {
//                             panic!("Object iterator expected Object for \"{}\" but found {}", name, type_definition.type_name())
//                         }
//                     },
//                     None => panic!("Object iterator failed to find \"{}\"", name),
//                 }
//             },
//             None => None,
//         }
//     }
// }


// #[derive(Debug)]
// pub struct InterfaceReferenceList {
//     pub schema: Rc<Schema>,
//     pub names: Vec<String>,
// }

// impl<'a> IntoIterator for &'a InterfaceReferenceList {
//     type Item = &'a Interface;
//     type IntoIter = InterfaceReferenceListItertor<'a>;
    
//     fn into_iter(self) -> Self::IntoIter {
//         InterfaceReferenceListItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// pub struct InterfaceReferenceListItertor<'a> {
//     list: &'a InterfaceReferenceList,
//     // iter: std::slice::Iter<'a, String>,
//     index: usize,
// }

// impl<'a> Iterator for InterfaceReferenceListItertor<'a> {
//     type Item = &'a Interface;

//     fn next(&mut self) -> Option<Self::Item> {        
//         match self.list.names.get(self.index) {
//             Some(name) => {
//                 self.index += 1;
//                 match self.list.schema.named_types.get(name) {
//                     Some(type_definition) => {
//                         if let TypeDefinition::Interface(object) = type_definition {
//                             Some(object)
//                         }
//                         else {
//                             panic!("Interface iterator expected Interface for \"{}\" but found {}", name, type_definition.type_name())
//                         }
//                     },
//                     None => panic!("Interface iterator failed to find \"{}\"", name),
//                 }
//             },
//             None => None,
//         }
//     }
// }


// fn visit_type(field_type: &graphql_parser::query::Type<'_, String>, non_null: bool) -> String {
   
//    match field_type {
//         graphql_parser::query::Type::NamedType(v) => {
//             let t = match v as &str {
//                 "Boolean" => String::from("bool"),
//                 "Date" => String::from("time::Date"),
//                 "DateTime" => String::from("time::OffsetDateTime"),
//                 "Float" => String::from("f64"),
//                 "ID" => String::from("String"),
//                 "Int" => String::from("i32"),
//                 "String" => String::from("String"),

//                 _ => to_pascal_case(v),
//             };

//             if non_null {
//                 t
//             }
//             else {
//                 format!("Option<{}>", t)
//             }
//         },
//         graphql_parser::query::Type::ListType(t) => {
//             format!("Vec<{}>", visit_type(&*t, non_null))
//         },
//         graphql_parser::query::Type::NonNullType(t) => {
//             visit_type(&*t, true)
//         },
//     }
// }


// #[derive(Debug)]
// pub struct VariableDefinition {
//     pub position: Pos,
//     pub name: String,
//     pub ty: Type,
//     pub optional: bool,
//     pub multiple: bool,
//     // pub rust_type: String,
//     // pub var_type: Type<'a, T>,
//     // pub default_value: Option<Value<'a, T>>,
// }

// impl VariableDefinition {

//     fn do_new(position: Pos, name: String, field_type: graphql_parser::query::Type<'_, String>, optional: bool, multiple: bool) -> VariableDefinition {
//         match field_type {
//             graphql_parser::query::Type::NamedType(v) => {
//                 let ty = match &v as &str {
//                     "Boolean" => Type::Boolean,
//                     "Float" => Type::Float,
//                     "ID" => Type::ID,
//                     "Int" => Type::Int,
//                     "String" => Type::String,

//                     _ => Type::DefinedType(v.clone()),
//                 };

//                 VariableDefinition {
//                     position,
//                     name,
//                     ty,
//                     optional,
//                     multiple,
//                 }
//             },
//             graphql_parser::query::Type::ListType(t) => {
//                 Self::do_new(position, name, *t, optional, true)
//             },
//             graphql_parser::query::Type::NonNullType(t) => {
//                 Self::do_new(position, name, *t, true, multiple)
//             },
//          }
//      }


//     pub fn new(out: &mut ErrorCollector, f: graphql_parser::query::VariableDefinition<'_, String> ) -> VariableDefinition {
//         Self::do_new(f.position, f.name, f.field_type, false, false)
        
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "VariableDefinition {{")?;
//         {
//             let mut out = out.indent();

//             writeln!(out, "position:  {}", &self.position)?;
//             writeln!(out, "name:      {}", &self.name)?;
//             writeln!(out, "type:      {}", &self.ty)?;
//             writeln!(out, "optional:  {}", self.optional)?;
//             writeln!(out, "multiple:  {}", self.multiple)?;
//         }
//         writeln!(out, "}}")
//     }
// }



