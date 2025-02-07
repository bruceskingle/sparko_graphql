use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

use graphql_parser::Pos;
use indexmap::IndexMap;
use inflections::case::to_snake_case;

use crate::parsed_model::{BuiltinType, DefinedTypeName, OperationType};
use crate::utils::to_pascal_case;
use crate::{Atom, BagOfNamed, BuildError, Error, ErrorCollector, Isomorphic, Print, Registry, TYPE_NAME};
use crate::{parsed_model, Output};

pub enum Optionality {
    Optional,
    Default,
    Required,
}

impl Optionality {
    pub fn required_by_default(&self) -> Optionality {
        match self {
            Optionality::Optional => Optionality::Optional,
            Optionality::Default => Optionality::Required,
            Optionality::Required => Optionality::Required,
        }
    }


    pub fn optional_by_default(&self) -> Optionality {
        match self {
            Optionality::Optional => Optionality::Optional,
            Optionality::Default => Optionality::Optional,
            Optionality::Required => Optionality::Required,
        }
    }
}

// const TYPE_NAME: &str = "__typename";
// pub type FieldMap = IndexMap<Atom, Rc<Field>>;


// pub enum View {
//     Type(Type),
//     Selection(Selection),
//     Invalid(BuildError),
// }

// impl View {
//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         match self {
//             View::Type(type_) => writeln!(out, "{}", type_),
//             View::Selection(selection) => selection.print(&mut out.indent()),
//             View::Invalid(build_error) => writeln!(out, "INVALID {}", build_error),
//         }
//     }
// }

// pub struct ViewField {
//     pub name: Atom,
//     pub view: View,
// }

// impl ViewField {
//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         write!(out, "{}: ", self.name)?;
//         self.view.print(out)
//     }

//     pub fn generate_field(&self, out: &mut Output) -> std::io::Result<()> {
//         match &self.view {
//             View::Type(type_) => write!(out, "{}: {},", self.name, type_),
//             View::Selection(selection) =>  write!(out, "{}: SELECTION,", self.name),
//             View::Invalid(build_error) => write!(out, "// {}: {},", self.name, build_error),
//         }
        
//     }
    
//     fn from_field(field: &Field) -> Rc<Self> {
//         Rc::new(Self {
//             name: field.parsed.name.clone(),
//             view: View::Type(field.ty.clone())
//         })
//     }
    
//     fn from_error(name: &Atom, error: BuildError) -> Rc<Self> {
//         Rc::new(Self {
//             name: name.clone(),
//             view: View::Invalid(error)
//         })
//     }
    
//     fn from_selection(field: &Field, selection: Selection) -> Rc<Self> {
//         Rc::new(Self {
//             name: field.parsed.name.clone(),
//             view: View::Selection(selection)
//         })
//     }
    
//     // fn from_selection(selection: Selection) -> Rc<Self> {
//     //     Rc::new(Self {
//     //         name: field.parsed.name.clone(),
//     //         view: View::Selection(selection)
//     //     })
//     // }
// }

pub type FieldMap = SharedMap<Field>;

pub type RawSharedMap<T> = IndexMap<Atom, Rc<T>>;

#[derive(Debug, Clone)]
pub struct SharedMap<T> {
    pub map: Rc<RawSharedMap<T>>,
}

impl<T> SharedMap<T> {
    pub fn builder() -> SharedMapBuilder<T> {
        SharedMapBuilder {
            map: IndexMap::new(),
        }
    }
    
    pub fn values(&self) -> indexmap::map::Values<'_, Atom, Rc<T>> {
        self.map.values()
    }
    
    pub fn get(&self, name: &String) -> Option<&Rc<T>> {
        self.map.get(name)
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    pub fn keys(&self) -> indexmap::map::Keys<'_, Atom, Rc<T>> {
        self.map.keys()
    }
}

impl<'h, T> IntoIterator for &'h SharedMap<T> {
    type Item = <&'h RawSharedMap<T> as IntoIterator>::Item;
    type IntoIter = <&'h RawSharedMap<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        (&self.map).iter()
    }
}

pub struct SharedMapBuilder<T> {
    pub map: RawSharedMap<T>,
}

impl<T> SharedMapBuilder<T> {
    pub fn new() -> SharedMapBuilder<T> {
        SharedMapBuilder {
            map: IndexMap::new(),
        }
    }

    pub fn insert(&mut self, key: Atom, value: Rc<T>) -> Option<Rc<T>> {
        self.map.insert(key, value)
    }

    pub fn build(self) -> SharedMap<T> {
        SharedMap {
            map: Rc::new(self.map),
        }
    }
}

// type ViewMap = SharedMap<ViewField>;
// pub type RawSelection = IndexMap<Atom, ViewMap>;
// type VariantMap = SharedMap<FieldMap>;


#[derive(Debug)]
pub struct Selection {
    pub index: usize,
    pub graphql_type_name: Atom,
    pub parsed: Rc<parsed_model::SelectionList>,
    pub names: Vec<Atom>,
    pub fields: IndexMap<Atom, SelectionField>,
    pub variants: Vec<Rc<Variant>>,
    pub interface: Option<Rc<Interface>>,
}

impl Print for Selection {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "fields {{")?;
        {
            let mut out = out.indent();

            for field in self.fields.values() {
                field.print(&mut out)?;
                // writeln!(out, "{}: {}", name, field)?;
            }
        }
        writeln!(out, "}}")?;

        writeln!(out, "variants {{")?;
        {
            let mut out = out.indent();

            for variant in &self.variants {
                writeln!(out, "{:?} {{", variant.names)?;
                {
                    let mut out = out.indent();

                    for (_name, field) in &variant.fields {
                        field.print(&mut out)?;
                        // writeln!(out, "{}: {}", name, field)?;
                    }
                }
                writeln!(out, "}}")?;
            }
        }
        writeln!(out, "}}")
    }
}

impl Selection {
    
    // fn values(&self) -> indexmap::map::Values<'_, Atom, ViewMap> {
    //     self.variants.values()
    // }
    
    // fn get(&self, name: &String) -> Option<&ViewMap> {
    //     self.variants.get(name)
    // }
    
    // fn is_empty(&self) -> bool {
    //     self.variants.is_empty()
    // }
    
    // fn keys(&self) -> indexmap::map::Keys<'_, Atom, ViewMap> {
    //     self.variants.keys()
    // }
    
    pub fn generate_query(&self, out: &mut Output<'_>, variables: &FieldMap, fragments: &mut HashSet<Atom>) -> Result<(), Error> {
        self.parsed.generate_query(out, variables, fragments)
    }
    
    fn generate_fields(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        for field in self.fields.values() {
            field.generate_field(out, selection_manager, schema)?;
        }

        Ok(())
    }
    
    // fn generate_variant_fields(&self, out: &mut Output<'_>, variant_name: &Atom, selection_manager: &SelectionManager) -> Result<(), Error> {
    //     let fields = self.variants.get(variant_name).unwrap();

    //     for (name, field) in fields {
    //         field.generate_field(out, selection_manager)?;
    //     }

    //     Ok(())
    // }


    
    // fn generate_structs(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, field_map: &IndexMap<Atom, Rc<OldSelectionField>>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
    //     for (name, selection_field) in field_map {
    //         if name.as_ref() == TYPE_NAME {}
    //         else {
    //             if let Some(field) = context.fields().get(&selection_field.parsed.name) {
    //                 if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
    //                     println!("defined_type.name()e={}", defined_type.name());
    //                     println!("&field.parsed.name={}", &field.parsed.name);
    //                     println!("dependencies={:?}", dependencies);

    //                     if ! dependencies.contains(defined_type.name()) {
    //                         defined_type.generate_selection(out, schema, executable_document, &selection_field.selections, &None, dependencies)?;
                            
    //                         //(out, schema, executable_document, Some(&selection_field.selections), &alias, type_set)?;
    //                     }
                        
    //                     // println!("defined_type.name()={}", defined_type.name());
    //                     // if defined_type.name().as_ref() == "PageInfo" {
    //                     //     println!("STOP {:?}", type_set);
    //                     // }
    //                     // if type_set.insert(defined_type.name().clone()) {
    //                     //     let alias = if name == &selection_field.parsed.name {
    //                     //         None
    //                     //     }
    //                     //     else {
    //                     //         Some(name.clone())
    //                     //     };

                            
    //                     // }
    //                 }
    //             }
    //             else {
    //                 writeln!(out, "UNKNOWN FIELD 2 {}", &selection_field.parsed.name)?;
    //             }
    //         }
    //     }
    //     Ok(())
    // }
    
    fn generate(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        let name = selection_manager.get_name(&self.index);

        // let rust_name = to_pascal_case(name);

        if let Some(interface) = &self.interface {
            writeln!(out, "// interface {} {:?}", interface.parsed.name, interface.implemented_by)?;
        }
        
        match &self.interface {
            None => {

                writeln!(out, "/* No variants */")?;
                
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                if *name != self.graphql_type_name {
                    writeln!(out, "#[serde(rename = \"{}\")]", self.graphql_type_name)?;
                }
                writeln!(out, "pub struct {} {{", name)?;
        
                {
                    let mut out = out.indent();
                    
                    self.generate_fields(&mut out, selection_manager, schema)?;
                }
        
                writeln!(out, "}}")?;
                writeln!(out, "")?;
        
                // selections.generate_structs(out, context, schema, executable_document, &field_map, dependencies)?;
            },
            Some(interface) => {
                if self.variants.len() < 2 && interface.implemented_by.len() < 2 {
                    writeln!(out, "/* <2 variant */")?;
                    
                    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                    if *name != self.graphql_type_name {
                        writeln!(out, "#[serde(rename = \"{}\")]", self.graphql_type_name)?;
                    }
                    writeln!(out, "pub struct {} {{", name)?;
            
                    {
                        let mut out = out.indent();
                        
                        self.generate_fields(&mut out, selection_manager, schema)?;
                        for variant_selection in &self.variants {
                            for (name, field) in &variant_selection.fields {
                                field.generate_field(&mut out, selection_manager, schema)?;
                            }
                        }
        
                    }
            
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;
                }
                else 
                {
                    writeln!(out, "/* {} variants {} implementors */", self.variants.len(), interface.implemented_by.len())?;
        
                    let abstract_name = Rc::new(format!("Abstract{}", name));
                    let base_type_name = to_pascal_case(name);
                    let base_member_name = to_snake_case(name);
                    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                    writeln!(out, "#[serde(tag = \"{}\")]", TYPE_NAME)?;
                    writeln!(out, "pub enum {} {{", base_type_name)?;

                    
                       

                    let mut variant_map = HashMap::new();
                    writeln!(out, "/* variants")?;
                    for variant in &self.variants {

                        writeln!(out, "{} {:?}", &variant.type_condition, &variant)?;
                        variant_map.insert(&variant.type_condition, variant);
                    }
                    writeln!(out, "*/")?;

                    for implementor in &interface.implemented_by {
                        let enum_variant_name = to_pascal_case(&implementor);

                        if let Some(variant) = variant_map.get(implementor) {
                            let name = selection_manager.get_name(&variant.index);
                            writeln!(out, "    {}({}),", enum_variant_name, name)?;
                        }
                        else {
                            writeln!(out, "    {}({}),", enum_variant_name, abstract_name)?;
                        }
                    }

                    // if ! self.fields.is_empty() {
                    //     writeln!(out, "    {}({}),", abstract_name, abstract_name)?;
                    // }
                    // for variant in &self.variants {
                    //     let name = selection_manager.get_name(&variant.index);
                    //     writeln!(out, "    {}({}),", name, name)?;
                    // }
            
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;
        
                    if ! self.fields.is_empty() {
                        writeln!(out, "impl {} {{", base_type_name)?;
                
                        {
                            let mut out = out.indent();
                            
                            writeln!(out, "pub fn as_{}(&self) -> &{} {{", base_member_name, abstract_name)?;
                            {
                                let mut out = out.indent();
                                writeln!(out, "match self {{")?;
                                {
                                    let mut out = out.indent();
        
                                    // writeln!(out, "{}::{}(content) => content.as_{}(),", base_type_name, abstract_name, base_member_name)?;


                                    // for variant in &self.variants {
                                    //     let name = selection_manager.get_name(&variant.index);
                                    //     writeln!(out, "{}::{}(content) => content.as_{}(),", base_type_name, name, base_member_name)?;
                                    // }


                                    for implementor in &interface.implemented_by {
                                        let enum_variant_name = to_pascal_case(&implementor);
                
                                        if let Some(variant) = variant_map.get(implementor) {
                                            writeln!(out, "{}::{}(content) => content.as_{}(),", base_type_name, enum_variant_name, base_member_name)?;
                                        }
                                        else {
                                            writeln!(out, "{}::{}(content) => content,", base_type_name, enum_variant_name)?;
                                        }
                                    }
                                }
                                writeln!(out, "}}")?;
                            }
                            writeln!(out, "}}")?;
                            writeln!(out, "")?;
                        }
                
                        writeln!(out, "}}")?;
                        writeln!(out, "")?;
                    }
        
                    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                    // writeln!(out, "#[serde(rename = \"{}\")]", name)?;
                    writeln!(out, "pub struct {} {{", abstract_name)?;
            
                    {
                        let mut out = out.indent();
                        
                        self.generate_fields(&mut out, selection_manager, schema)?;
                    }
            
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;
    
                    writeln!(out, "impl {} {{", abstract_name)?;
            
                    {
                        let mut out = out.indent();
                        
                        writeln!(out, "pub fn as_{}(&self) -> &{} {{", base_member_name, abstract_name)?;
                        writeln!(out, "    self")?;
                        writeln!(out, "}}")?;
                    }
            
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;

                    
        
                    for variant in &self.variants {
                        let variant_name = selection_manager.get_name(&variant.index);
                        let rust_variant_name = to_pascal_case(&variant_name);
        
                        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                        writeln!(out, "#[serde(rename = \"{}\")]", variant_name)?;
                        writeln!(out, "pub struct {} {{", rust_variant_name)?;
                
                        {
                            let mut out = out.indent();
        
                            if ! self.fields.is_empty() {
                                writeln!(out, "#[serde(flatten)]")?;
                                // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
                                writeln!(out, "pub {}_: {},", base_member_name, abstract_name)?;
                            }
                            
                            // self.generate_variant_fields(&mut out, variant_name, selection_manager)?;
                            for (_name, field) in &variant.fields {
                                field.generate_field(&mut out, selection_manager, schema)?;
                            }
                        }
                
                        writeln!(out, "}}")?;
                        writeln!(out, "")?;
        
                        writeln!(out, "impl {} {{", rust_variant_name)?;
                
                        {
                            let mut out = out.indent();
                            
                            writeln!(out, "pub fn as_{}(&self) -> &{} {{", base_member_name, abstract_name)?;
                            writeln!(out, "    &self.{}_", base_member_name)?;
                            writeln!(out, "}}")?;
                        }
                
                        writeln!(out, "}}")?;
                        writeln!(out, "")?;
                        
                    }
                }
            },
        }
        Ok(())
    }
    
    // fn generate_query(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
    //     let mut is_spread = false;
    //     for (variant_name, variant) in self {
    //         if is_spread {
    //             let out = out.indent();

    //             for (name, field) in variant {
    //                 field.generate_query()
    //             }
    //         }
    //         else {
    //             is_spread = true;

    //             for (name, field) in variant {
    //                 field.generate_query()
    //             }
    //         }
    //     }
        
