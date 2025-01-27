use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;
use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::to_snake_case;

use crate::intern;
use crate::intern_option;
use crate::intern_vec;
use crate::Atom;
// use crate::utils::to_pascal_case;
use crate::BuildError;
use crate::BuildWarning;
use crate::Error;
use crate::ErrorCollector;
use crate::Output;
use crate::validated_model;


pub type FieldMap = IndexMap<Atom, Rc<Field>>;

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
pub struct DefinedType {
    pub name: Atom,
    pub position: Pos
}

impl DefinedType {
    fn new(v: String, position: Pos) -> ScalarType {
        ScalarType::DefinedType(Rc::new(DefinedType{
            name: intern(v),
            position
        }))
    }
}

#[derive(Debug)]
pub enum ScalarType {
    DefinedType(Rc<DefinedType>),
    BuiltinType(BuiltinType),
}

impl ScalarType {
    fn new(v: String, position: Pos) -> ScalarType {
        if let Some(builtin_type) = BuiltinType::from_str(&v) {
            ScalarType::BuiltinType(builtin_type)
        }
        else {
            DefinedType::new(v, position)
        }
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::DefinedType(defined_type) => write!(f, "{} (referenced at {})", &defined_type.name, &defined_type.position),
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
    fn new(parsed: graphql_parser::schema::Type<'_, String>, position: Pos) -> Type {
        match parsed {
            graphql_parser::query::Type::NamedType(name) => Type::Scalar(ScalarType::new(name, position)),
            graphql_parser::query::Type::ListType(wrapped) => Type::Array(Box::new(Type::new(*wrapped, position))),
            graphql_parser::query::Type::NonNullType(wrapped) => Type::Required(Box::new(Type::new(*wrapped, position))),
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

#[derive(Debug, PartialEq, Eq)]
pub enum DefinedTypeName {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
}

impl DefinedTypeName {
    pub fn name(&self) -> &'static str {
        match self {
            DefinedTypeName::Scalar => "Scalar",
            DefinedTypeName::Object => "Object",
            DefinedTypeName::Interface => "Interface",
            DefinedTypeName::Union => "Union",
            DefinedTypeName::Enum => "Enum",
            DefinedTypeName::InputObject => "InputObject",
        }
    }
}

#[derive(Debug)]
pub enum TypeDefinition {
    Scalar(Rc<Scalar>),
    Object(Rc<Object>),
    Interface(Rc<Interface>),
    Union(Rc<Union>),
    Enum(Rc<Enum>),
    InputObject(Rc<Object>),
}

impl TypeDefinition {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            TypeDefinition::Scalar(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            TypeDefinition::Interface(content) => content.print(out),
            TypeDefinition::Union(content) => content.print(out),
            TypeDefinition::Enum(content) => content.print(out),
            TypeDefinition::InputObject(content) => content.print(out),
        }
    }

    pub fn name(&self) -> &Atom {
        match self {
            TypeDefinition::Scalar(content) => &content.name,
            TypeDefinition::Object(content) => &content.name,
            TypeDefinition::Interface(content) => &content.name,
            TypeDefinition::Union(content) => &content.name,
            TypeDefinition::Enum(content) => &content.name,
            TypeDefinition::InputObject(content) => &content.name,
        }
    }

    pub fn position(&self) -> &Pos {
        match self {
            TypeDefinition::Scalar(content) => &content.position,
            TypeDefinition::Object(content) => &content.position,
            TypeDefinition::Interface(content) => &content.position,
            TypeDefinition::Union(content) => &content.position,
            TypeDefinition::Enum(content) => &content.position,
            TypeDefinition::InputObject(content) => &content.position,
        }
    }

    pub fn defined_type_name(&self) -> DefinedTypeName {
        match self {
            TypeDefinition::Scalar(_) => DefinedTypeName::Scalar,
            TypeDefinition::Object(_) => DefinedTypeName::Object,
            TypeDefinition::Interface(_) => DefinedTypeName::Interface,
            TypeDefinition::Union(_) => DefinedTypeName::Union,
            TypeDefinition::Enum(_) => DefinedTypeName::Enum,
            TypeDefinition::InputObject(_) => DefinedTypeName::InputObject,
        }
    }

    pub fn type_name(&self) -> &str {
        self.defined_type_name().name()
    }
}

#[derive(Debug)]
pub struct Scalar {
    pub position: Pos,
    pub name: Atom,
}

impl Scalar {
    fn new(scalar_type: graphql_parser::schema::ScalarType<'_, String>) -> TypeDefinition {
        TypeDefinition::Scalar(Rc::new(Scalar {
            position: scalar_type.position,
            name: intern(scalar_type.name),
        }))
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
pub struct Object {
    pub position: Pos,
    pub name: Atom,
    pub fields: FieldMap,
    pub implements: Vec<Atom>,
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

            if !self.implements.is_empty() {
                writeln!(out, "implements {{")?;
                {
                    let mut out = out.indent();

                    for name in &self.implements {
                        writeln!(out, "{}", name)?;
                    }
                }
                writeln!(out, "}}")?;
            }
        }
        writeln!(out, "}}")

        
    }

    pub fn from_object(_err: &mut ErrorCollector, object_type: graphql_parser::schema::ObjectType<'_, String>) -> TypeDefinition {
        let mut fields: FieldMap = IndexMap::new();

        for f in object_type.fields {
            let value = Field::from_field(f);
            fields.insert(value.name.clone(), value);
        }

        TypeDefinition::Object(Rc::new(Object {
            position: object_type.position,
            name: intern(object_type.name),
            fields,
            implements: intern_vec(object_type.implements_interfaces),
            is_input: false,
        }))
    }
    
    fn from_input_object(_err: &mut ErrorCollector<'_>, object_type: graphql_parser::schema::InputObjectType<'_, String>) -> TypeDefinition {
        let mut fields = IndexMap::new();

        for f in object_type.fields {
            let value = Field::from_input_value(f);
            fields.insert(value.name.clone(), value);
        }

        TypeDefinition::InputObject(Rc::new(Object {
            position: object_type.position,
            name: intern(object_type.name),
            fields,
            implements: Vec::new(),
            is_input: true,
        }))
    }
}


#[derive(Debug)]
pub struct Interface {
    pub position: Pos,
    pub name: Atom,
    pub fields: FieldMap,
    pub implements: Vec<Atom>,
}

impl Interface {
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

                for name in &self.implements {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    pub fn new(_out: &mut ErrorCollector, interface_type: graphql_parser::schema::InterfaceType<'_, String>) -> TypeDefinition {
        let mut fields = IndexMap::new();

        for f in interface_type.fields {
            let value = Field::from_field(f);
            fields.insert(value.name.clone(), value);
            
        }

        TypeDefinition::Interface(Rc::new(Interface {
            position: interface_type.position,
            name: intern(interface_type.name),
            fields,
            implements: intern_vec(interface_type.implements_interfaces),
        }))
    }
}

#[derive(Debug)]
pub struct Union {
    pub position: Pos,
    pub name: Atom,
    pub types: Vec<Atom>,
}

impl Union {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Union {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "types {{")?;
            {
                let mut out = out.indent();

                for name in &self.types {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    fn new(_err: &mut ErrorCollector, union_type: graphql_parser::schema::UnionType<'_, String>) -> TypeDefinition {
        TypeDefinition::Union(Rc::new(Union {
            position: union_type.position,
            name: intern(union_type.name),
            types: intern_vec(union_type.types),
        }))
    }
}

#[derive(Debug)]
pub struct Variant {
    pub name: Atom,
    pub position: Pos,
    pub description: Option<String>,
}

impl Variant {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Variant {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "description:  {:?}", self.description)?;
        }
        writeln!(out, "}}")
    }

    pub fn new(_err: &mut ErrorCollector, enum_type: graphql_parser::schema::EnumValue<'_, String>) -> Self {
        Variant {
            name: intern(enum_type.name),
            position: enum_type.position,
            description: enum_type.description,
        }
    }
    
    fn new_vec(err: &mut ErrorCollector, values: Vec<graphql_parser::schema::EnumValue<'_, String>>) -> Vec<Variant> {
        let mut result =  Vec::new();

        for value in values {
            result.push(Self::new(err, value));
        }

        result
    }
}

#[derive(Debug)]
pub struct Enum {
    pub name: Atom,
    pub position: Pos,
    pub variants: Vec<Variant>,
}

impl Enum {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Enum {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "variants {{")?;
            {
                let mut out = out.indent();

                for variant in &self.variants {
                    writeln!(out, "{}", variant.name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn new(err: &mut ErrorCollector, enum_type: graphql_parser::schema::EnumType<'_, String>) -> TypeDefinition {
        TypeDefinition::Enum(Rc::new(Enum {
            name: intern(enum_type.name),
            position: enum_type.position,
            variants: Variant::new_vec(err, enum_type.values),
        }))
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: Atom,
    pub position:Pos,
    pub ty: Type,
}

impl Field {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "ty:        {}", self.ty)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }

    fn do_new(position: Pos, name: String, field_type: graphql_parser::schema::Type<'_, String>) -> Rc<Field> {
        let ty = Type::new(field_type, position.clone());
        Rc::new(Field {
            position,
            name: intern(name),
            ty,
        })
        
     }

     pub fn from_input_value(field: graphql_parser::schema::InputValue<'_, String>) -> Rc<Field> {
         Self::do_new(field.position, field.name, field.value_type)
     }

     pub fn from_field(field: graphql_parser::schema::Field<'_, String>) -> Rc<Field> {
         Self::do_new(field.position, field.name, field.field_type)
     }
    
    fn from_variable(variable: graphql_parser::query::VariableDefinition<'_, String>) -> Rc<Field> {
        Self::do_new(variable.position, variable.name, variable.var_type)
    }
}

#[derive(Debug)]
pub struct SchemaDefinition {
    pub position: Pos,
    pub query: Option<Atom>,
    pub mutation: Option<Atom>,
    pub subscription: Option<Atom>,
}

impl SchemaDefinition {
    fn new(_err: &mut ErrorCollector, schema_definition: graphql_parser::schema::SchemaDefinition<'_, String>) -> Self {
        SchemaDefinition {
            position: schema_definition.position,
            query: intern_option(schema_definition.query),
            mutation: intern_option(schema_definition.mutation),
            subscription: intern_option(schema_definition.subscription),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub named_types: IndexMap<Atom, TypeDefinition>,
    pub schema_definition: Option<SchemaDefinition>,
}

impl Schema {

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

    pub fn new(err: &mut ErrorCollector, ast: graphql_parser::schema::Document<'_, String>) -> Result<Rc<Self>, Error> {
        let mut model = Schema {
            named_types: IndexMap::new(),
            schema_definition: None,
        };

        for def in ast.definitions {
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
                    let value = match type_definition {
                        graphql_parser::schema::TypeDefinition::Scalar(scalar_type) => Scalar::new(scalar_type),
                        graphql_parser::schema::TypeDefinition::Object(ast) => Object::from_object(err, ast),
                        graphql_parser::schema::TypeDefinition::Interface(ast) => Interface::new(err, ast),
                        graphql_parser::schema::TypeDefinition::Union(ast) => Union::new(err, ast),
                        graphql_parser::schema::TypeDefinition::Enum(ast) => Enum::new(err, ast),
                        graphql_parser::schema::TypeDefinition::InputObject(ast) => Object::from_input_object(err, ast),
                    };
                    let position = value.position().clone(); // TODO: Can we remove this?

                    if let Some(existing) = model.named_types.insert(value.name().clone(), value) {
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

        err.ok(Rc::new(model))
        // Ok(model)
    }

    // pub fn get<T>(&self, name: &Atom, defined_type_name: DefinedTypeName) -> Result<T, Error> {
    //     if let Some(type_definition) = self.named_types.get(name) {
    //         if type_definition.defined_type_name() == defined_type_name {
    //             Ok(type_definition)
    //         }
    //         else {
    //             Err(Error::BuildFailed(format!("Expected {} for \"{}\" but found {}", defined_type_name.name(), name, type_definition.type_name())))
    //         }
    //     }
    //     else {
    //         Err(Error::BuildFailed(format!("Failed to find {} \"{}\"", defined_type_name.name(), name)))
    //     }
    // }

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

    pub fn get_scalar(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Scalar>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::Scalar(type_definition) = type_definition {
                Some(type_definition)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected Scalar for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingScalarError(position.clone(), format!("Failed to find Scalar \"{}\"", name)));
            None
        }
    }

    pub fn get_object(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Object>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::Object(object) = type_definition {
                Some(object)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected Object for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingObjectError(position.clone(), format!("Failed to find Object \"{}\"", name)));
            None
        }
    }

    pub fn get_interface(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Interface>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::Interface(interface) = type_definition {
                Some(interface)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected Interface for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingInterfaceError(position.clone(), format!("Failed to find Interface \"{}\"", name)));
            None
        }
    }
    
    fn get_union(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Union>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::Union(union) = type_definition {
                Some(union)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected Union for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingUnionError(position.clone(), format!("Failed to find Union \"{}\"", name)));
            None
        }
    }
    
    fn get_enum(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Enum>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::Enum(type_definition) = type_definition {
                Some(type_definition)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected Enum for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingEnumError(position.clone(), format!("Failed to find Enum \"{}\"", name)));
            None
        }
    }

    pub fn get_input_object(&self, err: &mut ErrorCollector, position: &Pos, name: &Atom) -> Option<&Rc<Object>> {
        if let Some(type_definition) = &self.named_types.get(name) {
            if let TypeDefinition::InputObject(object) = type_definition {
                Some(object)
            }
            else {
                err.error(BuildError::TypeMismatchError(position.clone(), format!("Expected InputObject for \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            err.error(BuildError::MissingInputObjectError(position.clone(), format!("Failed to find InputObject \"{}\"", name)));
            None
        }
    }
}

#[derive(Debug)]
pub enum Selection {
    Field(Rc<SelectionField>),
    Fragment(Rc<FragmentSpread>),
    // InlineFragment(InlineFragment),
} 

impl Selection{
    pub fn new(err: &mut ErrorCollector, selection: graphql_parser::query::Selection<'_, String> ) -> Selection {
        match selection {
            graphql_parser::query::Selection::OptionalField(field) => 
                SelectionField::new(err, *field, true),
            graphql_parser::query::Selection::Field(field) => SelectionField::new(err, field, false),
            graphql_parser::query::Selection::FragmentSpread(fragment) => FragmentSpread::new(err, fragment),
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
pub struct SelectionField {
    pub name: Atom,
    pub alias: Option<Atom>,
    pub position: Pos,
    pub optional: bool,
    pub arguments: Vec<Rc<Argument>>,
    pub selections: Vec<Selection>,
}

impl SelectionField {
    pub fn new(out: &mut ErrorCollector, field: graphql_parser::query::Field<'_, String>, optional: bool) -> Selection {
        Selection::Field(Rc::new(SelectionField {
            name: intern(field.name),
            alias: intern_option(field.alias),
            position: field.position,
            optional,
            arguments: build_arguments(out, field.arguments),
            selections: build_selections(out, field.selection_set),
        }))
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
pub struct GenericOperation {
    pub name: Atom,
    pub position: Pos,
    pub operation: OperationType,
    pub selections: Vec<Selection>,
    pub variables: FieldMap,
}

impl GenericOperation {
    pub fn from_query(err: &mut ErrorCollector, query: graphql_parser::query::Query<'_, String>, schema: &validated_model::Schema ) -> Result<Rc<GenericOperation>, Error> {
        Self::new(err, OperationType::Query, query.name, query.position, query.selection_set, query.variable_definitions, schema)
    }

    pub fn from_mutation(err: &mut ErrorCollector, query: graphql_parser::query::Mutation<'_, String>, schema: &validated_model::Schema ) -> Result<Rc<GenericOperation>, Error> {
        Self::new(err, OperationType::Query, query.name, query.position, query.selection_set, query.variable_definitions, schema)
    }

    pub fn new(err: &mut ErrorCollector, operation: OperationType, name: Option<String>, position: Pos, selection_set: graphql_parser::query::SelectionSet<'_, String>, 
        variable_definitions: Vec<graphql_parser::query::VariableDefinition<'_, String>>, _schema: &validated_model::Schema ) -> Result<Rc<GenericOperation>, Error> {
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
        

        Ok(Rc::new(GenericOperation {
            name: intern(name),
            position,
            operation,
            selections,
            variables,
            // query_object,
        }))
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

                for variable in self.variables.values() {
                    variable.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}



fn build_variables(_out: &mut ErrorCollector, variable_definitions: Vec<graphql_parser::query::VariableDefinition<'_, String>>) -> FieldMap {
    let mut variables = IndexMap::new();

    for variable in variable_definitions {
        let field = Field::from_variable(variable);
        variables.insert(field.name.clone(), field);
    }

    variables
}

fn build_selections(out: &mut ErrorCollector, selection_set: graphql_parser::query::SelectionSet<'_, String>) -> Vec<Selection> {

    let mut selections = Vec::new();

    for selection in selection_set.items {
        selections.push(Selection::new(out, selection));
    }

    selections
}



fn build_arguments(out: &mut ErrorCollector, arguments: Vec<(String, graphql_parser::query::Value<'_, String>)>) -> Vec<Rc<Argument>> {
    let mut result = Vec::new();

    for (name, value) in arguments {
        result.push(Argument::new(out, name, value));
    }

    result
}

#[derive(Debug)]
pub enum Value {
    Variable(Atom),
    Int(i64),
    // Float(f64),
    String(String),
    // Boolean(bool),
    // Null,
    // Enum,
    // List,
    // Object,
}

impl Value {
    pub fn new(_out: &mut ErrorCollector, value: graphql_parser::query::Value<'_, String> ) -> Self {
        match value {
            graphql_parser::query::Value::Variable(name) => Value::Variable(intern(name)),
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

impl Display for Value {
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
pub struct Argument {
    pub name: Atom,
    pub value: Value,
}

impl Argument {
    pub fn new(out: &mut ErrorCollector, name: String, value: graphql_parser::query::Value<'_, String> ) -> Rc<Self> {
        Rc::new(Argument {
            name: intern(name),
            value: Value::new(out, value),
        })
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
    
    pub fn generate_query(&self, out: &mut Output<'_>) -> Result<(), Error> {
        writeln!(out, "{}: {}", self.name, self.value)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FragmentDefinition {
    pub name: Atom,
    pub position: Pos,
    pub selections: Vec<Selection>,
    pub type_condition: Atom,
}

impl FragmentDefinition{
    pub fn new(err: &mut ErrorCollector, fragment_definition: graphql_parser::query::FragmentDefinition<'_, String>, _schema: &validated_model::Schema ) -> Result<Rc<Self>, Error> {

        let selections = build_selections(err, fragment_definition.selection_set);

        if selections.is_empty() {
            return err.fail(BuildError::InvalidQueryError(fragment_definition.position, format!("FragmentDefinition {} has no selection set", &fragment_definition.name)));
        }

        let type_condition = match fragment_definition.type_condition {
            graphql_parser::query::TypeCondition::On(value) => intern(value),
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
        

        Ok(Rc::new(FragmentDefinition {
            name: intern(fragment_definition.name),
            position: fragment_definition.position,
            selections,
            type_condition,
            // query_object,
        }))
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
pub struct FragmentSpread {
    pub name: Atom,
    pub position: Pos,
}

impl FragmentSpread {
    pub fn new(_err: &mut ErrorCollector, fragment: graphql_parser::query::FragmentSpread<'_, String>) -> Selection {
        Selection::Fragment(Rc::new(FragmentSpread {
            name: intern(fragment.fragment_name),
            position: fragment.position,
        }))
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
pub struct ExecutableDocument {
    pub name: Atom,
    pub fragments: IndexMap<Atom, Rc<FragmentDefinition>>,
    pub queries: Vec<Rc<GenericOperation>>,
    pub mutations: Vec<Rc<GenericOperation>>
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector,
        definitions: Vec<graphql_parser::query::Definition<'_, String>>, model_name: &str,
        schema: &Rc<validated_model::Schema>) -> Rc<Self> {

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
                            if let  Ok(query) = GenericOperation::from_query(err, query, schema) {
                                queries.push(query);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Mutation(mutation) => {
                            if let Ok(mutation) = GenericOperation::from_mutation(err, mutation, schema) {
                                mutations.push(mutation);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Subscription(_subscription) => {
                            unimplemented!()
                            
                        },
                    }
                },
                graphql_parser::query::Definition::Fragment(fragment_definition) => {
                    if let Ok(fragment) = FragmentDefinition::new(err, fragment_definition, schema) {
                        fragments.insert(fragment.name.clone(), fragment);
                    }
                },
            }
        }

        Rc::new(ExecutableDocument {
                name: intern(name),
                fragments,
                queries,
                mutations,
            })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "ExecutableDocument {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "fragments {{")?;
            {
                let mut out = out.indent();

                for query in self.fragments.values() {
                    query.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "queries {{")?;
            {
                let mut out = out.indent();

                for query in &self.queries {
                    query.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "mutations {{")?;
            {
                let mut out = out.indent();

                for query in &self.mutations {
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
// struct ObjectReferenceList {
//     schema: &'a Schema,
//     names: Vec<String>,
// }

// impl IntoIterator for &'a ObjectReferenceList {
//     type Item = &'a Object;
//     type IntoIter = ObjectReferenceListItertor;
    
//     fn into_iter(self) -> Self::IntoIter {
//         ObjectReferenceListItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// struct ObjectReferenceListItertor {
//     list: &'a ObjectReferenceList,
//     // iter: std::slice::Iter<'_, String>,
//     index: usize,
// }

// impl Iterator for ObjectReferenceListItertor {
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

// impl IntoIterator for &'a InterfaceReferenceList {
//     type Item = &'a Interface;
//     type IntoIter = InterfaceReferenceListItertor;
    
//     fn into_iter(self) -> Self::IntoIter {
//         InterfaceReferenceListItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// pub struct InterfaceReferenceListItertor {
//     list: &'a InterfaceReferenceList,
//     // iter: std::slice::Iter<'_, String>,
//     index: usize,
// }

// impl Iterator for InterfaceReferenceListItertor {
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



