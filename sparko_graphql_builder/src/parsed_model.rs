use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;
use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::to_snake_case;

use crate::Name;
use crate::BuildError;
use crate::BuildWarning;
use crate::Error;
use crate::ErrorCollector;
use crate::Isomorphic;
use crate::Output;
use crate::NameRegistry;
use crate::TYPE_NAME;


pub type FieldMap = IndexMap<Name, Rc<Field>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub name: Name,
    pub position: Pos
}

impl Isomorphic for DefinedType {
    fn is_isomorphic(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl DefinedType {
    fn new(registry: &mut NameRegistry, v: String, position: Pos) -> ScalarType {
        ScalarType::DefinedType(Rc::new(DefinedType{
            name: registry.intern(v),
            position
        }))
    }
}

#[derive(Debug)]
pub enum ScalarType {
    DefinedType(Rc<DefinedType>),
    BuiltinType(BuiltinType),
}

impl Isomorphic for ScalarType {
    fn is_isomorphic(&self, other: &Self) -> bool {
        match self {
            ScalarType::DefinedType(defined_type) => {
                if let ScalarType::DefinedType(other_type) = other {
                    defined_type.is_isomorphic(&other_type)
                }
                else {
                    false
                }
            },
            ScalarType::BuiltinType(builtin_type) => {
                if let ScalarType::BuiltinType(other_type) = other {
                    builtin_type == other_type
                }
                else {
                    false
                }
            },
        }
    }
}

impl ScalarType {
    fn new(registry: &mut NameRegistry, v: String, position: Pos) -> ScalarType {
        if let Some(builtin_type) = BuiltinType::from_str(&v) {
            ScalarType::BuiltinType(builtin_type)
        }
        else {
            DefinedType::new(registry, v, position)
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

impl Isomorphic for Type {
    fn is_isomorphic(&self, other: &Self) -> bool {
        match self {
            Type::Scalar(scalar_type) => {
                if let Type::Scalar(other_type) = other {
                    scalar_type.is_isomorphic(other_type)
                }
                else {
                    false
                }
            },
            Type::Required(wrapped) => {
                if let Type::Required(other_type) = other {
                    wrapped.is_isomorphic(other_type)
                }
                else {
                    false
                }
            },
            Type::Array(wrapped) => {
                if let Type::Array(other_type) = other {
                    wrapped.is_isomorphic(other_type)
                }
                else {
                    false
                }
            },
        }
    }
}

impl Type {
    fn new(registry: &mut NameRegistry, parsed: graphql_parser::schema::Type<'_, String>, position: Pos) -> Type {
        match parsed {
            graphql_parser::query::Type::NamedType(name) => Type::Scalar(ScalarType::new(registry, name, position)),
            graphql_parser::query::Type::ListType(wrapped) => Type::Array(Box::new(Type::new(registry, *wrapped, position))),
            graphql_parser::query::Type::NonNullType(wrapped) => Type::Required(Box::new(Type::new(registry, *wrapped, position))),
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
    Selection,
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
            DefinedTypeName::Selection => "Selection",
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

    pub fn name(&self) -> &Name {
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
    pub name: Name,
}

impl Scalar {
    fn new(registry: &mut NameRegistry, scalar_type: graphql_parser::schema::ScalarType<'_, String>) -> TypeDefinition {
        TypeDefinition::Scalar(Rc::new(Scalar {
            position: scalar_type.position,
            name: registry.intern(scalar_type.name),
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
    pub name: Name,
    pub fields: FieldMap,
    pub implements: Vec<Name>,
    pub is_input: bool,
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

    pub fn from_object(_err: &mut ErrorCollector, registry: &mut NameRegistry, object_type: graphql_parser::schema::ObjectType<'_, String>) -> TypeDefinition {
        let mut fields: FieldMap = IndexMap::new();

        for f in object_type.fields {
            let value = Field::from_field(registry, f);
            fields.insert(value.name.clone(), value);
        }

        TypeDefinition::Object(Rc::new(Object {
            position: object_type.position,
            name: registry.intern(object_type.name),
            fields,
            implements: registry.intern_vec(object_type.implements_interfaces),
            is_input: false,
        }))
    }
    
    fn from_input_object(_err: &mut ErrorCollector<'_>, registry: &mut NameRegistry, object_type: graphql_parser::schema::InputObjectType<'_, String>) -> TypeDefinition {
        let mut fields = IndexMap::new();

        for f in object_type.fields {
            let value = Field::from_input_value(registry, f);
            fields.insert(value.name.clone(), value);
        }

        TypeDefinition::InputObject(Rc::new(Object {
            position: object_type.position,
            name: registry.intern(object_type.name),
            fields,
            implements: Vec::new(),
            is_input: true,
        }))
    }
}


#[derive(Debug)]
pub struct Interface {
    pub position: Pos,
    pub name: Name,
    pub fields: FieldMap,
    pub implements: Vec<Name>,
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
    
    pub fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, interface_type: graphql_parser::schema::InterfaceType<'_, String>) -> TypeDefinition {
        let mut fields = IndexMap::new();

        for f in interface_type.fields {
            let value = Field::from_field(registry, f);
            fields.insert(value.name.clone(), value);
            
        }

        TypeDefinition::Interface(Rc::new(Interface {
            position: interface_type.position,
            name: registry.intern(interface_type.name),
            fields,
            implements: registry.intern_vec(interface_type.implements_interfaces),
        }))
    }
}

#[derive(Debug)]
pub struct Union {
    pub position: Pos,
    pub name: Name,
    pub types: Vec<Name>,
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

    fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, union_type: graphql_parser::schema::UnionType<'_, String>) -> TypeDefinition {
        TypeDefinition::Union(Rc::new(Union {
            position: union_type.position,
            name: registry.intern(union_type.name),
            types: registry.intern_vec(union_type.types),
        }))
    }
}

#[derive(Debug)]
pub struct Variant {
    pub name: Name,
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

    pub fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, enum_type: graphql_parser::schema::EnumValue<'_, String>) -> Self {
        Variant {
            name: registry.intern(enum_type.name),
            position: enum_type.position,
            description: enum_type.description,
        }
    }
    
    fn new_vec(err: &mut ErrorCollector, registry: &mut NameRegistry, values: Vec<graphql_parser::schema::EnumValue<'_, String>>) -> Vec<Variant> {
        let mut result =  Vec::new();

        for value in values {
            result.push(Self::new(err, registry, value));
        }

        result
    }
}

#[derive(Debug)]
pub struct Enum {
    pub name: Name,
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

    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, enum_type: graphql_parser::schema::EnumType<'_, String>) -> TypeDefinition {
        TypeDefinition::Enum(Rc::new(Enum {
            name: registry.intern(enum_type.name),
            position: enum_type.position,
            variants: Variant::new_vec(err, registry, enum_type.values),
        }))
    }
}

#[derive(Debug)]
pub struct Field {
    pub name: Name,
    pub position:Pos,
    pub ty: Type,
}

impl Isomorphic for Field {
    fn is_isomorphic(&self, other: &Self) -> bool {
        self.name == other.name && self.ty.is_isomorphic(&other.ty)
    }
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

    fn do_new(registry: &mut NameRegistry, position: Pos, name: String, field_type: graphql_parser::schema::Type<'_, String>) -> Rc<Field> {
        let ty = Type::new(registry, field_type, position.clone());
        Rc::new(Field {
            position,
            name: registry.intern(name),
            ty,
        })
        
    }

    pub fn from_input_value(registry: &mut NameRegistry, field: graphql_parser::schema::InputValue<'_, String>) -> Rc<Field> {
        Self::do_new(registry, field.position, field.name, field.value_type)
    }

    pub fn from_field(registry: &mut NameRegistry, field: graphql_parser::schema::Field<'_, String>) -> Rc<Field> {
        Self::do_new(registry, field.position, field.name, field.field_type)
    }
    
    fn from_variable(registry: &mut NameRegistry, variable: graphql_parser::query::VariableDefinition<'_, String>) -> Rc<Field> {
        Self::do_new(registry, variable.position, variable.name, variable.var_type)
    }
}

#[derive(Debug)]
pub struct SchemaDefinition {
    pub position: Pos,
    pub query: Option<Name>,
    pub mutation: Option<Name>,
    pub subscription: Option<Name>,
}

impl SchemaDefinition {
    fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, schema_definition: graphql_parser::schema::SchemaDefinition<'_, String>) -> Self {
        SchemaDefinition {
            position: schema_definition.position,
            query: registry.intern_option(schema_definition.query),
            mutation: registry.intern_option(schema_definition.mutation),
            subscription: registry.intern_option(schema_definition.subscription),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    pub named_types: IndexMap<Name, TypeDefinition>,
    pub schema_definition: Option<SchemaDefinition>,
    pub __typename: Rc<Field>,
}

impl Schema {
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, ast: graphql_parser::schema::Document<'_, String>) -> Result<Rc<Self>, Error> {
        let mut model = Schema {
            named_types: IndexMap::new(),
            schema_definition: None,
            __typename: Rc::new(Field {
                position: Pos { line: 0, column: 0 },
                name: registry.intern_str(TYPE_NAME),
                ty: Type::Scalar(ScalarType::BuiltinType(BuiltinType::String)),
            }),
        };

        for def in ast.definitions {
            match def {
                graphql_parser::schema::Definition::SchemaDefinition(schema_definition) => {
                    if let Some(existing) = &model.schema_definition {
                        err.error(BuildError::DuplicateName(schema_definition.position, existing.position.clone(), "Schema".to_string()));
                    }
                    else {
                        model.schema_definition = Some(SchemaDefinition::new(err, registry, schema_definition));
                    }
                },
                graphql_parser::schema::Definition::TypeDefinition(type_definition) => {
                    let value = match type_definition {
                        graphql_parser::schema::TypeDefinition::Scalar(scalar_type) => Scalar::new(registry, scalar_type),
                        graphql_parser::schema::TypeDefinition::Object(ast) => Object::from_object(err, registry, ast),
                        graphql_parser::schema::TypeDefinition::Interface(ast) => Interface::new(err, registry, ast),
                        graphql_parser::schema::TypeDefinition::Union(ast) => Union::new(err, registry, ast),
                        graphql_parser::schema::TypeDefinition::Enum(ast) => Enum::new(err, registry, ast),
                        graphql_parser::schema::TypeDefinition::InputObject(ast) => Object::from_input_object(err, registry, ast),
                    };
                    let position = value.position().clone();

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

    pub fn get_scalar(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Scalar>> {
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

    pub fn get_object(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Object>> {
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

    pub fn get_interface(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Interface>> {
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
    
    fn get_union(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Union>> {
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
    
    fn get_enum(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Enum>> {
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

    pub fn get_input_object(&self, err: &mut ErrorCollector, position: &Pos, name: &Name) -> Option<&Rc<Object>> {
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
pub struct SelectionList {
    pub selections: Vec<Selection>,
}

impl SelectionList {
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "selections {{")?;
        {
            let mut out = out.indent();

            for selection in &self.selections {
                selection.print(&mut out)?;
            }
        }
        writeln!(out, "}}")
    }
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Rc<SelectionField>),
    FragmentSpread(Rc<FragmentSpread>),
    InlineFragment(Rc<InlineFragmentSpread>),
} 

impl Selection{
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, selection: graphql_parser::query::Selection<'_, String> ) -> Result<Selection, BuildError> {
        Ok(match selection {
            graphql_parser::query::Selection::OptionalField(field) => 
                SelectionField::new(err, registry, *field, true),
            graphql_parser::query::Selection::Field(field) => SelectionField::new(err, registry, field, false),
            graphql_parser::query::Selection::FragmentSpread(fragment) => FragmentSpread::new(err, registry, fragment),
            graphql_parser::query::Selection::InlineFragment(inline_fragment) => InlineFragmentSpread::new(err, registry, inline_fragment)?,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::Field(selection_field) => selection_field.print(out),
            Selection::FragmentSpread(selection_fragment) => selection_fragment.print(out),
            Selection::InlineFragment(inline_fragment_spread) => inline_fragment_spread.print(out),
        }
    }
}

#[derive(Debug)]
pub struct SelectionField {
    pub name: Name,
    pub alias: Option<Name>,
    pub position: Pos,
    pub optional: bool,
    pub arguments: Vec<Rc<Argument>>,
    pub selections: Rc<SelectionList>,
}

impl SelectionField {
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, field: graphql_parser::query::Field<'_, String>, optional: bool) -> Selection {
        Selection::Field(Rc::new(SelectionField {
            name: registry.intern(field.name),
            alias: registry.intern_option(field.alias),
            position: field.position,
            optional,
            arguments: build_arguments(err, registry, field.arguments),
            selections: build_selections(err, registry, field.selection_set),
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

            self.selections.print(&mut out)?;
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
}

#[derive(Debug)]
pub struct GenericOperation {
    pub name: Name,
    pub position: Pos,
    pub operation: OperationType,
    pub selections: Rc<SelectionList>,
    pub variables: FieldMap,
}

impl GenericOperation {
    pub fn from_query(err: &mut ErrorCollector, registry: &mut NameRegistry, query: graphql_parser::query::Query<'_, String>) -> Result<Rc<GenericOperation>, Error> {
        Self::new(err, registry, OperationType::Query, query.name, query.position, query.selection_set, query.variable_definitions)
    }

    pub fn from_mutation(err: &mut ErrorCollector, registry: &mut NameRegistry, query: graphql_parser::query::Mutation<'_, String>) -> Result<Rc<GenericOperation>, Error> {
        Self::new(err, registry, OperationType::Mutation, query.name, query.position, query.selection_set, query.variable_definitions)
    }

    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, operation: OperationType, name: Option<String>, position: Pos, selection_set: graphql_parser::query::SelectionSet<'_, String>, 
        variable_definitions: Vec<graphql_parser::query::VariableDefinition<'_, String>>) -> Result<Rc<GenericOperation>, Error> {
        let name = match name {
            Some(name) => name,
            None => {
                return err.fail(BuildError::UnsupportedError(position.clone(), String::from("Anonymous query")))
            },
        };

        let selections = build_selections(err, registry, selection_set);
        let variables = build_variables(err, registry, variable_definitions);

        if selections.is_empty() {
            return err.fail(BuildError::InvalidQueryError(position.clone(), format!("Query {} has no selection set", name)));
        }

        Ok(Rc::new(GenericOperation {
            name: registry.intern(name),
            position,
            operation,
            selections,
            variables,
        }))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{} {{", &self.operation)?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            self.selections.print(&mut out)?;

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



fn build_variables(_err: &mut ErrorCollector, registry: &mut NameRegistry, variable_definitions: Vec<graphql_parser::query::VariableDefinition<'_, String>>) -> FieldMap {
    let mut variables = IndexMap::new();

    for variable in variable_definitions {
        let field = Field::from_variable(registry, variable);
        variables.insert(field.name.clone(), field);
    }

    variables
}

fn build_selections(err: &mut ErrorCollector, registry: &mut NameRegistry, selection_set: graphql_parser::query::SelectionSet<'_, String>) -> Rc<SelectionList> {

    let mut selections = Vec::new();

    for selection in selection_set.items {
        match Selection::new(err, registry, selection) {
            Ok(selection) => selections.push(selection),
            Err(error) => err.error(error),
        }
        
    }

    Rc::new( SelectionList{
        selections,
    })
}

fn build_arguments(err: &mut ErrorCollector, registry: &mut NameRegistry, arguments: Vec<(String, graphql_parser::query::Value<'_, String>)>) -> Vec<Rc<Argument>> {
    let mut result = Vec::new();

    for (name, value) in arguments {
        result.push(Argument::new(err, registry, name, value));
    }

    result
}

#[derive(Debug)]
pub enum Value {
    Variable(Name),
    Int(i64),
    String(String),
}

impl Value {
    pub fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, value: graphql_parser::query::Value<'_, String> ) -> Self {
        match value {
            graphql_parser::query::Value::Variable(name) => Value::Variable(registry.intern(name)),
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
            Value::String(string) => write!(f, "\\\"{}\\\"", string),
        }
    }
}


#[derive(Debug)]
pub struct Argument {
    pub name: Name,
    pub value: Value,
}

impl Argument {
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, name: String, value: graphql_parser::query::Value<'_, String> ) -> Rc<Self> {
        Rc::new(Argument {
            name: registry.intern(name),
            value: Value::new(err, registry, value),
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
    
    pub fn generate_query(&self, out: &mut Output<'_>, variables: &crate::validated_model::FieldMap) -> Result<(), Error> {
        if let Value::Variable(name) = &self.value {
            let field = variables.get(&name).unwrap();
            if field.ty.is_optional() {
                writeln!(out, "if self.variables.{}_.is_some() {{", to_snake_case(&field.parsed.name))?;
                writeln!(out, "    buf.push_str(\"{}: {},\\n\");", self.name, self.value)?;
                writeln!(out, "}}")?;
            }
            else {
                writeln!(out, "buf.push_str(\"{}: {},\\n\");", self.name, self.value)?;
            }
        }
        else {
            writeln!(out, "buf.push_str(\"{}: {},\\n\");", self.name, self.value)?;
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct FragmentDefinition {
    pub name: Name,
    pub position: Pos,
    pub selections: Rc<SelectionList>,
    pub type_condition: Name,
}

impl FragmentDefinition{
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, fragment_definition: graphql_parser::query::FragmentDefinition<'_, String>) -> Result<Rc<Self>, Error> {

        let selections = build_selections(err, registry, fragment_definition.selection_set);

        if selections.is_empty() {
            return err.fail(BuildError::InvalidQueryError(fragment_definition.position, format!("FragmentDefinition {} has no selection set", &fragment_definition.name)));
        }

        let type_condition = match fragment_definition.type_condition {
            graphql_parser::query::TypeCondition::On(value) => registry.intern(value),
        };

        Ok(Rc::new(FragmentDefinition {
            name: registry.intern(fragment_definition.name),
            position: fragment_definition.position,
            selections,
            type_condition,
        }))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "FragmentDefinition {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:        {}", self.position)?;
            writeln!(out, "name:            {}", self.name)?;
            writeln!(out, "type_condition:  {}", self.type_condition)?;
            self.selections.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub struct FragmentSpread {
    pub name: Name,
    pub position: Pos,
}

impl FragmentSpread {
    pub fn new(_err: &mut ErrorCollector, registry: &mut NameRegistry, fragment: graphql_parser::query::FragmentSpread<'_, String>) -> Selection {
        Selection::FragmentSpread(Rc::new(FragmentSpread {
            name: registry.intern(fragment.fragment_name),
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
pub struct InlineFragmentSpread {
    pub position: Pos,
    pub selections: Rc<SelectionList>,
    pub type_condition: Option<Name>,
}

impl InlineFragmentSpread{
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, fragment_definition: graphql_parser::query::InlineFragment<'_, String>) -> Result<Selection, BuildError> {

        let selections = build_selections(err, registry, fragment_definition.selection_set);

        if selections.is_empty() {
            return Err(BuildError::InvalidQueryError(fragment_definition.position, format!("InlineFragmentDefinition has no selection set")));
        }

        let type_condition = match fragment_definition.type_condition {
            Some(cond) => match cond {
                graphql_parser::query::TypeCondition::On(name) => Some(registry.intern(name)),
            },
            None => None,
        };

        Ok(Selection::InlineFragment(Rc::new(InlineFragmentSpread {
            position: fragment_definition.position,
            selections,
            type_condition,
        })))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "InlineFragmentSpread {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:        {}", self.position)?;
            writeln!(out, "type_condition:  {:?}", self.type_condition)?;
            self.selections.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
}

#[derive(Debug)]
pub struct ExecutableDocument {
    pub name: Name,
    pub fragments: IndexMap<Name, Rc<FragmentDefinition>>,
    pub queries: Vec<Rc<GenericOperation>>,
    pub mutations: Vec<Rc<GenericOperation>>
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, definitions: Vec<graphql_parser::query::Definition<'_, String>>, model_name: &str) -> Rc<Self> {

        let name = to_snake_case(model_name);
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
                            if let  Ok(query) = GenericOperation::from_query(err, registry, query) {
                                queries.push(query);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Mutation(mutation) => {
                            if let Ok(mutation) = GenericOperation::from_mutation(err, registry, mutation) {
                                mutations.push(mutation);
                            }
                        },
                        graphql_parser::query::OperationDefinition::Subscription(_subscription) => {
                            unimplemented!()
                            
                        },
                    }
                },
                graphql_parser::query::Definition::Fragment(fragment_definition) => {
                    if let Ok(fragment) = FragmentDefinition::new(err, registry, fragment_definition) {
                        fragments.insert(fragment.name.clone(), fragment);
                    }
                },
            }
        }

        Rc::new(ExecutableDocument {
                name: registry.intern(name),
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