    //     if ! &self.selections.is_empty() {
    //         writeln!(out, "{{")?;
    //         {
    //             let mut out = out.indent();

            
    //             for selection in self.selections.values() {
    //                 selection.generate_query(&mut out, context, schema, executable_document, true)?;
    //             }
    //         }
    //         writeln!(out, "}}")?;
    //     }
    //     Ok(())
    // }
    //     pub fn generate_query(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
//         if let Some(alias) = &self.parsed.alias {
//             writeln!(out, "{}: {}", alias, &self.parsed.name)?;
//         }
//         else {
//             writeln!(out, "{}", &self.parsed.name)?;
//         }

//         if ! self.arguments.is_empty() {
//             writeln!(out, "(")?;
//             {
//                 let mut out = out.indent();

            
//                 for argument in &self.arguments {
//                     argument.generate_query(&mut out)?;
//                 }
//             }
//             writeln!(out, ")")?;
//         }

//         self.selections.generate_query(out, context, schema, executable_document, true)?;
//         Ok(())
//     }

}

// impl<'h> IntoIterator for &'h Selection {
//     type Item = <&'h RawSelection as IntoIterator>::Item;
//     type IntoIter = <&'h RawSelection as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter {
//         (&self.variants).iter()
//     }
// }

#[derive(Debug)]
pub struct Variant {
    pub index: usize,
    pub names: Vec<Atom>,
    pub fields: IndexMap<Atom, SelectionField>,
    pub type_condition: Atom,
}

// impl Variant {
//     fn generate(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema, selection_index: usize,) -> Result<(), Error> {
//         let base_name = selection_manager.get_name(&selection_index);
//         let abstract_name = Rc::new(format!("Abstract{}", base_name));
//             let base_type_name = to_pascal_case(base_name);
//             let base_member_name = to_snake_case(base_name);




//         let variant_name = selection_manager.get_name(&self.index);
//         let rust_variant_name = to_pascal_case(&variant_name);

//         writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
//         writeln!(out, "#[serde(rename = \"{}\")]", variant_name)?;
//         writeln!(out, "pub struct {} {{", rust_variant_name)?;

//         {
//             let mut out = out.indent();

//             if ! self.fields.is_empty() {
//                 writeln!(out, "#[serde(flatten)]")?;
//                 // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
//                 writeln!(out, "pub {}_: {},", base_member_name, abstract_name)?;
//             }
            
//             // self.generate_variant_fields(&mut out, variant_name, selection_manager)?;
//             for (_name, field) in &self.fields {
//                 field.generate_field(&mut out, selection_manager, schema)?;
//             }
//         }

//         writeln!(out, "}}")?;
//         writeln!(out, "")?;

//         writeln!(out, "impl {} {{", rust_variant_name)?;

//         {
//             let mut out = out.indent();
            
//             writeln!(out, "pub fn as_{}(&self) -> &{} {{", base_member_name, abstract_name)?;
//             writeln!(out, "    &self.{}_", base_member_name)?;
//             writeln!(out, "}}")?;
//         }

//         writeln!(out, "}}")?;
//         writeln!(out, "")?;
//         Ok(())
//     }
// }

pub struct SelectionBuilder {
    pub graphql_type_name: Atom,
    pub parsed: Rc<parsed_model::SelectionList>,
    pub preferred_type_names: Vec<Atom>,
    pub fields: IndexMap<Atom, SelectionField>,
    pub variants: IndexMap<Atom, (Atom, IndexMap<Atom, SelectionField>)>,
    interface: Option<Rc<Interface>>,
}

impl SelectionBuilder {
    pub fn new(graphql_type_name: Atom, selections: &Rc<parsed_model::SelectionList>, interface: Option<Rc<Interface>>) -> SelectionBuilder {
        SelectionBuilder {
            graphql_type_name,
            parsed: selections.clone(),
            preferred_type_names: Vec::new(),
            fields: IndexMap::new(),
            variants: IndexMap::new(),
            interface,
        }
    }

    // pub fn insert_field(&mut self, field_name: &Atom, field: &Rc<Field>) {
    //     self.fields.insert(field_name.clone(), field.clone());

    // }

    // pub fn insert_variant(&mut self, variant_name: &Atom, field_name: &Atom, field: &Rc<Field>) {
    //     self.variants.entry(variant_name.clone()).or_insert(SharedMapBuilder::new()).insert(field_name.clone(), field.clone());

    // }

    pub fn with_field(&mut self, variant_name: &Option<(Atom, Atom)>, field_name: &Atom, field: SelectionField) -> &mut Self {
        if let Some((variant_name, variant_type_condition)) = variant_name {
            if let Some((tc, variant)) = self.variants.get_mut(variant_name) {
                variant.insert(field_name.clone(), field);
            }
            else {
                let mut fields = IndexMap::new();
                fields.insert(field_name.clone(), field);
                self.variants.insert(variant_name.clone(), (variant_type_condition.clone(), fields));
            }
            // self.variants.entry(variant_name.clone()).or_insert(IndexMap::new()).insert(field_name.clone(), field);
        }
        else {
            self.fields.insert(field_name.clone(), field); 
        }
        self
    }

    pub fn with_name(&mut self, name: &Atom) -> &mut Self {
        self.preferred_type_names.push(name.clone());
        self
    }

    pub fn with_preferred_type_names(&mut self, preferred_type_names: Vec<Atom>) -> &mut Self {
        for name in preferred_type_names {
            self.preferred_type_names.push(name);
        }
        self
    }

    pub fn build(self, manager: &mut NameSpaceManager) -> Rc<Selection> {
        
        manager.insert(self.graphql_type_name, self.parsed, self.preferred_type_names, self.fields, self.variants, self.interface)
        // let index = manager.selections.len();

        // manager.selections.push( Rc::new(Selection {
        //     index,
        //     parsed: self.parsed,
        //     names: self.names,
        //     fields: self.fields,
        //     variants: self.variants,
        // }));

        // let check = manager.selections.get(index);

        // if let Some(check) = check {
        //     if check.index == index {
        //         return check.clone()
        //     }
        //     else {
        //         panic!("SelectionManager index error expected {} got {}", index, check.index);
        //     }
        // }
        // else  {
        //     panic!("SelectionManager index error expected {} got None", index);
        // }
        
    }
    
    // fn get_variant(&mut self, variant_name: &Atom) -> &SharedMapBuilder<Field> {
    //     self.variants.entry(variant_name.clone()).or_insert(SharedMapBuilder::new())
    // }
}

// pub trait SelectionContext {
//     fn get_executable_document<'a>() -> &'a ExecutableDocument;
//     fn get_selections<'a>() ->&'a SelectionList;
// }


#[derive(Debug)]
pub enum SelectionFieldType {
    BuiltinType(BuiltinType),
    Scalar(Atom),
    Enum(Atom),
    Selection(usize),
    Required(Box<SelectionFieldType>),
    Array(Box<SelectionFieldType>),
}

impl Display for SelectionFieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionFieldType::BuiltinType(content) => content.fmt(f),
            SelectionFieldType::Scalar(content) => content.fmt(f),
            SelectionFieldType::Enum(content) => content.fmt(f),
            SelectionFieldType::Selection(content) => content.fmt(f),
            SelectionFieldType::Required(wrapped) => write!(f, "{}!", wrapped),
            SelectionFieldType::Array(wrapped) => write!(f, "[{}]", wrapped),
        }
    }
}

impl SelectionFieldType {
    // pub fn rust_type(&self, selection_manager: &SelectionManager) -> String {
    //     match self {
    //         SelectionFieldType::BuiltinType(builtin_type) => builtin_type.rust_type().to_string(),
    //         SelectionFieldType::Scalar(name) => to_pascal_case(name),
    //         SelectionFieldType::Enum(name) => to_pascal_case(name),
    //         SelectionFieldType::Selection(id) => selection_manager.get_name(id).to_string(),
    //         SelectionFieldType::Required(selection_field_type) => todo!(),
    //         SelectionFieldType::Array(selection_field_type) => todo!(),
    //     }
    // }

    // if nonnull then the type is coerced to be nonnull at all levels
    pub fn rust_type(&self, selection_manager: &NameSpaceManager, schema: &Schema, nonnull: bool) -> String {
        // Will never be called with Required variant.
        fn do_rust_type(input: &SelectionFieldType, selection_manager: &NameSpaceManager, schema: &Schema, nonnull: bool) -> String {
            match input {
                SelectionFieldType::BuiltinType(builtin_type) => builtin_type.rust_type().to_string(),
                SelectionFieldType::Scalar(name) => schema.defined_types.get(name).unwrap().rust_name().to_string(),
                SelectionFieldType::Enum(name) => schema.defined_types.get(name).unwrap().rust_name().to_string(),
                SelectionFieldType::Selection(id) => selection_manager.get_name(id).to_string(),
                SelectionFieldType::Required(_) => unreachable!(),
                SelectionFieldType::Array(wrapped) => format!("Vec<{}>", wrapped.rust_type(selection_manager, schema, nonnull)),
            }
        }

        if let SelectionFieldType::Required(wrapped) = self {
            do_rust_type(wrapped, selection_manager, schema, nonnull)
        }
        else {
            if nonnull {
                do_rust_type(self, selection_manager, schema, nonnull)
            }
            else {
                format!("Option<{}>", do_rust_type(self, selection_manager, schema, nonnull))
            }
        }
    }
}


#[derive(Debug)]
pub struct SelectionField {
    pub name: Atom,
    pub selection_type: SelectionFieldType,
    pub optional: bool,
}
impl SelectionField {
    fn generate_field(&self, out: &mut Output, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), std::io::Error> {
        if *self.name != TYPE_NAME {
            let field_name = to_snake_case(&self.name);

            writeln!(out, "#[serde(rename = \"{}\")]", &self.name)?;
            writeln!(out, "pub {}_: {}, // T1", field_name, self.selection_type.rust_type(selection_manager, schema, !self.optional))
        }
        else {
            Ok(())
        }
    }
}

impl Print for SelectionField {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{}: {}", self.name, self.selection_type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinedType {
    Scalar(Atom),
    Object(Atom),
    Interface(Atom),
    Union(Atom),
    Enum(Atom),
    InputObject(Atom),
}

