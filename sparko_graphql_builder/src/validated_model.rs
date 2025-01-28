use std::collections::HashSet;
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::{to_snake_case, to_constant_case};

use crate::parsed_model::{BuiltinType, DefinedTypeName, OperationType};
use crate::utils::to_pascal_case;
use crate::{intern, Atom, BuildError, Error, ErrorCollector};
use crate::{parsed_model, Output};

const TYPE_NAME: &str = "__typename";
pub type FieldMap = IndexMap<Atom, Rc<Field>>;

#[derive(Debug)]
pub enum DefinedType {
    Scalar(Atom),
    Object(Atom),
    Interface(Atom),
    Union(Atom),
    Enum(Atom),
    InputObject(Atom),
}

impl Display for DefinedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinedType::Scalar(name) => write!(f, "Scalar {}", name),
            DefinedType::Object(name) => write!(f, "Object {}", name),
            DefinedType::Interface(name) => write!(f, "Interface {}", name),
            DefinedType::Union(name) => write!(f, "Union {}", name),
            DefinedType::Enum(name) => write!(f, "Enum {}", name),
            DefinedType::InputObject(name) => write!(f, "InputObject {}", name),
        }
    }
}

impl DefinedType {
    pub fn defined_type_name(&self) -> DefinedTypeName {
        match self {
            DefinedType::Scalar(_) => DefinedTypeName::Scalar,
            DefinedType::Object(_) => DefinedTypeName::Object,
            DefinedType::Interface(_) => DefinedTypeName::Interface,
            DefinedType::Union(_) => DefinedTypeName::Union,
            DefinedType::Enum(_) => DefinedTypeName::Enum,
            DefinedType::InputObject(_) => DefinedTypeName::InputObject,
        }
    }

    pub fn name(&self) -> &str {
        self.defined_type_name().name()
    }

    pub fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<Atom>) -> Result<(), Error> {
        match self {
            DefinedType::Scalar(name) => schema.get_scalar(name)?.generate(out, schema),
            DefinedType::Object(name) => schema.get_object(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::Interface(name) => schema.get_interface(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::Union(name) => schema.get_union(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::Enum(name) => schema.get_enum(name)?.generate(out, schema, executable_document, selections, alias),
            DefinedType::InputObject(name) => schema.get_input_object(name)?.generate(out, schema, executable_document, selections, alias),
        }
    }
    
    fn rust_type(&self) -> String {
        match self {
            DefinedType::Scalar(name) => to_pascal_case(name),
            DefinedType::Object(name) => to_pascal_case(name),
            DefinedType::Interface(name) => to_pascal_case(name),
            DefinedType::Union(name) => to_pascal_case(name),
            DefinedType::Enum(name) => to_pascal_case(name),
            DefinedType::InputObject(name) => to_pascal_case(name),
        }
    }
}

#[derive(Debug)]
pub enum ScalarType {
    DefinedType(DefinedType),
    BuiltinType(BuiltinType),
}

impl ScalarType {
    fn new(err: &mut ErrorCollector, parsed: &parsed_model::ScalarType, schema: &Rc<parsed_model::Schema>) -> Result<Self, Error> {
        match parsed {
            parsed_model::ScalarType::DefinedType(defined_type) => {
                if let Some(def) = schema.named_types.get(&defined_type.name) {
                    Ok(match def {
                        parsed_model::TypeDefinition::Scalar(content) =>ScalarType::DefinedType(DefinedType::Scalar(content.name.clone())),
                        parsed_model::TypeDefinition::Object(content) =>ScalarType::DefinedType(DefinedType::Object(content.name.clone())),
                        parsed_model::TypeDefinition::Interface(content) =>ScalarType::DefinedType(DefinedType::Interface(content.name.clone())),
                        parsed_model::TypeDefinition::Union(content) =>ScalarType::DefinedType(DefinedType::Union(content.name.clone())),
                        parsed_model::TypeDefinition::Enum(content) => ScalarType::DefinedType(DefinedType::Enum(content.name.clone())),
                        parsed_model::TypeDefinition::InputObject(content) =>ScalarType::DefinedType(DefinedType::InputObject(content.name.clone())),
                    })
                }
                else {
                    err.fail(BuildError::UndefinedTypeError(defined_type.position.clone(), format!("Missing type {}", defined_type.name.clone())))
                }
            },
            parsed_model::ScalarType::BuiltinType(builtin_type) => Ok(ScalarType::BuiltinType(builtin_type.clone())),
        }
    }

    fn from_validated(err: &mut ErrorCollector, parsed: &parsed_model::ScalarType, schema: &Rc<Schema>) -> Result<Self, Error> {
        match parsed {
            parsed_model::ScalarType::DefinedType(defined_type) => {
                if let Some(def) = schema.defined_types.get(&defined_type.name) {
                    Ok(match def {
                        TypeDefinition::Scalar(content) =>ScalarType::DefinedType(DefinedType::Scalar(content.parsed.name.clone())),
                        TypeDefinition::Object(content) =>ScalarType::DefinedType(DefinedType::Object(content.parsed.name.clone())),
                        TypeDefinition::Interface(content) =>ScalarType::DefinedType(DefinedType::Interface(content.parsed.name.clone())),
                        TypeDefinition::Union(content) =>ScalarType::DefinedType(DefinedType::Union(content.parsed.name.clone())),
                        TypeDefinition::Enum(content) => ScalarType::DefinedType(DefinedType::Enum(content.parsed.name.clone())),
                        TypeDefinition::InputObject(content) =>ScalarType::DefinedType(DefinedType::InputObject(content.parsed.name.clone())),
                    })
                }
                else {
                    err.fail(BuildError::UndefinedTypeError(defined_type.position.clone(), format!("Missing type {}", defined_type.name.clone())))
                }
            },
            parsed_model::ScalarType::BuiltinType(builtin_type) => Ok(ScalarType::BuiltinType(builtin_type.clone())),
        }
    }
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::DefinedType(defined_type) => write!(f, "{}", defined_type),
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
    fn new(err: &mut ErrorCollector, parsed: &parsed_model::Type, schema: &Rc<parsed_model::Schema>,) -> Result<Self, Error> {
       Ok( match parsed {
            parsed_model::Type::Scalar(scalar_type) => Type::Scalar(ScalarType::new(err, scalar_type, schema)?),
            parsed_model::Type::Required(wrapped) => Type::Required(Box::new(Type::new(err, wrapped, schema)?)),
            parsed_model::Type::Array(wrapped) => Type::Array(Box::new(Type::new(err, wrapped, schema)?)),
        })
    }

    fn from_validated(err: &mut ErrorCollector, parsed: &parsed_model::Type, schema: &Rc<Schema>) -> Result<Self, Error> {
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

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Scalar(scalar_type) => scalar_type.fmt(f),
            Type::Required(wrapped) => write!(f, "{}!", wrapped),
            Type::Array(wrapped) => write!(f, "[{}]", wrapped),
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

    // pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>, selections: Option<&SelectionList>) -> Result<(), Error> {
    //     match self {
    //         TypeDefinition::Enum(content) => content.generate(out, schema, selections),
    //         TypeDefinition::Union(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Object(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Interface(content) => content.generate(out, schema, selections, &None),
    //         TypeDefinition::Scalar(content) => content.generate(out, schema),
    //     }
    // }
}

pub trait Context {
    fn fields(&self) -> &FieldMap;
    // fn get_context(&self, schema: &Rc<Schema>) -> Result<IndexMap<Atom, &FieldMap>, Error>;
}

#[derive(Debug)]
pub struct Scalar {
    pub parsed: Rc<parsed_model::Scalar>,
    pub rust_name: String,
    pub rust_type: String,
}

impl Scalar {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "TypeDef {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;

            writeln!(out, "rust_name: {}", self.rust_name)?;
            writeln!(out, "rust_type: {}", self.rust_type)?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")

        
    }
    
    fn new(parsed: &Rc<parsed_model::Scalar>) -> TypeDefinition {

        let rust_type = "serde_json::Value".to_string(); // TODO: get proper types

        TypeDefinition::Scalar(Rc::new(Scalar {
            parsed: parsed.clone(),
            rust_name: to_pascal_case(&parsed.name),
            rust_type,
        }))
    }
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Rc<Schema>) -> Result<(), Error> {
        writeln!(out, "type {} = {};", &self.rust_name, &self.rust_type)?;
        writeln!(out, "")?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Object {
    pub parsed: Rc<parsed_model::Object>,
    pub fully_implements: Vec<Atom>,
    pub fields: FieldMap,
    pub is_input: bool,
}

impl Context for Object {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }

    // fn get_context(&self, schema: &Rc<Schema>) -> Result<IndexMap<Atom, &XFieldMap>, Error> {
    //     let mut context = IndexMap::new();

    //     context.insert(self.parsed.name.clone(), &self.fields);

    //     Ok(context)
    // }
}

impl Object {

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
    
    fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Object>, schema: &Rc<parsed_model::Schema>,) -> Result<Rc<Self>, Error> {
        let mut fully_implements = Vec::new();

        for interface_name in &parsed.implements {
            if let Some(interface) = schema.get_interface(err, &parsed.position, &interface_name) {
                let mut ok = true;
                    for field_name in interface.fields.keys() {
                        if ! parsed.fields.contains_key(field_name) {
                            ok = false;
                            break;
                        }
                    }
                    if ok {
                        fully_implements.push(interface_name.clone());
                    }
            }
        }

        let mut fields = IndexMap::new();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                fields.insert(name.clone(), field);
            }
        }

        err.ok(Rc::new(Object {
                parsed: parsed.clone(),
                fully_implements,
                fields,
                is_input: parsed.is_input,
            }))
    }
    
    fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<Atom>) -> Result<(), Error> {
    //(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<String>) -> Result<(), Error> {
        generate_struct(out, schema, executable_document, selections, alias, &self.parsed.name, self, self.is_input)
    }
}



