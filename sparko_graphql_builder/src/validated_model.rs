use std::fmt::Display;
use std::io::Write;

use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::{to_snake_case, to_constant_case};

use crate::parsed_model::{BuiltinType, OperationType};
use crate::utils::to_pascal_case;
use crate::{BuildError, Error, ErrorCollector};
use crate::{parsed_model, Output};

const TYPE_NAME: &str = "__typename";
pub type FieldMap<'a> = IndexMap<&'a str, Field<'a>>;
pub type FieldMapRef<'a> = &'a FieldMap<'a>;


#[derive(Debug)]
pub enum ScalarType<'a> {
    DefinedType(DefinedType<'a>),
    BuiltinType(BuiltinType),
}

impl<'a> ScalarType<'a> {
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::ScalarType<'a>, schema: &'a parsed_model::Schema) -> Result<Self, Error> {
        match parsed {
            parsed_model::ScalarType::DefinedType{name, position} => {
                if let Some(def) = schema.named_types.get(name) {
                    Ok(match def {
                        parsed_model::TypeDefinition::Scalar(content) =>ScalarType::DefinedType(DefinedType::Scalar(content.name)),
                        parsed_model::TypeDefinition::Object(content) =>ScalarType::DefinedType(DefinedType::Object(content.name)),
                        parsed_model::TypeDefinition::Interface(content) =>ScalarType::DefinedType(DefinedType::Interface(content.name)),
                        parsed_model::TypeDefinition::Union(content) =>ScalarType::DefinedType(DefinedType::Union(content.name)),
                        parsed_model::TypeDefinition::Enum(content) => ScalarType::DefinedType(DefinedType::Enum(content.name)),
                    })
                }
                else {
                    err.fail(BuildError::UndefinedTypeError(*position.clone(), format!("Missing type {}", name)))
                }
            },
            parsed_model::ScalarType::BuiltinType(builtin_type) => Ok(ScalarType::BuiltinType(*builtin_type)),
        }
    }

    fn from_validated(err: &mut ErrorCollector, parsed: &parsed_model::ScalarType, schema: &'a Schema) -> Result<Self, Error> {
        match parsed {
            parsed_model::ScalarType::DefinedType{name, position} => {
                if let Some(def) = schema.defined_types.get(*name) {
                    Ok(match def {
                        TypeDefinition::Scalar(content) =>ScalarType::DefinedType(DefinedType::Scalar(content.parsed.name)),
                        TypeDefinition::Object(content) =>ScalarType::DefinedType(DefinedType::Object(content.parsed.name)),
                        TypeDefinition::Interface(content) =>ScalarType::DefinedType(DefinedType::Interface(content.parsed.name)),
                        TypeDefinition::Union(content) =>ScalarType::DefinedType(DefinedType::Union(content.parsed.name)),
                        TypeDefinition::Enum(content) => ScalarType::DefinedType(DefinedType::Enum(content.parsed.name)),
                    })
                }
                else {
                    err.fail(BuildError::UndefinedTypeError(*position.clone(), format!("Missing type {}", name)))
                }
            },
            parsed_model::ScalarType::BuiltinType(builtin_type) => Ok(ScalarType::BuiltinType(*builtin_type)),
        }
    }
}

impl Display for ScalarType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::DefinedType(defined_type) => write!(f, "{}", defined_type),
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
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Type, schema: &'a parsed_model::Schema) -> Result<Self, Error> {
       Ok( match parsed {
            parsed_model::Type::Scalar(scalar_type) => Type::Scalar(ScalarType::new(err, scalar_type, schema)?),
            parsed_model::Type::Required(wrapped) => Type::Required(Box::new(Type::new(err, wrapped, schema)?)),
            parsed_model::Type::Array(wrapped) => Type::Array(Box::new(Type::new(err, wrapped, schema)?)),
        })
    }

    fn from_validated(err: &mut ErrorCollector, parsed: &'a parsed_model::Type, schema: &'a Schema) -> Result<Self, Error> {
        Ok( match parsed {
             parsed_model::Type::Scalar(scalar_type) => Type::Scalar(ScalarType::from_validated(err, scalar_type, schema)?),
             parsed_model::Type::Required(wrapped) => Type::Required(Box::new(Type::from_validated(err, wrapped, schema)?)),
             parsed_model::Type::Array(wrapped) => Type::Array(Box::new(Type::from_validated(err, wrapped, schema)?)),
         })
     }

    fn graphql_name(&self) -> String {
        match self {
            Type::Scalar(scalar_type) => {
                match scalar_type {
                    ScalarType::DefinedType(defined_type) => defined_type.name().to_string(),
                    ScalarType::BuiltinType(builtin_type) => builtin_type.name().to_string(),
                }
            },
            Type::Required(wrapped) => format!("{}!", wrapped.graphql_name()),
            Type::Array(wrapped) => format!("[{}]", wrapped.graphql_name()),
        }
    }

    pub fn get_scalar(&self) -> &ScalarType {
        match self {
            Type::Scalar(scalar_type) => scalar_type,
            Type::Required(wrapped) => wrapped.get_scalar(),
            Type::Array(wrapped) => wrapped.get_scalar(),
        }
    }

    
    // if nonnull then the type is coerced to be nonnull at all levels
    pub fn rust_type(&self, nonnull: bool) -> String {
        // Will never be called with Required variant.
        fn do_rust_type(input: &Type, nonnull: bool) -> String {
            match input {
                Type::Scalar(wrapped) => {
                    match wrapped {
                        ScalarType::DefinedType(defined_type) => defined_type.rust_type(),
                        ScalarType::BuiltinType(builtin_type) => builtin_type.rust_type().to_string(),
                    }
                },
                Type::Required(_) => unreachable!(),
                Type::Array(wrapped) => format!("Vec<{}>", wrapped.rust_type(nonnull)),
            }
        }

        if let Type::Required(wrapped) = self {
            do_rust_type(wrapped, nonnull)
        }
        else {
            if nonnull {
                do_rust_type(self, nonnull)
            }
            else {
                format!("Option<{}>", do_rust_type(self, nonnull))
            }
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

    pub fn type_name(&self) -> &str {
        match self {
            TypeDefinition::Scalar(_) => "scalar",
            TypeDefinition::Object(_) => "object",
            TypeDefinition::Interface(_) => "interface",
            TypeDefinition::Union(_) => "union",
            TypeDefinition::Enum(_) => "enum",
        }
    }

    // fn validate2(out: &mut Output, name: &String, defined_types: IndexMap<String, TypeDefinition>, parsed_types: IndexMap<String, parsed_model::TypeDefinition>, loop_detect: HashSet<String>) {
    //     if loop_detect.contains(name) {
    //         out.error(GraphQLError::);
    //     }
    // }

    // pub fn validate(out: &mut Output, name: &String, defined_types: IndexMap<String, TypeDefinition>, parsed_types: IndexMap<String, parsed_model::TypeDefinition>) {
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

    // pub fn generate(&self, out: &mut Output, schema: &Schema, selections: Option<&Vec<Selection>>) -> Result<(), Error> {
    //     match self {
    //         TypeDefinition::Enum(content) => content.generate(out, schema, selections),
    //         TypeDefinition::Union(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Object(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Interface(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Scalar(content) => content.generate(out, schema),
    //     }
    // }
}

trait Context<'a> {
    fn get_context(&'a self, schema: &'a Schema) -> Result<IndexMap<&'a str, &'a IndexMap<&'a str, Field<'a>>>, Error>;
}