impl Isomorphic for DefinedType {
    fn is_isomorphic(&self, other: &Self) -> bool {
        match self {
            DefinedType::Scalar(name) => {
                if let DefinedType::Scalar(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
            DefinedType::Object(name) => {
                if let DefinedType::Object(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
            DefinedType::Interface(name) => {
                if let DefinedType::Interface(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
            DefinedType::Union(name) => {
                if let DefinedType::Union(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
            DefinedType::Enum(name) => {
                if let DefinedType::Enum(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
            DefinedType::InputObject(name) => {
                if let DefinedType::InputObject(other_name) = other {
                    name == other_name
                }
                else {
                    false
                }
            },
        }
    }
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
    // pub fn defined_type_name(&self) -> DefinedTypeName {
    //     match self {
    //         DefinedType::Scalar(_) => DefinedTypeName::Scalar,
    //         DefinedType::Object(_) => DefinedTypeName::Object,
    //         DefinedType::Interface(_) => DefinedTypeName::Interface,
    //         DefinedType::Union(_) => DefinedTypeName::Union,
    //         DefinedType::Enum(_) => DefinedTypeName::Enum,
    //         DefinedType::InputObject(_) => DefinedTypeName::InputObject,
    //     }
    // }

    // pub fn name(&self) -> &str {
    //     self.defined_type_name().name()
    // }

    pub fn generate(&self, out: &mut Output<'_>, schema: &Rc<Schema>, alias: &Option<Atom>) -> Result<(), Error> {
        match self {
            DefinedType::Scalar(name) => schema.get_scalar(name)?.generate(out),
            DefinedType::Object(name) => schema.get_object(name)?.generate(out),
            DefinedType::Interface(name) => schema.get_interface(name)?.generate(out),
            DefinedType::Union(name) => schema.get_union(name)?.generate(out),
            DefinedType::Enum(name) => schema.get_enum(name)?.generate(out),
            DefinedType::InputObject(name) => schema.get_input_object(name)?.generate(out),
        }
    }

    // pub fn generate_selection(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, alias: &Option<Atom>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
    //     match self {
    //         DefinedType::Scalar(name) => Err(Error::FatalBuildError("Can't generate selections on Scalar")),
    //         DefinedType::Object(name) => schema.get_object(name)?.generate_selection(out, schema, executable_document, selections, alias, dependencies),
    //         DefinedType::Interface(name) => schema.get_interface(name)?.generate_selection(out, schema, executable_document, selections, alias, dependencies),
    //         DefinedType::Union(name) => schema.get_union(name)?.generate_selection(out, schema, executable_document, selections, alias, dependencies),
    //         DefinedType::Enum(name) => Err(Error::FatalBuildError("Can't generate selections on Enum")),
    //         DefinedType::InputObject(name) => schema.get_input_object(name)?.generate_selection(out, schema, executable_document, selections, alias, dependencies),
    //     }
    // }
    
    // fn gather_selection_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     match self {
    //         DefinedType::Scalar(name) => {
    //             dependencies.insert(name.clone());
    //             Ok(())
    //         },
    //         DefinedType::Object(name) => schema.get_object(name)?.gather_selection_dependencies(schema, executable_document, selections, dependencies),
    //         DefinedType::Interface(name) => schema.get_interface(name)?.gather_selection_dependencies(schema, executable_document, selections, dependencies),
    //         DefinedType::Union(name) => schema.get_union(name)?.gather_selection_dependencies(schema, executable_document, selections, dependencies),
    //         DefinedType::Enum(name) => {
    //             dependencies.insert(name.clone());
    //             Ok(())
    //         },
    //         DefinedType::InputObject(name) => schema.get_input_object(name)?.gather_selection_dependencies(schema, executable_document, selections, dependencies),
    //     }
    // }
    
    // fn gather_dependencies(&self, schema: &Rc<Schema>, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     match self {
    //         DefinedType::Scalar(_) => Ok(()),
    //         DefinedType::Object(name) => schema.get_object(name)?.gather_dependencies(schema, dependencies),
    //         DefinedType::Interface(name) => schema.get_interface(name)?.gather_dependencies(schema, dependencies),
    //         DefinedType::Union(name) => schema.get_union(name)?.gather_dependencies(schema, dependencies),
    //         DefinedType::Enum(_) => Ok(()),
    //         DefinedType::InputObject(name) => schema.get_input_object(name)?.gather_dependencies(schema, dependencies),
    //     }
    // }
    
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
    
    fn graphql_type(&self) -> &str {
        match self {
            DefinedType::Scalar(name) => name,
            DefinedType::Object(name) => name,
            DefinedType::Interface(name) => name,
            DefinedType::Union(name) => name,
            DefinedType::Enum(name) => name,
            DefinedType::InputObject(name) => name,
        }
    }
    
    fn name(&self) -> &Rc<String> {
        match self {
            DefinedType::Scalar(name) => name,
            DefinedType::Object(name) => name,
            DefinedType::Interface(name) => name,
            DefinedType::Union(name) => name,
            DefinedType::Enum(name) => name,
            DefinedType::InputObject(name) => name,
        }
    }
    
    pub fn get_fields<'a>(&self, schema: &'a Schema) -> Result<Option<&'a FieldMap>, Error> {
        Ok(match self {
            DefinedType::Scalar(_) => None,
            DefinedType::Object(name) => Some(&schema.get_object(name)?.fields),
            DefinedType::Interface(name) => Some(&schema.get_interface(name)?.fields),
            DefinedType::Union(name) => Some(&schema.get_union(name)?.fields),
            DefinedType::Enum(_) => None,
            DefinedType::InputObject(name) => Some(&schema.get_input_object(name)?.fields),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarType {
    DefinedType(DefinedType),
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
                        TypeDefinition::Object(content) =>ScalarType::DefinedType(DefinedType::Object(content.name.clone())),
                        TypeDefinition::Interface(content) =>ScalarType::DefinedType(DefinedType::Interface(content.parsed.name.clone())),
                        TypeDefinition::Union(content) =>ScalarType::DefinedType(DefinedType::Union(content.parsed.name.clone())),
                        TypeDefinition::Enum(content) => ScalarType::DefinedType(DefinedType::Enum(content.parsed.name.clone())),
                        TypeDefinition::InputObject(content) =>ScalarType::DefinedType(DefinedType::InputObject(content.name.clone())),
                    })
                }
                else {
                    err.fail(BuildError::UndefinedTypeError(defined_type.position.clone(), format!("Missing type {}", defined_type.name.clone())))
                }
            },
            parsed_model::ScalarType::BuiltinType(builtin_type) => Ok(ScalarType::BuiltinType(builtin_type.clone())),
        }
    }
    
    pub fn get_fields<'a>(&self, schema: &'a Schema) -> Result<Option<&'a FieldMap>, Error> {
        match self {
            ScalarType::DefinedType(defined_type) => defined_type.get_fields(schema),
            ScalarType::BuiltinType(_) => Ok(None),
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
                    ScalarType::DefinedType(defined_type) => defined_type.graphql_type().to_string(),
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

    pub fn get_fields<'a>(&self, schema: &'a Schema) -> Result<Option<&'a FieldMap>, Error> {
        self.get_scalar().get_fields(schema)
    }
    
    // if nonnull then the type is coerced to be nonnull at all levels
    pub fn rust_type(&self, optionality: Optionality) -> String {
        // Will never be called with Required variant.
        fn do_rust_type(input: &Type, optionality: Optionality) -> String {
            match input {
                Type::Scalar(wrapped) => {
                    match wrapped {
                        ScalarType::DefinedType(defined_type) => defined_type.rust_type(),
                        ScalarType::BuiltinType(builtin_type) => builtin_type.rust_type().to_string(),
                    }
                },
                Type::Required(_) => unreachable!(),
                Type::Array(wrapped) => format!("Vec<{}>", wrapped.rust_type(optionality)),
            }
        }

        let (new_optionality, new_type) = if let Type::Required(wrapped) = self {
            (optionality.required_by_default(), &**wrapped)
        }
        else {
            (optionality.optional_by_default(), self)
        };

        if let Optionality::Optional = new_optionality {
            format!("Option<{}>", do_rust_type(new_type, optionality))
        }
        else {
            do_rust_type(new_type, optionality)
        }


        // if let Type::Required(wrapped) = self {
        //     do_rust_type(wrapped, optionality.required_by_default())
        // }
        // else {
            
        //     if nonnull {
        //         do_rust_type(self, nonnull)
        //     }
        //     else {
        //         format!("Option<{}>", do_rust_type(self, nonnull))
        //     }
        // }
    }
    
    pub fn is_optional(&self) -> bool {
        if let Type::Required(_) = self {
            false
        }
        else {
            true
        }
    }
    
    // fn rust_type_is_optional(&self) -> bool {
    //     match self {
    //         Type::Scalar(scalar_type) => true,
    //         Type::Required(wrapped) => false,
    //         Type::Array(wrapped) => wrapped.rust_type_is_optional(),
    //     }
    // }
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


pub enum TypeDefinition {
    Scalar(Rc<Scalar>),
    Object(Rc<Object>),
    Interface(Rc<Interface>),
    Union(Rc<Union>),
    Enum(Rc<Enum>),
    InputObject(Rc<Object>),
    // Selection(Variants),
}

impl Print for TypeDefinition {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            TypeDefinition::Scalar(content) => content.print(out),
            TypeDefinition::Object(content) => content.print(out),
            TypeDefinition::Interface(content) => content.print(out),
            TypeDefinition::Union(content) => content.print(out),
            TypeDefinition::Enum(content) => content.print(out),
            TypeDefinition::InputObject(content) => content.print(out),
            // TypeDefinition::Selection(content) => content.print(out),
        }
    }
}

impl TypeDefinition {
    pub fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
        match self {
            TypeDefinition::Scalar(content) => content.generate(out),
            TypeDefinition::Object(content) => content.generate(out),
            TypeDefinition::Interface(content) => content.generate(out),
            TypeDefinition::Union(content) => content.generate(out),
            TypeDefinition::Enum(content) => content.generate(out),
            TypeDefinition::InputObject(content) => content.generate(out),
        }
    }
    // pub fn generate_selection(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &SelectionList, alias: &Option<Atom>) -> Result<(), Error> {
    //     match self {
    //         TypeDefinition::Scalar(_) => Err(Error::FatalBuildError("Can't generate selections on Scalar")),
    //         TypeDefinition::Object(content) => content.generate_selection(out, schema, executable_document, selections, alias),
    //         TypeDefinition::Interface(content) => content.generate_selection(out, schema, executable_document, selections, alias),
    //         TypeDefinition::Union(content) => content.generate_selection(out, schema, executable_document, selections, alias),
    //         TypeDefinition::Enum(_) => Err(Error::FatalBuildError("Can't generate selections on Enum")),
    //         TypeDefinition::InputObject(content) => content.generate_selection(out, schema, executable_document, selections, alias),
    //     }
    // }

    

    pub fn defined_type_name(&self) -> DefinedTypeName {
        match self {
            TypeDefinition::Scalar(_) => DefinedTypeName::Scalar,
            TypeDefinition::Object(_) => DefinedTypeName::Object,
            TypeDefinition::Interface(_) => DefinedTypeName::Interface,
            TypeDefinition::Union(_) => DefinedTypeName::Union,
            TypeDefinition::Enum(_) => DefinedTypeName::Enum,
            TypeDefinition::InputObject(_) => DefinedTypeName::InputObject,
            // TypeDefinition::Selection(_) => DefinedTypeName::Selection,
        }
    }

    pub fn type_name(&self) -> &str {
        self.defined_type_name().name()
    }

    pub fn rust_name(&self) -> &Atom {
        match self {
            TypeDefinition::Scalar(object) => &object.rust_name,
            TypeDefinition::Object(object) => &object.rust_name,
            TypeDefinition::Interface(object) => &object.rust_name,
            TypeDefinition::Union(object) => &object.rust_name,
            TypeDefinition::Enum(object) => &object.rust_name,
            TypeDefinition::InputObject(object) => &object.rust_name,
        }
    }
}

pub trait Context {
    fn fields(&self) -> &FieldMap;
}


pub struct Scalar {
    pub parsed: Rc<parsed_model::Scalar>,
    pub rust_name: Atom,
    pub rust_type: Atom,
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
    
    fn new(registry: &mut Registry, parsed: &Rc<parsed_model::Scalar>, types: &HashMap<String, String>) -> TypeDefinition {
        println!("&*parsed.name={}", &*parsed.name);
        let rust_name =  registry.intern(to_pascal_case(&parsed.name));
        let rust_type = if let Some(fqn) = types.get(&*parsed.name) {
            registry.intern_str(fqn)
        }
        else {
            registry.intern_str("serde_json::Value")
        };

        TypeDefinition::Scalar(Rc::new(Scalar {
            parsed: parsed.clone(),
            rust_name,
            rust_type,
        }))
    }
    
    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
        writeln!(out, "type {} = {}; // B1", &self.rust_name, &self.rust_type)?;
        writeln!(out, "")?;
        Ok(())
    }
}


pub struct Object {
    // pub parsed: Rc<parsed_model::Object>,
    pub name: Atom,
    pub rust_name: Atom,
    pub fully_implements: Vec<Atom>,
    pub fields: FieldMap,
    pub is_input: bool,
}

impl Context for Object {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
}

impl Object {
    fn new(err: &mut ErrorCollector, registry: &mut Registry, parsed: &Rc<parsed_model::Object>, schema: &Rc<parsed_model::Schema>,) -> Result<Rc<Self>, Error> {
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

        let mut builder = FieldMap::builder();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                builder.insert(name.clone(), field);
            }
        }