fn generate_struct(out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<Atom>, name: &Atom, context: &dyn Context, is_input: bool) -> Result<(), Error> {
    if let Some(selections) = selections {
        let abstract_name = Rc::new(format!("Abstract{}", to_pascal_case(name)));
        let variants = selections.get_variants(&abstract_name, schema, executable_document, context)?;
        if variants.len() > 1 {
            writeln!(out, "/* {} variants */", variants.len())?;
            let base_type_name = to_pascal_case(name);
            let base_member_name = to_snake_case(name);
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            writeln!(out, "pub enum {} {{", base_type_name)?;
            for (name, variant) in &variants {
                writeln!(out, "  {}({}),", name, name)?;
            }
    
            // for object in self.implemented_by.iterator(schema) {
                
            //     writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
            // }
    
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            for (name, (context, selections)) in &variants {
                let rust_name = to_pascal_case(&name);

                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "#[serde(rename = \"{}\")]", name)?;
                writeln!(out, "pub struct {} {{", rust_name)?;

                if name.as_ref() != name.as_ref() {
                    writeln!(out, "#[serde(flatten)]")?;
                    // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
                    writeln!(out, "pub {}_: {},", base_member_name, base_type_name)?;
                    
                }
                //context nees to come from vRIANT
                let mut field_map = IndexMap::new();
                selections.gather_fields(&mut field_map, *context, schema, executable_document, true)?;

                selections.generate_fields(out, *context, schema, executable_document, true, &field_map)?;

                writeln!(out, "}}")?;
                writeln!(out, "")?;
            }
        }
        else {
            writeln!(out, "/* No variants */")?;



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



            let mut field_map = IndexMap::new();
            selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

            selections.generate_fields(out, context, schema, executable_document, true, &field_map)?;
    
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            selections.generate_structs(out, context, schema, executable_document, true, &field_map, alias)?;
        }
    }
    else {
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

        for field in context.fields().values() {
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
                for field in context.fields().values() {
                    writeln!(out, "        {}_: None,", to_snake_case(&name))?;
                }
                writeln!(out, "    }}")?;
                writeln!(out, "}}")?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "#[derive(Debug)]")?;
            writeln!(out, "pub struct {}Builder {{", rust_name)?;

            for field in context.fields().values() {
                writeln!(out, "    {}_: {},", to_snake_case(&name), field.ty.rust_type(false))?;
            }
        
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl {}Builder {{", rust_name)?;
            {
                let mut out = out.indent();

                for field in context.fields().values() {
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

                    for field in context.fields().values() {
                        if let Type::Required(_) = field.ty {
                            writeln!(out, "if let None = self.{}_ {{", to_snake_case(&name))?;
                            writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", name)?;
                            writeln!(out, "}}")?;
                        }
                    }

                    writeln!(out, "Ok({} {{", rust_name)?;
                    {
                        let mut out = out.indent();
    
                        for field in context.fields().values() {
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
pub struct Interface {
    pub parsed: Rc<parsed_model::Interface>,
    pub implemented_by: Vec<Atom>,
    pub fields: FieldMap,
}

impl Context for Interface {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }

//     fn get_context(&self, schema: &Rc<Schema>) -> Result<IndexMap<Atom, &FieldMap>, Error> {
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
}

impl Interface {

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

    fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Interface>, schema: &Rc<parsed_model::Schema>,) -> Result<TypeDefinition, Error> {
        let mut implemented_by = Vec::new();

        
        for defined_type in schema.named_types.values() {
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
        //         out.error(BuildError::MissingInterfaceError(parsed.position, format!("Interface {} Not Found", parsed.name)))
        //     };

            
        // }



        let mut fields = IndexMap::new();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                fields.insert(name.clone(), field);
            }
        }

        err.ok(TypeDefinition::Interface(Rc::new(Interface {
                parsed: parsed.clone(),
                implemented_by,
                fields,
            })))
    }
    
    fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<Atom>) -> Result<(), Error> {
        generate_struct(out, schema, executable_document, selections, alias, &self.parsed.name, self, false)?;
        // if let Some(selections) = selections {
        //     let variants = selections.get_variants(&self.parsed.name, schema, executable_document, self)?;
        //         if variants.len() > 0 {
        //             writeln!(out, "/* {} variants */", variants.len())?;
        //             let base_type_name = to_pascal_case(&self.parsed.name);
        //             let base_member_name = to_snake_case(&self.parsed.name);
        //             writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //             writeln!(out, "pub enum {} {{", base_type_name)?;
        //             for (name, variant) in &variants {
        //                 writeln!(out, "  {}({}),", name, name)?;
        //             }
            
        //             // for object in self.implemented_by.iterator(schema) {
                        
        //             //     writeln!(out, "    {}({}),", to_pascal_case(&object.name), to_pascal_case(&object.name))?;
        //             // }
            
        //             writeln!(out, "}}")?;
        //             writeln!(out, "")?;

        //             for (name, (context, selections)) in &variants {
        //                 let rust_name = to_pascal_case(&name);

        //                 writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        //                 writeln!(out, "#[serde(rename = \"{}\")]", name)?;
        //                 writeln!(out, "pub struct {} {{", rust_name)?;

        //                 if name.as_ref() != self.parsed.name.as_ref() {
        //                     writeln!(out, "#[serde(flatten)]")?;
        //                     // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
        //                     writeln!(out, "pub {}_: {},", base_member_name, base_type_name)?;
                            
        //                 }
        //                 //context nees to come from vRIANT
        //                 let mut field_map = IndexMap::new();
        //                 selections.gather_fields(&mut field_map, *context, schema, executable_document, true)?;

        //                 selections.generate_fields(out, *context, schema, executable_document, true, &field_map)?;

        //                 writeln!(out, "}}")?;
        //                 writeln!(out, "")?;
        //             }
        //         }
        //         else {
        //             writeln!(out, "/* No variants */")?;
                    
        //         }
        // }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Union {
    pub parsed: Rc<parsed_model::Union>,
    pub fields: FieldMap,
}

impl Context for Union {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
    // fn get_context(&self, schema: &Rc<Schema>) -> Result<IndexMap<Atom, &XFieldMap>, Error> {
    //     let mut context = IndexMap::new();

    //     context.insert(self.parsed.name.clone(), &self.fields);

    //     Ok(context)
    // }
}

impl Union {
    fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Union>, schema: &Rc<parsed_model::Schema>,) -> Result<TypeDefinition, Error> {
        let mut fields = IndexMap::new();
        let mut err = err.child();

        for type_name in &parsed.types {
            if let Some(object) = schema.get_object(&mut err, &parsed.position, type_name) {
                for (name, field) in &object.fields {
                    if let Ok(field) = Field::new(&mut err, field, schema) {
                        fields.insert(name.clone(), field);
                    }
                }
            }
            else {
                err.error(BuildError::MissingInterfaceError(parsed.position.clone(), format!("Interface {} Not Found", parsed.name)))
            };
        }

        err.ok(TypeDefinition::Union(Rc::new(Union {
            parsed: parsed.clone(),
            fields,
        })))
    }

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
    

    fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: Option<&SelectionList>, alias: &Option<Atom>) -> Result<(), Error> {
        generate_struct(out, schema, executable_document, selections, alias, &self.parsed.name, self, false)
    }
    
    // fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, selections: Option<&SelectionList>, alias: &Option<String>) -> Result<(), Error> {
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
pub struct Enum {
    pub parsed: Rc<parsed_model::Enum>,
}

impl Enum {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Enum {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
        }
        writeln!(out, "}}")
    }

    pub fn new(parsed: &Rc<parsed_model::Enum>) -> TypeDefinition {
        TypeDefinition::Enum(Rc::new(Enum {
            parsed: parsed.clone(),
        }))
    }
    
    fn generate(&self, out: &mut Output<'_>, _schema: &Rc<Schema>, _executable_document: &ExecutableDocument, _selections: Option<&SelectionList>, _alias: &Option<Atom>) -> Result<(), Error> {
    // fn generate(&self, out: &mut Output<'_>, _schema: &Rc<Schema>, _selections: Option<&SelectionList>) -> Result<(), Error> {
        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
        writeln!(out, "pub enum {} {{", to_pascal_case(&self.parsed.name))?;

        for variant in &self.parsed.variants {
            writeln!(out, "    #[serde(rename = \"{}\")]", &variant.name)?;
            writeln!(out, "    {},", to_constant_case(&variant.name))?;
        }
        writeln!(out, "}}")?;
        writeln!(out, "")?;
        Ok(())
    }
}

// #[derive(Debug)]
// pub struct ObjectProxy {
//     pub name: Atom,
// }

// impl ObjectProxy {
//     pub fn new(name: Atom) -> Self {
//         ObjectProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &Rc<Schema>) -> Result<&'a Object, Error> {
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

//     pub fn option_as_str(proxy: &'a Option<ObjectProxy>) -> Atom {
//         match proxy {
//             Some(proxy) => proxy.as_str(),
//             None => "None",
//         }
//     }
// }

// #[derive(Debug)]
// pub struct InterfaceProxy {
//     pub name: Atom,
// }

// impl InterfaceProxy {
//     pub fn new(name: Atom) -> Self {
//         InterfaceProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &Rc<Schema>) -> Result<&'a Interface, Error> {
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
// pub struct ScalarProxy {
//     pub name: Atom,
// }

// impl ScalarProxy {
//     pub fn new(name: Atom) -> Self {
//         ScalarProxy {
//             name,
//         }
//     }
    
//     pub fn get(&self, schema: &Rc<Schema>) -> Result<&'a Scalar, Error>{
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
// pub struct ObjectListProxy {
//     pub names: Vec<Atom>,
// }

// impl ObjectListProxy {
//     pub fn new(names: Vec<Atom>) -> Self {
//         ObjectListProxy { names }
//     }

//     fn iterator(&self, schema: &Rc<Schema>) -> ObjectListProxyItertor {
//         ObjectListProxyItertor {
//             list: self,
//             schema,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// // impl IntoIterator for &'a ObjectListProxy {
// //     type Item = &'a Interface;
// //     type IntoIter = ObjectListProxyItertor;
    
// //     fn into_iter(self) -> Self::IntoIter {
// //         ObjectListProxyItertor {
// //             list: &self,
// //             index: 0,
// //             // iter: self.names.iter(),
// //         }
// //     }
// // }

// pub struct ObjectListProxyItertor {
//     list: &'a ObjectListProxy,
//     schema: &Rc<Schema>,
//     index: usize,
// }

// impl /*Iterator for*/ ObjectListProxyItertor {
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

//     fn _iterator(&self, schema: &Rc<Schema>) -> InterfaceListProxyItertor {
//         InterfaceListProxyItertor {
//             list: self,
//             schema,
//             index: 0,
//             // iter: self.names.iter(),
//         }
//     }
// }

// // impl IntoIterator for &'a InterfaceListProxy {
// //     type Item = &'a Interface;
// //     type IntoIter = InterfaceListProxyItertor;
    
// //     fn into_iter(self) -> Self::IntoIter {
// //         InterfaceListProxyItertor {
// //             list: &self,
// //             index: 0,
// //             // iter: self.names.iter(),
// //         }
// //     }
// // }

// pub struct InterfaceListProxyItertor {
//     list: &'a InterfaceListProxy,
//     schema: &Rc<Schema>,
//     index: usize,
// }

// impl InterfaceListProxyItertor {
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
// // pub struct FragmentProxy {
// //     pub name: Atom,
// // }

// // impl FragmentProxy {
// //     pub fn new(name: Atom) -> Self {
// //         FragmentProxy {
// //             name,
// //         }
// //     }
    
// //     pub fn get(&self, executable_document: &Rc<parsed_model::ExecutableDocument>) -> Result<&'a parsed_model::FragmentDefinition, Error> {
// //         match executable_document.fragments.get(self.name) {
// //             Some(fragment_definition) => {
// //                 Ok(fragment_definition)
// //             },
// //             None => Err(Error::BuildFailed(format!("FragmentProxy failed to find \"{}\"", &self.name))),
// //         }
// //     }
// // }

#[derive(Debug)]
pub struct Field {
    pub parsed: Rc<parsed_model::Field>,
    pub ty: Type,
}

impl Field{
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
    
    fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Field>, schema: &Rc<parsed_model::Schema>) -> Result<Rc<Field>, Error> {
        Ok(Rc::new(Field {
            parsed: parsed.clone(),
            ty: Type::new(err, &parsed.ty, schema)?,
        }))
    }

    // pub fn rust_type(&self, schema: &Rc<Schema>, maybe_optional: Maybe, alias: &Option<String>) -> String {
    //     self.ty.rust_type()   .rust_type(self.multiple, self.nonnull, schema, maybe_optional, alias)
    // }

    pub fn graphql_name(&self) -> String {
        self.ty.graphql_name()
    }
    
    fn from_variable(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Field>, schema: &Rc<Schema>) -> Result<Rc<Self>, Error> {
        Ok(Rc::new(Field {
            parsed: parsed.clone(),
            ty: Type::from_validated(err, &parsed.ty, schema)?,
        }))
    }
}

#[derive(Debug)]
pub struct Schema {
    pub defined_types: IndexMap<Atom, TypeDefinition>,
    pub query: Atom,
    pub mutation: Option<Atom>,
    pub subscription: Option<Atom>,
}

impl Schema {
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

    pub fn get_scalar(&self, name: &Atom) -> Result<&Rc<Scalar>, Error> {
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

    pub fn get_object(&self, name: &Atom) -> Result<&Rc<Object>, Error> {
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

    pub fn get_interface(&self, name: &Atom) -> Result<&Rc<Interface>, Error> {
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
    
    fn get_union(&self, name: &Atom) -> Result<&Rc<Union>, Error> {
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
    
    fn get_enum(&self, name: &Atom) -> Result<&Rc<Enum>, Error> {
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

    pub fn get_input_object(&self, name: &Atom) -> Result<&Rc<Object>, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            if let TypeDefinition::InputObject(object) = type_definition {
                Ok(object)
            }
            else {
                Err(Error::BuildFailed(format!("Expected InputObject for \"{}\" but found {}", name, type_definition.type_name())))
            }
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find InputObject \"{}\"", name)))
        }
    }

    fn new_get_object(out: &mut ErrorCollector, name: &Option<Atom>, position: &Pos, defined_types: &IndexMap<Atom, TypeDefinition>) -> Option<Atom> {
        if let Some(name) = name {
            if let Some(type_definition) = &defined_types.get(name) {
                if let TypeDefinition::Object(_) = type_definition {
                    Some(name.clone())
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

    fn new_get_default_object(out: &mut ErrorCollector, name: &str, defined_types: &IndexMap<Atom, TypeDefinition>, missing_error: Option<BuildError>) -> Option<Atom> {
        let name = intern(String::from(name));
        if let Some(type_definition) = &defined_types.get(&name) {
            if let TypeDefinition::Object(_) = type_definition {
                Some(name)
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

    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Schema>) -> Result<Rc<Self>, Error> {

        let mut defined_types: IndexMap<Atom, TypeDefinition> = IndexMap::new();

        for parsed_type in parsed.named_types.values() {
            let defined_type = match parsed_type {
                parsed_model::TypeDefinition::Scalar(type_def) => Scalar::new(&type_def),
                parsed_model::TypeDefinition::Object(object) => TypeDefinition::Object(Object::new(err, &object, &parsed)?),
                parsed_model::TypeDefinition::Interface(interface) => Interface::new(err, interface, &parsed)?,
                parsed_model::TypeDefinition::Union(union) => Union::new(err, union, &parsed)?,
                parsed_model::TypeDefinition::Enum(enum_definition) => Enum::new(enum_definition),
                parsed_model::TypeDefinition::InputObject(object) => TypeDefinition::InputObject(Object::new(err, &object, &parsed)?),
            };
            defined_types.insert(parsed_type.name().clone(), defined_type);
        }
        
        

        let mutation = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.mutation, &schema_definition.position, &defined_types) 
        }
        else {
            Self::new_get_default_object(err, "Mutation", &defined_types, None)
        };
        
        let subscription = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.subscription, &schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, "Subscription", &defined_types, None)
        };

        let query = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.query, &schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, "Query", &defined_types, Some(BuildError::NoQueryDefinition))
        };

        err.ok(Rc::new(Schema {
                defined_types,
                query: query.ok_or(Error::BuildFailed(format!("Failed to find previously validated query")))?,
                mutation,
                subscription,
            }))
    }


    // pub fn generate(&self, out: &mut Output) -> Result<(), Error> {
    //     for (_, defined_type) in &self.defined_types {
    //         defined_type.generate(out, self, None)?;
    //     }
    //     Ok(())
    // }
    
}

#[derive(Debug)]
pub enum Selection {
    List(Rc<SelectionList>),
    Field(Rc<SelectionField>),
    FragmentSpread(Rc<FragmentSpread>),
    // InlineFragment(InlineFragment),
} 

impl Selection {
    pub fn new(err: &mut ErrorCollector, selection: &parsed_model::Selection, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument, context: &dyn Context) -> Result<Self, Error> {
        match selection {
            parsed_model::Selection::Field(selection_field) => {

            // writeln!(out, "/* Selection Field:" );
            // selection_field.print(out);
            // writeln!(out, "Context:" );
            // context.print(out);
            // writeln!(out, "*/" );

                Ok(Selection::Field(Rc::new(SelectionField::new(err, selection_field, schema, executable_document, context)?)))
            },
            parsed_model::Selection::Fragment(fragment_spread) => Ok(Selection::FragmentSpread(Rc::new(FragmentSpread::new(err, fragment_spread,
                 schema, executable_document, context)?))),
            
            // graphql_parser::query::Selection::FragmentSpread(fragment_spread) => todo!(),
            // graphql_parser::query::Selection::InlineFragment(inline_fragment) => todo!(),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            Selection::List(selection_list) => selection_list.print(out),
            Selection::Field(selection_field) => selection_field.print(out),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.print(out),
        }
    }

    // pub fn position(&self) -> Pos {
    //     match self {
    //         Selection::Field(selection_field) => selection_field.position,
    //     }
    // }

    pub fn generate_query(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
        match self {
            Selection::List(selection_list) => selection_list.generate_query(out, context, schema, executable_document, maybe_optional),
            Selection::Field(selection_field) => selection_field.generate_query(out, context, schema, executable_document, maybe_optional),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.generate_query(out, context, schema, executable_document, maybe_optional),
        }
    }

    // pub fn generate_fields(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
    //     match self {
    //         Selection::List(selection_list) => selection_list.generate_fields(out, context, schema, executable_document, maybe_optional),
    //         Selection::Field(selection_field) => selection_field.generate_fields(out, context, schema, executable_document, maybe_optional),
    //         Selection::FragmentSpread(fragment_spread) => fragment_spread.generate_fields(out, context, schema, executable_document, maybe_optional),
    //     }
    // }

    // fn gather_fields(&self, field_map: &mut IndexMap<Atom, Rc<SelectionField>>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
    //     match self {
    //         Selection::List(selection_list) => selection_list.gather_fields(field_map, context, schema, executable_document, maybe_optional),
    //         Selection::Field(selection_field) => selection_field.gather_fields(field_map, context, schema, executable_document, maybe_optional),
    //         Selection::FragmentSpread(fragment_spread) => fragment_spread.gather_fields(field_map, context, schema, executable_document, maybe_optional),
    //     }
    // }
    
    // fn generate_structs(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
    //     match self {
    //         Selection::List(selection_list) => selection_list.generate_structs(out, context, schema, executable_document),
    //         Selection::Field(selection_field) => selection_field.generate_structs(out, context, schema, executable_document),
    //         Selection::FragmentSpread(fragment_spread) =>fragment_spread.generate_structs(out, context, schema, executable_document),
    //     }
    // }
}

#[derive(Debug)]
pub struct SelectionList {
    selections: Vec<Selection>,
}

impl SelectionList {
    fn new() -> Self {
        Self {
            selections: Vec::new(),
        }
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "selections {{")?;
        {
            let mut out = out.indent();

            for item in &self.selections {
                item.print(&mut out)?;
            }
        }
        writeln!(out, "}}")
    }
    
    // fn generate_structs(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
    //     for selection in &self.selections {
    //         selection.generate_structs(out, context, schema, executable_document)?;
    //     }
    //     Ok(())
    // }
    
    fn generate_query(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
        if ! &self.selections.is_empty() {
            writeln!(out, "{{")?;
            {
                let mut out = out.indent();

            
                for selection in &self.selections {
                    selection.generate_query(&mut out, context, schema, executable_document, true)?;
                }
            }
            writeln!(out, "}}")?;
        }
        Ok(())
    }
    
    fn gather_fields(&self, field_map: &mut IndexMap<Atom, Rc<SelectionField>>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
        for selection in &self.selections {

            match selection {
                Selection::Field(selection_field) => {
                    field_map.insert(selection_field.parsed.name.clone(), selection_field.clone());
                },
                Selection::FragmentSpread(fragment_spread) => {
                    let fragment = executable_document.get_fragment(&fragment_spread.parsed.name)?;
                    
                    fragment.selections.gather_fields(field_map, context, schema, executable_document, maybe_optional)?;
                },
                Selection::List(selection_list) => {
                    selection_list.gather_fields(field_map, context, schema, executable_document, maybe_optional)?;
                },
            };
        }
        Ok(())
    }
    
    fn generate_structs(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool, field_map: &IndexMap<Atom, Rc<SelectionField>>, alias: &Option<Atom>) -> Result<(), Error> {
        for (name, selection_field) in field_map {
            if name.as_ref() == TYPE_NAME {}
            else {
                if let Some(field) = context.fields().get(name) {
                    if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
                        defined_type.generate(out, schema, executable_document, Some(&selection_field.selections), alias)?;
                    }
                }
                else {
                    writeln!(out, "UNKNOWN FIELD 2 {}", &selection_field.parsed.name)?;
                }
            }
        }
        Ok(())
    }
    
    fn generate_fields(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool, field_map: &IndexMap<Atom, Rc<SelectionField>>) -> Result<(), Error> {
        

        for field in field_map.values() {
            field.generate_fields(out, context, schema, executable_document, maybe_optional)?;
        }

        Ok(())
    }

    fn get_variants<'a>(&self, name: &Rc<String>, schema: &'a Rc<Schema>, executable_document: &ExecutableDocument, context: &'a dyn Context) -> Result<IndexMap<Rc<String>, (&'a dyn Context, SelectionList)>, Error> {
        let mut variants = IndexMap::new();

        self.gather_variants(name, schema, executable_document, context, &mut variants)?;

        Ok(variants)
    }

    // fn get_or_create_variant(variants: &mut IndexMap<Rc<String>, Vec<Selection>>, name: &Rc<String>) -> &mut Vec<Selection> {
    //     variants.entry(name.clone()).or_insert(Vec::new())
    // }

    fn gather_variants<'a>(&self, name: &Rc<String>, schema: &'a Rc<Schema>, executable_document: &ExecutableDocument, context: &'a dyn Context, variants: &mut IndexMap<Rc<String>, (&'a dyn Context, SelectionList)>) -> Result<(), Error> {
        for selection in &self.selections {
            match selection {
                Selection::Field(selection_field) => {
                    variants.entry(name.clone()).or_insert((context, SelectionList::new())).1.selections.push(Selection::Field(selection_field.clone()));
                },
                Selection::FragmentSpread(fragment_spread) => {
                    let fragment = executable_document.get_fragment(&fragment_spread.parsed.name)?;
                    let name = &fragment.parsed.type_condition;
                    let fragment_context = schema.get_object(name)?.as_ref();
                    variants.entry(name.clone()).or_insert((fragment_context, SelectionList::new())).1.selections.push(Selection::FragmentSpread(fragment_spread.clone()));
                },
                Selection::List(selection_list) => {
                    selection_list.gather_variants(name, schema, executable_document, context, variants)?;
                },
            };
        }
        Ok(())
    }
    
    // fn selections_to_fields(selections: &SelectionList, executable_document: &ExecutableDocument, context: &dyn Context) -> Result<IndexMap<Atom, Rc<SelectionField>>, Error> {
        
    
    //     gather_fields(&mut field_map, selections, executable_document, context)?;
    
    //     Ok(field_map)
    // }
    
    // fn generate_record(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
    //     // if self.has_variants() {
    //     //     // we are generating an enum
    //     // }
    //     // else {
    //     //     // we are generating  struct
    //     // }
    //     writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
    //     // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
    //     writeln!(out, "pub struct Response {{")?;
    //     {
    //         let mut out = out.indent();

    //         for selection in &self.selections {
    //             // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

    //             selection.generate_fields(&mut out, context, schema, executable_document, true)?;
    //             // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
    //             // writeln!(out, "    {},", to_constant_case(&selection.name))?;
    //         }
    //     }
    //     writeln!(out, "}}")?;
    //     writeln!(out, "")?;
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct SelectionField {
    pub parsed: Rc<parsed_model::SelectionField>,
    pub nonnull: bool,
    pub arguments: Vec<Rc<parsed_model::Argument>>,
    pub selections: SelectionList,
}