#[derive(Debug)]
pub struct Scalar<'a> {
    pub parsed: &'a parsed_model::Scalar<'a>,
    pub rust_name: String,
    pub rust_type: String,
}

impl<'a> Scalar<'a> {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "TypeDef {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out);

            writeln!(out, "rust_name: {}", self.rust_name)?;
            writeln!(out, "rust_type: {}", self.rust_type)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(parsed: &'a parsed_model::Scalar<'a>) -> Self {

        let rust_type = "serde_json::Value".to_string(); // TODO: get proper types

        Scalar {
            parsed,
            rust_name: to_pascal_case(&parsed.name),
            rust_type,
        }
    }
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Schema) -> Result<(), Error> {
        writeln!(out, "type {} = {};", &self.rust_name, &self.rust_type)?;
        writeln!(out, "")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Object<'a> {
    pub parsed: &'a parsed_model::Object<'a>,
    pub fully_implements: Vec<&'a str>,
    pub fields: IndexMap<&'a str, Field<'a>>,
    pub is_input: bool,
}

impl<'a> Context<'a> for Object<'a> {
    fn get_context(&'a self, _schema: &'a Schema) -> Result<IndexMap<&'a str, &'a IndexMap<&'a str, Field<'a>>>, Error> {
        let mut context = IndexMap::new();

        context.insert(self.parsed.name, &self.fields);

        Ok(context)
    }
}

impl<'a> Object<'a> {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Object {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;

            writeln!(out, "fully_implements {{")?;
            {
                let mut out = out.indent();

                for name in &self.fully_implements {
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
    
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Object<'a>, schema: &'a parsed_model::Schema<'a>) -> Result<Self, Error> {
        let mut fully_implements = Vec::new();

        if let Some(implements) = parsed.implements {
            for interface_name in implements {
                if let Some(defined_type) = schema.named_types.get(interface_name as &str) {
                    if let parsed_model::TypeDefinition::Interface(interface) = defined_type {
                        let mut ok = true;
                        for field_name in interface.fields.keys() {
                            if ! parsed.fields.contains_key(field_name as &str) {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            fully_implements.push(interface_name as &str);
                        }
                    }
                    else {
                        err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Expected interface {} but found {}", parsed.name, defined_type.type_name())));
                    }
                }
                else {
                    err.error(BuildError::MissingInterfaceError(parsed.position.clone(), format!("Interface {} Not Found", parsed.name)))
                };
            }
        }

        let mut fields: IndexMap<&str, Field<'_>> = IndexMap::new();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                fields.insert(name, field);
            }
        }

        err.ok(Object {
                parsed,
                fully_implements,
                fields,
                is_input: parsed.is_input,
            })
    }
    
    fn generate(&'a self, out: &mut Output<'_>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>, selections: Option<&'a Vec<Selection<'a>>>, alias: &'a Option<String>) -> Result<(), Error> {
    //(&self, out: &mut Output<'_>, schema: &Schema, executable_document: &ExecutableDocument, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), Error> {
        generate_struct(out, schema, executable_document, selections, alias, &self.parsed.name, &self.fields, self.is_input)
    }
}

fn selections_to_fields<'a>(selections: &'a Vec<Selection>, executable_document: &ExecutableDocument, context: FieldMapRef<'a>) -> Result<Vec<&'a SelectionField<'a>>, Error> {
    let mut selected_fields = Vec::new();
    for selection in selections {
        match selection {
            Selection::Field(selection_field) => {
                selected_fields.push(selection_field);
                // writeln!(out, "// selected field {} = {}", selection_field.name, selection_field.optional)?;
            },
            Selection::FragmentSpread(fragment_spread) => {
                let fragment = executable_document.get_fragment(fragment_spread.parsed.name)?;

                let type_condition = fragment.parsed.type_condition;
                // let object = context.get(type_condition)
                todo!()
            },
        };
    }
    Ok(selected_fields)
}