        err.ok(Rc::new(Object {
                // parsed: parsed.clone(),
                name: parsed.name.clone(),
                rust_name: registry.intern(to_pascal_case(&parsed.name)),
                fully_implements,
                fields: builder.build(),
                is_input: parsed.is_input,
            }))
    }

    fn from_fields(registry: &mut Registry, name: Atom, fields: FieldMap) -> Rc<Self> {
        Rc::new(Object {
            rust_name: registry.intern(to_pascal_case(&name)),
            name,
            fully_implements: Vec::new(),
            fields,
            is_input: false,
        })
    }
    

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Object {{")?;
        {
            let mut out = out.indent();

            // self.parsed.print(&mut out)?;

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
    
    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
        generate_struct(out, &self.name, &self.rust_name, self, self.is_input)
    }
    
    // fn generate_selection(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, alias: &Option<Atom>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
    //     generate_struct_selection(out, schema, executable_document, selections, alias, &self.parsed.name, self, self.is_input, dependencies)
    // }
    
    // fn gather_selection_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_selection_dependencies(schema, executable_document, selections, &self.parsed.name, self, dependencies)
    // }
    
    // fn gather_dependencies(&self, schema: &Rc<Schema>, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_dependencies(schema,self, dependencies)
    // }
}



fn generate_struct(out: &mut Output<'_>, name: &Atom, rust_name: &Atom, context: &dyn Context, is_input: bool) -> Result<(), Error> {
   

    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
    if name != rust_name {
        writeln!(out, "#[serde(rename = \"{}\")]", name)?;
    }
    writeln!(out, "/* BRUCE */ pub struct {} {{", rust_name)?;

    for field in context.fields().values() {
        writeln!(out, "    #[serde(rename = \"{}\")]", &field.parsed.name)?;
        writeln!(out, "    pub {}_: {},", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Default))?;
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
                writeln!(out, "        {}_: None,", to_snake_case(&field.parsed.name))?;
            }
            writeln!(out, "    }}")?;
            writeln!(out, "}}")?;
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        writeln!(out, "")?;
        writeln!(out, "pub struct {}Builder {{", rust_name)?;

        for field in context.fields().values() {
            writeln!(out, "    {}_: {},", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Default))?;
        }
    
        writeln!(out, "}}")?;
        writeln!(out, "")?;

        writeln!(out, "impl {}Builder {{", rust_name)?;
        {
            let mut out = out.indent();

            for field in context.fields().values() {
                writeln!(out, "pub fn with_{}(mut self, value: {}) -> Self {{", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Required))?;
                {
                    let mut out = out.indent();

                    writeln!(out, "self.{}_ = Some(value);", to_snake_case(&field.parsed.name))?;
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
                        writeln!(out, "if let None = self.{}_ {{", to_snake_case(&field.parsed.name))?;
                        writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", field.parsed.name)?;
                        writeln!(out, "}}")?;
                    }
                }

                writeln!(out, "Ok({} {{", rust_name)?;
                {
                    let mut out = out.indent();

                    for field in context.fields().values() {
                        if let Type::Required(_) = field.ty {
                            writeln!(out, "{}_: self.{}_.unwrap(),", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
                        }
                        else {
                            writeln!(out, "{}_: self.{}_,", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
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

    Ok(())
}

// fn generate_struct_selection(out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, alias: &Option<Atom>, name: &Atom, context: &dyn Context, is_input: bool, dependencies: &HashSet<Atom>) -> Result<(), Error> {
//     let name = if let Some(alias) = alias {
//         alias
//     }
//     else {
//         name
//     };
//     let rust_name = to_pascal_case(name);
    
//     let abstract_name = Rc::new(format!("Abstract{}", to_pascal_case(name)));
//     let variants = selections.get_variants(&abstract_name, schema, executable_document, context)?;
//     if variants.len() > 1 {
//         writeln!(out, "/* {} variants */", variants.len())?;
//         let base_type_name = to_pascal_case(name);
//         let base_member_name = to_snake_case(name);
//         writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
//         writeln!(out, "pub enum {} {{", base_type_name)?;
//         for (name, _variant) in &variants {
//             writeln!(out, "    {}({}),", name, name)?;
//         }

//         writeln!(out, "}}")?;
//         writeln!(out, "")?;

//         for (name, (context, selections)) in &variants {
//             let rust_name = to_pascal_case(&name);

//             writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
//             writeln!(out, "#[serde(rename = \"{}\")]", name)?;
//             writeln!(out, "pub struct {} {{", rust_name)?;

//             let mut field_map = IndexMap::new();
//             selections.gather_fields(&mut field_map, *context, schema, executable_document, true)?;

//             {
//                 let mut out = out.indent();

//                 if name.as_ref() != name.as_ref() {
//                     writeln!(out, "#[serde(flatten)]")?;
//                     // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
//                     writeln!(out, "pub {}_: {},", base_member_name, base_type_name)?;
//                 }
//                 selections.generate_fields(&mut out, *context, &field_map)?;
//             }
//             writeln!(out, "}}")?;
//             writeln!(out, "")?;

//             selections.generate_structs(out, *context, schema, executable_document, &field_map, dependencies)?;
//         }
//     }
//     else {
//         writeln!(out, "/* No variants */")?;
        
//         writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
//         writeln!(out, "#[serde(rename = \"{}\")]", name)?;
//         writeln!(out, "pub struct {} {{", rust_name)?;

//         let mut field_map = IndexMap::new();
//         selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

//         {
//             let mut out = out.indent();
            
//             selections.generate_fields(&mut out, context, &field_map)?;
//         }

//         writeln!(out, "}}")?;
//         writeln!(out, "")?;

//         selections.generate_structs(out, context, schema, executable_document, &field_map, dependencies)?;
//     }

//     Ok(())
// }


// fn gather_struct_selection_dependencies(schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, name: &Atom, context: &dyn Context, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
//     // TODO: Work out if all fields are selected and add if they are
//     let abstract_name = Rc::new(format!("Abstract{}", to_pascal_case(name)));
//     let variants = selections.get_variants(&abstract_name, schema, executable_document, context)?;
//     if variants.len() > 1 {
//         for (_, (context, selections)) in &variants {
//             let mut field_map = IndexMap::new();
//             selections.gather_fields(&mut field_map, *context, schema, executable_document, true)?;

//             selections.gather_dependencies(schema, executable_document, *context, &field_map, dependencies)?;
//             //(*context, &field_map, type_set)?;
//         }
//     }
//     else {
//         let mut field_map = IndexMap::new();
//         selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

//         selections.gather_dependencies(schema, executable_document, context, &field_map, dependencies)?;
//     }

//     Ok(())
// }

// fn gather_struct_dependencies(schema: &Rc<Schema>, context: &dyn Context, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {

//     for (name, field) in context.fields() {
//         if name.as_ref() == TYPE_NAME {}
//         else {
//             if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
//                 defined_type.gather_dependencies(schema, dependencies)?;
//             }
//         }
//     }

//     Ok(())
// }

#[derive(Debug)]
pub struct Interface {
    pub parsed: Rc<parsed_model::Interface>,
    pub rust_name: Atom,
    pub implemented_by: Vec<Atom>,
    pub fields: FieldMap,
}

impl Context for Interface {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
}

impl Interface {

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Interface {{")?;
        self.parsed.print(out)?;
        
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

    fn new(err: &mut ErrorCollector, registry: &mut Registry, parsed: &Rc<parsed_model::Interface>, schema: &Rc<parsed_model::Schema>,) -> Result<TypeDefinition, Error> {
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

        let mut builder = FieldMap::builder();

        for (name, field) in &parsed.fields {
            if let Ok(field) = Field::new(err, field, schema) {
                builder.insert(name.clone(), field);
            }
        }

        err.ok(TypeDefinition::Interface(Rc::new(Interface {
                parsed: parsed.clone(),
                rust_name: registry.intern(to_pascal_case(&parsed.name)),
                implemented_by,
                fields: builder.build(),
            })))
    }
    
    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
        generate_struct(out, &self.parsed.name, &self.rust_name, self, false)?;
        Ok(())
    }
    
    // fn generate_selection(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, alias: &Option<Atom>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
    //     generate_struct_selection(out, schema, executable_document, selections, alias, &self.parsed.name, self, false, dependencies)?;
    //     Ok(())
    // }
    
    // fn gather_selection_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_selection_dependencies(schema, executable_document, selections, &self.parsed.name, self, dependencies)?;
    //     Ok(())
    // }
    
    // fn gather_dependencies(&self, schema: &Rc<Schema>, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_dependencies(schema,self, dependencies)
    // }
}


pub struct Union {
    pub parsed: Rc<parsed_model::Union>,
    pub rust_name: Atom,
    pub fields: FieldMap,
}

impl Context for Union {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
}

impl Union {
    fn new(err: &mut ErrorCollector, registry: &mut Registry, parsed: &Rc<parsed_model::Union>, schema: &Rc<parsed_model::Schema>,) -> Result<TypeDefinition, Error> {
        let mut builder = FieldMap::builder();
        let mut err = err.child();

        for type_name in &parsed.types {
            if let Some(object) = schema.get_object(&mut err, &parsed.position, type_name) {
                for (name, field) in &object.fields {
                    if let Ok(field) = Field::new(&mut err, field, schema) {
                        builder.insert(name.clone(), field);
                    }
                }
            }
            else {
                err.error(BuildError::MissingInterfaceError(parsed.position.clone(), format!("Interface {} Not Found", parsed.name)))
            };
        }

        err.ok(TypeDefinition::Union(Rc::new(Union {
            parsed: parsed.clone(),
            rust_name: registry.intern(to_pascal_case(&parsed.name)),
            fields: builder.build(),
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

    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
        generate_struct(out, &self.parsed.name, &self.rust_name, self, false)
    }

    // fn generate_selection(&self, out: &mut Output<'_>, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, alias: &Option<Atom>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
    //     generate_struct_selection(out, schema, executable_document, selections, alias, &self.parsed.name, self, false, dependencies)
    // }
    
    // fn gather_selection_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selections: &OldSelectionList, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_selection_dependencies(schema, executable_document, selections, &self.parsed.name, self, dependencies)?;
    //     Ok(())
    // }
    
    // fn gather_dependencies(&self, schema: &Rc<Schema>, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
    //     gather_struct_dependencies(schema,self, dependencies)
    // }
}


pub struct Enum {
    pub parsed: Rc<parsed_model::Enum>,
    pub rust_name: Atom,
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

    pub fn new(registry: &mut Registry, parsed: &Rc<parsed_model::Enum>) -> TypeDefinition {
        TypeDefinition::Enum(Rc::new(Enum {
            parsed: parsed.clone(),
            rust_name: registry.intern(to_pascal_case(&parsed.name)),
        }))
    }
    
    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
    // fn generate(&self, out: &mut Output<'_>, _schema: &Rc<Schema>, _selections: &SelectionList) -> Result<(), Error> {
        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
        if self.parsed.name != self.rust_name {
            writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
        }
        writeln!(out, "pub enum {} {{", &self.rust_name)?;

        for variant in &self.parsed.variants {
            writeln!(out, "    #[serde(rename = \"{}\")]", &variant.name)?;
            writeln!(out, "    {},", to_pascal_case(&variant.name))?;
        }
        writeln!(out, "}}")?;
        writeln!(out, "")?;
        Ok(())
    }
}



#[derive(Debug, Clone)]
pub struct Field {
    pub parsed: Rc<parsed_model::Field>,
    pub ty: Type,
}

impl Isomorphic for Field {
    fn is_isomorphic(&self, other: &Self) -> bool {
        self.parsed.name == other.parsed.name && self.ty.is_isomorphic(&other.ty)
    }
}

impl Field{
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "Field {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
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


pub struct Schema {
    pub defined_types: IndexMap<Atom, TypeDefinition>,
    pub query: Atom,
    pub mutation: Option<Atom>,
    pub subscription: Option<Atom>,
    pub __typename: Rc<Field>,
}

impl Schema {
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Schema {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "query        {}", self.query)?;
            writeln!(out, "mutation     {:?}", self.mutation)?;
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

    fn new_get_default_object(out: &mut ErrorCollector, name: Atom, defined_types: &IndexMap<Atom, TypeDefinition>, missing_error: Option<BuildError>) -> Option<Atom> {
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

    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Schema>, registry: &mut Registry, types: &HashMap<String, String>) -> Result<Rc<Self>, Error> {

        let mut defined_types: IndexMap<Atom, TypeDefinition> = IndexMap::new();

        for parsed_type in parsed.named_types.values() {
            let defined_type = match parsed_type {
                parsed_model::TypeDefinition::Scalar(type_def) => Scalar::new(registry, &type_def, types),
                parsed_model::TypeDefinition::Object(object) => TypeDefinition::Object(Object::new(err, registry, &object, &parsed)?),
                parsed_model::TypeDefinition::Interface(interface) => Interface::new(err, registry, interface, &parsed)?,
                parsed_model::TypeDefinition::Union(union) => Union::new(err, registry, union, &parsed)?,
                parsed_model::TypeDefinition::Enum(enum_definition) => Enum::new(registry, enum_definition),
                parsed_model::TypeDefinition::InputObject(object) => TypeDefinition::InputObject(Object::new(err, registry, &object, &parsed)?),
            };
            defined_types.insert(parsed_type.name().clone(), defined_type);
        }
        
        

        let mutation = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.mutation, &schema_definition.position, &defined_types) 
        }
        else {
            Self::new_get_default_object(err, registry.intern_str("Mutation"), &defined_types, None)
        };
        
        let subscription = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.subscription, &schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, registry.intern_str("Subscription"), &defined_types, None)
        };

        let query = if let Some(schema_definition) = &parsed.schema_definition {
            Self::new_get_object(err, &schema_definition.query, &schema_definition.position, &defined_types)
        }
        else {
            Self::new_get_default_object(err, registry.intern_str("Query"), &defined_types, Some(BuildError::NoQueryDefinition))
        };

        err.ok(Rc::new(Schema {
                defined_types,
                query: query.ok_or(Error::BuildFailed(format!("Failed to find previously validated query")))?,
                mutation,
                subscription,
                __typename: Rc::new(Field {
                    parsed: parsed.__typename.clone(),
                    ty: Type::Scalar(ScalarType::BuiltinType(BuiltinType::String)),
                }),
            }))
    }
}


// pub enum OldSelection {
//     // List(Rc<SelectionList>),
//     Field(Rc<OldSelectionField>),
//     FragmentSpread(Rc<FragmentSpread>),
//     // InlineFragment(InlineFragment),
// } 

// impl OldSelection {
//     pub fn new(err: &mut ErrorCollector, selection: &parsed_model::Selection, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument, context: &dyn Context) -> Result<Self, Error> {
//         match selection {
//             parsed_model::Selection::Field(selection_field) => {
//                 Ok(OldSelection::Field(Rc::new(OldSelectionField::new(err, selection_field, schema, executable_document, context)?)))
//             },
//             parsed_model::Selection::FragmentSpread(fragment_spread) => Ok(OldSelection::FragmentSpread(Rc::new(FragmentSpread::new(err, fragment_spread,
//                  schema, executable_document)?))),
//         }
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         match self {
//             // Selection::List(selection_list) => selection_list.print(out),
//             OldSelection::Field(selection_field) => selection_field.print(out),
//             OldSelection::FragmentSpread(fragment_spread) => fragment_spread.print(out),
//         }
//     }

//     pub fn generate_query(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
//         match self {
//             // Selection::List(selection_list) => selection_list.generate_query(out, context, schema, executable_document, maybe_optional),
//             OldSelection::Field(selection_field) => selection_field.generate_query(out, context, schema, executable_document),
//             OldSelection::FragmentSpread(fragment_spread) => fragment_spread.generate_query(out, context, schema, executable_document, maybe_optional),
//         }
//     }
// }

// pub struct OldSelectionList {
//     selections: IndexMap<Atom, OldSelection>,
// }

// impl Isomorphic for OldSelectionList {
//     fn is_isomorphic(&self, other: &Self) -> bool {
//         self.parsed.name == other.parsed.name && 
//         self.parsed.optional == other.parsed.optional && 
//         self.selections.is_isomorphic(&other.selections) && 
//         self.field.is_isomorphic(&other.field)
//     }
// }

// impl OldSelectionList {
//     fn new() -> Self {
//         Self {
//             selections: IndexMap::new(),
//         }
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "selections {{")?;
//         {
//             let mut out = out.indent();

//             for item in self.selections.values() {
//                 item.print(&mut out)?;
//             }
//         }
//         writeln!(out, "}}")
//     }
    
//     fn generate_query(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
//         if ! &self.selections.is_empty() {
//             writeln!(out, "{{")?;
//             {
//                 let mut out = out.indent();

            
//                 for selection in self.selections.values() {
//                     selection.generate_query(&mut out, context, schema, executable_document, true)?;
//                 }
//             }
//             writeln!(out, "}}")?;
//         }
//         Ok(())
//     }
    
//     fn gather_fields(&self, field_map: &mut IndexMap<Atom, Rc<OldSelectionField>>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, maybe_optional: bool) -> Result<(), Error> {
//         for selection in self.selections.values() {

//             match selection {
//                 OldSelection::Field(selection_field) => {
//                     let name = if let Some(alias) = &selection_field.parsed.alias {
//                         alias
//                     }
//                     else {
//                         &selection_field.parsed.name
//                     };
//                     field_map.insert(name.clone(), selection_field.clone());
//                 },
//                 OldSelection::FragmentSpread(fragment_spread) => {
//                     let fragment = executable_document.get_fragment(&fragment_spread.parsed.name)?;
                    
//                     fragment.selections.gather_fields(field_map, context, schema, executable_document, maybe_optional)?;
//                 },
//                 // Selection::List(selection_list) => {
//                 //     selection_list.gather_fields(field_map, context, schema, executable_document, maybe_optional)?;
//                 // },
//             };
//         }
//         Ok(())
//     }
    
//     fn generate_structs(&self, out: &mut Output<'_>, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument, field_map: &IndexMap<Atom, Rc<OldSelectionField>>, dependencies: &HashSet<Atom>) -> Result<(), Error> {
//         for (name, selection_field) in field_map {
//             if name.as_ref() == TYPE_NAME {}
//             else {
//                 if let Some(field) = context.fields().get(&selection_field.parsed.name) {
//                     if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
//                         println!("defined_type.name()e={}", defined_type.name());
//                         println!("&field.parsed.name={}", &field.parsed.name);
//                         println!("dependencies={:?}", dependencies);

//                         if ! dependencies.contains(defined_type.name()) {
//                             defined_type.generate_selection(out, schema, executable_document, &selection_field.selections, &None, dependencies)?;
                            
//                             //(out, schema, executable_document, Some(&selection_field.selections), &alias, type_set)?;
//                         }
                        
//                         // println!("defined_type.name()={}", defined_type.name());
//                         // if defined_type.name().as_ref() == "PageInfo" {
//                         //     println!("STOP {:?}", type_set);
//                         // }
//                         // if type_set.insert(defined_type.name().clone()) {
//                         //     let alias = if name == &selection_field.parsed.name {
//                         //         None
//                         //     }
//                         //     else {
//                         //         Some(name.clone())
//                         //     };

                            
//                         // }
//                     }
//                 }
//                 else {
//                     writeln!(out, "UNKNOWN FIELD 2 {}", &selection_field.parsed.name)?;
//                 }
//             }
//         }
//         Ok(())
//     }
    
//     fn gather_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument, context: &dyn Context, field_map: &IndexMap<Atom, Rc<OldSelectionField>>, dependencies: &mut HashSet<Atom>) -> Result<(), Error> {
//         for (name, selection_field) in field_map {
//             if name.as_ref() == TYPE_NAME {}
//             else {
//                 println!("field {}", &selection_field.parsed.name);
//                 if let Some(field) = context.fields().get(&selection_field.parsed.name) {
//                     if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
//                         defined_type.gather_selection_dependencies(schema, executable_document, &selection_field.selections, dependencies)?;
//                         // match defined_type {
//                         //     DefinedType::Scalar(_) => {
//                         //         dependencies.insert(defined_type.name().clone());
//                         //     },
//                         //     DefinedType::Object(content) => {
//                         //         // TODO: check to see if all fields selected
//                         //     },
//                         //     DefinedType::Interface(_) => {
//                         //         // TODO: check to see if all fields selected
//                         //     },
//                         //     DefinedType::Union(_) => {
//                         //         // TODO: check to see if all fields selected
//                         //     },
//                         //     DefinedType::Enum(_) => {
//                         //         dependencies.insert(defined_type.name().clone());
//                         //     },
//                         //     DefinedType::InputObject(_) => {
//                         //         dependencies.insert(defined_type.name().clone());
//                         //     },
//                         // };
//                     }
//                 }
//                 else {
//                     println!("Failed to find field {}", &selection_field.parsed.name);
//                 }
//             }
//         }
//         Ok(())
//     }
    
//     fn generate_fields(&self, out: &mut Output<'_>, context: &dyn Context, field_map: &IndexMap<Atom, Rc<OldSelectionField>>) -> Result<(), Error> {
        
//         writeln!(out, "// generate_fields for SelectionList {}", &field_map.len())?;
//         for field in field_map.values() {
//             field.generate_fields(out, context)?;
//         }

//         Ok(())
//     }

//     fn get_variants<'a>(&self, name: &Rc<String>, schema: &'a Rc<Schema>, executable_document: &ExecutableDocument, context: &'a dyn Context) -> Result<IndexMap<Rc<String>, (&'a dyn Context, OldSelectionList)>, Error> {
//         let mut variants = IndexMap::new();

//         self.gather_variants(name, schema, executable_document, context, &mut variants)?;

//         Ok(variants)
//     }

//     fn gather_variants<'a>(&self, name: &Rc<String>, schema: &'a Rc<Schema>, executable_document: &ExecutableDocument, context: &'a dyn Context, variants: &mut IndexMap<Rc<String>, (&'a dyn Context, OldSelectionList)>) -> Result<(), Error> {
//         for selection in self.selections.values() {
//             match selection {
//                 OldSelection::Field(selection_field) => {
//                     variants.entry(name.clone()).or_insert((context, OldSelectionList::new())).1.selections.insert(selection_field.parsed.name, OldSelection::Field(selection_field.clone()));
//                 },
//                 OldSelection::FragmentSpread(fragment_spread) => {
//                     let fragment = executable_document.get_fragment(&fragment_spread.parsed.name)?;
//                     let name = &fragment.parsed.type_condition;
//                     let fragment_context = schema.get_object(name)?.as_ref();
//                     variants.entry(name.clone()).or_insert((fragment_context, OldSelectionList::new())).1.selections.insert(OldSelection::FragmentSpread(fragment_spread.clone()));
//                 },
//                 // Selection::List(selection_list) => {
//                 //     selection_list.gather_variants(name, schema, executable_document, context, variants)?;
//                 // },
//             };
//         }
//         Ok(())
//     }
// }


// pub struct OldSelectionField {
//     pub parsed: Rc<parsed_model::SelectionField>,
//     pub nonnull: bool,
//     pub arguments: Vec<Rc<parsed_model::Argument>>,
//     pub selections: OldSelectionList,
//     pub field: Rc<Field>,
// }

// impl Isomorphic for OldSelectionField {
//     fn is_isomorphic(&self, other: &Self) -> bool {
//         self.parsed.name == other.parsed.name && 
//         self.parsed.optional == other.parsed.optional && 
//         self.selections.is_isomorphic(&other.selections) && 
//         self.field.is_isomorphic(&other.field)
//     }
// }

// impl OldSelectionField {
//     pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::SelectionField>, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument, context: &dyn Context) -> Result<Self, Error> {
//         let mut arguments = Vec::new();
//         let mut selections = OldSelectionList::new();

//         println!("SlectionField name={} alias={:?}", &parsed.name, &parsed.alias);

//         if *parsed.name == TYPE_NAME {
//             println!("HERE TYPE_NAME");
//             Ok(OldSelectionField {
//                 parsed: parsed.clone(),
//                 nonnull: !parsed.optional,
//                 arguments,
//                 selections,
//                 field: schema.__typename.clone(),
//             })
//         }
//         else {
//             if let Some(field) = context.fields().get(&parsed.name) {
//                 for parsed_argument in &parsed.arguments {
//                     // arguments.push(Argument::new(prsed_argument));
//                     arguments.push(parsed_argument.clone());
//                 }

//                 if ! parsed.selections.is_empty() {
//                     if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
//                         let optional_context: Option<&dyn Context> =  match defined_type {
//                             DefinedType::Scalar(_) => None,
//                             DefinedType::Object(name) => Some(schema.get_object(name)?.as_ref()),
//                             DefinedType::Interface(name) => Some(schema.get_interface(name)?.as_ref()),
//                             DefinedType::Union(name) => Some(schema.get_union(name)?.as_ref()),
//                             DefinedType::Enum(_) => None,
//                             DefinedType::InputObject(_) => {
//                                 err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Selection on InputObject {}", field.graphql_name())));
//                                 None
//                             },
//                         };
        
//                         if let Some(context) = optional_context {
//                             for selection in &parsed.selections {
//                                 if let Ok(selection) = OldSelection::new(err, &selection, schema, executable_document, context) {
//                                     selections.selections.push(selection);
//                                 }
//                             }
//                         }
//                         else {
//                             println!("Attribute selection given on incompatible type \"{}\"", field.ty);
//                             err.error(BuildError::TypeMismatchError(parsed.position.clone(), format!("Attribute selection given on incompatible type \"{}\"", field.ty)));
//                         }
//                     }
//                 }
//                 err.ok(OldSelectionField {
//                     parsed: parsed.clone(),
//                     nonnull: !parsed.optional,
//                     arguments,
//                     selections,
//                     field: field.clone(),
//                 })
//             }
//             else {
//                 err.fail(BuildError::MissingFieldError(parsed.position.clone(), parsed.name.to_string()))
//             }
//         }

        
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "SelectionField {{")?;
//         {
//             let mut out = out.indent();

//             self.parsed.print(&mut out)?;
//             writeln!(out, "nonnull:  {}", self.nonnull)?;
//             writeln!(out, "arguments {{")?;
//             {
//                 let mut out = out.indent();

//                 for argument in &self.arguments {
//                     argument.print(&mut out)?;
//                 }
//             }
//             writeln!(out, "}}")?;

//             self.selections.print(&mut out)?;
//         }
//         writeln!(out, "}}")
//     }

//     pub fn generate_query(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
//         if let Some(alias) = &self.parsed.alias {
//             writeln!(out, "{}: {}", alias, &self.parsed.name)?;
//         }
//         else {
//             writeln!(out, "{}", &self.parsed.name)?;
//         }

//         if ! self.arguments.is_empty() {
//             writeln!(out, "(")?;
//             {
//                 let mut out = out.indent();

            
//                 for argument in &self.arguments {
//                     argument.generate_query(&mut out)?;
//                 }
//             }
//             writeln!(out, ")")?;
//         }

//         self.selections.generate_query(out, context, schema, executable_document, true)?;
//         Ok(())
//     }

//     pub fn generate_fields(&self, out: &mut Output, context: &dyn Context) -> Result<(), Error> {
//         if *self.parsed.name == TYPE_NAME {
//             println!("HERE");
//             writeln!(out, "/* HERE1 */ pub __typename: String,")?;
//         }
//         else {
//             let field = match context.fields().get(&self.parsed.name) {
//                 Some(f) => f,
//                 None => {
//                     println!("&self.parsed.name={:?}", &self.parsed.name);
//                     // println!("context.fields={:?}", context.fields());
//                     panic!("Failed to find field {}", self.parsed.name);
//                 },
//             };
//             let name = if let Some(alias) = &self.parsed.alias {
//                 alias
//             }
//             else {
//                 &self.parsed.name
//             };

//             let rust_type = if let Some(alias) = &self.parsed.alias {
//                 to_pascal_case(alias)
//             }
//             else {
//                 field.ty.rust_type(self.nonnull)
//             };

//             writeln!(out, "#[serde(rename = \"{}\")]", &name)?;
//             writeln!(out, "pub {}_: {},", to_snake_case(&name), rust_type)?;
//         }
//         Ok(())
//     }

//     pub fn generate_structs(&self, out: &mut Output, context: &dyn Context, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<(), Error> {
//         let field = context.fields().get(&self.parsed.name).unwrap();

//         if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
//             writeln!(out, "// {} is {}", &self.parsed.name, defined_type)?;
            
//             defined_type.generate(out, schema, &self.parsed.alias)?;
//         }
//         else {
//             writeln!(out, "// Nothing to generate because {} is {}", &self.parsed.name, field.ty)?;
//         }
//         Ok(())
//     }
// }

// pub struct Dependencies {
//     pub next_id: u32,
//     pub selections: BagOfNamed<Selection>,
//     // pub operation_types: BagOfNamed<TypeDefinition>,
//     pub schema_types: HashSet<Atom>,
// }

// impl Dependencies {
//     fn new() -> Dependencies {
//         Dependencies {
//             next_id: 0,
//             selections: BagOfNamed::new(),
//             // operation_types: BagOfNamed::new(),
//             schema_types: HashSet::new(),
//         }
//     }

//     fn insert_schema_type(&mut self, name: &Atom) {
//         self.schema_types.insert(name.clone());
//     }

//     fn insert_operation_type(&mut self, name: &Atom, operation_type: TypeDefinition) {
//         self.operation_types.insert(name, operation_type);
//     }

//     fn insert_selection(&mut self, name: &Atom, selection: Selection) {
//         self.selections.insert(name, selection);
//     }
    
//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "Dependencies {{")?;
//         {
//             let mut out = out.indent();

//             self.selections.print(&mut out)?;
//             writeln!(out, "types {{")?;
//             {
//                 let mut out = out.indent();

//                 for item in &self.schema_types {
//                     writeln!(out, "{}", item)?;
//                 }
//             }
//             writeln!(out, "}}")?;
//         }
//         writeln!(out, "}}")
//     }
// }

/*
{
                    match defined_type {
                        DefinedType::Scalar(type_name) => dependencies.insert(type_name, TypeDefinition::Scalar(schema.get_scalar(type_name)?.clone())),
                        DefinedType::Object(type_name) => dependencies.insert(type_name, TypeDefinition::Object(schema.get_object(type_name)?.clone())),
                        DefinedType::Interface(type_name) => dependencies.insert(type_name, TypeDefinition::Interface(schema.get_interface(type_name)?.clone())),
                        DefinedType::Union(type_name) => dependencies.insert(type_name, TypeDefinition::Union(schema.get_union(type_name)?.clone())),
                        DefinedType::Enum(type_name) => dependencies.insert(type_name, TypeDefinition::Enum(schema.get_enum(type_name)?.clone())),
                        DefinedType::InputObject(type_name) => dependencies.insert(type_name, TypeDefinition::InputObject(schema.get_input_object(type_name)?.clone())),
                    }
                },
*/

pub struct GenericOperation {
    pub parsed: Rc<parsed_model::GenericOperation>,
    pub variables: FieldMap,
    // pub selections: SelectionList,
    // pub selection: Selection,
    // pub dependencies: Dependencies,
    pub response: Rc<Selection>,
    // pub schema_dependencies: HashSet<Atom>,
}

impl GenericOperation {
    fn get_fields<'a>(err: &mut ErrorCollector, operation: &OperationType, name: &Atom, position: &Pos, schema: &'a Rc<Schema>) -> Result<&'a FieldMap, Error> {
        match operation {
            OperationType::Query => {
                Ok(&schema.get_object(&schema.query)?.fields)
            },
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

    // fn get_selection(err: &mut ErrorCollector, selections: &Rc<parsed_model::SelectionList>, name: &Atom, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>, fields: &FieldMap) -> Result<Selection, Error> {
    //     let mut selection_builder: SelectionBuilder = Selection::builder(selections);

    //     Self::gather_selection(err, selections, name, schema, fragments, fields, &mut selection_builder)?;

    //     Ok(selection_builder.build())
    // }

    // fn gather_selection(err: &mut ErrorCollector, selections: &Rc<parsed_model::SelectionList>, name: &Atom, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>, fields: &FieldMap, selection_builder: &mut SelectionBuilder) -> Result<(), Error> {
    //     for selection in &selections.selections {
    //         match selection {
    //             parsed_model::Selection::Field(selection_field) => {
    //                 let view_field: Rc<ViewField> = if let Some(field) = get_field(schema, fields, &selection_field.name) {
    //                     if selection_field.selections.is_empty() {
    //                         ViewField::from_field(field)
    //                     }
    //                     else {
    //                         if let Some(fields) = field.ty.get_fields(schema)? {
    //                             ViewField::from_selection(field, Self::get_selection(err, &selection_field.selections, &selection_field.name, schema, fragments, fields)?)
    //                         }
    //                         else {
    //                             ViewField::from_error(&selection_field.name, BuildError::SelectionOnNonObjectError(selection_field.position.clone().clone(), selection_field.name.clone()))
    //                         }
    //                     }
    //                 }
    //                 else {
    //                         ViewField::from_error(&selection_field.name, BuildError::MissingFieldError(selection_field.position.clone(), selection_field.name.to_string()))
    //                 };

    //                 if let View::Invalid(error) = &view_field.view {
    //                     err.error(error.clone());
    //                 }

    //                 selection_builder.insert(name, &selection_field.name, view_field);
    //             },
    //             parsed_model::Selection::FragmentSpread(fragment_spread) => {
    //                 if let Some(fragment) = fragments.get(&fragment_spread.name) {
    //                     Self::gather_selection(err, &fragment.selections, &fragment.type_condition, schema, fragments, &schema.get_object(&fragment.type_condition)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     err.error(BuildError::MissingFragmentError(fragment_spread.position.clone(), fragment_spread.name.to_string()));
    //                 }
                
    //             },
    //             parsed_model::Selection::InlineFragment(inline_fragment_spread) => {
    //                 if let Some(fragment_name) = &inline_fragment_spread.type_condition {
    //                     Self::gather_selection(err, &inline_fragment_spread.selections, fragment_name, schema, fragments, &schema.get_object(fragment_name)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     // anonymous inline spread is essentially a no op nesting
    //                     Self::gather_selection(err, &inline_fragment_spread.selections, name, schema, fragments, fields, selection_builder)?;
    //                 }
    //             },
    //         };
    //     }
    //     Ok(())
    // }




    // fn new_gather_selection(err: &mut ErrorCollector, selections: &Rc<parsed_model::SelectionList>, name: &Atom, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>,
    //     fields: &FieldMap, dependencies: &mut Dependencies, field_builder: &SharedMapBuilder<Field>, selection_builder: &mut SelectionBuilder) -> Result<(), Error>
    // {
    //     for selection in &selections.selections {
    //         match selection {
    //             parsed_model::Selection::Field(selection_field) => {
    //                 if let Some(field) = get_field(schema, fields, &selection_field.name) {
    //                     field_builder.insert(selection_field.name.clone(), field.clone());
    //                     if ! selection_field.selections.is_empty() {
    //                         if let Some(field_fields) = field.ty.get_fields(schema)? {
    //                             let field_is_interface = if let ScalarType::DefinedType(defined_type) = field.ty.get_scalar() {
    //                                 if let DefinedType::Interface(_) = defined_type {
    //                                     true
    //                                 }
    //                                 else {
    //                                     false
    //                                 }
    //                             }
    //                             else {
    //                                 false
    //                             };
                                
    //                             Self::new_gather_dependencies(err, &selection_field.selections, &selection_field.name, schema, fragments, field_fields, field_is_interface, dependencies);
    //                         }
    //                         else {
    //                             err.error(BuildError::SelectionOnNonObjectError(selection_field.position.clone().clone(), selection_field.name.clone()));
    //                         }
    //                     }
    //                 }
    //                 else {
    //                     err.error(BuildError::MissingFieldError(selection_field.position.clone(), selection_field.name.to_string()));
    //                 }
    //             },
    //             parsed_model::Selection::FragmentSpread(fragment_spread) => {
    //                 if let Some(fragment) = fragments.get(&fragment_spread.name) {
    //                     let object = schema.get_object(&fragment.type_condition)?;
    //                     let fragment_field_builder = selection_builder.get_variant(&fragment.type_condition);
    //                     Self::new_gather_selection(err, &fragment.selections, name, schema, fragments, &object.fields, dependencies, fragment_field_builder, selection_builder);




    //                         Self::gather_selection(err, &fragment.selections, &fragment.type_condition, schema, fragments, &schema.get_object(&fragment.type_condition)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     err.error(BuildError::MissingFragmentError(fragment_spread.position.clone(), fragment_spread.name.to_string()));
    //                 }
                
    //             },
    //             parsed_model::Selection::InlineFragment(inline_fragment_spread) => {
    //                 if let Some(fragment_name) = &inline_fragment_spread.type_condition {
    //                     Self::gather_selection(err, &inline_fragment_spread.selections, fragment_name, schema, fragments, &schema.get_object(fragment_name)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     // anonymous inline spread is essentially a no op nesting
    //                     Self::gather_selection(err, &inline_fragment_spread.selections, name, schema, fragments, fields, selection_builder)?;
    //                 }
    //             },
    //         };
    //     }

    //     Ok(())
    // }
    
    fn create_selection(err: &mut ErrorCollector, registry: &mut Registry, selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>,
        preferred_type_names: Vec<Atom>, graphql_type_name: Atom, selections: &Rc<parsed_model::SelectionList>, fields: &FieldMap, interface: Option<Rc<Interface>>) -> Result<Rc<Selection>, Error>
    {
        let mut selection_builder = SelectionBuilder::new(graphql_type_name, selections, interface);
        
        selection_builder.with_preferred_type_names(preferred_type_names);

        Self::new_gather_dependencies(err, registry, selection_manager, selections, schema, fragments, None, fields, &mut selection_builder)?;

        Ok(selection_builder.build(selection_manager))
    }

    fn create_selection_field(err: &mut ErrorCollector, registry: &mut Registry, selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>,
        field_name: Atom, preferred_type_names: Vec<Atom>, graphql_type_name: Atom, selections: &Rc<parsed_model::SelectionList>, optional: bool, fields: &FieldMap, interface: Option<Rc<Interface>>) -> Result<SelectionField, Error>
    {
        // let mut selection_builder = SelectionBuilder::new(selections);
        
        // selection_builder.with_names(names);

        // Self::new_gather_dependencies(err, registry, selection_manager, schema_dependencies, selections, schema, fragments, None, fields, &mut selection_builder)?;

        let index = Self::create_selection(err, registry, selection_manager, schema, fragments, preferred_type_names, graphql_type_name, selections, fields, interface)?.index;

        // println!("insert {}", index);
        // selection_manager.get_context_mut().imports.insert(index);
        
        Ok(SelectionField {
            name: field_name,
            selection_type: SelectionFieldType::Selection(index),
            optional,
        })
    }



    fn new_get_dependencies(selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fields: &FieldMap) -> Result<(), Error> {
        for (name, field) in fields {
            match field.ty.get_scalar() {
                ScalarType::DefinedType(defined_type) => {
                    match defined_type {
                        DefinedType::Scalar(name) => {
                            selection_manager.import_schema(name.clone());
                        },
                        DefinedType::Object(name) => {
                            selection_manager.import_schema(name.clone());

                            let object = schema.get_object(name)?;

                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                        },
                        DefinedType::Interface(name) => {
                            selection_manager.import_schema(name.clone());
                            let object = schema.get_interface(name)?;

                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                        },
                        DefinedType::Union(name) => {
                            selection_manager.import_schema(name.clone());
                            let object = schema.get_union(name)?;

                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                        }
                        DefinedType::Enum(name) => {
                            selection_manager.import_schema(name.clone());
                        },
                        DefinedType::InputObject(name) => {
                            selection_manager.import_schema(name.clone());
                            let object = schema.get_input_object(name)?;
                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                        },
                    }
                },
                ScalarType::BuiltinType(builtin_type) => {}
            }
        }
        Ok(())
    }
    
    fn wrap_type(ty: &Type, selection_field: SelectionField) -> SelectionField {
        SelectionField {
            name: selection_field.name,
            selection_type: Self::wrap_type2(ty, selection_field.selection_type),
            optional: selection_field.optional,
        }
    }

    fn wrap_type2(ty: &Type, selection_type: SelectionFieldType) -> SelectionFieldType {
        match ty {
            Type::Required(wrapped) => SelectionFieldType::Required(Box::new(Self::wrap_type2(wrapped, selection_type))),
            Type::Array(wrapped) => SelectionFieldType::Array(Box::new(Self::wrap_type2(wrapped, selection_type))),
            Type::Scalar(_) => selection_type,
        }
        
    }
    
    fn new_gather_dependencies(err: &mut ErrorCollector, registry: &mut Registry, selection_manager: &mut NameSpaceManager, selections: &Rc<parsed_model::SelectionList>, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>,
        variant_name: Option<(Atom, Atom)>, fields: &FieldMap, selection_builder: &mut SelectionBuilder) -> Result<(), Error>
    {
        // let mut is_interface = is_interface;
        // let mut fields_builder = FieldMap::builder();
        // let mut selection_builder = SelectionBuilder::new( /*&parsed.selections*/ );

        for selection in &selections.selections {
            match selection {
                parsed_model::Selection::Field(selection_field) => {
                    let field_name  = if let Some(alias) = &selection_field.alias { alias } else {&selection_field.name};
                    if let Some(field) = get_field(schema, fields, &selection_field.name) {
                        selection_builder.with_field(&variant_name,  &field_name, 
                            Self::wrap_type(&field.ty, 
                                match field.ty.get_scalar() {
                                ScalarType::DefinedType(defined_type) => {
                                    match defined_type {
                                        DefinedType::Scalar(name) => {
                                            selection_manager.import_schema(name.clone());
                                            SelectionField {
                                                name: field_name.clone(), 
                                                selection_type: SelectionFieldType::Scalar(name.clone()),
                                                optional: selection_field.optional,
                                            }
                                        },
                                        DefinedType::Object(name) => {
                                            let object = schema.get_object(name)?;
                                            let mut preferred_type_names = Vec::new();

                                            if let Some(alias) = &selection_field.alias {
                                                preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
                                            }
                                            preferred_type_names.push(registry.intern(to_pascal_case(&object.name)));
                                            preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments,
                                                field_name.clone(), preferred_type_names, name.clone(), &selection_field.selections, selection_field.optional, &object.fields, None)?
                                        },
                                        DefinedType::Interface(name) => {
                                            let object = schema.get_interface(name)?;
                                            let mut preferred_type_names = Vec::new();

                                            if let Some(alias) = &selection_field.alias {
                                                preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
                                            }
                                            preferred_type_names.push(registry.intern(to_pascal_case(&object.parsed.name)));
                                            preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_names, name.clone(), &selection_field.selections, selection_field.optional, &object.fields, Some(object.clone()))?
                                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                                        },
                                        DefinedType::Union(name) => {
                                            let object = schema.get_union(name)?;
                                            let mut preferred_type_names = Vec::new();

                                            if let Some(alias) = &selection_field.alias {
                                                preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
                                            }
                                            preferred_type_names.push(registry.intern(to_pascal_case(&object.parsed.name)));
                                            preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_names, name.clone(), &selection_field.selections, selection_field.optional, &object.fields, None)?
                                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                                        }
                                        DefinedType::Enum(name) => {
                                            selection_manager.import_schema(name.clone());
                                            SelectionField {
                                                name: field_name.clone(), 
                                                selection_type: SelectionFieldType::Enum(name.clone()),
                                                optional: selection_field.optional,
                                            }
                                        },
                                        DefinedType::InputObject(name) => {
                                            let object = schema.get_input_object(name)?;
                                            let mut preferred_type_names = Vec::new();

                                            if let Some(alias) = &selection_field.alias {
                                                preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
                                            }
                                            preferred_type_names.push(registry.intern(to_pascal_case(&object.name)));
                                            preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_names, name.clone(), &selection_field.selections, selection_field.optional, &object.fields, None)?
                                                //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
                                        },
                                    }
                                },
                                ScalarType::BuiltinType(builtin_type) => SelectionField {
                                    name: field_name.clone(), 
                                    selection_type: SelectionFieldType::BuiltinType(builtin_type.clone()),
                                    optional: selection_field.optional,
                                }
                        }));

                        // selection_builder.with_field(variant_name,  &selection_field.name, selection_field);
                    }
                    else {
                        println!("fields");
                        for (name, field) in fields {
                            println!("{}: {:?}", name, field.graphql_name());
                        }
                        println!("not found: {}",selection_field.name);
                        err.error(BuildError::MissingFieldError(selection_field.position.clone(), selection_field.name.to_string()));
                    }
                },
                parsed_model::Selection::FragmentSpread(fragment_spread) => {
                    if let Some(fragment) = fragments.get(&fragment_spread.name) {
                        let object = schema.get_object(&fragment.type_condition)?;


                        Self::new_gather_dependencies(err, registry, selection_manager, &fragment.selections, schema, fragments, Some((fragment_spread.name.clone(), fragment.type_condition.clone())), 
                            &object.fields, selection_builder)?;


                        // let fragment_field_builder = selection_builder.get_variant(&fragment.type_condition);
                        // Self::new_gather_selection(err, &fragment.selections, name, schema, fragments, &object.fields, dependencies, fragment_field_builder, selection_builder);




                        //     Self::gather_selection(err, &fragment.selections, &fragment.type_condition, schema, fragments, &schema.get_object(&fragment.type_condition)?.fields, selection_builder)?;
                    }
                    else {
                        err.error(BuildError::MissingFragmentError(fragment_spread.position.clone(), fragment_spread.name.to_string()));
                    }
                
                },
                parsed_model::Selection::InlineFragment(inline_fragment_spread) => {
                    if let Some(type_condition) = &inline_fragment_spread.type_condition {
                        let object = schema.get_object(type_condition)?;
                        Self::new_gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, Some((type_condition.clone(), type_condition.clone())),
                            &object.fields, selection_builder)?;

                        // Self::gather_selection(err, &inline_fragment_spread.selections, fragment_name, schema, fragments, &schema.get_object(fragment_name)?.fields, selection_builder)?;
                    }
                    else {
                        // anonymous inline spread is essentially a no op nesting

                        Self::new_gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, None,
                            fields, selection_builder)?;

                        // Self::gather_selection(err, &inline_fragment_spread.selections, name, schema, fragments, fields, selection_builder)?;
                    }
                },
            };
        }

        
        Ok(())
       
    }

    // fn new_get_dependencies(selection_manager: &mut SelectionManager, schema: &Rc<Schema>, fields: &FieldMap) -> Result<(), Error> {
    //     for (name, field) in fields {
    //         match field.ty.get_scalar() {
    //             ScalarType::DefinedType(defined_type) => {
    //                 match defined_type {
    //                     DefinedType::Scalar(name) => {
    //                         let object = schema.get_scalar(name)?;

    //                         selection_manager.import_schema(object.rust_name.clone());
    //                     },
    //                     DefinedType::Object(name) => {
    //                         let object = schema.get_object(name)?;

    //                         selection_manager.import_schema(object.rust_name.clone());
    //                         Self::new_get_dependencies(selection_manager, schema, &object.fields)?
    //                     },
    //                     DefinedType::Interface(name) => {
    //                         let object = schema.get_interface(name)?;
    //                         selection_manager.import_schema(object.rust_name.clone());

    //                         Self::new_get_dependencies(selection_manager, schema, &object.fields)?
    //                             //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                     },
    //                     DefinedType::Union(name) => {
    //                         let object = schema.get_union(name)?;
    //                         selection_manager.import_schema(object.rust_name.clone());

    //                         Self::new_get_dependencies(selection_manager, schema, &object.fields)?
    //                             //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                     }
    //                     DefinedType::Enum(name) => {
    //                         selection_manager.import_schema(name.clone());
    //                     },
    //                     DefinedType::InputObject(name) => {
    //                         let object = schema.get_input_object(name)?;
    //                         selection_manager.import_schema(object.rust_name.clone());
    //                         Self::new_get_dependencies(selection_manager, schema, &object.fields)?
    //                             //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                     },
    //                 }
    //             },
    //             ScalarType::BuiltinType(builtin_type) => {}
    //         }
    //     }
    //     Ok(())
    // }
    
    // fn new_gather_dependencies(err: &mut ErrorCollector, registry: &mut Registry, selection_manager: &mut SelectionManager, selections: &Rc<parsed_model::SelectionList>, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>,
    //     variant_name: Option<Atom>, fields: &FieldMap, selection_builder: &mut SelectionBuilder) -> Result<(), Error>
    // {
    //     // let mut is_interface = is_interface;
    //     // let mut fields_builder = FieldMap::builder();
    //     // let mut selection_builder = SelectionBuilder::new( /*&parsed.selections*/ );

    //     for selection in &selections.selections {
    //         match selection {
    //             parsed_model::Selection::Field(selection_field) => {
    //                 let s: &Rc<parsed_model::SelectionField> = selection_field;
    //                 if let Some(field) = get_field(schema, fields, &selection_field.name) {
    //                     selection_builder.with_field(&variant_name,  &selection_field.name, 
    //                         match field.ty.get_scalar() {
    //                         ScalarType::DefinedType(defined_type) => {
    //                             match defined_type {
    //                                 DefinedType::Scalar(name) => {
    //                                     selection_manager.import_schema(name.clone());
    //                                     SelectionField {
    //                                         name: selection_field.name.clone(), 
    //                                         selection_type: SelectionFieldType::Scalar(name.clone()),
    //                                     }
    //                                 },
    //                                 DefinedType::Object(name) => {
    //                                     let object = schema.get_object(name)?;
    //                                     let mut preferred_type_names = Vec::new();

    //                                     if let Some(alias) = &selection_field.alias {
    //                                         preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
    //                                     }
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&object.name)));
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

    //                                     Self::create_selection_field(err, registry, selection_manager, schema, fragments,
    //                                         selection_field.name.clone(), preferred_type_names, name.clone(), &selection_field.selections, &object.fields)?
    //                                 },
    //                                 DefinedType::Interface(name) => {
    //                                     let object = schema.get_interface(name)?;
    //                                     let mut preferred_type_names = Vec::new();

    //                                     if let Some(alias) = &selection_field.alias {
    //                                         preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
    //                                     }
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&object.parsed.name)));
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

    //                                     Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
    //                                         selection_field.name.clone(), preferred_type_names, name.clone(), &selection_field.selections, &object.fields)?
    //                                         //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                                 },
    //                                 DefinedType::Union(name) => {
    //                                     let object = schema.get_union(name)?;
    //                                     let mut preferred_type_names = Vec::new();

    //                                     if let Some(alias) = &selection_field.alias {
    //                                         preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
    //                                     }
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&object.parsed.name)));
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

    //                                     Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
    //                                         selection_field.name.clone(), preferred_type_names, name.clone(), &selection_field.selections, &object.fields)?
    //                                         //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                                 }
    //                                 DefinedType::Enum(name) => {
    //                                     selection_manager.import_schema(name.clone());
    //                                     SelectionField {
    //                                         name: selection_field.name.clone(), 
    //                                         selection_type: SelectionFieldType::Enum(name.clone()),
    //                                     }
    //                                 },
    //                                 DefinedType::InputObject(name) => {
    //                                     let object = schema.get_input_object(name)?;
    //                                     let mut preferred_type_names = Vec::new();

    //                                     if let Some(alias) = &selection_field.alias {
    //                                         preferred_type_names.push(registry.intern(to_pascal_case(&alias)));
    //                                     }
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&object.name)));
    //                                     preferred_type_names.push(registry.intern(to_pascal_case(&selection_field.name)));

    //                                     Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
    //                                         selection_field.name.clone(), preferred_type_names, name.clone(), &selection_field.selections, &object.fields)?
    //                                         //selection_field.name.clone(), names, &selection_field.selections, &object.fields)?
    //                                 },
    //                             }
    //                         },
    //                         ScalarType::BuiltinType(builtin_type) => SelectionField {
    //                             name: selection_field.name.clone(), 
    //                             selection_type: SelectionFieldType::BuiltinType(builtin_type.clone()),
    //                         }
    //                     });

    //                     // selection_builder.with_field(variant_name,  &selection_field.name, selection_field);
    //                 }
    //                 else {
    //                     println!("fields");
    //                     for (name, field) in fields {
    //                         println!("{}: {:?}", name, field.graphql_name());
    //                     }
    //                     println!("not found: {}",selection_field.name);
    //                     err.error(BuildError::MissingFieldError(selection_field.position.clone(), selection_field.name.to_string()));
    //                 }
    //             },
    //             parsed_model::Selection::FragmentSpread(fragment_spread) => {
    //                 if let Some(fragment) = fragments.get(&fragment_spread.name) {
    //                     let object = schema.get_object(&fragment.type_condition)?;


    //                     Self::new_gather_dependencies(err, registry, selection_manager, &fragment.selections, schema, fragments, Some(fragment_spread.name.clone()),
    //                         &object.fields, selection_builder)?;


    //                     // let fragment_field_builder = selection_builder.get_variant(&fragment.type_condition);
    //                     // Self::new_gather_selection(err, &fragment.selections, name, schema, fragments, &object.fields, dependencies, fragment_field_builder, selection_builder);




    //                     //     Self::gather_selection(err, &fragment.selections, &fragment.type_condition, schema, fragments, &schema.get_object(&fragment.type_condition)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     err.error(BuildError::MissingFragmentError(fragment_spread.position.clone(), fragment_spread.name.to_string()));
    //                 }
                
    //             },
    //             parsed_model::Selection::InlineFragment(inline_fragment_spread) => {
    //                 if let Some(type_condition) = &inline_fragment_spread.type_condition {
    //                     let object = schema.get_object(type_condition)?;
    //                     Self::new_gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, Some(type_condition.clone()),
    //                         &object.fields, selection_builder)?;

    //                     // Self::gather_selection(err, &inline_fragment_spread.selections, fragment_name, schema, fragments, &schema.get_object(fragment_name)?.fields, selection_builder)?;
    //                 }
    //                 else {
    //                     // anonymous inline spread is essentially a no op nesting

    //                     Self::new_gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, None,
    //                         fields, selection_builder)?;

    //                     // Self::gather_selection(err, &inline_fragment_spread.selections, name, schema, fragments, fields, selection_builder)?;
    //                 }
    //             },
    //         };
    //     }

        
    //     Ok(())
       
    // }

    // fn gather_dependencies(err: &mut ErrorCollector, selection: &Selection, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>, dependencies: &mut Dependencies) -> Result<(), Error> {
    //     for (_variant_name, field_map) in selection {
    //         for (_field_name, field) in field_map {
    //             match &field.view {
    //                 View::Type(type_) => match type_.get_scalar() {
    //                     ScalarType::DefinedType(defined_type) => dependencies.insert_schema_type(defined_type.name()),
    //                     ScalarType::BuiltinType(_) => {}, // Nothing
    //                 },
    //                 View::Selection(selection) => {
    //                     Self::gather_dependencies(err, selection, schema, fragments, dependencies)?;
    //                 },
    //                 View::Invalid(_) => {}, // Nothing
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    fn new(err: &mut ErrorCollector, registry: &mut Registry, selection_manager: &mut NameSpaceManager, parsed: &Rc<parsed_model::GenericOperation>, schema: &Rc<Schema>, fragments: &IndexMap<Atom, Rc<parsed_model::FragmentDefinition>>)-> Result<Self, Error> {

        let mut variable_builder = FieldMap::builder();

       

        for (name, variable) in &parsed.variables {
            if let Ok(field) = Field::from_variable(err, variable, schema) {
                if let Some(existing) = variable_builder.insert(name.clone(), field) {
                    err.error(BuildError::DuplicateName(existing.parsed.position.clone(), variable.position.clone(), variable.name.to_string()));
                }
            }
        }
        let variables = variable_builder.build();

        Self::new_get_dependencies(selection_manager, schema, &variables)?;

        let fields = Self::get_fields(err, &parsed.operation, &parsed.name, &parsed.position, schema)?;
        // let mut dependencies = Dependencies::new();
        

        
        let name = registry.intern_str("Response");
        let response = Self::create_selection(err, registry, selection_manager, schema, fragments, 
            vec!(name.clone()), name, &parsed.selections, fields, None)?;
        
        

        

        // Self::gather_dependencies(err, &selection, schema, fragments, &mut dependencies);

        err.ok(GenericOperation {
                parsed: parsed.clone(),
                variables,
                response,
                // dependencies,
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

            self.response.print(&mut out)?;
            // self.dependencies.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
    
    pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selection_manager: &NameSpaceManager) -> Result<(), Error> {
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

            for name in selection_manager.get_schema_dependencies() {
                if let Some(object) = schema.defined_types.get(name) {
                    writeln!(out, "use super::super::{};", object.rust_name())?;
                }
                else {
                    writeln!(out, "compile_error!(\"Failed to find type {}\");", name)?;
                }
            }



            writeln!(out, "// Start dependencies")?;
            println!("DEPENDENCIES doc={:?} op={:?} {:?}", &selection_manager.document_name, &selection_manager.operation_name,
            &selection_manager.get_context());
            for (name, id) in &selection_manager.get_context().names {
                writeln!(out, "// name={} id={}", name, id)?;
                let selection = selection_manager.get(*id);

                selection.generate(&mut out, selection_manager, schema)?;
            }
            writeln!(out, "// End dependencies")?;


            if !self.variables.is_empty() {
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                writeln!(out, "pub struct Variables {{")?;

                {
                    let mut out = out.indent();
                
                    for (name, field) in &self.variables {
                        writeln!(out, "#[serde(rename = \"{}\")]", &field.parsed.name)?;
                        if field.ty.is_optional() {
                            writeln!(out, "#[serde(skip_serializing_if = \"Option::is_none\")]")?;
                        }
                        writeln!(out, "{}_: {},", to_snake_case(&name), field.ty.rust_type(Optionality::Default))?;
                    }
                }

                writeln!(out, "}}")?;
                writeln!(out, "")?;




                //-------

                let rust_name = "Variables";

                writeln!(out, "impl {} {{", rust_name)?;
                {
                    let mut out = out.indent();
        
                    writeln!(out, "pub fn builder() -> {}Builder {{", rust_name)?;
                    writeln!(out, "    {}Builder {{", rust_name)?;
                    for field in self.variables.values() {
                        writeln!(out, "        {}_: None,", to_snake_case(&field.parsed.name))?;
                    }
                    writeln!(out, "    }}")?;
                    writeln!(out, "}}")?;


        
                    writeln!(out, "pub fn get_formal_params(&self, buf: &mut String) {{")?;
                    // writeln!(out, "    let mut buf = String::new();")?;
                    for (name, field) in &self.variables {
                        if field.ty.is_optional() {
                            writeln!(out, "    if self.{}_.is_some() {{", to_snake_case(&field.parsed.name))?;
                            writeln!(out, "        buf.push_str(\"${}: {},\");", &name, field.graphql_name())?;
                            writeln!(out, "    }}")?;
                        }
                        else {
                            writeln!(out, "    buf.push_str(\"${}: {},\");", &name, field.graphql_name())?;
                        }
                    }

                    // writeln!(out, "    buf")?;
                    // writeln!(out, "    }}")?;
                    writeln!(out, "}}")?;
                }
            
                writeln!(out, "}}")?;
                writeln!(out, "")?;
        
                writeln!(out, "")?;
                writeln!(out, "pub struct {}Builder {{", rust_name)?;
        
                for field in self.variables.values() {
                    writeln!(out, "    {}_: {},", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Optional))?;
                }
            
                writeln!(out, "}}")?;
                writeln!(out, "")?;
        
                writeln!(out, "impl {}Builder {{", rust_name)?;
                {
                    let mut out = out.indent();
        
                    for field in self.variables.values() {
                        writeln!(out, "pub fn with_{}(mut self, value: {}) -> Self {{", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Required))?;
                        {
                            let mut out = out.indent();
        
                            writeln!(out, "self.{}_ = Some(value);", to_snake_case(&field.parsed.name))?;
                            writeln!(out, "self")?;
                        }
                        writeln!(out, "}}")?;
                        writeln!(out, "")?;
                    }
                    writeln!(out, "pub fn build(self) -> Result<{}, sparko_graphql::error::Error> {{", rust_name)?;
                    {
                        let mut out = out.indent();
        
                        for field in self.variables.values() {
                            if let Type::Required(_) = field.ty {
                                writeln!(out, "if let None = self.{}_ {{", to_snake_case(&field.parsed.name))?;
                                writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", field.parsed.name)?;
                                writeln!(out, "}}")?;
                            }
                        }
        
                        writeln!(out, "Ok({} {{", rust_name)?;
                        {
                            let mut out = out.indent();
        
                            for field in self.variables.values() {
                                if let Type::Required(_) = field.ty {
                                    writeln!(out, "{}_: self.{}_.unwrap(),", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
                                }
                                else {
                                    writeln!(out, "{}_: self.{}_,", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
                                }
                            }
                            writeln!(out, "}})")?;
                        }
                    }
                    writeln!(out, "}}")?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;

                //-------
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

                // if self.variables.is_empty() {
                //     writeln!(out, "const {}: &str = r#\"{} {}", self.parsed.operation.to_upper_case(), self.parsed.operation.to_lower_case(), self.parsed.name)?;
                // }
                // else {
                //     writeln!(out, "const {}: &str = r#\"{} {}(", self.parsed.operation.to_upper_case(), self.parsed.operation.to_lower_case(), self.parsed.name)?;
                //     {
                //         let mut out = out.indent();
                    
                //         for (name, field) in &self.variables {
                //             writeln!(out, "${}: {},", to_snake_case(&name), field.graphql_name()
                //             // field.rust_type(schema, Maybe::Maybe, &None)
                //             )?;
                //         }
                //     }
                //     write!(out, ")")?;
                // }
                // self.response.generate_query(&mut out)?;
                
                // writeln!(out, "\"#;")?;
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
                            writeln!(out, "{}_: {},", to_snake_case(&name), field.ty.rust_type(Optionality::Default))?;
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

                                for name in self.variables.keys() {
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

                writeln!(out, "fn get_query(&self) -> String {{")?;
                // writeln!(out.indent(), "Self::{}", self.parsed.operation.to_upper_case())?;
                {
                    let mut out = out.indent();
                    writeln!(out, "let mut buf = String::from(\"{} {}\");", self.parsed.operation.to_lower_case(), self.parsed.name)?;

                    if ! self.variables.is_empty() {

                        writeln!(out, "buf.push('(');")?;
                        writeln!(out, "self.variables.get_formal_params(&mut buf);")?;
                        writeln!(out, "buf.push(')');")?;
                        
                    }

                    let mut done_fragments = HashSet::new();
                    let mut fragments = HashSet::new();
                    self.response.generate_query(&mut out, &self.variables, &mut fragments)?;

                    while ! fragments.is_empty() {
                        fragments = Self::generate_fragments(&mut out, &self.variables, executable_document, &fragments, &mut done_fragments)?;
                        // let mut new_fragments = HashSet::new();
                        // for fragment_name in &fragments {
                        //     if ! done_fragments.contains(fragment_name) {
                        //         let fragment = executable_document.parsed.fragments.get(fragment_name).unwrap();

                        //         fragment.generate_query(&mut out, &self.variables, &mut new_fragments)?;
                        //         done_fragments.insert(fragment_name);
                        //     }
                        // }
                        // if new_fragments.is_empty() {
                        //     break;
                        // }
                        // fragments = new_fragments;
                    }
                    writeln!(out, "buf")?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;

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


            // let mut field_map = IndexMap::new();
            // self.selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

            // writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
            // // writeln!(out, "#[serde(rename = \"{}\")]", self.parsed.name)?;
            // writeln!(out, "pub struct Response {{")?;
            // {
            //     let mut out = out.indent();
        
            //     self.response.generate_fields(&mut out, selection_manager)?;
            // }
            // writeln!(out, "}}")?;
            // writeln!(out, "")?;

            writeln!(out, "impl NewGraphQLResponse for Response {{")?;
            writeln!(out, "}}")?;
            
            // self.selections.generate_structs(&mut out, context, schema, executable_document, &field_map, dependencies)?;

            // for (_name, field) in &self.variables {
            //     if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
            //         if ! dependencies.contains(defined_type.name()) {
            //             defined_type.generate(&mut out, schema, &None)?;
            //         }
            //     }
            // }

        }
        writeln!(out, "}}")?;
        Ok(())
    }
    
    fn generate_fragments(out: &mut Output<'_>, variables: &SharedMap<Field>, executable_document: &ExecutableDocument, fragments: &HashSet<Rc<String>>, done_fragments: &mut HashSet<Rc<String>>) -> Result<HashSet<Rc<String>>, Error> {
        let mut new_fragments = HashSet::new();
        for fragment_name in fragments {
            if ! done_fragments.contains(fragment_name) {
                let fragment = executable_document.parsed.fragments.get(fragment_name).unwrap();

                fragment.generate_query(out, variables, &mut new_fragments)?;
                done_fragments.insert(fragment_name.clone());
            }
        }

        Ok(new_fragments)
    }
    
    
    
    // pub fn gather_dependencies(&self, schema: &Rc<Schema>, executable_document: &ExecutableDocument) -> Result<HashSet<Atom>, Error> {
    //     let mut dependencies: HashSet<Atom> = HashSet::new();
    //     let context = *self.get_context( schema)?;
    //     let mut field_map = IndexMap::new();
        
    //     self.selections.gather_fields(&mut field_map, context, schema, executable_document, true)?;

    //     self.selections.gather_dependencies(schema, executable_document, context, &field_map, &mut dependencies)?;

    //     for (_name, field) in &self.variables {
    //         if let ScalarType::DefinedType(defined_type) = &field.ty.get_scalar() {
    //             defined_type.gather_dependencies(schema, &mut dependencies)?;
    //         }
    //     }

    //     Ok(dependencies)
    // }
}

fn get_field<'a>(schema: &'a Rc<Schema>, fields: &'a SharedMap<Field>, name: &String) -> Option<&'a Rc<Field>> {
    if name == TYPE_NAME {
        Some(&schema.__typename)
    }
    else {
        fields.get(name)
    }
}

// pub struct FragmentDefinition {
//     pub parsed: Rc<parsed_model::FragmentDefinition>,
//     pub selections: OldSelectionList, // TODO: can we delete this? in fact this whole class I think
// }

// impl FragmentDefinition {
//     pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::FragmentDefinition>, schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument) -> Result<Self, Error> {

//         if let Some(type_definition) = schema.defined_types.get(&parsed.type_condition) {
//             if let TypeDefinition::Object(object) = type_definition {
//                 let mut selections = OldSelectionList::new();
//                 for selection in &parsed.selections {
                    
//                     // selections.push(Selection::new(selection, schema, &schema.query.get(schema).fields, out));
//                     if let Ok(selection) = OldSelection::new(err, selection, schema, executable_document, object.as_ref()) {
//                         selections.selections.push(selection);
//                     }
//                 }
        
//                 if selections.selections.is_empty() {
//                     return err.fail(BuildError::InvalidQueryError(parsed.position.clone(), format!("FragmentDefinition {} has no selection set", &parsed.name)));
//                 }

//                 err.ok(FragmentDefinition {
//                     parsed: parsed.clone(),
//                     selections,
//                     // query_object,
//                 })
//             }
//             else {
//                 err.fail(BuildError::TypeMismatchError(parsed.position.clone(), format!("expected Object \"{}\" but found {}", &parsed.name, type_definition.type_name())))
//             }
//         }
//         else {
//             err.fail(BuildError::MissingObjectError(parsed.position.clone(), format!("Fragment {} has missing type condition {}", parsed.name, &parsed.type_condition)))
//         }
//     }

//     pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
//         writeln!(out, "Query {{")?;
//         {
//             let mut out = out.indent();

//             self.parsed.print(&mut out)?;
//             writeln!(out, "selections {{")?;
//             {
//                 let mut out = out.indent();

//                 for selection in &self.selections.selections {
//                     selection.print(&mut out)?;
//                 }
//             }
//             writeln!(out, "}}")?;
//         }
//         writeln!(out, "}}")
//     }
// }



pub struct FragmentSpread {
    pub parsed: Rc<parsed_model::FragmentSpread>,
}

impl FragmentSpread {
    fn new(err: &mut ErrorCollector<'_>, parsed: &Rc<parsed_model::FragmentSpread>, _schema: &Rc<Schema>, executable_document: &parsed_model::ExecutableDocument) -> Result<Self, Error> {
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
    
    // pub fn generate_query(&self, out: &mut Output, _context: &dyn Context, _schema: &Rc<Schema>, _executable_document: &ExecutableDocument, _maybe_optional: bool) -> Result<(), Error> {
    //     writeln!(out, "...{}", &self.parsed.name)?;
    //     Ok(())
    // }
}

#[derive(Debug)]
pub struct SelectionContext {
    pub imports: HashSet<usize>,
    pub schema_imports: HashSet<Atom>,
    pub names: IndexMap<Atom, usize>,
}

impl SelectionContext {
    pub fn new() -> Self {
        SelectionContext {
            imports: HashSet::new(),
            schema_imports: HashSet::new(),
            names: IndexMap::new(),
        }
    }
}

impl Print for SelectionContext {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "imports {:?}", self.imports)?;
        writeln!(out, "schema_imports {:?}", self.schema_imports)?;
        writeln!(out, "names {:?}", self.names)
    }
}


#[derive(Debug)]
pub enum NameSpaceItem {
    Selection(Rc<Selection>),
    Variant(Rc<Variant>),
}

impl NameSpaceItem {
    fn generate(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        match self {
            NameSpaceItem::Selection(selection) => selection.generate(out, selection_manager, schema),
            NameSpaceItem::Variant(variant) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct NameSpaceManager {
    pub items: Vec<NameSpaceItem>,
    // pub imports: IndexMap<Atom, IndexMap<Atom, HashSet<usize>>>,
    pub all_schema_imports: HashSet<Atom>,
    // pub schema_imports: IndexMap<Atom, IndexMap<Atom, HashSet<Atom>>>,
    pub contexts: IndexMap<Atom, IndexMap<Atom, SelectionContext>>,
    pub names: IndexMap<usize, Atom>,
    pub document_name: Option<Atom>,
    pub operation_name: Option<Atom>,
}

impl Print for NameSpaceManager {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionManager {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "all_schema_imports: {:?}", self.all_schema_imports)?;

            for s in &self.all_schema_imports {
                writeln!(out, "-{}", s)?;

            }
            // writeln!(out, "schema_imports: {:?}", self.schema_imports)?;
            // writeln!(out, "imports: {:?}", self.imports)?;

            writeln!(out, "contexts {{")?;
            {
                let mut out = out.indent();

                for (document, map) in &self.contexts {
                    writeln!(out, "{} {{", document)?;
                    {
                        let mut out = out.indent();

                        for (operation, context) in map {
                            writeln!(out, "{} {{", operation)?;
                            context.print(&mut out.indent())?;
                            writeln!(out, "}}")?;

                        }
                        
                    }
                        writeln!(out, "}}")?;
                }
            }
            writeln!(out, "}}")?;

            // // writeln!(out, "mutations {{")?;
            // // {
            // //     let mut out = out.indent();

            // //     for query in &self.mutations {
            // //         query.print(&mut out)?;
            // //     }
            // // }
            // // writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }
}

impl NameSpaceManager {
    pub fn new() -> NameSpaceManager {
        NameSpaceManager {
            items: Vec::new(),
            all_schema_imports: HashSet::new(),
            // imports: IndexMap::new(),
            // schema_imports: IndexMap::new(),
            contexts: IndexMap::new(),
            names: IndexMap::new(),
            document_name: None,
            operation_name: None,
        }
    }

    pub fn allocate_names(&mut self) {
        println!("Allocate Names");
        for (document, map) in self.contexts.iter_mut() {
            println!("Document {}", document);
            for (operation, context) in map {
                println!("operation {}", operation);
                for id in &context.imports {
                    if let Some(item) = self.items.get(*id) {
                        let selection_names = match item {
                            NameSpaceItem::Selection(content ) => &content.names,
                            NameSpaceItem::Variant(content ) => &content.names,
                        };


                        let default_name = Rc::new(format!("Type{}", id));
                        let mut name = &default_name;
                        for posible_name in selection_names {
                            if ! context.names.contains_key(posible_name) {
                                name = posible_name;
                                break;
                            }
                        }
                        println!("allocate_name({}) = {}", id, name);
                        context.names.insert(name.clone(), *id);
                        println!("context.names = {:?}", &context.names);
                        self.names.insert(*id, name.clone());
                    }
                    else {
                        panic!("Failed to find selection {}", id);
                    }
                }
            }
       }
    }

    pub fn create_document(&mut self, document_name: Atom) {
        println!("create_document({})", &document_name);
        // self.imports.insert(document_name.clone(), IndexMap::new());
        // self.schema_imports.insert(document_name.clone(), IndexMap::new());
        self.contexts.insert(document_name.clone(), IndexMap::new());
        self.document_name = Some(document_name);
    }

    pub fn create_operation(&mut self, operation_name: Atom) {
        println!("create_operation({})", &operation_name);
        // if let Some(document_name) = &self.document_name {
        //     if let Some(map) = self.imports.get_mut(document_name) {
        //         map.insert(operation_name.clone(), HashSet::new());

        //         if let Some(map) = self.schema_imports.get_mut(document_name) {
        //             map.insert(operation_name.clone(), HashSet::new());
        //             self.operation_name = Some(operation_name);
        //             return;
        //         }
        //     }
        // }
        if let Some(document_name) = &self.document_name {
            if let Some(map) = self.contexts.get_mut(document_name) {
                map.insert(operation_name.clone(), SelectionContext::new());
                self.operation_name = Some(operation_name);
                return;
            }
        }
        panic!("Failed to create_operation?");
    }

    pub fn set_document(&mut self, document_name: Atom) {
        println!("set_document({})", &document_name);
        self.document_name = Some(document_name);
    }

    pub fn set_operation(&mut self, operation_name: Atom) {
        println!("set_operation({})", &operation_name);
        self.operation_name = Some(operation_name);
    }

    fn get_context_mut(&mut self) -> &mut SelectionContext {
        if let Some(document_name) = &self.document_name {
            if let Some(map) = self.contexts.get_mut(document_name) {
                if let Some(operation_name) = &self.operation_name {
                    if let Some(context) = map.get_mut(operation_name) {
                        return context;
                    }
                }
            }
        }
        panic!("Failed to set_document?");
    }
    pub fn get_context(&self) -> &SelectionContext {
        if let Some(document_name) = &self.document_name {
            if let Some(map) = self.contexts.get(document_name) {
                let xm = format!("{:?}", map);
                if let Some(operation_name) = &self.operation_name {
                    if let Some(context) = map.get(operation_name) {
                        let xc = format!("{:?}", context);
                        return context;
                    }
                }
            }
        }
        panic!("Failed to set_document?");
    }

    fn import_schema(&mut self, name: Atom) {
        self.get_context_mut().schema_imports.insert(name.clone());
        self.all_schema_imports.insert(name);

        
        // if let Some(document_name) = &self.document_name {
        //     if let Some(map) = self.schema_imports.get_mut(document_name) {
        //         if let Some(operation_name) = &self.operation_name {
        //             if let Some(set) = map.get_mut(operation_name) {
        //                 println!("ZZ insert {}", name);
        //                 println!("ZZ set={:?}", set);
        //                 println!("ZZall_schema_imports={:?}", self.all_schema_imports);
        //                 set.insert(name.clone());
        //                 self.all_schema_imports.insert(name);
        //                 return;
        //             }
        //         }
        //     }
        // }
        // panic!("Failed to set_document?");
    }

    fn insert(&mut self, graphql_type_name: Atom, parsed: Rc<parsed_model::SelectionList>, names: Vec<Rc<String>>, fields: IndexMap<Atom, SelectionField>, p_variants: IndexMap<Atom, (Atom, IndexMap<Atom, SelectionField>)>, interface: Option<Rc<Interface>>) -> Rc<Selection> {
        
        
        println!("insert {:?}", &names);

        let mut variants: Vec<Rc<Variant>> = Vec::new();
        for (name, (type_condition, fields)) in p_variants {
            let index = self.items.len();

            let variant = Rc::new(Variant{
                index,
                names: vec!(name.clone()),
                fields,
                type_condition,
            });

            
            self.items.push( NameSpaceItem::Variant(variant.clone()));
            self.get_context_mut().imports.insert(index);

            variants.push(variant);
        }


        let selection_index = self.items.len();

        let content = Rc::new(Selection {
            index: selection_index,
            graphql_type_name,
            parsed,
            names,
            fields,
            variants,
            interface,
        });

        let result = content.clone();

        self.items.push( NameSpaceItem::Selection(content));
        self.get_context_mut().imports.insert(selection_index);
        

        result

        // let check = self.items.get(index);

        // if let Some(check) = check {
        //     if let NameSpaceItem::Selection { id, content } = check {
        //         if content.index == index {
        //             let result = check.clone();
        //             self.get_context_mut().imports.insert(index);

        //             return result
        //         }
        //         else {
        //             panic!("SelectionManager index error expected {} got {}", index, check.index);
        //         }

        //         // if let Some(document_name) = &self.document_name {
        //         //     if let Some(map) = self.imports.get_mut(document_name) {
        //         //         if let Some(operation_name) = &self.operation_name {
        //         //             if let Some(set) = map.get_mut(operation_name) {
        //         //                 set.insert(index);
                            
        //         //                 return check.clone()
        //         //             }
        //         //         }
        //         //     }
        //         // }
        //         // panic!("Failed to set_document?");
        //     }
        //     else {
        //         panic!("SelectionManager index error expected {} got {}", index, check.index);
        //     }
        // }
        // else  {
        //     panic!("SelectionManager index error expected {} got None", index);
        // }
    }
    
    fn get_schema_dependencies(&self) -> &HashSet<Atom> {
        &self.get_context().schema_imports

        // if let Some(document_name) = &self.document_name {
        //     if let Some(map) = self.schema_imports.get(document_name) {
        //         if let Some(operation_name) = &self.operation_name {
        //             if let Some(set) = map.get(operation_name) {
        //                 return &set
        //             }
        //         }
        //     }
        // }
        // panic!("Failed to set_document?");
    }
    
    fn get_name(&self, id: &usize) -> &Atom {
        println!("get_name({}) = {:?}", id, self.names.get(id));
        self.names.get(id).unwrap()
    }
    
    fn get(&self, id: usize) -> &NameSpaceItem {
        if let Some(item) = self.items.get(id) {
            // match item {
            //     NameSpaceItem::Selection(content ) => content,
            //     NameSpaceItem::Variant(_) => panic!("Expecting selection {} but found Variant", id),
            // }
            item
        }
        else {
            panic!("Cant find selection {}", id);
        }
    }
    
    // pub fn insert(&mut self, type_name: Atom, alias: Option<Atom>, selections: &parsed_model::SelectionList) -> Result<usize, Error> {
    //     let field_builder = FieldMap::builder();
    //     let variant_builder = VariantMap::builder();

    //     self.selections.push(Selection {
    //         type_name,
    //         alias,
    //         fields: field_builder.build(),
    //         variants: variant_builder.build(),
    //     });

    //     Ok(self.selections.len())
    // }

    
}

pub struct ExecutableDocument {
    pub parsed: Rc<parsed_model::ExecutableDocument>,
    // pub fragments: IndexMap<Atom, FragmentDefinition>,
    pub operations: Vec<GenericOperation>,
    // pub mutations: Vec<GenericOperation>,
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector, registry: &mut Registry,  selection_manager: &mut NameSpaceManager, parsed: &Rc<parsed_model::ExecutableDocument>, schema: &Rc<Schema>) ->  Result<Self, Error> {

        // let mut fragments: IndexMap<Rc<String>, FragmentDefinition> = IndexMap::new();

        // for fragment_definition in parsed.fragments.values() {
        //     fragments.insert(fragment_definition.name.clone(), FragmentDefinition::new(err, fragment_definition, schema, &parsed)?);
        // }

        

        let mut operations = Vec::new();

        for query in &parsed.queries {
            selection_manager.create_operation(query.name.clone());
            operations.push(GenericOperation::new(err, registry, selection_manager, query, schema, &parsed.fragments)?);
        }

        // let mut mutations = Vec::new();

        for mutation in &parsed.mutations {
            selection_manager.create_operation(mutation.name.clone());
            operations.push(GenericOperation::new(err, registry, selection_manager, mutation, schema, &parsed.fragments)?);
        }

        Ok(ExecutableDocument {
            parsed: parsed.clone(),
            // fragments,
            operations,
            // mutations,
        })
    }

    // pub fn get_fragment(&self, name: &Atom) -> Result<&FragmentDefinition, Error> {
    //     match self.fragments.get(name) {
    //         Some(fragment_definition) => {
    //             Ok(fragment_definition)
    //         },
    //         None => Err(Error::BuildFailed(format!("Failed to find Fragment \"{}\"", name))),
    //     }
    // }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "ExecutableDocument {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
            // writeln!(out, "fragments {{")?;
            // {
            //     let mut out = out.indent();

            //     for item in self.fragments.values() {
            //         item.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;

            writeln!(out, "operations {{")?;
            {
                let mut out = out.indent();

                for operation in &self.operations {
                    operation.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            // writeln!(out, "mutations {{")?;
            // {
            //     let mut out = out.indent();

            //     for query in &self.mutations {
            //         query.print(&mut out)?;
            //     }
            // }
            // writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

    // fn operation_dependencies<'a>(dependencies: &'a IndexMap<Atom, HashSet<Atom>>, operation: &GenericOperation) -> Result<&'a HashSet<Rc<std::string::String>>, Error> {
    //     if let Some(d) = dependencies.get(&operation.parsed.name) {
    //         Ok(d)
    //     }
    //     else {
    //         Err(Error::BuildFailed(format!("Unable to fined dependencies for operation {}", operation.parsed.name)))
    //     }
    // }

    pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>, selection_manager: &mut NameSpaceManager) -> Result<(), Error> {
       writeln!(out, "pub mod {} {{", self.parsed.name)?;
        {
            let mut out = out.indent();

            writeln!(out, "// Start dependencies")?;
            // we dont have document level context yet
            writeln!(out, "// End dependencies")?;

            for query in &self.operations {
                selection_manager.set_operation(query.parsed.name.clone());
                query.generate(&mut out, schema, self, selection_manager)?;
            }

            // for mutation in &self.mutations {
            //     mutation.generate(&mut out, schema, self, Self::operation_dependencies(dependencies, mutation)?)?;
            // }
        }

        // let mut new_type_set: HashSet<Atom> = HashSet::new();

        // for name in type_set {
        //     writeln!(out, "// type {}", name)?;
        //     if let Some(defined_type) = schema.defined_types.get(&name) {
        //         defined_type.generate(out, schema, self, None, &None)?;
        //     }

            
        // }
        writeln!(out, "}} // End of executable_document {}", self.parsed.name)?;
        Ok(())
    }
    
    // pub fn gather_dependencies(&self, schema: &Rc<Schema>) -> Result<IndexMap<Atom, HashSet<Atom>>, Error> {
    //     let mut dependencies: IndexMap<Atom, HashSet<Rc<String>>> = IndexMap::new();

    //     for query in &self.queries {
    //         dependencies.insert(query.parsed.name.clone(), query.gather_dependencies(schema, self)?);
    //     }

    //     for mutation in &self.mutations {
    //         dependencies.insert(mutation.parsed.name.clone(), mutation.gather_dependencies(schema, self)?);
    //     }

    //     Ok(dependencies)
    // }
}