impl SelectionField {
    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::SelectionField>, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument, context: &dyn Context) -> Result<Self, Error> {
        let mut arguments = Vec::new();
        let mut selections = SelectionList::new();

        println!("SlectionField name={} alias={:?}", &parsed.name, &parsed.alias);

        if *parsed.name == TYPE_NAME {
            println!("HERE TYPE_NAME");
        }
        else {
            if let Some(field) = context.fields().get(&parsed.name) {
                for parsed_argument in &parsed.arguments {
                    // arguments.push(Argument::new(prsed_argument));
                    arguments.push(parsed_argument.clone());
                }

                if ! parsed.selections.is_empty() {
                    if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
                        let optional_context: Option<&dyn Context> =  match defined_type {
                            DefinedType::Scalar(_) => None,
                            DefinedType::Object(name) => Some(schema.get_object(name)?.as_ref()),
                            DefinedType::Interface(name) => Some(schema.get_interface(name)?.as_ref()),
                            DefinedType::Union(name) => Some(schema.get_union(name)?.as_ref()),
                            DefinedType::Enum(_) => None,
                            DefinedType::InputObject(_) => {
                                err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Selection on InputObject {}", field.graphql_name())));
                                None
                            },
                        };
        
                        if let Some(context) = optional_context {
                            for selection in &parsed.selections {
                                if let Ok(selection) = Selection::new(err, &selection, schema, executable_document, context) {
                                    selections.selections.push(selection);
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
            parsed: parsed.clone(),
            nonnull: !parsed.optional,
            arguments,
            selections,
        })
    }

    // pub fn old(err: &mut ErrorCollector, parsed: &Rc<parsed_model::SelectionField>, schema: &Rc<Schema>, executable_document: &Rc<parsed_model::ExecutableDocument>, context: FieldMapRef) -> Result<Self, Error> {

        
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
    //             parsed: parsed.clone(),
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

            self.selections.print(&mut out)?;
        }
        writeln!(out, "}}")
    }

    pub fn generate_query(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
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
                    argument.generate_query(&mut out)?;
                }
            }
            writeln!(out, ")")?;
        }

        self.selections.generate_query(out, context, schema, executable_document, true)?;
        Ok(())
    }

    pub fn generate_fields(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
        if *self.parsed.name == TYPE_NAME {
            println!("HERE");
            writeln!(out, "/* HERE1 */ pub __typename: String,")?;
        }
        else {
            // let field = context.fields().get(&self.parsed.name).unwrap();
            let field = match context.fields().get(&self.parsed.name) {
                Some(f) => f,
                None => {
                    panic!("Failed to find field {}", self.parsed.name);
                },
            };
            let name = if let Some(alias) = &self.parsed.alias {
                alias
            }
            else {
                &self.parsed.name
            };

            let rust_type = if let Some(alias) = &self.parsed.alias {
                to_pascal_case(alias)
            }
            else {
                field.ty.rust_type(self.nonnull)
            };

            writeln!(out, "#[serde(rename = \"{}\")]", &name)?;
            writeln!(out, "/* HERE1 */ pub {}_: {},", to_snake_case(&name), rust_type)?;
        }
        Ok(())
    }

    pub fn generate_structs(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
        let field = context.fields().get(&self.parsed.name).unwrap();

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
pub struct GenericOperation {
    pub parsed: Rc<parsed_model::GenericOperation>,
    // pub context: FieldMap,
    pub variables: FieldMap,
    pub selections: SelectionList,
}

impl GenericOperation {
    // pub fn from_query(err: &mut ErrorCollector, parsed: parsed_model::Query, schema:  &Rc<Schema>) ->  Result<GenericOperation, Error> {
    //     Self::new(err, OperationType::Query, 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema)
    // }

    // pub fn from_mutation(err: &mut ErrorCollector, parsed: parsed_model::Mutation, schema:  &Rc<Schema>) ->  Result<GenericOperation, Error> {
    //     Self::new(err, OperationType::Mutation, 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema)
    // }
    
    // pub fn from_subscription(parsed: graphql_parser::Subscription, schema:  &Rc<Schema>, out: &mut Output) ->  Result<GenericOperation, Error> {
    //     Self::new("Subscription", 
    //         parsed.name, parsed.position,
    //         parsed.variables, parsed.selections, schema, out)
    // }

    fn get_context2<'a>(err: Option<&mut ErrorCollector>, operation: &OperationType, name: &Atom, position: &Pos, schema: &'a Rc<Schema>) -> Result<Box<&'a dyn Context>, Error> {
        match operation {
            OperationType::Query => Ok(Box::new(schema.get_object(&schema.query)?.as_ref())),
            OperationType::Mutation => {
                if let Some(mutation) = &schema.mutation {
                    Ok(Box::new(schema.get_object(mutation)?.as_ref()))
                }
                else {
                    if let Some(err) = err {
                        err.fail(BuildError::MissingObjectError(position.clone(), format!("Mutation {} used but no Mutation root found", name)))
                    }
                    else {
                        Err(Error::BuildFailed(format!("Previously validated Mutation {} used but no Mutation root found", name)))
                    }
                }
            },
        }
    }

    fn get_context<'a>(&self, schema: &'a Rc<Schema>) -> Result<Box<&'a dyn Context>, Error> {
        Self::get_context2(None, &self.parsed.operation, &self.parsed.name, &self.parsed.position, schema)
    }

    // fn new(err: &mut ErrorCollector, operation: OperationType, name: String, position: Pos, parsed_variables: Vec<parsed_model::Field>, parsed_selections: Vec<parsed_model::Selection>, schema: &Rc<Schema>) -> Result<GenericOperation, Error> {
    fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::GenericOperation>, schema: &Rc<Schema>, executable_document: &Rc<parsed_model::ExecutableDocument>)-> Result<Self, Error> {

        let mut variables = IndexMap::new();

        for (name, variable) in &parsed.variables {
            if let Ok(field) = Field::from_variable(err, variable, schema) {
                if let Some(existing) = variables.insert(name.clone(), field) {
                    err.error(BuildError::DuplicateName(existing.parsed.position.clone(), variable.position.clone(), variable.name.to_string()));
                }
            }
        }

        let context= *Self::get_context2(Some(err), &parsed.operation, &parsed.name, &parsed.position, schema)?;
        
        let mut selections = SelectionList::new();
        for selection in &parsed.selections {
            
            // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
            if let Ok(selection) = Selection::new(err, selection, schema, executable_document, context) {
                selections.selections.push(selection);
            }
        }

        err.ok(GenericOperation {
                parsed: parsed.clone(),
                // context,
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

            self.selections.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
    
    pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
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

        // fn get_context2(err: &mut ErrorCollector, operation: &OperationType, name: &String, position: &Pos, schema: &Rc<Schema>) -> Result<&'a IndexMap<String, Field>, Error> {
           
        // }
    
        // fn get_context(&self, err: &mut ErrorCollector, schema: &Rc<Schema>) -> Result<&'a IndexMap<String, Field>, Error> {
        //     Self::get_context2(err, &self.operation, &self.name, &self.position, schema)
        // }

        let context = *self.get_context( schema)?;

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
                    write!(out, ")")?;
                }
                self.selections.generate_query(&mut out, context, schema, executable_document, true)?;
                
                writeln!(out, "\"#;")?;
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


            let mut field_map = IndexMap::new();
            self.selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

            // self.selections.generate_record(&mut out, context, schema, executable_document, true)?;
            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
            writeln!(out, "pub struct Response {{")?;
            {
                let mut out = out.indent();
        
                self.selections.generate_fields(&mut out, context, schema, executable_document, true, &field_map)?;
                // for selection in &self.selections {
                //     // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

                //     selection.generate_fields(&mut out, context, schema, executable_document, true)?;
                //     // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
                //     // writeln!(out, "    {},", to_constant_case(&selection.name))?;
                // }
            }
            writeln!(out, "}}")?;
            writeln!(out, "")?;

            writeln!(out, "impl NewGraphQLResponse for Response {{")?;
            writeln!(out, "}}")?;
            
            self.selections.generate_structs(&mut out, context, schema, executable_document, true, &field_map, &None)?;
            // for selection in &self.selections.selections {
            //     // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

            //     selection.generate_structs(&mut out, context, schema, executable_document)?;
            //     // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
            //     // writeln!(out, "    {},", to_constant_case(&selection.name))?;
            // }

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
pub struct FragmentDefinition {
    pub parsed: Rc<parsed_model::FragmentDefinition>,
    pub selections: SelectionList,
}

impl FragmentDefinition {
    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::FragmentDefinition>, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument) -> Result<Self, Error> {

        if let Some(type_definition) = schema.defined_types.get(&parsed.type_condition) {
            if let TypeDefinition::Object(object) = type_definition {
                let mut selections = SelectionList::new();
                for selection in &parsed.selections {
                    
                    // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
                    if let Ok(selection) = Selection::new(err, selection, schema, executable_document, object.as_ref()) {
                        selections.selections.push(selection);
                    }
                }
        
                if selections.selections.is_empty() {
                    return err.fail(BuildError::InvalidQueryError(parsed.position.clone(), format!("FragmentDefinition {} has no selection set", &parsed.name)));
                }

                err.ok(FragmentDefinition {
                    parsed: parsed.clone(),
                    selections,
                    // query_object,
                })
            }
            else {
                err.fail(BuildError::TypeMismatchError(parsed.position.clone(), format!("expected Object \"{}\" but found {}", &parsed.name, type_definition.type_name())))
            }
        }
        else {
            err.fail(BuildError::MissingObjectError(parsed.position.clone(), format!("Fragment {} has missing type condition {}", parsed.name, &parsed.type_condition)))
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

                for selection in &self.selections.selections {
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
    pub parsed: Rc<parsed_model::FragmentSpread>,
}

impl FragmentSpread {
    fn new(err: &mut ErrorCollector<'_>, parsed: &Rc<parsed_model::FragmentSpread>, _schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument, context: &dyn Context) -> Result<Self, Error> {
        if let Some(_fragment) = executable_document.fragments.get(&parsed.name) {

            Ok(FragmentSpread {
                parsed: parsed.clone(),
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
    
    pub fn generate_query(&self, out: &mut Output, _context: &dyn Context, _schema: &Rc<Schema>, _executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
        writeln!(out, "...{}", &self.parsed.name)?;
        Ok(())
    }

    // pub fn generate_fields(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
    //     if *self.parsed.name == TYPE_NAME {
    //         println!("HERE");
    //     }
    //     let field = context.fields().get(&self.parsed.name).unwrap();
    //     let name = if let Some(alias) = &self.parsed.alias {
    //         alias
    //     }
    //     else {
    //         &self.parsed.name
    //     };

    //     let rust_type = if let Some(alias) = &self.parsed.alias {
    //         to_pascal_case(alias)
    //     }
    //     else {
    //         field.ty.rust_type(self.nonnull)
    //     };

    //     writeln!(out, "#[serde(rename = \"{}\")]", &name)?;
    //     writeln!(out, "/* HERE1 */ pub {}: {},", to_snake_case(&name), rust_type)?;
    //     Ok(())
    // }

    // pub fn generate_structs(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
    //     let field = context.fields().get(&self.parsed.name).unwrap();

    //     if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
    //         writeln!(out, "// {} is {}", &self.parsed.name, defined_type)?;
            
    //         defined_type.generate(out, schema, executable_document, Some(&self.selections), &self.parsed.alias)?;
    //     }
    //     else {
    //         writeln!(out, "// Nothing to generate because {} is {}", &self.parsed.name, field.ty)?;
    //     }
    //     // if !self.selections.is_empty() {
    //     //     let name = to_pascal_case(&self.name);
    //     //     self.

            

    //     //     for selection in &self.selections {
    //     //         // let context: IndexMap<String, Field> = schema.query.get(schema).fields;

    //     //         selection.generate_structs(out, &schema.query.get(schema).fields, schema)?;
    //     //         // writeln!(out, "    #[serde(rename = \"{}\")]", &selection.name)?;
    //     //         // writeln!(out, "    {},", to_constant_case(&selection.name))?;
    //     //     }

    //     // }
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct ExecutableDocument {
    pub parsed: Rc<parsed_model::ExecutableDocument>,
    pub fragments: IndexMap<Atom, FragmentDefinition>,
    pub queries: Vec<GenericOperation>,
    pub mutations: Vec<GenericOperation>,
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::ExecutableDocument>, schema: &Rc<Schema>) ->  Result<Self, Error> {

        let mut fragments = IndexMap::new();

        for fragment_definition in parsed.fragments.values() {
            fragments.insert(fragment_definition.name.clone(), FragmentDefinition::new(err, fragment_definition, schema, &parsed)?);
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
            parsed: parsed.clone(),
            fragments,
            queries,
            mutations,
        })
    }

    pub fn get_fragment(&self, name: &Atom) -> Result<&FragmentDefinition, Error> {
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

    pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>) -> Result<(), Error> {
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
// #[derive(Debug, Clone, Copy)]
// pub enum Maybe {
//     True,
//     False,
//     Maybe,
// }

// impl Display for Maybe {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Maybe::True => writeln!(f, "True"),
//             Maybe::False => writeln!(f, "False"),
//             Maybe::Maybe => writeln!(f, "Maybe"),
//         }
//     }
// }

// impl Maybe {
//     pub fn isit(&self, default: bool) -> bool {
//         match self {
//             Maybe::True => true,
//             Maybe::False => false,
//             Maybe::Maybe => default,
//         }
//     }
// }

// #[derive(Debug)]
// pub struct VariableDefinition {
//     parsed: Rc<parsed_model::VariableDefinition>,
//     // queries: Vec<Query>,
// }

// impl VariableDefinition {
//     pub fn new(parsed: parsed_model::VariableDefinition, out: &mut Output) -> Result<VariableDefinition, Error> {


//         Ok(VariableDefinition {
//             parsed: parsed.clone(),
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