fn generate_struct<'a>(out: &mut Output<'_>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>, selections: Option<&'a Vec<Selection<'a>>>, alias: &'a Option<String>, name: &'a str, fields: FieldMapRef<'a>, is_input: bool) -> Result<(), Error> {
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
        let selected_fields = selections_to_fields(selections, executable_document, fields)?;

        for selection_field in &selected_fields {
            writeln!(out, "// selection_field {}", &selection_field.parsed.name)?;
        }

        for selection_field in &selected_fields {
            if &selection_field.parsed.name == &TYPE_NAME {
                writeln!(out, "    #[serde(rename = \"{}\")]", &selection_field.parsed.name)?;
                writeln!(out, "    {}: String,", &selection_field.parsed.name)?;
            }
            else {
                if let Some(field) = fields.get(&selection_field.parsed.name) {
                    // writeln!(out, "// T3 {} {}", name, selection_field.optional)?;
                    writeln!(out, "    #[serde(rename = \"{}\")]", &field.parsed.name)?;
                    writeln!(out, "    pub {}_: {},", to_snake_case(&selection_field.parsed.name), field.ty.rust_type(selection_field.nonnull));
                }
                else {
                    writeln!(out, "UNKNOWN FIELD 1 {}", &selection_field.parsed.name)?;
                }
            }
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        for selection_field in &selected_fields {
            if &selection_field.parsed.name == &TYPE_NAME {}
            else {
                if let Some(field) = fields.get(&selection_field.parsed.name) {
                    if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
                        defined_type.generate(out, schema, executable_document, Some(&selection_field.selections), &selection_field.parsed.alias)?;
                    }
                }
                else {
                    writeln!(out, "UNKNOWN FIELD 2 {}", &selection_field.parsed.name)?;
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
        for field in fields.values() {
            writeln!(out, "    #[serde(rename = \"{}\")]", &field.parsed.name)?;
            writeln!(out, "    pub {}_: {},", to_snake_case(&name), field.ty.rust_type(false))?;
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        if is_input {
            writeln!(out, "impl {} {{", rust_name)?;
            {
                let mut out = out.indent();

                writeln!(out, "pub fn builder() -> {}Builder {{", rust_name)?;
                writeln!(out, "    {}Builder {{", rust_name)?;
                for field in fields.values() {
                    writeln!(out, "        {}_: None,", to_snake_case(&name))?;
                }
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "#[derive(Debug)]")?;
            writeln!(out, "pub struct {}Builder {{", rust_name)?;

            for field in fields.values() {
                writeln!(out, "    {}_: {},", to_snake_case(&name), field.ty.rust_type(false))?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl {}Builder {{", rust_name)?;
            {
                let mut out = out.indent();

                for field in fields.values() {
                    writeln!(out, "pub fn with_{}(mut self, value: {}) -> Self {{", to_snake_case(&name), field.ty.rust_type(true))?;
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

                    for field in fields.values() {
                        if let Type::Required(_) = field.ty {
                            writeln!(out, "if let None = self.{}_ {{", to_snake_case(&name))?;
                            writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", name)?;
                            writeln!(out, "}}")?;
                        }
                    }

                    writeln!(out, "Ok({} {{", rust_name)?;
                    {
                        let mut out = out.indent();
    
                        for field in fields.values() {
                            if let Type::Required(_) = field.ty {
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
pub struct Interface<'a> {
    pub parsed: &'a parsed_model::Interface<'a>,
    pub implemented_by: Vec<&'a str>,
    pub fields: IndexMap<&'a str, Field<'a>>,
}

// impl<'a> Context<'a> for Interface<'a> {
//     fn get_context(&self, schema: &'a Schema) -> Result<IndexMap<&'a str, &'a IndexMap<&'a str, Field<'a>>>, Error> {
//         let mut context = IndexMap::new();

//         for name in self.implemented_by {
//             context.insert(name, schema.get_object(name)?);
//         }
//         let mut it = self.implemented_by.iterator(schema);
//         loop {
//             match it.next()? {
//                 Some(object) => {
//                     context.insert(object.parsed.name, &object.fields);
//                 },
//                 None => {
//                     break;
//                 },
//             }
//         }

//         Ok(context)
//     }
// }

impl<'a> Interface<'a> {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Interface {{")?;
        self.parsed.print(out);
        
        {
            let mut out = out.indent();

            writeln!(out, "implemented_by {{")?;
            {
                let mut out = out.indent();

                for name in &self.implemented_by {
                    writeln!(out, "{}", name)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Interface<'a>, schema: &'a parsed_model::Schema) -> Result<Self, Error> {
        let mut implemented_by = Vec::new();

        
        for (_, defined_type) in &schema.named_types {
            if let parsed_model::TypeDefinition::Object(object_model) = defined_type {
                if let Some(implements) = &object_model.implements {
                    for implements in *implements {
                        if implements == &parsed.name {
                            implemented_by.push(object_model.name.clone());
                            break;
                        }
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
        //         out.error(BuildError::MissingInterfaceError(parsed.position, format!("Interface {} Not Found", parsed.name)))
        //     };

            
        // }



        let mut fields = IndexMap::new();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                fields.insert(name as &str, field);
            }
        }

        err.ok(Interface {
                parsed,
                implemented_by,
                fields,
            })
    }
    
    fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), Error> {

        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        writeln!(out, "pub enum {} {{", to_pascal_case(&self.parsed.name))?;

        // for object in self.implemented_by.iterator(schema) {
            
        //     writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
        // }

        writeln!(out, "}}")?;
        writeln!(out, "")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Union<'a> {
    pub parsed: &'a parsed_model::Union<'a>,
    pub fields: IndexMap<&'a str, Field<'a>>,
}

impl<'a> Context<'a> for Union<'a> {
    fn get_context(&'a self, schema: &'a Schema) -> Result<IndexMap<&'a str, &'a IndexMap<&'a str, Field<'a>>>, Error> {
        let mut context = IndexMap::new();

        context.insert(self.parsed.name, &self.fields);

        Ok(context)
    }
}

impl<'a> Union<'a> {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Union {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;

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
    
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Union, schema: &'a parsed_model::Schema) -> Result<Self, Error> {
        let mut fields = IndexMap::new();
        let mut err = err.child();

        for type_name in parsed.types {
            if let Some(defined_type) = schema.named_types.get(type_name as &str) {
                if let parsed_model::TypeDefinition::Object(object) = defined_type {
                    for (name, field) in &object.fields {
                        if let Ok(field) = Field::new(&mut err, field, schema) {
                            fields.insert(name.clone(), field);
                        }
                    }
                }
                else {
                    err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Expected interface {} but found {}", parsed.name, defined_type.type_name())));
                }
            }
            else {
                err.error(BuildError::MissingInterfaceError(parsed.position.clone(), format!("Interface {} Not Found", parsed.name)))
            };
        }

        err.ok(Union {
            parsed,
            fields,
        })
    }

    fn generate(&'a self, out: &mut Output<'_>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>, selections: Option<&'a Vec<Selection<'a>>>, alias: &'a Option<String>) -> Result<(), Error> {
        generate_struct(out, schema, executable_document, selections, alias, &self.parsed.name, &self.fields, false)
    }
    
    // fn generate(&self, out: &mut Output<'_>, schema: &Schema, selections: Option<&Vec<Selection>>, alias: &Option<String>) -> Result<(), Error> {
    //     generate_struct(out, schema, selections, alias, &self.name, &self.fields, false)

    //     // writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
    //     // writeln!(out, "#[serde(rename = \"{}\")]", &self.name)?;
    //     // writeln!(out, "pub struct {} {{", to_pascal_case(&self.name))?;

    //     // if let Some(selected_fields) = selected_fields {
    //     //     for (name, field) in &self.fields {
    //     //         if let Some(selection_field) = selected_fields.get(name) {
    //     //             writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
    //     //             writeln!(out, "    /* BRUCE */ {}_: {},", to_snake_case(&name), field.rust_type(schema, selection_field.optional.clone(), &selection_field.alias))?;
    //     //         }
    //     //     }
    //     // }
    //     // else {
    //     //     for (name, field) in &self.fields {
    //     //         writeln!(out, "    #[serde(rename = \"{}\")]", &field.name)?;
    //     //         writeln!(out, "    /* BRUCE2 */ {}_: {},", to_snake_case(&name), field.rust_type(schema, true, &None))?;
    //     //     }
    //     // }
    //     // writeln!(out, "}}")?;
    //     // writeln!(out, "")?;
    //     // Ok(())
    // }
}

#[derive(Debug)]
pub struct Enum<'a> {
    pub parsed: &'a parsed_model::Enum<'a>,
}

impl<'a> Enum<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Enum {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
        }
        writeln!(out, "}}")
    }

    pub fn new(parsed: &'a parsed_model::Enum<'a>) -> Self {
        Enum {
            parsed,
        }
    }
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Schema, _executable_document: &ExecutableDocument, _selections: Option<&Vec<Selection>>, _alias: &Option<String>) -> Result<(), Error> {
    // fn generate(&self, out: &mut Output<'_>, _schema: &Schema, _selections: Option<&Vec<Selection>>) -> Result<(), Error> {
        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
        writeln!(out, "pub enum {} {{", to_pascal_case(&self.parsed.name))?;

        for variant in self.parsed.variants {
            writeln!(out, "    #[serde(rename = \"{}\")]", &variant.name)?;
            writeln!(out, "    {},", to_constant_case(&variant.name))?;
        }
        writeln!(out, "}}")?;
        writeln!(out, "")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum DefinedType<'a> {
    Scalar(&'a str),
    Object(&'a str),
    Interface(&'a str),
    Union(&'a str),
    Enum(&'a str),
}

impl Display for DefinedType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinedType::Scalar(name) => write!(f, "Scalar {}", name),
            DefinedType::Object(name) => write!(f, "Object {}", name),
            DefinedType::Interface(name) => write!(f, "Interface {}", name),
            DefinedType::Union(name) => write!(f, "Union {}", name),
            DefinedType::Enum(name) => write!(f, "Enum {}", name),
        }
    }
}

impl<'a> DefinedType<'a> {
    pub fn name(&self) -> &str {
        match self {
            DefinedType::Scalar(name) => name,
            DefinedType::Object(name) => name,
            DefinedType::Interface(name) => name,
            DefinedType::Union(name) => name,
            DefinedType::Enum(name) => name,
        }
    }

    pub fn generate(&self, out: &mut Output<'_>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>, selections: Option<&'a Vec<Selection<'a>>>, alias: &'a Option<String>) -> Result<(), Error> {
        match self {
            DefinedType::Scalar(name) => schema.get_scalar(name)?.generate(out, schema),
            DefinedType::Object(name) => schema.get_object(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::Interface(name) => schema.get_interface(name)?.generate(out, schema, selections, alias),
            DefinedType::Union(name) => schema.get_union(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::Enum(name) => schema.get_enum(name)?.generate(out, schema, executable_document, selections, alias),
        }
    }
    
    fn rust_type(&self) -> String {
        match self {
            DefinedType::Scalar(name) => to_pascal_case(name),
            DefinedType::Object(name) => to_pascal_case(name),
            DefinedType::Interface(name) => to_pascal_case(name),
            DefinedType::Union(name) => to_pascal_case(name),
            DefinedType::Enum(name) => to_pascal_case(name),
        }
    }
}

// #[derive(Debug)]
// pub struct ObjectProxy<'a> {
//     pub name: &'a str,
// }

// impl<'a> ObjectProxy<'a> {
//     pub fn new(name: &'a str) -> Self {
//         ObjectProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &'a Schema) -> Result<&'a Object<'a>, Error> {
//         match schema.defined_types.get(self.name) {
//             Some(type_definition) => {
//                 if let TypeDefinition::Object(object) = type_definition {
//                     Ok(object)
//                 }
//                 else {
//                     Err(Error::BuildFailed(format!("ObjectProxy expected Object for \"{}\" but found {}", &self.name, type_definition.type_name())))
//                 }
//             },
//             None => Err(Error::BuildFailed(format!("ObjectProxy failed to find \"{}\"", &self.name))),
//         }
//     }

//     pub fn as_str(&self) -> &str {
//         &self.name
//     }

//     pub fn option_as_str(proxy: &'a Option<ObjectProxy<'a>>) -> &'a str {
//         match proxy {
//             Some(proxy) => proxy.as_str(),
//             None => "None",
//         }
//     }
// }

// #[derive(Debug)]
// pub struct InterfaceProxy<'a> {
//     pub name: &'a str,
// }

// impl<'a> InterfaceProxy<'a> {
//     pub fn new(name: &'a str) -> Self {
//         InterfaceProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &'a Schema) -> Result<&'a Interface<'a>, Error> {
//         match schema.defined_types.get(&self.name as &str) {
//             Some(type_definition) => {
//                 if let TypeDefinition::Interface(content) = type_definition {
//                 Ok(content)
//                 }
//                 else {
//                     Err(Error::BuildFailed(format!("InterfaceProxy expected Interface for \"{}\" but found {}", &self.name, type_definition.type_name())))
//                 }
//             },
//             None => Err(Error::BuildFailed(format!("InterfaceProxy failed to find \"{}\"", &self.name))),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ScalarProxy<'a> {
//     pub name: &'a str,
// }

// impl<'a> ScalarProxy<'a> {
//     pub fn new(name: &'a str) -> Self {
//         ScalarProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &'a Schema) -> Result<&'a Scalar<'a>, Error>{
//         match schema.defined_types.get(self.name) {
//             Some(type_definition) => {
//                 if let TypeDefinition::Scalar(content) = type_definition {
//                 Ok(content)
//                 }
//                 else {
//                     Err(Error::BuildFailed(format!("ScalarProxy expected Scalar for \"{}\" but found {}", &self.name, type_definition.type_name())))
//                 }
//             },
//             None => Err(Error::BuildFailed(format!("ScalarProxy failed to find \"{}\"", &self.name))),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct ObjectListProxy<'a> {
//     pub names: Vec<&'a str>,
// }

// impl<'a> ObjectListProxy<'a> {
//     pub fn new(names: Vec<&'a str>) -> Self {
//         ObjectListProxy { names }
//     }

//     fn iterator(&self, schema: &'a Schema) -> ObjectListProxyItertor<'a> {
//         ObjectListProxyItertor {
//             list: self,
//             schema,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// // impl<'a> IntoIterator for &'a ObjectListProxy {
// //     type Item = &'a Interface;
// //     type IntoIter = ObjectListProxyItertor<'a>;
    
// //     fn into_iter(self) -> Self::IntoIter {
// //         ObjectListProxyItertor {
// //             list: &self,
// //             index: 0,
// //             // iter: self.names.iter(),
// //         }
// //     }
// // }

// pub struct ObjectListProxyItertor<'a> {
//     list: &'a ObjectListProxy<'a>,
//     schema: &'a Schema<'a>,
//     index: usize,
// }

// impl<'a> /*Iterator for*/ ObjectListProxyItertor<'a> {
//     // type Item = &'a Object;

//     fn next(&mut self) -> Result<Option<&'a Object>, Error> {        
//         match self.list.names.get(self.index) {
//             Some(name) => {
//                 self.index += 1;
//                 match self.schema.defined_types.get(name as &str) {
//                     Some(type_definition) => {
//                         if let TypeDefinition::Object(object) = type_definition {
//                             Ok(Some(object))
//                         }
//                         else {
//                             Err(Error::BuildFailed(format!("Object iterator expected Object for \"{}\" but found {}", name, type_definition.type_name())))
//                         }
//                     },
//                     None => Err(Error::BuildFailed(format!("Object iterator failed to find \"{}\"", name))),
//                 }
//             },
//             None => Ok(None),
//         }
//     }
// }

// #[derive(Debug)]
// pub struct InterfaceListProxy {
//     // pub schema: Rc<Schema>,
//     pub names: Vec<String>,
// }

// impl InterfaceListProxy {
//     pub fn new(names: Vec<String>) -> Self {
//         InterfaceListProxy { names }
//     }

//     fn _iterator<'a>(&self, schema: &'a Schema) -> InterfaceListProxyItertor<'a> {
//         InterfaceListProxyItertor {
//             list: self,
//             schema,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// // impl<'a> IntoIterator for &'a InterfaceListProxy {
// //     type Item = &'a Interface;
// //     type IntoIter = InterfaceListProxyItertor<'a>;
    
// //     fn into_iter(self) -> Self::IntoIter {
// //         InterfaceListProxyItertor {
// //             list: &self,
// //             index: 0,
// //             // iter: self.names.iter(),
// //         }
// //     }
// // }

// pub struct InterfaceListProxyItertor<'a> {
//     list: &'a InterfaceListProxy,
//     schema: &'a Schema<'a>,
//     index: usize,
// }

// impl<'a> InterfaceListProxyItertor<'a> {
//     fn next(&mut self) -> Result<Option<&'a Interface>, Error> {        
//         match self.list.names.get(self.index) {
//             Some(name) => {
//                 self.index += 1;
//                 match self.schema.defined_types.get(name as &str) {
//                     Some(type_definition) => {
//                         if let TypeDefinition::Interface(object) = type_definition {
//                             Ok(Some(object))
//                         }
//                         else {
//                             Err(Error::BuildFailed(format!("Interface iterator expected Interface for \"{}\" but found {}", name, type_definition.type_name())))
//                         }
//                     },
//                     None => Err(Error::BuildFailed(format!("Interface iterator failed to find \"{}\"", name))),
//                 }
//             },
//             None => Ok(None),
//         }
//     }
// }

// // #[derive(Debug)]
// // pub struct FragmentProxy<'a> {
// //     pub name: &'a str,
// // }

// // impl<'a> FragmentProxy<'a> {
// //     pub fn new(name: &'a str) -> Self {
// //         FragmentProxy {
// //             name,
// //         }
// //     }
    
// //     pub fn get(&self, executable_document: &'a parsed_model::ExecutableDocument) -> Result<&'a parsed_model::FragmentDefinition, Error> {
// //         match executable_document.fragments.get(self.name) {
// //             Some(fragment_definition) => {
// //                 Ok(fragment_definition)
// //             },
// //             None => Err(Error::BuildFailed(format!("FragmentProxy failed to find \"{}\"", &self.name))),
// //         }
// //     }
// // }

#[derive(Debug)]
pub struct Field<'a> {
    pub parsed: &'a parsed_model::Field<'a>,
    pub ty: Type<'a>,
}

impl<'a> Field<'a>{
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out);
            writeln!(out, "ty:        {}", self.ty)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
    
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Field<'a>, schema: &'a parsed_model::Schema) -> Result<Self, Error> {
        Ok(Field {
            parsed,
            ty: Type::new(err, &parsed.ty, schema)?,
        })
    }

    // pub fn rust_type(&self, schema: &Schema, maybe_optional: Maybe, alias: &Option<String>) -> String {
    //     self.ty.rust_type()   .rust_type(self.multiple, self.nonnull, schema, maybe_optional, alias)
    // }

    pub fn graphql_name(&self) -> String {
        self.ty.graphql_name()
    }
    
    fn from_variable(err: &mut ErrorCollector, parsed: &'a parsed_model::Field<'a>, schema: &'a Schema) -> Result<Self, Error> {
        Ok(Field {
            parsed,
            ty: Type::from_validated(err, &parsed.ty, schema)?,
        })
    }
}

#[derive(Debug)]
pub struct Schema<'a> {
    pub defined_types: IndexMap<&'a str, TypeDefinition<'a>>,
    pub query: &'a str,
    pub mutation: Option<&'a str>,
    pub subscription: Option<&'a str>,
}

impl<'a> Schema<'a> {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "query        {}", self.query)?;
            writeln!(out, "mutation     {:?}", self.mutation);
            writeln!(out, "subscription {:?}", self.subscription)?;
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

    pub fn get_scalar(&'a self, name: &str) -> Result<&'a Scalar<'a>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::Scalar(type_definition) = type_definition {
                Ok(type_definition)
            }
            else {
                Err(Error::BuildFailed(format!("Expected Scalar for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Scalar \"{}\"", name)))
        }
    }

    pub fn get_object(&'a self, name: &str) -> Result<&'a Object<'a>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::Object(object) = type_definition {
                Ok(object)
            }
            else {
                Err(Error::BuildFailed(format!("Expected Object for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Object \"{}\"", name)))
        }
    }

    pub fn get_interface(&'a self, name: &str) -> Result<&'a Interface<'a>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::Interface(interface) = type_definition {
                Ok(interface)
            }
            else {
                Err(Error::BuildFailed(format!("Expected Interface for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Interface \"{}\"", name)))
        }
    }
    
    fn get_union(&'a self, name: &'a str) -> Result<&'a Union<'a>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::Union(union) = type_definition {
                Ok(union)
            }
            else {
                Err(Error::BuildFailed(format!("Expected Union for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Union \"{}\"", name)))
        }
    }
    
    fn get_enum(&'a self, name: &'a str) -> Result<&'a Enum<'a>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::Enum(type_definition) = type_definition {
                Ok(type_definition)
            }
            else {
                Err(Error::BuildFailed(format!("Expected Enum for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Enum \"{}\"", name)))
        }
    }

    fn new_get_object(out: &mut ErrorCollector, name: &'a Option<String>, position: &'a Pos, defined_types: &IndexMap<&'a str, TypeDefinition>) -> Option<&'a str> {
        if let Some(name) = name {
            if let Some(type_definition) = &defined_types.get(name as &str) {
                if let TypeDefinition::Object(_) = type_definition {
                    Some(name as &str)
                }
                else {
                    out.error(BuildError::TypeMismatchError(position.clone(), format!("Expected object \"{}\" but found {}", name, type_definition.type_name())));
                    None
                }
            }
            else {
                out.error(BuildError::MissingObjectError(position.clone(), format!("Missing schema reference \"{}\"", name)));
                None
            }
        }
        else {
            None
        }
    }

    fn new_get_default_object(out: &mut ErrorCollector, name: &'a str, defined_types: &IndexMap<&'a str, TypeDefinition>, missing_error: Option<BuildError>) -> Option<&'a str> {
        if let Some(type_definition) = &defined_types.get(name) {
            if let TypeDefinition::Object(_) = type_definition {
                Some(name as &str)
            }
            else {
                out.error(BuildError::TypeMismatchError(Pos { line: 0, column: 0 }, format!("Expected object \"{}\" but found {}", name, type_definition.type_name())));
                None
            }
        }
        else {
            if let Some(error) = missing_error { 
                out.error(error);
            }
            None
        }
    }

    pub fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::Schema<'a>) -> Result<Self, Error> {

        let mut defined_types: IndexMap<&'a str, TypeDefinition<'a>> = IndexMap::new();
        
        for(name, ty) in &parsed.named_types {
            let defined_type = match ty {
                parsed_model::TypeDefinition::Scalar(type_def) => TypeDefinition::Scalar(Scalar::new(type_def)),
                parsed_model::TypeDefinition::Object(object) => TypeDefinition::Object(Object::new(err, object, &parsed)?),
                parsed_model::TypeDefinition::Interface(interface) => TypeDefinition::Interface(Interface::new(err, interface, &parsed)?),
                parsed_model::TypeDefinition::Union(union) => TypeDefinition::Union(Union::new(err, union, &parsed)?),
                parsed_model::TypeDefinition::Enum(enum_definition) => TypeDefinition::Enum(Enum::new(enum_definition)),
            };
            defined_types.insert(name, defined_type);
        }

        let mutation = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.mutation, schema_definition.position, &defined_types) 
        }
        else {
            Self::new_get_default_object(err, "Mutation", &defined_types, None)
        };
        
        let subscription = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.subscription, schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, "Subscription", &defined_types, None)
        };

        let query = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.query, schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, "Query", &defined_types, Some(BuildError::NoQueryDefinition))
        };

        err.ok(Schema {
                defined_types,
                query: query.ok_or(Error::BuildFailed(format!("Failed to find previously validated query")))?,
                mutation,
                subscription,
            })
    }


    // pub fn generate(&self, out: &mut Output) -> Result<(), Error> {
    //     for (_, defined_type) in &self.defined_types {
    //         defined_type.generate(out, self, None)?;
    //     }
    //     Ok(())
    // }
    
}

