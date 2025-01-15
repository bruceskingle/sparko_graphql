use std::fmt::Display;
use std::rc::Rc;
use std::collections::HashMap;
use std::io::Write;

use inflections::case::{to_snake_case, to_constant_case};

use crate::utils::to_pascal_case;
use crate::{error::GraphQLError, parsed_model, Output};

const TYPE_NAME: &str = "__typename";

#[derive(Debug, Clone, Copy)]
pub enum Maybe {
    True,
    False,
    Maybe,
}

impl Display for Maybe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Maybe::True => writeln!(f, "True"),
            Maybe::False => writeln!(f, "False"),
            Maybe::Maybe => writeln!(f, "Maybe"),
        }
    }
}

impl Maybe {
    pub fn isit(&self, default: bool) -> bool {
        match self {
            Maybe::True => true,
            Maybe::False => false,
            Maybe::Maybe => default,
        }
    }
}

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
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Schema) -> Result<(), GraphQLError> {
        writeln!(out, "type {} = {};", &self.rust_name, &self.rust_type)?;
        writeln!(out, "")?;
        Ok(())
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
            TypeDefinition::Enum(_) => "enum",
            TypeDefinition::Union(_) => "union",
            TypeDefinition::Object(_) => "object",
            TypeDefinition::Interface(_) => "interface",
            TypeDefinition::Scalar(_) => "scalar",
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

    pub fn generate(&self, out: &mut Output, schema: &Schema, selections: Option<&Vec<Selection>>) -> Result<(), GraphQLError> {
        match self {
            TypeDefinition::Enum(content) => content.generate(out, schema, selections),
            TypeDefinition::Union(content) => content.generate(out, schema, selections, &None),
            TypeDefinition::Object(content) => content.generate(out, schema, selections, &None),
            TypeDefinition::Interface(content) => content.generate(out, schema, selections, &None),
            TypeDefinition::Scalar(content) => content.generate(out, schema),
        }
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

impl DefinedType {
    pub fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), GraphQLError> {
        match self {
            DefinedType::Enum(proxy) => proxy.get(schema).generate(out, schema, selections,),
            DefinedType::Union(proxy) => proxy.get(schema).generate(out, schema, selections, alias),
            DefinedType::Object(proxy) => proxy.get(schema).generate(out, schema, selections, alias),
            DefinedType::Interface(proxy) => proxy.get(schema).generate(out, schema, selections, alias),
            DefinedType::Scalar(proxy) => proxy.get(schema).generate(out, schema),
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
                        parsed_model::TypeDefinition::Object(content) =>Type::DefinedType(DefinedType::Object(ObjectProxy::new(content.name.clone()))),
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

    fn graphql_name(&self, multiple: bool, nonnull: bool) -> String {
        let base_type = match self {
            Type::Int => "Int",
                Type::Float => "Float",
                Type::String => "String",
                Type::Boolean => "Boolean",
                Type::ID => "ID",
                Type::DefinedType(defined_type) => match defined_type {
                    DefinedType::Enum(proxy) => &proxy.name,
                    DefinedType::Union(proxy) => &proxy.name,
                    DefinedType::Object(proxy) => &proxy.name,
                    DefinedType::Interface(proxy) => &proxy.name,
                    DefinedType::Scalar(proxy) => &proxy.name,
                },
        };

        let type2 = if nonnull {
            format!("{}!", base_type)
        }
        else {
            base_type.to_string()
        };

        if multiple {
            format!("[{}]", type2)
        }
        else {
            type2
        }
    }
     
    fn rust_type(&self, multiple: bool, nonnull: bool, schema: &Schema, maybe_optional: Maybe, alias: &Option<String>) -> String {
        let base_type = 
        match alias {
            Some(alias) => to_pascal_case(alias),
            None =>
             match self {
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
        }
            
        };

        let type2 = if multiple {
            format!("Vec<{}>", base_type)
        }
        else {
            base_type
        };

        if maybe_optional.isit(!nonnull) {
            format!("Option<{}>", type2)
        }
        else {
            type2
        }
        // if nonnull || !maybe_optional {
        //     type2
        // }
        // else {
        //     format!("Option<{}>", type2)
        // }
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

    pub fn rust_type(&self, schema: &Schema, maybe_optional: Maybe, alias: &Option<String>) -> String {
        self.ty.rust_type(self.multiple, self.nonnull, schema, maybe_optional, alias)
    }

    pub fn graphql_name(&self) -> String {
        self.ty.graphql_name(self.multiple, self.nonnull)
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

    pub fn new(_out: &mut Output, parsed: &parsed_model::Enum) -> Enum {
        Enum {
            position: parsed.position,
            name: parsed.name.clone(),
            variants: parsed.variants.clone(),
        }
    }
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Schema, _selections: Option<&Vec<Selection>>) -> Result<(), GraphQLError> {
        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        writeln!(out, "#[serde(rename = \"{}\")]", self.name)?;
        writeln!(out, "pub enum {} {{", to_pascal_case(&self.name))?;

        for name in &self.variants {
            writeln!(out, "    #[serde(rename = \"{}\")]", name)?;
            writeln!(out, "    {},", to_constant_case(name))?;
        }
        writeln!(out, "}}")?;
        writeln!(out, "")?;
        Ok(())
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

                for (_name, field) in &self.fields {
                    field.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    fn new(out: &mut Output, parsed: &parsed_model::Union, named_types: &HashMap<String, parsed_model::TypeDefinition>) -> Union {
        let mut fields = HashMap::new();

        for type_name in &parsed.types {
            if let Some(defined_type) = named_types.get(type_name) {
                if let parsed_model::TypeDefinition::Object(interface) = defined_type {
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
    
    fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), GraphQLError> {
        generate_struct(out, schema, selections, alias, &self.name, &self.fields, false)

        // writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        // writeln!(out, "#[serde(rename = \"{}\")]", &self.name)?;
        // writeln!(out, "pub struct {} {{", to_pascal_case(&self.name))?;

        // if let Some(selected_fields) = selected_fields {
        //     for (name, field) in &self.fields {
        //         if let Some(selection_field) = selected_fields.get(name) {
        //             writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
        //             writeln!(out, "    /* BRUCE */ {}_: {},", to_snake_case(&name), field.rust_type(schema, selection_field.optional.clone(), &selection_field.alias))?;
        //         }
        //     }
        // }
        // else {
        //     for (name, field) in &self.fields {
        //         writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
        //         writeln!(out, "    /* BRUCE2 */ {}_: {},", to_snake_case(&name), field.rust_type(schema, true, &None))?;
        //     }
        // }
        // writeln!(out, "}}")?;
        // writeln!(out, "")?;
        // Ok(())
    }
}


#[derive(Debug)]
pub struct Object {
    pub position: graphql_parser::Pos,
    pub name: String,
    pub fully_implements: InterfaceListProxy,
    pub fields: HashMap<String, Field>,
    pub is_input: bool,
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

                for (_, field) in &self.fields {
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
            is_input: parsed.is_input,
        }
    }
    
    fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), GraphQLError> {
        generate_struct(out, schema, selections, alias, &self.name, &self.fields, self.is_input)
        // let name = if let Some(alias) = &alias {
        //     to_pascal_case(alias)
        //     // format!("{}{}", to_pascal_case(alias),to_pascal_case(&self.name))
        // }
        // else {
        //     to_pascal_case(&self.name)
        // };
        
        // writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        // writeln!(out, "#[serde(rename = \"{}\")]", &self.name)?;
        // writeln!(out, "pub struct {} {{", name)?;

        // if let Some(selected_fields) = selected_fields {

        //     for (k, v) in &selected_fields {
        //         writeln!(out, "//selected_fields {} = {} {} {}", k, v.name, v.optional, match &v.alias {
        //             Some(alias) => alias,
        //             None => "None",
        //         })?;
        //     }

        //     for (name, field) in &self.fields {
        //         // writeln!(out, "// T2 {}", name);
        //         // field.print(out);
        //         if let Some(selection_field) = selected_fields.get(name) {
        //             writeln!(out, "// T3 {} {}", name, selection_field.optional)?;
        //             writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
        //             writeln!(out, "    /* BRUCE3 */ {}_: {},", to_snake_case(&name), field.rust_type(schema, selection_field.optional.clone(), &selection_field.alias))?;
        //         }
        //         writeln!(out, "// T4 {}", name)?;
        //     }
        // }
        // else {
        //     for (name, field) in &self.fields {
        //         writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
        //         writeln!(out, "    {}_: {},", to_snake_case(&name), field.rust_type(schema, true, &None))?;
        //     }
        // }
        
        // writeln!(out, "}}")?;
        // writeln!(out, "")?;
        // Ok(())
    }
}

fn selections_to_fields<'a>(selections: &'a Vec<Selection>) -> Vec<&'a SelectionField> {
    let mut selected_fields = Vec::new();
    for selection in selections {
        match selection {
            Selection::Field(selection_field) => {
                selected_fields.push(selection_field);
                // writeln!(out, "// selected field {} = {}", selection_field.name, selection_field.optional)?;
            },
        };
    }
    selected_fields
}

fn generate_struct(out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>, name: &str, fields: &HashMap<String, Field>, is_input: bool) -> Result<(), GraphQLError> {
    let rust_name = if let Some(alias) = &alias {
        to_pascal_case(alias)
        // format!("{}{}", to_pascal_case(alias),to_pascal_case(&self.name))
    }
    else {
        to_pascal_case(name)
    };


    
    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
    writeln!(out, "#[serde(rename = \"{}\")]", name)?;
    writeln!(out, "pub struct {} {{", rust_name)?;

    if let Some(selections) = selections {
        let selected_fields = selections_to_fields(selections);

        for selection_field in &selected_fields {
            writeln!(out, "// selection_field {}", &selection_field.name)?;
        }

        for selection_field in &selected_fields {
            if &selection_field.name == TYPE_NAME {
                writeln!(out, "    #[serde(rename = \"{}\")]", &selection_field.name)?;
                writeln!(out, "    {}: String,", &selection_field.name)?;
            }
            else {
                if let Some(field) = fields.get(&selection_field.name) {
                    // writeln!(out, "// T3 {} {}", name, selection_field.optional)?;
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
                    writeln!(out, "    pub {}_: {},", to_snake_case(&selection_field.name), field.rust_type(schema, selection_field.optional, &selection_field.alias))?;
                }
                else {
                    writeln!(out, "UNKNOWN FIELD 1 {}", &selection_field.name)?;
                }
            }
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        for selection_field in &selected_fields {
            if &selection_field.name == TYPE_NAME {}
            else {
                if let Some(field) = fields.get(&selection_field.name) {
                    if let Type::DefinedType(defined_type) = &field.ty {
                        defined_type.generate(out, schema, Some(&selection_field.selections), &selection_field.alias)?;
                    }
                }
                else {
                    writeln!(out, "UNKNOWN FIELD 2 {}", &selection_field.name)?;
                }
            }
        }

        // if is_input {
        //     writeln!(out, "#[derive(Debug)]")?;
        //     writeln!(out, "pub struct {}Builder {{", rust_name)?;

        //     for selection_field in &selected_fields {
        //         // if &selection_field.name == TYPE_NAME {
        //         //     writeln!(out, "    #[serde(rename = \"{}\")]", name)?;
        //         //     writeln!(out, "    {}: String,", name)?;
        //         // }
        //         // else {
        //             if let Some(field) = fields.get(&selection_field.name) {
        //                 writeln!(out, "    {}_: {},", to_snake_case(&selection_field.name), field.rust_type(schema, Maybe::True, &selection_field.alias))?;
        //             }
        //             else {
        //                 writeln!(out, "UNKNOWN FIELD 3 {}", &selection_field.name)?;
        //             }
        //         // }
        //     }
        
        //     writeln!(out, "}}")?;
        //     writeln!(out, "")?;

        //     for selection_field in &selected_fields {
        //         if name == TYPE_NAME {}
        //         else {
        //             if let Some(field) = fields.get(&selection_field.name) {
        //                 if let Type::DefinedType(defined_type) = &field.ty {
        //                     defined_type.generate(out, schema, Some(&selection_field.selections), &selection_field.alias);
        //                 }
        //             }
        //             else {
        //                 writeln!(out, "UNKNOWN FIELD 4 {}", &selection_field.name)?;
        //             }
        //         }
        //     }
        // }
    }
    else {
        for (name, field) in fields {
            writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
            writeln!(out, "    pub {}_: {},", to_snake_case(&name), field.rust_type(schema, Maybe::Maybe, &None))?;
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        if is_input {
            writeln!(out, "impl {} {{", rust_name)?;
            {
                let mut out = out.indent();

                writeln!(out, "pub fn builder() -> {}Builder {{", rust_name)?;
                writeln!(out, "    {}Builder {{", rust_name)?;
                for (name, field) in fields {
                    writeln!(out, "        {}_: None,", to_snake_case(&name))?;
                }
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "#[derive(Debug)]")?;
            writeln!(out, "pub struct {}Builder {{", rust_name)?;

            for (name, field) in fields {
                writeln!(out, "    {}_: {},", to_snake_case(&name), field.rust_type(schema, Maybe::True, &None))?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl {}Builder {{", rust_name)?;
            {
                let mut out = out.indent();

                for (name, field) in fields {
                    writeln!(out, "pub fn with_{}(mut self, value: {}) -> Self {{", to_snake_case(&name), field.rust_type(schema, Maybe::False, &None))?;
                    {
                        let mut out = out.indent();

                        writeln!(out, "self.{}_ = Some(value);", to_snake_case(&name))?;
                        writeln!(out, "self")?;
                    }
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;
                }
                writeln!(out, "pub fn build(self) -> Result<{}, sparko_graphql::error::Error> {{", rust_name)?;
                {
                    let mut out = out.indent();

                    for (name, field) in fields {
                        if field.nonnull {
                            writeln!(out, "if let None = self.{}_ {{", to_snake_case(&name))?;
                            writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", name)?;
                            writeln!(out, "}}")?;
                        }
                    }

                    writeln!(out, "Ok({} {{", rust_name)?;
                    {
                        let mut out = out.indent();
    
                        for (name, field) in fields {
                            if field.nonnull {
                                writeln!(out, "{}_: self.{}_.unwrap(),", to_snake_case(&name), to_snake_case(&name))?;
                            }
                            else {
                                writeln!(out, "{}_: self.{}_,", to_snake_case(&name), to_snake_case(&name))?;
                            }
                        }
                        writeln!(out, "}})")?;
                    }
                }
                writeln!(out, "}}")?;
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;
        }
    };

    Ok(())
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

        for (_, defined_type) in named_types {
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
    
    fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), GraphQLError> {
        if let Some(_) = selections {
            generate_struct(out, schema, selections, alias, &self.name, &self.fields, false)
        }
        else {
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "pub enum {} {{", to_pascal_case(&self.name))?;

            for object in self.implemented_by.iterator(schema) {
                
                writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
            }

            writeln!(out, "}}")?;
            writeln!(out, "")?;
            Ok(())
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

    // pub fn as_str(&self) -> &str {
    //     &self.name
    // }

    // pub fn option_as_str(proxy: &Option<InterfaceProxy>) -> &str {
    //     match proxy {
    //         Some(proxy) => proxy.as_str(),
    //         None => "None",
    //     }
    // }
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

    // pub fn as_str(&self) -> &str {
    //     &self.name
    // }

    // pub fn option_as_str(proxy: &Option<UnionProxy>) -> &str {
    //     match proxy {
    //         Some(proxy) => proxy.as_str(),
    //         None => "None",
    //     }
    // }
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

    // pub fn as_str(&self) -> &str {
    //     &self.name
    // }

    // pub fn option_as_str(proxy: &Option<EnumProxy>) -> &str {
    //     match proxy {
    //         Some(proxy) => proxy.as_str(),
    //         None => "None",
    //     }
    // }
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

    // pub fn as_str(&self) -> &str {
    //     &self.name
    // }

    // pub fn option_as_str(proxy: &Option<ScalarProxy>) -> &str {
    //     match proxy {
    //         Some(proxy) => proxy.as_str(),
    //         None => "None",
    //     }
    // }
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

    fn _iterator<'a>(&'a self, schema: &'a Schema) -> InterfaceListProxyItertor<'a> {
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
                    ty.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }


    fn get_object(out: &mut Output, name: &Option<String>, position: graphql_parser::Pos, defined_types: &HashMap<String, TypeDefinition>) -> Option<ObjectProxy> {
        if let Some(name) = name {
            if let Some(type_definition) = &defined_types.get(name) {
                if let TypeDefinition::Object(_) = type_definition {
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
            if let TypeDefinition::Object(_) = type_definition {
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

    // fn get_known_union(&self, name: &str) -> &Union {
    //     if let Some(type_definition) = self.defined_types.get(name) {
    //         match type_definition {
    //             TypeDefinition::Union(union) => union,
    //             _ => panic!("Expected union {} but got {}", name, type_definition.type_name()),
    //         }
    //     }
    //     else {
    //         panic!("Expected union {} but found nothing", name)
    //     }
    // }

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
        for (_, defined_type) in &self.defined_types {
            defined_type.generate(out, self, None)?;
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
    pub fn new(selection: parsed_model::Selection, schema: &Schema, context: &HashMap<std::string::String, Field>, out: &mut Output) -> Result<Selection, GraphQLError> {
        match selection {
            parsed_model::Selection::Field(selection_field) => {

            // writeln!(out, "/* Selection Field:" );
            // selection_field.print(out);
            // writeln!(out, "Context:" );
            // context.print(out);
            // writeln!(out, "*/" );

                Ok(Selection::Field(SelectionField::new(selection_field, schema, context, out)?))
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

    // pub fn position(&self) -> graphql_parser::Pos {
    //     match self {
    //         Selection::Field(selection_field) => selection_field.position,
    //     }
    // }

    pub fn generate_query(&self, out: &mut Output, context: &HashMap<String, Field>, schema: &Schema, maybe_optional: Maybe) -> Result<(), GraphQLError> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_query(out, context, schema, maybe_optional),
        }
    }

    pub fn generate_fields(&self, out: &mut Output, context: &HashMap<String, Field>, schema: &Schema, maybe_optional: Maybe) -> Result<(), GraphQLError> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_fields(out, context, schema, maybe_optional),
        }
    }
    
    fn generate_structs(&self, out: &mut Output<'_>, context: &HashMap<String, Field>, schema: &Schema) -> Result<(), GraphQLError> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_structs(out, context, schema),
        }
    }
}




// #[derive(Debug)]
// pub struct Argument {
//     pub name: String,
//     pub value: parsed_model::Value,
// }

// impl Argument {
    
//     fn new(prsed_argument: parsed_model::Argument) -> Self {
//         Self {
//             name: prsed_argument.name,
//             value: prsed_argument.value,
//         }
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "Argument {{")?;
//         {
//             let mut out = out.indent();

//             writeln!(out, "name:      {}", self.name)?;
//             writeln!(out, "value:     {}", self.value)?;
//         }
//         writeln!(out, "}}")
//     }
// }

#[derive(Debug)]
pub struct SelectionField {
    pub name: String,
    pub alias: Option<String>,
    pub position: graphql_parser::Pos,
    pub optional: Maybe,
    pub arguments: Vec<parsed_model::Argument>,
    pub selections: Vec<Selection>,
}

impl SelectionField {
    pub fn new(parsed: parsed_model::SelectionField, schema: &Schema, context: &HashMap<std::string::String, Field>, out: &mut Output) -> Result<SelectionField, GraphQLError> {
        let mut arguments = Vec::new();
        
        for parsed_argument in parsed.arguments {
            // arguments.push(Argument::new(prsed_argument));
            arguments.push(parsed_argument);
        }
        
        let mut selections = Vec::new();

        if parsed.name == TYPE_NAME {
            println!("HERE1");
        }
        else  {
                if let Some(field) = context.get(&parsed.name) {
                    if parsed.optional && field.nonnull {
                        out.error(GraphQLError::OptionalNonNullFieldError(parsed.position,parsed.name.clone()));
                    }
        
                    if !parsed.selections.is_empty() {
                        let optional_fields = match &field.ty {
                            Type::Int => None,
                            Type::Float => None,
                            Type::String => None,
                            Type::Boolean => None,
                            Type::ID => None,
                            Type::DefinedType(defined_type) => match defined_type {
                                DefinedType::Enum(_) => None,
                                DefinedType::Union(proxy) => Some(&proxy.get(schema).fields),
                                DefinedType::Object(proxy) => Some(&proxy.get(schema).fields),
                                DefinedType::Interface(proxy) => Some(&proxy.get(schema).fields),
                                DefinedType::Scalar(_) => None,
                            },
                        };
        
                        if let Some(fields) = optional_fields {
                            for selection in parsed.selections {
                                match Selection::new(selection, schema, &fields, out) {
                                    Ok(selection) => selections.push(selection),
                                    Err(error) => out.error(error),
                                };
                            }
                        }
                        else {
                            out.error(GraphQLError::TypeMismatchError(parsed.position, format!("Attribute selection given on incompatible type \"{}\"", field.ty)));
                        }
                    }
                    
                }
                else {
                    return Err(GraphQLError::MissingFieldError(parsed.position,parsed.name.clone()));
                }
            }
        
        let optional = if parsed.optional {
            Maybe::Maybe
        }
        else {
            Maybe::False
        };

        Ok(SelectionField {
            name: parsed.name,
            alias: parsed.alias,
            position: parsed.position,
            optional: optional,
            arguments,
            selections,
        })
    }



    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionField {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
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

    pub fn generate_query(&self, out: &mut Output, context: &HashMap<String, Field>, schema: &Schema, maybe_optional: Maybe) -> Result<(), GraphQLError> {
        // let field = context.get(&self.name).unwrap();
        if let Some(alias) = &self.alias {
            writeln!(out, "{}: {}", alias, &self.name)?;
        }
        else {
            writeln!(out, "{}", &self.name)?;
        }

        if ! self.arguments.is_empty() {
            writeln!(out, "(")?;
            {
                let mut out = out.indent();

            
                for argument in &self.arguments {
                    argument.generate_query(&mut out, &context, schema, true)?;
                }
            }
            writeln!(out, ")")?;
        }

        if ! &self.selections.is_empty() {
            writeln!(out, "{{")?;
            {
                let mut out = out.indent();

            
                for selection in &self.selections {
                    selection.generate_query(&mut out, &context, schema, Maybe::Maybe)?;
                }
            }
            writeln!(out, "}}")?;
        }
        Ok(())
    }

    pub fn generate_fields(&self, out: &mut Output, context: &HashMap<String, Field>, schema: &Schema, maybe_optional: Maybe) -> Result<(), GraphQLError> {
        if &self.name == TYPE_NAME {
            println!("HERE");
        }
        let field = context.get(&self.name).unwrap();
        let name = if let Some(alias) = &self.alias {
            alias
        }
        else {
            &self.name
        };

        writeln!(out, "#[serde(rename = \"{}\")]", &name)?;
        writeln!(out, "/* HERE1 */ pub {}: {},", to_snake_case(&name), field.rust_type(schema, maybe_optional, &self.alias))?;
        Ok(())
    }

    pub fn generate_structs(&self, out: &mut Output, context: &HashMap<String, Field>, schema: &Schema) -> Result<(), GraphQLError> {
        let field = context.get(&self.name).unwrap();

        if let Type::DefinedType(defined_type) = &field.ty {
            writeln!(out, "// {} is {}", &self.name, defined_type)?;
            
            defined_type.generate(out, schema, Some(&self.selections), &self.alias)?;

            // let fields = defined_type.fields();

            // writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // writeln!(out, "#[serde(rename = \"{}\")]", defined_type.name)?;
            // writeln!(out, "pub struct {} {{", name)?;
            // {
            //     let mut out = out.indent();

            //     for selection in &self.selections {
            //         // let context: HashMap<String, Field> = schema.query.get(schema).fields;

            //         let ty: &TypeDefinition = schema.defined_types.get(&self.name).unwrap();
            //         let x = ty.
            //         selection.generate_fields(&mut out, &schema.defined_types.get(&self.name).unwrap().fields, schema)?;
            //         // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
            //         // writeln!(out, "    {},", to_constant_case(&selection.name))?;
            //     }
            // }
            // writeln!(out, "}}")?;
            // writeln!(out, "")?;
        }
        else {
            writeln!(out, "// Nothing to generate because {} is {}", &self.name, field.ty)?;
        }
        // if !self.selections.is_empty() {
        //     let name = to_pascal_case(&self.name);
        //     self.

            

        //     for selection in &self.selections {
        //         // let context: HashMap<String, Field> = schema.query.get(schema).fields;

        //         selection.generate_structs(out, &schema.query.get(schema).fields, schema)?;
        //         // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
        //         // writeln!(out, "    {},", to_constant_case(&selection.name))?;
        //     }

        // }
        Ok(())
    }
    
    // pub fn generate(&mut self, out: &mut Output) -> Result<(), GraphQLError> {
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
    pub name: String,
    pub position: graphql_parser::Pos,
    pub operation: OperationType,
    // context: &HashMap<String, Field>,
    variables: HashMap<String, Field>,
    selections: Vec<Selection>,
}

impl GenericOperation {
    pub fn from_query(parsed: parsed_model::Query, schema:  &Schema, out: &mut Output) ->  Result<GenericOperation, GraphQLError> {
        Self::new(OperationType::Query, 
            parsed.name, parsed.position,
            parsed.variables, parsed.selections, schema, out)
    }

    pub fn from_mutation(parsed: parsed_model::Mutation, schema:  &Schema, out: &mut Output) ->  Result<GenericOperation, GraphQLError> {
        Self::new(OperationType::Mutation, 
            parsed.name, parsed.position,
            parsed.variables, parsed.selections, schema, out)
    }
    
    // pub fn from_subscription(parsed: graphql_parser::Subscription, schema:  &Schema, out: &mut Output) ->  Result<GenericOperation, GraphQLError> {
    //     Self::new("Subscription", 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema, out)
    // }

    fn get_context2<'a>(operation: &OperationType, name: &String, position: &graphql_parser::Pos, schema: &'a Schema) -> Result<&'a HashMap<String, Field>, GraphQLError> {
        match operation {
            OperationType::Query => Ok(&schema.query.get(schema).fields),
            OperationType::Mutation => {
                if let Some(mutation_proxy) = &schema.mutation {
                    Ok(&mutation_proxy.get(schema).fields)
                }
                else {
                    Err(GraphQLError::MissingObjectError(position.clone(), format!("Mutation {} used but no Mutation root found", name)))
                }
            },
        }
    }

    fn get_context<'a>(&self, schema: &'a Schema) -> Result<&'a HashMap<String, Field>, GraphQLError> {
        Self::get_context2(&self.operation, &self.name, &self.position, schema)
    }

    fn new(operation: OperationType, name: String, position: graphql_parser::Pos, parsed_variables: Vec<parsed_model::Field>, parsed_selections: Vec<parsed_model::Selection>, schema: &Schema, out: &mut Output<'_>) -> Result<GenericOperation, GraphQLError> {

        let mut variables = HashMap::new();

        for variable in &parsed_variables {
            if let Some(existing) = variables.insert(variable.name.clone(), Field::from_variable(out, variable, schema)) {
                out.error(GraphQLError::DuplicateName(existing.position, variable.position, variable.name.clone()));
            }
        }

        let context= Self::get_context2(&operation, &name, &position, schema)?;
        
        let mut selections = Vec::new();
        for selection in parsed_selections {
            
            // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
            match Selection::new(selection, schema, context, out) {
                Ok(selection) => selections.push(selection),
                Err(error) => out.error(error),
            };
        }

        Ok(GenericOperation {
            // parsed,
            name,
            position,
            operation,
            // context,
            variables,
            selections,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{} {{", self.operation)?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
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
    
    pub fn generate(&self, out: &mut Output, schema: &Schema) -> Result<(), GraphQLError> {
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

        // let name = to_pascal_case(&self.name);

        let context= self.get_context(schema)?;

        writeln!(out, "pub mod {} {{", to_snake_case(&self.name))?;
        {
            let mut out = out.indent();
            writeln!(out, r#"
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
use sparko_graphql::{{NewGraphQLResponse, NewGraphQLQuery}};
"#
            )?;


            if !self.variables.is_empty() {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "pub struct Variables {{")?;

                {
                    let mut out = out.indent();
                
                    for (name, field) in &self.variables {
                        writeln!(out, "#[serde(rename = \"{}\")]", &field.name)?;
                        writeln!(out, "{}_: {},", to_snake_case(&name), field.rust_type(schema, Maybe::Maybe, &None))?;
                    }
                }

                writeln!(out, "}}")?;
                writeln!(out, "")?;
            }

            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // writeln!(out, "#[serde(rename = \"{}\")]", self.name)?;
            writeln!(out, "pub struct {} {{", self.operation)?;

            if !self.variables.is_empty() {
                writeln!(out.indent(), "variables: Variables,")?;
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl {} {{", self.operation)?;
            {
                let mut out = out.indent();

                writeln!(out, "const REQUEST_NAME: &str = \"{}\";", self.name)?;

                if self.variables.is_empty() {
                    writeln!(out, "const {}: &str = r#\"{} {} {{", self.operation.to_upper_case(), self.operation.to_lower_case(), self.name)?;
                }
                else {
                    writeln!(out, "const {}: &str = r#\"{} {}(", self.operation.to_upper_case(), self.operation.to_lower_case(), self.name)?;
                    {
                        let mut out = out.indent();
                    
                        for (name, field) in &self.variables {
                            writeln!(out, "${}: {},", to_snake_case(&name), field.graphql_name()
                            // field.rust_type(schema, Maybe::Maybe, &None)
                            )?;
                        }
                    }
                    writeln!(out, ") {{")?;
                }
                
                {
                    let mut out = out.indent();
    
                    for selection in &self.selections {
                        selection.generate_query(&mut out, &context, schema, Maybe::Maybe)?;
                    }
                }
                writeln!(out, "}}\"#;")?;
                writeln!(out, "")?;

                if self.variables.is_empty() {
                    writeln!(out, "pub fn new() -> {} {{", self.operation)?;
                    writeln!(out.indent(), "{} {{}}", self.operation)?;
                    writeln!(out, "}}")?;
                }
                else {

                    writeln!(out, "pub fn from(variables: Variables) -> {} {{", self.operation)?;
                    writeln!(out.indent(), "{} {{variables}}", self.operation)?;
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;

                    writeln!(out, "pub fn new(")?;
                    {
                        let mut out = out.indent();
                    
                        for (name, field) in &self.variables {
                            writeln!(out, "{}_: {},", to_snake_case(&name), field.rust_type(schema, Maybe::Maybe, &None))?;
                        }
                    }
                    writeln!(out, ") -> {} {{", self.operation)?;
                    {
                        let mut out = out.indent();

                        writeln!(out, "{} {{", self.operation)?;
                        {
                            writeln!(out, "variables: Variables {{")?;
                            {
                                let mut out = out.indent();
        
                                for (name, field) in &self.variables {
                                    writeln!(out, "{}_,", to_snake_case(&name))?;
                                }
                            }
                            writeln!(out, "}}")?;
                        }
                        writeln!(out, "}}")?;
                    }
                    writeln!(out, "}}")?;
                }
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl NewGraphQLQuery<Response> for {} {{", self.operation)?;
            {
                let mut out = out.indent();

                writeln!(out, "fn get_request_name() -> &'static str {{")?;
                writeln!(out.indent(), "Self::REQUEST_NAME")?;
                writeln!(out, "}}")?;

                writeln!(out, "fn get_query() -> &'static str {{")?;
                writeln!(out.indent(), "Self::{}", self.operation.to_upper_case())?;
                writeln!(out, "}}")?;

                writeln!(out, "fn get_variables(&self) -> Result<std::string::String, serde_json::Error> {{")?;
                if self.variables.is_empty() {
                    writeln!(out.indent(), "Ok(String::from(\"{{}}\"))")?;
                }
                else {
                    writeln!(out.indent(), "serde_json::to_string_pretty(&self.variables)")?;
                }
                writeln!(out, "}}")?;
            }
            writeln!(out, "}}")?;

            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // writeln!(out, "#[serde(rename = \"{}\")]", self.name)?;
            writeln!(out, "pub struct Response {{")?;
            {
                let mut out = out.indent();

                for selection in &self.selections {
                    // let context: HashMap<String, Field> = schema.query.get(schema).fields;

                    selection.generate_fields(&mut out, &context, schema, Maybe::Maybe)?;
                    // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
                    // writeln!(out, "    {},", to_constant_case(&selection.name))?;
                }
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl NewGraphQLResponse for Response {{")?;
            writeln!(out, "}}")?;
            

            for selection in &self.selections {
                // let context: HashMap<String, Field> = schema.query.get(schema).fields;

                selection.generate_structs(&mut out, &context, schema)?;
                // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
                // writeln!(out, "    {},", to_constant_case(&selection.name))?;
            }

            for (name, field) in &self.variables {
                if let Type::DefinedType(defined_type) = &field.ty {
                    defined_type.generate(&mut out, schema, None, &None)?;
                }
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct Operations {
    name: String,
    queries: Vec<GenericOperation>,
    mutations: Vec<GenericOperation>,
}

impl Operations {
    pub fn new(parsed: parsed_model::Operations, out: &mut Output, schema: &Schema) ->  Result<Operations, GraphQLError>{

        let mut queries = Vec::new();

        for query in parsed.queries {
            queries.push(GenericOperation::from_query(query, schema, out)?);
        }

        let mut mutations = Vec::new();

        for mutation in parsed.mutations {
            mutations.push(GenericOperation::from_mutation(mutation, schema, out)?);
        }

        Ok(Operations {
            name: parsed.name,
            queries,
            mutations,
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

    pub fn generate(&self, out: &mut Output, schema: &Schema) -> Result<(), GraphQLError> {
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
        writeln!(out, "}} // End of operations {}", self.name)?;
        Ok(())
    }
}


