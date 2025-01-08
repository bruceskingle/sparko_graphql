use std::fmt::Display;
use std::rc::Rc;
use std::collections::HashMap;
use std::io::Write;

use inflections::case::{to_snake_case, to_constant_case};

use crate::utils::to_pascal_case;
use crate::{error::GraphQLError, parsed_model, Output};

#[derive(Debug)]
pub struct Scalar {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub rust_name: String,
    pub rust_type: String,
}

impl Scalar {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "TypeDef {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "rust_name: {}", self.rust_name)?;
            writeln!(out, "rust_type: {}", self.rust_type)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(parsed: &parsed_model::TypeDef) -> Self {

        let rust_type = "serde_json::Value".to_string(); // TODO: get proper types

        Scalar {
            position: parsed.position,
            rust_name: to_pascal_case(&parsed.name),
            name: parsed.name.clone(),
            rust_type,
        }
    }
}

#[derive(Debug)]
pub enum TypeDefinition {
    Enum(Enum),
    Union(Union),
    Object(Object),
    Interface(Interface),
    Scalar(Scalar),
}

impl TypeDefinition {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            TypeDefinition::Enum(content) => content.print(out),
            TypeDefinition::Union(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            TypeDefinition::Interface(content) => content.print(out),
            TypeDefinition::Scalar(content) => content.print(out),
        }
    }

    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Enum(union) => "enum",
            TypeDefinition::Union(union) => "union",
            TypeDefinition::Object(object) => "object",
            TypeDefinition::Interface(interface) => "interface",
            TypeDefinition::Scalar(type_def) => "scalar",
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

    pub fn generate(&self, out: &mut Output, schema: &Schema) -> Result<(), GraphQLError> {
        match self {
            TypeDefinition::Enum(enum_def) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "#[serde(rename = \"{}\")]", enum_def.name)?;
                writeln!(out, "pub enum {} {{", to_pascal_case(&enum_def.name))?;
        
                for name in &enum_def.variants {
                    writeln!(out, "    #[serde(rename = \"{}\")]", name)?;
                    writeln!(out, "    {},", to_constant_case(name))?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;

            }
            TypeDefinition::Union(union) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "#[serde(rename = \"{}\")]", &union.name)?;
                writeln!(out, "pub struct {} {{", to_pascal_case(&union.name))?;
        
                for (name, field) in &union.fields {
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
                    writeln!(out, "    {}_: {},", to_snake_case(&field.name), field.rust_type(schema))?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
            TypeDefinition::Object(object_model) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "#[serde(rename = \"{}\")]", &object_model.name)?;
                writeln!(out, "pub struct {} {{", to_pascal_case(&object_model.name))?;
        
                for (name, field) in &object_model.fields {
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
                    writeln!(out, "    {}_: {},", to_snake_case(&field.name), field.rust_type(schema))?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
            TypeDefinition::Interface(interface_model) => {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "pub enum {} {{", to_pascal_case(&interface_model.name))?;

                for object in interface_model.implemented_by.iterator(schema) {
                    
                    writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
                }

                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
            TypeDefinition::Scalar(scalar) => {
                writeln!(out, "type {} = {};", &scalar.rust_name, &scalar.rust_type)?;
                writeln!(out, "")?;
            }
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
pub enum DefinedType {
    Enum(EnumProxy),
    Union(UnionProxy),
    Object(ObjectProxy),
    Interface(InterfaceProxy),
    Scalar(ScalarProxy),
}

impl Display for DefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinedType::Enum(proxy) => write!(f, "Enum {}", proxy.name),
            DefinedType::Union(proxy) => write!(f, "Union {}", proxy.name),
            DefinedType::Object(proxy) => write!(f, "Object {}", proxy.name),
            DefinedType::Interface(proxy) => write!(f, "Interface {}", proxy.name),
            DefinedType::Scalar(proxy) => write!(f, "Scalar {}", proxy.name),
        }
    }
}


#[derive(Debug)]
pub enum Type {
    Int,
    Float,
    String,
    Boolean,
    ID,
    DefinedType(DefinedType),
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
            Type::DefinedType(defined_type) => defined_type.fmt(f),
        }
    }
}