#[derive(Debug)]
pub enum Selection<'a> {
    Field(SelectionField<'a>),
    FragmentSpread(FragmentSpread<'a>),
    // InlineFragment(InlineFragment),
} 

impl<'a> Selection<'a> {
    pub fn new(err: &mut ErrorCollector, selection: &'a parsed_model::Selection<'a>, schema: &'a Schema<'a>, executable_document: &'a parsed_model::ExecutableDocument<'a>, context: FieldMapRef<'a>) -> Result<Self, Error> {
        match selection {
            parsed_model::Selection::Field(selection_field) => {

            // writeln!(out, "/* Selection Field:" );
            // selection_field.print(out);
            // writeln!(out, "Context:" );
            // context.print(out);
            // writeln!(out, "*/" );

                Ok(Selection::Field(SelectionField::new(err, selection_field, schema, executable_document, context)?))
            },
            parsed_model::Selection::Fragment(fragment_spread) => Ok(Selection::FragmentSpread(FragmentSpread::new(err, fragment_spread,
                 schema, executable_document, context)?)),
            
            // graphql_parser::query::Selection::FragmentSpread(fragment_spread) => todo!(),
            // graphql_parser::query::Selection::InlineFragment(inline_fragment) => todo!(),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::Field(selection_field) => selection_field.print(out),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.print(out),
        }
    }

    // pub fn position(&self) -> Pos {
    //     match self {
    //         Selection::Field(selection_field) => selection_field.position,
    //     }
    // }

    pub fn generate_query(&self, out: &mut Output, context: FieldMapRef<'a>, schema: &Schema, maybe_optional: Maybe) -> Result<(), Error> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_query(out, context, schema, maybe_optional),
            Selection::FragmentSpread(fragment_spread) => todo!(),
        }
    }

    pub fn generate_fields(&self, out: &mut Output, context: FieldMapRef<'a>, schema: &Schema, maybe_optional: Maybe) -> Result<(), Error> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_fields(out, context, schema, maybe_optional),
            Selection::FragmentSpread(fragment_spread) => todo!(),
        }
    }
    
    fn generate_structs(&'a self, out: &mut Output<'_>, context: FieldMapRef<'a>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>) -> Result<(), Error> {
        match self {
            Selection::Field(selection_field) => selection_field.generate_structs(out, context, schema, executable_document),
            Selection::FragmentSpread(fragment_spread) => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct SelectionField<'a> {
    pub parsed: &'a parsed_model::SelectionField<'a>,
    pub nonnull: bool,
    pub arguments: Vec<&'a parsed_model::Argument<'a>>,
    pub selections: Vec<Selection<'a>>,
}

impl<'a> SelectionField<'a> {
    pub fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::SelectionField, schema: &'a Schema<'a>, executable_document: &'a parsed_model::ExecutableDocument<'a>, context: FieldMapRef<'a>) -> Result<Self, Error> {
        let mut arguments = Vec::new();
        let mut selections = Vec::new();

        println!("SlectionField name={} alias={:?}", &parsed.name, &parsed.alias);

        if parsed.name == TYPE_NAME {
            println!("HERE TYPE_NAME");
        }
        else {
            if let Some(field) = context.get(&parsed.name) {
                for parsed_argument in &parsed.arguments {
                    // arguments.push(Argument::new(prsed_argument));
                    arguments.push(parsed_argument);
                }

                if ! parsed.selections.is_empty() {
                    if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
                        let optional_fields =  match defined_type {
                            DefinedType::Scalar(_) => None,
                            DefinedType::Object(name) => Some(&schema.get_object(name)?.fields),
                            DefinedType::Interface(name) => Some(&schema.get_interface(name)?.fields),
                            DefinedType::Union(name) => Some(&schema.get_union(name)?.fields),
                            DefinedType::Enum(_) => None,
                        };
        
                        if let Some(fields) = optional_fields {
                            for selection in &parsed.selections {
                                if let Ok(selection) = Selection::new(err, &selection, schema, executable_document, &fields) {
                                    selections.push(selection);
                                }
                            }
                        }
                        else {
                            println!("Attribute selection given on incompatible type \"{}\"", field.ty);
                            err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Attribute selection given on incompatible type \"{}\"", field.ty)));
                        }
                    }
                }
            }
            else {
                err.error(BuildError::MissingFieldError(parsed.position.clone(), parsed.name.to_string()));
            }
        }

        err.ok(SelectionField {
            parsed,
            nonnull: !parsed.optional,
            arguments,
            selections,
        })
    }

    // pub fn old(err: &mut ErrorCollector, parsed: &'a parsed_model::SelectionField, schema: &'a Schema, executable_document: &'a parsed_model::ExecutableDocument, context: FieldMapRef<'a>) -> Result<Self, Error> {

        
    //     let mut arguments = Vec::new();
        
    //     for parsed_argument in &parsed.arguments {
    //         // arguments.push(Argument::new(prsed_argument));
    //         arguments.push(parsed_argument);
    //     }
        
    //     println!("SlectionField name={} alias={:?}", &parsed.name, &parsed.alias);
    //     let mut selections = Vec::new();

    //     if parsed.name == TYPE_NAME {
    //         println!("HERE1");
    //     }
    //     else {
    //         if ! parsed.selections.is_empty() {
    //             if let Some(field) = context.get(&parsed.name) {
    //                 if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
    //                     let optional_fields =  match defined_type {
    //                         DefinedType::Scalar(_) => None,
    //                         DefinedType::Object(proxy) => Some(&proxy.get(schema)?.fields),
    //                         DefinedType::Interface(proxy) => Some(&proxy.get(schema)?.fields),
    //                         // DefinedType::Union(proxy) => Some(&proxy.get(schema)?.fields),
    //                         // DefinedType::Enum(_) => None,
    //                     };
        
    //                     if let Some(fields) = optional_fields {
    //                         for selection in &parsed.selections {
    //                             if let Ok(selection) = Selection::new(err, &selection, schema, executable_document, &fields) {
    //                                 selections.push(selection);
    //                             }
    //                         }
    //                     }
    //                     else {
    //                         println!("Attribute selection given on incompatible type \"{}\"", field.ty);
    //                         err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Attribute selection given on incompatible type \"{}\"", field.ty)));
    //                     }
    //                 }
                    
    //             }
    //             else {
    //                 err.error(BuildError::MissingFieldError(parsed.position.clone(), parsed.name.to_string()));
    //             }
    //         }
    //     }

    //     err.ok(SelectionField {
    //             parsed,
    //             nonnull: !parsed.optional,
    //             arguments,
    //             selections,
    //         })
    // }



    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionField {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
            writeln!(out, "nonnull:  {}", self.nonnull)?;
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

    pub fn generate_query(&self, out: &mut Output, context: FieldMapRef<'a>, schema: &Schema, maybe_optional: Maybe) -> Result<(), Error> {
        // let field = context.get(&self.name).unwrap();
        if let Some(alias) = &self.parsed.alias {
            writeln!(out, "{}: {}", alias, &self.parsed.name)?;
        }
        else {
            writeln!(out, "{}", &self.parsed.name)?;
        }

        if ! self.arguments.is_empty() {
            writeln!(out, "(")?;
            {
                let mut out = out.indent();

            
                for argument in &self.arguments {
                    argument.generate_query(&mut out, context, schema, true)?;
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

    pub fn generate_fields(&self, out: &mut Output, context: FieldMapRef<'a>, schema: &Schema, maybe_optional: Maybe) -> Result<(), Error> {
        if &self.parsed.name == &TYPE_NAME {
            println!("HERE");
        }
        let field = context.get(&self.parsed.name).unwrap();
        let name = if let Some(alias) = &self.parsed.alias {
            alias
        }
        else {
            self.parsed.name
        };

        let rust_type = if let Some(alias) = &self.parsed.alias {
            to_pascal_case(alias)
        }
        else {
            field.ty.rust_type(self.nonnull)
        };

        writeln!(out, "#[serde(rename = \"{}\")]", &name)?;
        writeln!(out, "/* HERE1 */ pub {}: {},", to_snake_case(&name), rust_type)?;
        Ok(())
    }

    pub fn generate_structs(&'a self, out: &mut Output, context: FieldMapRef<'a>, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>) -> Result<(), Error> {
        let field = context.get(&self.parsed.name).unwrap();

        if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
            writeln!(out, "// {} is {}", &self.parsed.name, defined_type)?;
            
            defined_type.generate(out, schema, executable_document, Some(&self.selections), &self.parsed.alias)?;
        }
        else {
            writeln!(out, "// Nothing to generate because {} is {}", &self.parsed.name, field.ty)?;
        }
        // if !self.selections.is_empty() {
        //     let name = to_pascal_case(&self.name);
        //     self.

            

        //     for selection in &self.selections {
        //         // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

        //         selection.generate_structs(out, &schema.query.get(schema).fields, schema)?;
        //         // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
        //         // writeln!(out, "    {},", to_constant_case(&selection.name))?;
        //     }

        // }
        Ok(())
    }
    
    // pub fn generate(&mut self, out: &mut Output) -> Result<(), Error> {
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
pub struct GenericOperation<'a> {
    pub parsed: &'a parsed_model::GenericOperation<'a>,
    pub context: FieldMapRef<'a>,
    pub variables: FieldMap<'a>,
    pub selections: Vec<Selection<'a>>,
}

impl<'a> GenericOperation<'a> {
    // pub fn from_query(err: &mut ErrorCollector, parsed: parsed_model::Query, schema:  &'a Schema) ->  Result<GenericOperation, Error> {
    //     Self::new(err, OperationType::Query, 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema)
    // }

    // pub fn from_mutation(err: &mut ErrorCollector, parsed: parsed_model::Mutation, schema:  &'a Schema) ->  Result<GenericOperation, Error> {
    //     Self::new(err, OperationType::Mutation, 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema)
    // }
    
    // pub fn from_subscription(parsed: graphql_parser::Subscription, schema:  &Schema, out: &mut Output) ->  Result<GenericOperation, Error> {
    //     Self::new("Subscription", 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema, out)
    // }

    fn get_context2(err: &mut ErrorCollector, operation: &'a OperationType, name: &'a str, position: &'a Pos, schema: &'a Schema<'a>) -> Result<FieldMapRef<'a>, Error> {
        match operation {
            OperationType::Query => Ok(&schema.get_object(schema.query)?.fields),
            OperationType::Mutation => {
                if let Some(mutation) = &schema.mutation {
                    Ok(&schema.get_object(mutation)?.fields)
                }
                else {
                    err.fail(BuildError::MissingObjectError(position.clone(), format!("Mutation {} used but no Mutation root found", name)))
                }
            },
        }
    }

    // fn get_context(&self, err: &mut ErrorCollector, schema: &'a Schema) -> Result<&'a IndexMap<String, Field>, Error> {
    //     Self::get_context2(err, &self.operation, &self.name, &self.position, schema)
    // }

    // fn new(err: &mut ErrorCollector, operation: OperationType, name: String, position: Pos, parsed_variables: Vec<parsed_model::Field>, parsed_selections: Vec<parsed_model::Selection>, schema: &'a Schema) -> Result<GenericOperation, Error> {
    fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::GenericOperation, schema: &'a Schema<'a>, executable_document: &'a parsed_model::ExecutableDocument<'a>)-> Result<Self, Error> {

        let mut variables = IndexMap::new();

        for variable in &parsed.variables {
            if let Ok(field) = Field::from_variable(err, variable, schema) {
                if let Some(existing) = variables.insert(variable.name, field) {
                    err.error(BuildError::DuplicateName(existing.parsed.position.clone(), variable.position.clone(), variable.name.to_string()));
                }
            }
        }

        let context= Self::get_context2(err, &parsed.operation, parsed.name, &parsed.position, schema)?;
        
        let mut selections = Vec::new();
        for selection in &parsed.selections {
            
            // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
            if let Ok(selection) = Selection::new(err, selection, schema, executable_document, context) {
                selections.push(selection);
            }
        }

        err.ok(GenericOperation {
                parsed,
                context,
                variables,
                selections,
            })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{} {{", self.parsed.operation)?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
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
    
    pub fn generate(&'a self, out: &mut Output, schema: &'a Schema<'a>, executable_document: &ExecutableDocument<'_>) -> Result<(), Error> {
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

        // let context=  //self.get_context(schema)?;

        // match &self.operation {
        //     OperationType::Query => Ok(&schema.query.get(schema).fields),
        //     OperationType::Mutation => {
        //         if let Some(mutation_proxy) = &schema.mutation {
        //             Ok(&mutation_proxy.get(schema).fields)
        //         }
        //         else {
        //             err.fail(BuildError::MissingObjectError(&self.position.clone(), format!("Mutation {} used but no Mutation root found", name)))
        //         }
        //     },
        // }

        // fn get_context2<'a>(err: &mut ErrorCollector, operation: &OperationType, name: &String, position: &Pos, schema: &'a Schema) -> Result<&'a IndexMap<String, Field>, Error> {
           
        // }
    
        // fn get_context<'a>(&self, err: &mut ErrorCollector, schema: &'a Schema) -> Result<&'a IndexMap<String, Field>, Error> {
        //     Self::get_context2(err, &self.operation, &self.name, &self.position, schema)
        // }

        writeln!(out, "pub mod {} {{", to_snake_case(&self.parsed.name))?;
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
                        writeln!(out, "#[serde(rename = \"{}\")]", &field.parsed.name)?;
                        writeln!(out, "{}_: {},", to_snake_case(&name), field.ty.rust_type(false))?;
                    }
                }

                writeln!(out, "}}")?;
                writeln!(out, "")?;
            }

            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // writeln!(out, "#[serde(rename = \"{}\")]", self.name)?;
            writeln!(out, "pub struct {} {{", self.parsed.operation)?;

            if !self.variables.is_empty() {
                writeln!(out.indent(), "variables: Variables,")?;
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl {} {{", self.parsed.operation)?;
            {
                let mut out = out.indent();

                writeln!(out, "const REQUEST_NAME: &str = \"{}\";", self.parsed.name)?;

                if self.variables.is_empty() {
                    writeln!(out, "const {}: &str = r#\"{} {} {{", self.parsed.operation.to_upper_case(), self.parsed.operation.to_lower_case(), self.parsed.name)?;
                }
                else {
                    writeln!(out, "const {}: &str = r#\"{} {}(", self.parsed.operation.to_upper_case(), self.parsed.operation.to_lower_case(), self.parsed.name)?;
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
                        selection.generate_query(&mut out, &self.context, schema, Maybe::Maybe)?;
                    }
                }
                writeln!(out, "}}\"#;")?;
                writeln!(out, "")?;

                if self.variables.is_empty() {
                    writeln!(out, "pub fn new() -> {} {{", self.parsed.operation)?;
                    writeln!(out.indent(), "{} {{}}", self.parsed.operation)?;
                    writeln!(out, "}}")?;
                }
                else {

                    writeln!(out, "pub fn from(variables: Variables) -> {} {{", self.parsed.operation)?;
                    writeln!(out.indent(), "{} {{variables}}", self.parsed.operation)?;
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;

                    writeln!(out, "pub fn new(")?;
                    {
                        let mut out = out.indent();
                    
                        for (name, field) in &self.variables {
                            writeln!(out, "{}_: {},", to_snake_case(&name), field.ty.rust_type(false))?;
                        }
                    }
                    writeln!(out, ") -> {} {{", self.parsed.operation)?;
                    {
                        let mut out = out.indent();

                        writeln!(out, "{} {{", self.parsed.operation)?;
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

            writeln!(out, "impl NewGraphQLQuery<Response> for {} {{", self.parsed.operation)?;
            {
                let mut out = out.indent();

                writeln!(out, "fn get_request_name() -> &'static str {{")?;
                writeln!(out.indent(), "Self::REQUEST_NAME")?;
                writeln!(out, "}}")?;

                writeln!(out, "fn get_query() -> &'static str {{")?;
                writeln!(out.indent(), "Self::{}", self.parsed.operation.to_upper_case())?;
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
            // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
            writeln!(out, "pub struct Response {{")?;
            {
                let mut out = out.indent();

                for selection in &self.selections {
                    // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

                    selection.generate_fields(&mut out, &self.context, schema, Maybe::Maybe)?;
                    // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
                    // writeln!(out, "    {},", to_constant_case(&selection.name))?;
                }
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl NewGraphQLResponse for Response {{")?;
            writeln!(out, "}}")?;
            

            for selection in &self.selections {
                // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

                selection.generate_structs(&mut out, &self.context, schema, executable_document)?;
                // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
                // writeln!(out, "    {},", to_constant_case(&selection.name))?;
            }

            for (_name, field) in &self.variables {
                if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
                    defined_type.generate(&mut out, schema, executable_document, None, &None)?;
                }
            }
        }
        writeln!(out, "}}")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct FragmentDefinition<'a> {
    pub parsed: &'a parsed_model::FragmentDefinition<'a>,
    pub selections: Vec<Selection<'a>>,
}

impl<'a> FragmentDefinition<'a> {
    pub fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::FragmentDefinition<'a>, schema: &'a Schema<'a>, executable_document: &'a parsed_model::ExecutableDocument<'a>) -> Result<Self, Error> {

        if let Some(type_definition) = schema.defined_types.get(&parsed.type_condition) {
            if let TypeDefinition::Object(object) = type_definition {
                let context = &object.fields;
                let mut selections = Vec::new();
                for selection in &parsed.selections {
                    
                    // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
                    if let Ok(selection) = Selection::new(err, selection, schema, executable_document, context) {
                        selections.push(selection);
                    }
                }
        
                if selections.is_empty() {
                    return err.fail(BuildError::InvalidQueryError(parsed.position.clone(), format!("FragmentDefinition {} has no selection set", &parsed.name)));
                }

                err.ok(FragmentDefinition {
                    parsed,
                    selections,
                    // query_object,
                })
            }
            else {
                err.fail(BuildError::TypeMismatchError(parsed.position.clone(), format!("expected Object \"{}\" but found {}", &parsed.name, type_definition.type_name())))
            }
        }
        else {
            err.fail(BuildError::MissingObjectError(parsed.position.clone(), parsed.name.to_string()))
        }
    }
    
    // pub fn validate(&mut self, _out: &mut ErrorCollector) -> Result<(), Error> {
    //     Ok(())
    // }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Query {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
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
    pub parsed: &'a parsed_model::FragmentSpread<'a>,
}

impl<'a> FragmentSpread<'a> {
    fn new(err: &mut ErrorCollector<'_>, parsed: &'a parsed_model::FragmentSpread<'a>, _schema: &Schema, executable_document: &parsed_model::ExecutableDocument, context: FieldMapRef<'a>) -> Result<Self, Error> {
        if let Some(_fragment) = executable_document.fragments.get(parsed.name) {

            Ok(FragmentSpread {
                parsed,
            })
        }
        else {
            err.fail(BuildError::MissingFragmentError(parsed.position.clone(), parsed.name.to_string()))
        }
        
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "FragmentSpread {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
    
}

#[derive(Debug)]
pub struct ExecutableDocument<'a> {
    pub parsed: &'a parsed_model::ExecutableDocument<'a>,
    pub fragments: IndexMap<&'a str, FragmentDefinition<'a>>,
    pub queries: Vec<GenericOperation<'a>>,
    pub mutations: Vec<GenericOperation<'a>>,
}

impl<'a> ExecutableDocument<'a> {
    pub fn new(err: &mut ErrorCollector, parsed: &'a parsed_model::ExecutableDocument<'a>, schema: &'a Schema<'a>) ->  Result<Self, Error> {

        let mut fragments = IndexMap::new();

        for fragment_definition in parsed.fragments.values() {
            fragments.insert(fragment_definition.name as &str, FragmentDefinition::new(err, fragment_definition, schema, &parsed)?);
        }

        let mut queries = Vec::new();

        for query in &parsed.queries {
            queries.push(GenericOperation::new(err, query, schema, parsed)?);
        }

        let mut mutations = Vec::new();

        for mutation in &parsed.mutations {
            mutations.push(GenericOperation::new(err, mutation, schema, parsed)?);
        }

        Ok(ExecutableDocument {
            parsed,
            fragments,
            queries,
            mutations,
        })
    }

    pub fn get_fragment(&self, name: &str) -> Result<&'a FragmentDefinition, Error> {
        match self.fragments.get(name) {
            Some(fragment_definition) => {
                Ok(fragment_definition)
            },
            None => Err(Error::BuildFailed(format!("Failed to find Fragment \"{}\"", name))),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "ExecutableDocument {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out);
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

    pub fn generate(&'a self, out: &mut Output, schema: &'a Schema<'a>) -> Result<(), Error> {
        writeln!(out, "pub mod {} {{", self.parsed.name)?;

        {
            let mut out = out.indent();
            for query in &self.queries {
                query.generate(&mut out, schema, self)?;
            }

            for mutation in &self.mutations {
                mutation.generate(&mut out, schema, self)?;
            }
        }
        writeln!(out, "}} // End of executable_document {}", self.parsed.name)?;
        Ok(())
    }
}














// Unordered --------------------------------------------------------------------------------------------------------------------------------------------
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

// #[derive(Debug)]
// pub struct VariableDefinition {
//     parsed: Rc<parsed_model::VariableDefinition>,
//     // queries: Vec<Query>,
// }

// impl VariableDefinition {
//     pub fn new(parsed: parsed_model::VariableDefinition, out: &mut Output) -> Result<VariableDefinition, Error> {


//         Ok(VariableDefinition {
//             parsed,
//         })
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         // writeln!(out, "ExecutableDocument {{")?;
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
    
//     pub fn generate(&mut self, out: &mut Output) -> Result<(), Error> {
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