impl Type {
    // pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
    //     match self {
    //         // Type::DefinedType(content) => {content.print(out); Ok(0 as usize)},
    //         Type::Int => out.write(b"Int"),
    //         Type::Float => out.write(b"Float"),
    //         Type::String => out.write(b"String"),
    //         Type::Boolean => out.write(b"Boolean"),
    //         Type::ID => out.write(b"ID"),
    //         Type::Enum(content) => {content.print(out); Ok(0 as usize)},
    //         Type::Union(content) => {content.print(out); Ok(0 as usize)},
    //         Type::Object(content) => {content.print(out); Ok(0 as usize)},
    //         Type::Interface(content) => {content.print(out); Ok(0 as usize)},
    //         Type::Scalar(content) => {content.print(out); Ok(0 as usize)},
    //     }?;
    //     Ok(())
    // }

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
                        parsed_model::TypeDefinition::Enum(content) => Type::DefinedType(DefinedType::Enum(EnumProxy::new(content.name.clone()))),
                        parsed_model::TypeDefinition::Union(content) =>Type::DefinedType(DefinedType::Union(UnionProxy::new(content.name.clone()))),
                        parsed_model::TypeDefinition::Object(content) =>Type::DefinedType(DefinedType::Object(ObjectProxy { name: content.name.clone()})),
                        parsed_model::TypeDefinition::Interface(content) =>Type::DefinedType(DefinedType::Interface(InterfaceProxy::new(content.name.clone()))),
                        parsed_model::TypeDefinition::Scalar(content) =>Type::DefinedType(DefinedType::Scalar(ScalarProxy::new(content.name.clone()))),
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

    fn from_validated(out: &mut Output<'_>, variable: &parsed_model::Field, schema: &Schema) -> Type {
        match &variable.ty {
            parsed_model::Type::DefinedType(name) => {
                if let Some(def) = schema.defined_types.get(name) {
                    match def {
                        TypeDefinition::Enum(content) => Type::DefinedType(DefinedType::Enum(EnumProxy::new(content.name.clone()))),
                        TypeDefinition::Union(content) =>Type::DefinedType(DefinedType::Union(UnionProxy::new(content.name.clone()))),
                        TypeDefinition::Object(content) =>Type::DefinedType(DefinedType::Object(ObjectProxy { name: content.name.clone()})),
                        TypeDefinition::Interface(content) =>Type::DefinedType(DefinedType::Interface(InterfaceProxy::new(content.name.clone()))),
                        TypeDefinition::Scalar(content) =>Type::DefinedType(DefinedType::Scalar(ScalarProxy::new(content.name.clone()))),
                    }
                }
                else {
                    out.error(GraphQLError::UndefinedTypeError(variable.position.clone(), format!("Missing type {}", name)));
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
     
    fn rust_type(&self, multiple: bool, nonnull: bool, schema: &Schema) -> String {
        let base_type =  match self {
            // Type::DefinedType(defined_type) => write!(f, "{}", defined_type),
            Type::Int => "i32".to_string(),
            Type::Float => "f64".to_string(),
            Type::String => "String".to_string(),
            Type::Boolean => "bool".to_string(),
            Type::ID => "String".to_string(),
            Type::DefinedType(defined_type) => match defined_type {
                DefinedType::Enum(proxy) => to_pascal_case(&proxy.get(schema).name),
                DefinedType::Union(proxy) => to_pascal_case(&proxy.get(schema).name),
                DefinedType::Object(proxy) => to_pascal_case(&proxy.get(schema).name),
                DefinedType::Interface(proxy) => to_pascal_case(&proxy.get(schema).name),
                DefinedType::Scalar(proxy) => proxy.get(schema).rust_name.clone(),
            },
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
    pub position: graphql_parser::Pos,
    pub multiple: bool,
    pub nonnull: bool,
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
            writeln!(out, "nonnull:   {}", self.nonnull)?;
            writeln!(out, "multiple:  {}", self.multiple)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(out: &mut Output, field: &parsed_model::Field, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Field {
        Field {
            name: field.name.clone(),
            position: field.position,
            multiple: field.multiple,
            nonnull: field.nonnull,
            ty: Type::new(out, field, named_types),
        }
    }

    pub fn rust_type(&self, schema: &Schema) -> String {
        self.ty.rust_type(self.multiple, self.nonnull, schema)
    }
    
    fn from_variable(out: &mut Output, variable: &parsed_model::Field, schema: &Schema) -> Field {
        Field {
            name: variable.name.clone(),
            position: variable.position,
            multiple: variable.multiple,
            nonnull: variable.nonnull,
            ty: Type::from_validated(out, variable, schema),
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub variants: Vec<String>,
}

impl Enum {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Enum {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "variants {{")?;
            {
                let mut out = out.indent();

                for name in &self.variants {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn new(out: &mut Output, parsed: &parsed_model::Enum) -> Enum {
        Enum {
            position: parsed.position,
            name: parsed.name.clone(),
            variants: parsed.variants.clone(),
        }
    }
}

#[derive(Debug)]
pub struct Union {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub fields: HashMap<String, Field>,
}

impl Union {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Union {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;

            writeln!(out, "fields {{")?;
            {
                let mut out = out.indent();

                for (name, field) in &self.fields {
                    field.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    fn new(out: &mut Output, parsed: &parsed_model::Union, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Union {
        let mut fields = HashMap::new();

        for interface_name in &parsed.implements {
            if let Some(defined_type) = named_types.get(interface_name) {
                if let parsed_model::TypeDefinition::Interface(interface) = defined_type {
                    for (name, field) in &interface.fields {
                        fields.insert(name.clone(), Field::new(out, field, &named_types));
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

        Union {
            position: parsed.position,
            name: parsed.name.clone(),
            fields,
        }
    }
}


#[derive(Debug)]
pub struct Object {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub fully_implements: InterfaceListProxy,
    pub fields: HashMap<String, Field>,
}

impl Object {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Object {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "name:      {}", self.name)?;

            writeln!(out, "fully_implements {{")?;
            {
                let mut out = out.indent();

                for name in &self.fully_implements.names {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "fields {{")?;
            {
                let mut out = out.indent();

                for (name, field) in &self.fields {
                    field.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    fn new(out: &mut Output, parsed: &parsed_model::Object, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Object {
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
                        fully_implements.push(interface_name.clone());
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

        Object {
            position: parsed.position,
            name: parsed.name.clone(),
            fully_implements: InterfaceListProxy::new(fully_implements),
            fields,
        }
    }
}

#[derive(Debug)]
pub struct Interface {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub implemented_by: ObjectListProxy,
    pub fields: HashMap<String, Field>,
}

impl Interface {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Interface {{")?;

        writeln!(out, "position:  {}", self.position)?;
        writeln!(out, "name:      {}", self.name)?;
        {
            let mut out = out.indent();

            writeln!(out, "implemented_by {{")?;
            {
                let mut out = out.indent();

                for name in &self.implemented_by.names {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    pub fn new( out: &mut Output, parsed: &parsed_model::Interface, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Interface {
        let mut implemented_by = Vec::new();

        for (name, defined_type) in named_types {
            if let parsed_model::TypeDefinition::Object(object_model) = defined_type {
                for implements in &object_model.implements {
                    if implements == &parsed.name {
                        implemented_by.push(object_model.name.clone());
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

        Interface {
            position: parsed.position,
            name: parsed.name.clone(),
            implemented_by: ObjectListProxy::new(implemented_by),
            fields,
        }
    }
}

#[derive(Debug)]
pub struct ObjectProxy {
    pub name: String,
}

impl ObjectProxy {
    pub fn new(name: String) -> Self {
        ObjectProxy {
            name,
        }
    }
    
    pub fn get<'a>(&self, schema: &'a Schema) -> &'a Object {
        match schema.defined_types.get(&self.name) {
            Some(type_definition) => {
                if let TypeDefinition::Object(object) = type_definition {
                    object
                }
                else {
                    panic!("ObjectProxy expected Object for \"{}\" but found {}", &self.name, type_definition.type_name())
                }
            },
            None => panic!("ObjectProxy failed to find \"{}\"", &self.name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn option_as_str(proxy: &Option<ObjectProxy>) -> &str {
        match proxy {
            Some(proxy) => proxy.as_str(),
            None => "None",
        }
    }
}

#[derive(Debug)]
pub struct InterfaceProxy {
    pub name: String,
}

impl InterfaceProxy {
    pub fn new(name: String) -> Self {
        InterfaceProxy {
            name,
        }
    }
    
    pub fn get<'a>(&self, schema: &'a Schema) -> &'a Interface {
        match schema.defined_types.get(&self.name) {
            Some(type_definition) => {
                if let TypeDefinition::Interface(content) = type_definition {
                    content
                }
                else {
                    panic!("InterfaceProxy expected Interface for \"{}\" but found {}", &self.name, type_definition.type_name())
                }
            },
            None => panic!("InterfaceProxy failed to find \"{}\"", &self.name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn option_as_str(proxy: &Option<InterfaceProxy>) -> &str {
        match proxy {
            Some(proxy) => proxy.as_str(),
            None => "None",
        }
    }
}

#[derive(Debug)]
pub struct UnionProxy {
    pub name: String,
}

impl UnionProxy {
    pub fn new(name: String) -> Self {
        UnionProxy {
            name,
        }
    }
    
    pub fn get<'a>(&self, schema: &'a Schema) -> &'a Union {
        match schema.defined_types.get(&self.name) {
            Some(type_definition) => {
                if let TypeDefinition::Union(content) = type_definition {
                    content
                }
                else {
                    panic!("UnionProxy expected Union for \"{}\" but found {}", &self.name, type_definition.type_name())
                }
            },
            None => panic!("UnionProxy failed to find \"{}\"", &self.name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn option_as_str(proxy: &Option<UnionProxy>) -> &str {
        match proxy {
            Some(proxy) => proxy.as_str(),
            None => "None",
        }
    }
}

#[derive(Debug)]
pub struct EnumProxy {
    pub name: String,
}

impl EnumProxy {
    pub fn new(name: String) -> Self {
        EnumProxy {
            name,
        }
    }

    pub fn get<'a>(&self, schema: &'a Schema) -> &'a Enum {
        match schema.defined_types.get(&self.name) {
            Some(type_definition) => {
                if let TypeDefinition::Enum(content) = type_definition {
                    content
                }
                else {
                    panic!("EnumProxy expected Enum for \"{}\" but found {}", &self.name, type_definition.type_name())
                }
            },
            None => panic!("EnumProxy failed to find \"{}\"", &self.name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn option_as_str(proxy: &Option<EnumProxy>) -> &str {
        match proxy {
            Some(proxy) => proxy.as_str(),
            None => "None",
        }
    }
}

#[derive(Debug)]
pub struct ScalarProxy {
    pub name: String,
}

impl ScalarProxy {
    pub fn new(name: String) -> Self {
        ScalarProxy {
            name,
        }
    }
    
    pub fn get<'a>(&self, schema: &'a Schema) -> &'a Scalar {
        match schema.defined_types.get(&self.name) {
            Some(type_definition) => {
                if let TypeDefinition::Scalar(content) = type_definition {
                    content
                }
                else {
                    panic!("ScalarProxy expected Scalar for \"{}\" but found {}", &self.name, type_definition.type_name())
                }
            },
            None => panic!("ScalarProxy failed to find \"{}\"", &self.name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }

    pub fn option_as_str(proxy: &Option<ScalarProxy>) -> &str {
        match proxy {
            Some(proxy) => proxy.as_str(),
            None => "None",
        }
    }
}

#[derive(Debug)]
pub struct InterfaceListProxy {
    // pub schema: Rc<Schema>,
    pub names: Vec<String>,
}

impl InterfaceListProxy {
    pub fn new(names: Vec<String>) -> Self {
        InterfaceListProxy { names }
    }

    fn iterator<'a>(&'a self, schema: &'a Schema) -> InterfaceListProxyItertor<'a> {
        InterfaceListProxyItertor {
            list: self,
            schema,
            index: 0,
            // iter: self.names.iter(),
        }
    }
}

// impl<'a> IntoIterator for &'a InterfaceListProxy {
//     type Item = &'a Interface;
//     type IntoIter = InterfaceListProxyItertor<'a>;
    
//     fn into_iter(self) -> Self::IntoIter {
//         InterfaceListProxyItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

pub struct InterfaceListProxyItertor<'a> {
    list: &'a InterfaceListProxy,
    schema: &'a Schema,
    index: usize,
}

impl<'a> Iterator for InterfaceListProxyItertor<'a> {
    type Item = &'a Interface;

    fn next(&mut self) -> Option<Self::Item> {        
        match self.list.names.get(self.index) {
            Some(name) => {
                self.index += 1;
                match self.schema.defined_types.get(name) {
                    Some(type_definition) => {
                        if let TypeDefinition::Interface(object) = type_definition {
                            Some(object)
                        }
                        else {
                            panic!("Interface iterator expected Interface for \"{}\" but found {}", name, type_definition.type_name())
                        }
                    },
                    None => panic!("Interface iterator failed to find \"{}\"", name),
                }
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct ObjectListProxy {
    // pub schema: Rc<Schema>,
    pub names: Vec<String>,
}

impl ObjectListProxy {
    pub fn new(names: Vec<String>) -> Self {
        ObjectListProxy { names }
    }

    fn iterator<'a>(&'a self, schema: &'a Schema) -> ObjectListProxyItertor<'a> {
        ObjectListProxyItertor {
            list: self,
            schema,
            index: 0,
            // iter: self.names.iter(),
        }
    }
}

// impl<'a> IntoIterator for &'a ObjectListProxy {
//     type Item = &'a Interface;
//     type IntoIter = ObjectListProxyItertor<'a>;
    
//     fn into_iter(self) -> Self::IntoIter {
//         ObjectListProxyItertor {
//             list: &self,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

pub struct ObjectListProxyItertor<'a> {
    list: &'a ObjectListProxy,
    schema: &'a Schema,
    index: usize,
}

impl<'a> Iterator for ObjectListProxyItertor<'a> {
    type Item = &'a Object;

    fn next(&mut self) -> Option<Self::Item> {        
        match self.list.names.get(self.index) {
            Some(name) => {
                self.index += 1;
                match self.schema.defined_types.get(name) {
                    Some(type_definition) => {
                        if let TypeDefinition::Object(object) = type_definition {
                            Some(object)
                        }
                        else {
                            panic!("Object iterator expected Object for \"{}\" but found {}", name, type_definition.type_name())
                        }
                    },
                    None => panic!("Object iterator failed to find \"{}\"", name),
                }
            },
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    defined_types: HashMap<String, TypeDefinition>,
    query: ObjectProxy,
    mutation: Option<ObjectProxy>,
    subscription: Option<ObjectProxy>,
}

impl Schema {


    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "query        {}", self.query.as_str())?;
            writeln!(out, "mutation     {}", ObjectProxy::option_as_str(&self.mutation))?;
            writeln!(out, "subscription {}", ObjectProxy::option_as_str(&self.subscription))?;
            writeln!(out, "named_types  {{")?;
            {
                let mut out = out.indent();

                for (name, ty) in &self.defined_types {
                    write!(out, "{}\t", name)?;
                    ty.print(&mut out);
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }


    fn get_object(out: &mut Output, name: &Option<String>, position: graphql_parser::Pos, defined_types: &HashMap<String, TypeDefinition>) -> Option<ObjectProxy> {
        if let Some(name) = name {
            if let Some(type_definition) = &defined_types.get(name) {
                if let TypeDefinition::Object(object) = type_definition {
                    Some(ObjectProxy{
                        name:name.clone(),
                    })
                }
                else {
                    out.error(GraphQLError::TypeMismatchError(position, format!("Expected object \"{}\" but found {}", name, type_definition.type_name())));
                    None
                }
            }
            else {
                out.error(GraphQLError::MissingObjectError(position, format!("Missing schema reference \"{}\"", name)));
                None
            }
        }
        else {
            None
        }
    }

    fn get_default_object(out: &mut Output, name: &str, defined_types: &HashMap<String, TypeDefinition>, missing_error: Option<GraphQLError>) -> Option<ObjectProxy> {
        if let Some(type_definition) = &defined_types.get(name) {
            if let TypeDefinition::Object(object) = type_definition {
                Some(ObjectProxy{
                    name: name.to_string(),
                })
            }
            else {
                out.error(GraphQLError::TypeMismatchError(graphql_parser::Pos { line: 0, column: 0 }, format!("Expected object \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            if let Some(errror) = missing_error { 
                out.error(errror);
            }
            None
        }
    }

    fn get_known_union(&self, name: &str) -> &Union {
        if let Some(type_definition) = self.defined_types.get(name) {
            match type_definition {
                TypeDefinition::Union(union) => union,
                _ => panic!("Expected union {} but got {}", name, type_definition.type_name()),
            }
        }
        else {
            panic!("Expected union {} but found nothing", name)
        }
    }

    pub fn new(parsed: parsed_model::Schema, out: &mut Output) -> Result<Rc<Schema>, GraphQLError> {
        let mut defined_types: HashMap<String, TypeDefinition> = HashMap::new();
        
        
        for(name, ty) in &parsed.named_types {
            defined_types.insert(name.clone(), match ty {
                parsed_model::TypeDefinition::Enum(enum_definition) => TypeDefinition::Enum(Enum::new(out, enum_definition)),
                parsed_model::TypeDefinition::Union(union) => TypeDefinition::Union(Union::new(out, union, &parsed.named_types)),
                parsed_model::TypeDefinition::Object(object) => TypeDefinition::Object(Object::new(out, object, &parsed.named_types)),
                parsed_model::TypeDefinition::Interface(interface) => TypeDefinition::Interface(Interface::new(out, interface, &parsed.named_types)),
                parsed_model::TypeDefinition::Scalar(type_def) => TypeDefinition::Scalar(Scalar::new(type_def)),
            });
        }

        let query = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.query, schema_definition.position, &defined_types)
        }
        else {
            Self::get_default_object(out, "Query", &defined_types, Some(GraphQLError::NoQueryDefinition))
        };

        let mutation = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.mutation, schema_definition.position, &defined_types) 
        }
        else {
            Self::get_default_object(out, "Mutation", &defined_types, None)
        };
        
        let subscription = if let Some(schema_definition) = &parsed.schema_definition {
            Self::get_object(out, &schema_definition.subscription, schema_definition.position, &defined_types)
        }
        else {
            Self::get_default_object(out, "Subscription", &defined_types, None)
        };

        Ok(Rc::new(Schema {
            defined_types,
            query: query.ok_or(GraphQLError::ValidationError)?,
            mutation,
            subscription,
        }))
        
    }


    pub fn generate(&self, out: &mut Output) -> Result<(), GraphQLError> {
        for (name, defined_type) in &self.defined_types {
            defined_type.generate(out, self)?;
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

// #[derive(Debug)]
// pub struct VariableDefinition {
//     parsed: Rc<parsed_model::VariableDefinition>,
//     // queries: Vec<Query>,
// }

// impl VariableDefinition {
//     pub fn new(parsed: parsed_model::VariableDefinition, out: &mut Output) -> Result<VariableDefinition, GraphQLError> {


//         Ok(VariableDefinition {
//             parsed,
//         })
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         // writeln!(out, "Operations {{")?;
//         // {
//         //     let mut out = out.indent();

//         //     writeln!(out, "variables {{")?;
//         //     {
//         //         let mut out = out.indent();

//         //         for item in self.variables {
//         //             item.print(&mut out)?;
//         //         }
//         //     }
//         //     writeln!(out, "}}")?;

//         //     writeln!(out, "selections {{")?;
//         //     {
//         //         let mut out = out.indent();

//         //         for item in self.selections {
//         //             item.print(&mut out)?;
//         //         }
//         //     }
//         //     writeln!(out, "}}")?;
//         // }
//         // writeln!(out, "}}")
//         writeln!(out, "VariableDefinition is empty")
//     }
    
//     pub fn generate(&mut self, out: &mut Output) -> Result<(), GraphQLError> {
//         // if !self.variables.is_empty() {

//         //     writeln!(out, "struct {} {{", &to_pascal_case(&format!("{}Variables", &self.name)))?;
//         //     {
//         //         let mut out = out.indent();

//         //         for variable in &mut self.variables {
//         //             writeln!(out, "{}_: {},", &to_snake_case(&variable.name), &to_pascal_case(&variable.type_name))?;
//         //         }
//         //     }
//         //     writeln!(out, "}}")?;
//         // }
//         Ok(())
//     }
// }


#[derive(Debug)]
pub enum Selection {
    Field(SelectionField),
    // FragmentSpread(FragmentSpread),
    // InlineFragment(InlineFragment),
} 

impl Selection {
    pub fn new(selection: parsed_model::Selection, schema: &Schema, context: &HashMap<std::string::String, Field>, out: &mut Output) -> Selection {
        match selection {
            parsed_model::Selection::Field(selection_field) => {

            // writeln!(out, "/* Selection Field:" );
            // selection_field.print(out);
            // writeln!(out, "Context:" );
            // context.print(out);
            // writeln!(out, "*/" );

                Selection::Field(SelectionField::new(selection_field, schema, context, out))
            },
            
            // graphql_parser::query::Selection::FragmentSpread(fragment_spread) => todo!(),
            // graphql_parser::query::Selection::InlineFragment(inline_fragment) => todo!(),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::Field(selection_field) => selection_field.print(out),
        }
    }

    pub fn position(&self) -> graphql_parser::Pos {
        match self {
            Selection::Field(selection_field) => selection_field.position,
        }
    }
}



#[derive(Debug)]
pub struct SelectionField {
    pub name: String,
    pub position: graphql_parser::Pos,
    pub optional: bool,
    pub selections: Vec<Selection>,
}

impl SelectionField {
    pub fn new(parsed: parsed_model::SelectionField, schema: &Schema, context: &HashMap<std::string::String, Field>, out: &mut Output) -> SelectionField {
        let mut selections = Vec::new();

        if let Some(field) = context.get(&parsed.name) {
            if parsed.optional && field.nonnull {
                out.error(GraphQLError::OptionalNonNullFieldError(parsed.position,parsed.name.clone()));
            }

            if !selections.is_empty() {
                let optional_fields = match &field.ty {
                    Type::Int => None,
                    Type::Float => None,
                    Type::String => None,
                    Type::Boolean => None,
                    Type::ID => None,
                    Type::DefinedType(defined_type) => match defined_type {
                        DefinedType::Enum(proxy) => None,
                        DefinedType::Union(proxy) => Some(&proxy.get(schema).fields),
                        DefinedType::Object(proxy) => Some(&proxy.get(schema).fields),
                        DefinedType::Interface(proxy) => Some(&proxy.get(schema).fields),
                        DefinedType::Scalar(proxy) => None,
                    },
                };

                if let Some(fields) = optional_fields {
                    for selection in parsed.selections {
                        selections.push(Selection::new(selection, schema, &fields, out));
                    }
                }
                else {
                    out.error(GraphQLError::TypeMismatchError(parsed.position, format!("Attribute selection given on incompatible type \"{}\"", field.ty)));
                }
            }
            
        }
        else {
            out.error(GraphQLError::MissingFieldError(parsed.position,parsed.name.clone()));
        }

        SelectionField {
            name: parsed.name,
            position: parsed.position,
            optional: parsed.optional,
            selections
        }
    }



    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionField {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "optional:  {}", self.optional)?;
            // writeln!(out, "arguments {{")?;
            // {
            //     let mut out = out.indent();

            //     for argument in &self.arguments {
            //         argument.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

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
    
    pub fn generate(&mut self, out: &mut Output) -> Result<(), GraphQLError> {
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
    variables: HashMap<String, Field>,
    selections: Vec<Selection>,
}

impl Query {
    pub fn new(parsed: parsed_model::Query, schema:  &Schema, out: &mut Output) ->  Result<Query, GraphQLError> {
        let mut variables = HashMap::new();

        for variable in &parsed.variables {
            if let Some(existing) = variables.insert(variable.name.clone(), Field::from_variable(out, variable, schema)) {
                out.error(GraphQLError::DuplicateName(existing.position, variable.position, variable.name.clone()));
            }
        }

        let mut selections = Vec::new();
        for selection in parsed.selections {
            selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
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
    
    pub fn generate(&self, out: &mut Output) -> Result<(), GraphQLError> {
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
    pub fn new(parsed: parsed_model::Operations, out: &mut Output, schema: &Schema) ->  Result<Operations, GraphQLError>{
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

    pub fn generate(&self, out: &mut Output) -> Result<(), GraphQLError> {
        for query in &self.queries {
            query.generate(out)?;
        }
        Ok(())
    }
}


