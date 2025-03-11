use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::Write;
use std::rc::Rc;

use graphql_parser::Pos;
use indexmap::{IndexMap, IndexSet};
use inflections::case::to_snake_case;

use crate::parsed_model::{BuiltinType, DefinedTypeName, OperationType};
use crate::utils::to_pascal_case;
use crate::{Name, BuildError, Error, ErrorCollector, Isomorphic, Print, NameRegistry, TYPE_NAME};
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

pub type FieldMap = SharedMap<Field>;

pub type RawSharedMap<T> = IndexMap<Name, Rc<T>>;

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
    
    pub fn values(&self) -> indexmap::map::Values<'_, Name, Rc<T>> {
        self.map.values()
    }
    
    pub fn get(&self, name: &String) -> Option<&Rc<T>> {
        self.map.get(name)
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    pub fn keys(&self) -> indexmap::map::Keys<'_, Name, Rc<T>> {
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

    pub fn insert(&mut self, key: Name, value: Rc<T>) -> Option<Rc<T>> {
        self.map.insert(key, value)
    }

    pub fn build(self) -> SharedMap<T> {
        SharedMap {
            map: Rc::new(self.map),
        }
    }
}
#[derive(Debug)]
pub struct Selection {
    pub index: usize,
    pub graphql_type_name: Name,
    // pub parsed: Rc<parsed_model::SelectionList>,
    pub name: Name,
    pub fields: IndexMap<Name, SelectionField>,
    pub variants: Vec<Rc<Variant>>,
    // pub interface: Option<Rc<Interface>>,
    pub implemented_by: Option<Vec<Name>>,
}

impl Print for Selection {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "fields {{")?;
        {
            let mut out = out.indent();

            for field in self.fields.values() {
                field.print(&mut out)?;
            }
        }
        writeln!(out, "}}")?;

        writeln!(out, "variants {{")?;
        {
            let mut out = out.indent();

            for variant in &self.variants {
                writeln!(out, "{:?} {{", variant.name)?;
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
    fn generate_fields(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        for field in self.fields.values() {
            field.generate_field(out, selection_manager, schema)?;
        }

        Ok(())
    }
    
    fn generate(&self, out: &mut Output<'_>, namespace_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        let name = namespace_manager.get_name(&self.index);

        if let Some(implemented_by) = &self.implemented_by {
            writeln!(out, "// implemented_by {:?}", implemented_by)?;
        }
        
        match &self.implemented_by {
            None => {
                writeln!(out, "/* No variants */")?;
                
                writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                if *name != self.graphql_type_name {
                    writeln!(out, "#[serde(rename = \"{}\")]", self.graphql_type_name)?;
                }
                writeln!(out, "pub struct {} {{", name)?;
        
                {
                    let mut out = out.indent();
                    
                    self.generate_fields(&mut out, namespace_manager, schema)?;
                    for variant_selection in &self.variants {
                        writeln!(out, "// variant {:?}", variant_selection.name)?;
                        for field in variant_selection.fields.values() {
                            field.generate_field(&mut out, namespace_manager, schema)?;
                        }
                    }
                }
        
                writeln!(out, "}}")?;
                writeln!(out, "")?;
            },
            Some(implemented_by) => {
                if self.variants.len() < 2 && implemented_by.len() < 2 {
                    writeln!(out, "/* <2 variant */")?;
                    
                    writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                    if *name != self.graphql_type_name {
                        writeln!(out, "#[serde(rename = \"{}\")]", self.graphql_type_name)?;
                    }
                    writeln!(out, "pub struct {} {{", name)?;
            
                    {
                        let mut out = out.indent();
                        
                        self.generate_fields(&mut out, namespace_manager, schema)?;
                        for variant_selection in &self.variants {
                            for field in variant_selection.fields.values() {
                                field.generate_field(&mut out, namespace_manager, schema)?;
                            }
                        }
        
                    }
            
                    writeln!(out, "}}")?;
                    writeln!(out, "")?;
                }
                else 
                {
                    writeln!(out, "/* {} variants {} implementors */", self.variants.len(), implemented_by.len())?;
        
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

                    for implementor in implemented_by {
                        let enum_variant_name = to_pascal_case(&implementor);

                        if let Some(variant) = variant_map.get(implementor) {
                            let name = namespace_manager.get_name(&variant.index);
                            writeln!(out, "    {}({}),", enum_variant_name, name)?;
                        }
                        else {
                            writeln!(out, "    {}({}),", enum_variant_name, abstract_name)?;
                        }
                    }
            
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

                                    for implementor in implemented_by {
                                        let enum_variant_name = to_pascal_case(&implementor);
                
                                        if let Some(_) = variant_map.get(implementor) {
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
                    writeln!(out, "pub struct {} {{", abstract_name)?;
            
                    {
                        let mut out = out.indent();
                        
                        self.generate_fields(&mut out, namespace_manager, schema)?;
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
                        let variant_name = namespace_manager.get_name(&variant.index);
                        let rust_variant_name = to_pascal_case(&variant_name);
        
                        writeln!(out, "// Variant names {:?}", variant.name)?;
                        writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
                        writeln!(out, "#[serde(rename = \"{}\")]", variant_name)?;
                        writeln!(out, "pub struct {} {{", rust_variant_name)?;
                
                        {
                            let mut out = out.indent();
        
                            if ! self.fields.is_empty() {
                                writeln!(out, "#[serde(flatten)]")?;
                                writeln!(out, "pub {}_: {},", base_member_name, abstract_name)?;
                            }
                            
                            for (_name, field) in &variant.fields {
                                field.generate_field(&mut out, namespace_manager, schema)?;
                            }
                        }
                
                        writeln!(out, "}}")?;
                        writeln!(out, "")?;
        
                        if ! self.fields.is_empty() {
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
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Variant {
    pub index: usize,
    pub name: Name,
    pub fields: IndexMap<Name, SelectionField>,
    pub type_condition: Name,
}

pub struct SelectionBuilder {
    pub graphql_type_name: Name,
    pub preferred_type_name: Name,
    pub fields: IndexMap<Name, SelectionField>,
    pub variants: IndexMap<Name, (Name, IndexMap<Name, SelectionField>)>,
    pub implemented_by: Option<Vec<Name>>,
}

impl SelectionBuilder {
    pub fn new(graphql_type_name: Name, preferred_type_name: Name, implemented_by: &Option<Vec<Name>>) -> SelectionBuilder {
        // println!("SelectionBuilder new");
        SelectionBuilder {
            graphql_type_name,
            preferred_type_name,
            fields: IndexMap::new(),
            variants: IndexMap::new(),
            implemented_by: implemented_by.clone(),
        }
    }

    pub fn with_field(&mut self, variant_name: &Option<VariantID>, field_name: &Name, field: SelectionField) -> &mut Self {
        println!("SelectionBuilder field {}", field_name);
        if let Some(variant_id) = variant_name {
            for variant_type_condition in &variant_id.type_conditions {
                println!("variant_type_condition={}", &variant_type_condition);
                if let Some((names, variant)) = self.variants.get_mut(variant_type_condition) {
                    variant.insert(field_name.clone(), field.clone());
                    // names.insert(variant_name.clone());
                }
                else {
                    println!("insert variant_type_condition {}", &variant_type_condition);
                    let mut fields = IndexMap::new();
                    fields.insert(field_name.clone(), field.clone());
                    // let mut names = IndexSet::new();
                    // names.insert(variant_name.clone());
                    // self.variants.insert(variant_type_condition.clone(), (variant_id.fragment_name.clone(), fields));
                    self.variants.insert(variant_type_condition.clone(), (variant_type_condition.clone(), fields));
                }
            }
        }
        else {
            self.fields.insert(field_name.clone(), field); 
        }
        self
    }

    // pub fn with_preferred_type_name(&mut self, name: Name) -> &mut Self {
    //     self.preferred_type_name = Some(name);

    //     self
    // }

    pub fn build(self, manager: &mut NameSpaceManager) -> SelectionFieldType {

        // println!("SelectionBuilder build {}", self.graphql_type_name);
        // for name in self.fields.keys() {
        //     println!("    {}", name);
        // }

        // for (name, (_cond, variant)) in &self.variants {
        //     println!("  variant {}", name);
        //     for name in variant.keys() {
        //         println!("    {}", name);
        //     }
        // }

        // if self.graphql_type_name.as_ref() == "PageInfo" {
        //     println!("HERE");
        //     println!("fields {:?}", self.fields);
        //     println!("variants {:?}", self.variants);
        // }
        // Recognise pagination

        let mut has_cursor = false;
        let mut cantbe_page_of = false;
        let mut cantbe_edge_of = false;
        let mut cantbe_page_info = false;
        let mut page_info_fields = HashSet::new();
        let mut page_info_type = None;
        let mut edge_of_type = None;
        let mut page_of_type = None;
        
        // let keys = if self.variants.is_empty() { self.fields.keys() } else {
        //     let (a,b) = self.variants.values().next().unwrap();
        //     b.keys()
        // };

        let mut meta_map = Vec::new();

        meta_map.push(&self.fields);

        for (_n, v) in self.variants.values() {
            meta_map.push(v);
        }

        for map in meta_map {
            for (name, field) in map {
                let name = name.as_ref() as &str;

                if name == TYPE_NAME {
                    continue;
                }

                if name == "pageInfo" {
                    page_info_type = Some(SelectionFieldType::remove_wrapper(&field.selection_type));
                    cantbe_edge_of = true;
                    cantbe_page_info=true;
                }
                else if name == "edges" {
                    page_of_type = Some(SelectionFieldType::remove_wrapper(&field.selection_type));
                    cantbe_edge_of = true;
                    cantbe_page_info=true;
                }
                else if name == "cursor" {
                    has_cursor = true;
                    cantbe_page_of = true;
                    cantbe_page_info=true;
                }
                else if name == "node" {
                    edge_of_type = Some(SelectionFieldType::remove_wrapper(&field.selection_type));
                    cantbe_page_of = true;
                    cantbe_page_info=true;
                }
                else if name == "startCursor" || name == "endCursor" || name == "hasPreviousPage" || name == "hasNextPage" {
                    page_info_fields.insert(name);
                    cantbe_page_of = true;
                    cantbe_edge_of=true;
                }
                else {
                    cantbe_page_of = true;
                    cantbe_edge_of = true;
                    cantbe_page_info=true;
                }
            }
        }

        if cantbe_page_of==false && page_of_type.is_some() {
            if let Some(page_type) =  page_of_type {
                if let SelectionFieldType::EdgeOf(selection_field_type, _) = page_type.as_ref() {
                    if let Some(page_info) = page_info_type {
                        match page_info.as_ref() {
                            SelectionFieldType::PageInfo => {
                                println!("!! Its PageOf<{:?}>", &selection_field_type);
                                return SelectionFieldType::PageOf(selection_field_type.clone())
                            },
                            SelectionFieldType::ForwardPageInfo => {
                                println!("!! Its ForwardPageOf<{:?}>", &selection_field_type);
                                return SelectionFieldType::ForwardPageOf(selection_field_type.clone())
                            },
                            SelectionFieldType::ReversePageInfo => {
                                println!("!! Its ReversePageOf<{:?}>", &selection_field_type);
                                return SelectionFieldType::ReversePageOf(selection_field_type.clone())
                            },
                            _ => {}
                        }
                            
                    }
                }
            }
           
        }

        if cantbe_page_info==false && page_info_fields.len() > 1 {
            let maybe_forward_page = page_info_fields.contains("hasNextPage") && page_info_fields.contains("endCursor");
            let maybe_reverse_page = page_info_fields.contains("hasPreviousPage") && page_info_fields.contains("startCursor");

            if maybe_forward_page && maybe_reverse_page {
                println!("!! Its PageInfo");
                return SelectionFieldType::PageInfo;
            }
            else if maybe_forward_page {
                println!("!! Its ForwardPageInfo");
                return SelectionFieldType::ForwardPageInfo;
            }
            else if maybe_reverse_page {
                println!("!! Its ReversePageInfo");
                return SelectionFieldType::ReversePageInfo;
            }
        }

        if cantbe_edge_of==false && has_cursor && edge_of_type.is_some() {
            println!("!! ItsEdgeOf<{:?}>", edge_of_type.as_ref());
            return SelectionFieldType::EdgeOf(edge_of_type.unwrap(), SelectionCreationParams::new(self.graphql_type_name, self.preferred_type_name, self.fields, self.variants, self.implemented_by))
        }

        // check for unused pagination types:

        for field in self.fields.values() {
            if field.selection_type.is_pagination_internal() {
                println!("\n\nUNUSED    {}", &field.name);
            }
           
        }

        for (_cond, variant) in self.variants.values() {
            for field in variant.values() {
                if field.selection_type.is_pagination_internal() {
                    println!("\n\nUNUSED    {}", &field.name);
                }
            }
        }

        let selection = manager.insert(self.graphql_type_name, self.preferred_type_name, self.fields, self.variants, self.implemented_by);

        SelectionFieldType::Selection(selection.index)
    }
}

#[derive(Debug)]
pub enum SelectionFieldType {
    BuiltinType(BuiltinType),
    Scalar(Name),
    Enum(Name),
    Selection(usize),
    Required(Rc<SelectionFieldType>),
    Array(Rc<SelectionFieldType>),
    EdgeOf(Rc<SelectionFieldType>, SelectionCreationParams),
    PageInfo,
    ForwardPageInfo,
    ReversePageInfo,
    PageOf(Rc<SelectionFieldType>),
    ForwardPageOf(Rc<SelectionFieldType>),
    ReversePageOf(Rc<SelectionFieldType>),
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
            SelectionFieldType::EdgeOf(wrapped, _) => write!(f, "EdgeOf<{}>", wrapped),
            SelectionFieldType::PageInfo => write!(f, "PageInfo"),
            SelectionFieldType::ForwardPageInfo => write!(f, "ForwardPageInfo"),
            SelectionFieldType::ReversePageInfo => write!(f, "ReversePageInfo"),
            SelectionFieldType::PageOf(wrapped) => write!(f, "PageOf<{}>", wrapped),
            SelectionFieldType::ForwardPageOf(wrapped) => write!(f, "ForwardPageOf<{}>", wrapped),
            SelectionFieldType::ReversePageOf(wrapped) => write!(f, "ReversePageOf<{}>", wrapped),
        }
    }
}

impl SelectionFieldType {
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
                SelectionFieldType::EdgeOf(wrapped, _) => format!("sparko_graphql::types::EdgeOf<{}>", wrapped.rust_type(selection_manager, schema, nonnull)),
                SelectionFieldType::PageInfo => format!("sparko_graphql::types::PageInfo"),
                SelectionFieldType::ForwardPageInfo => format!("sparko_graphql::types::ForwardPageInfo"),
                SelectionFieldType::ReversePageInfo => format!("sparko_graphql::types::ReversePageInfo"),
                SelectionFieldType::PageOf(wrapped) => format!("sparko_graphql::types::PageOf<{}>", wrapped.rust_type(selection_manager, schema, nonnull)),
                SelectionFieldType::ForwardPageOf(wrapped) => format!("sparko_graphql::types::ForwardPageOf<{}>", wrapped.rust_type(selection_manager, schema, nonnull)),
                SelectionFieldType::ReversePageOf(wrapped) => format!("sparko_graphql::types::ReversePageOf<{}>", wrapped.rust_type(selection_manager, schema, nonnull)),
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
    
    fn remove_wrapper(this: &Rc<SelectionFieldType>) -> Rc<SelectionFieldType> {
        match this.as_ref() {
            SelectionFieldType::Required(wrapped) => Self::remove_wrapper(wrapped),
            SelectionFieldType::Array(wrapped) => Self::remove_wrapper(wrapped),
            _ => this.clone()
        }
    }
    
    fn is_pagination_internal(&self) -> bool {
        match self {
            SelectionFieldType::EdgeOf(_,_) => true,
            SelectionFieldType::PageInfo => true,
            SelectionFieldType::ForwardPageInfo => true,
            SelectionFieldType::ReversePageInfo => true,
            SelectionFieldType::BuiltinType(_) => false,
            SelectionFieldType::Scalar(_) => false,
            SelectionFieldType::Enum(_) => false,
            SelectionFieldType::Selection(_) => false,
            SelectionFieldType::Required(_) => false,
            SelectionFieldType::Array(_) => false,
            SelectionFieldType::PageOf(_) => false,
            SelectionFieldType::ForwardPageOf(_) => false,
            SelectionFieldType::ReversePageOf(_) => false,
        }
    }


    // pub fn is_array(&self) -> bool {
    //     match self {
    //         SelectionFieldType::BuiltinType(_) => false,
    //         SelectionFieldType::Scalar(_) => false,
    //         SelectionFieldType::Enum(_) => false,
    //         SelectionFieldType::Selection(_) => false,
    //         SelectionFieldType::Required(selection_field_type) => selection_field_type.is_array(),
    //         SelectionFieldType::Array(_) => true,
    //     }
    // }
}


#[derive(Debug, Clone)]
pub struct SelectionField {
    pub name: Name,
    pub selection_type: Rc<SelectionFieldType>,
    pub force_nonnull: bool,
}
impl SelectionField {
    fn generate_field(&self, out: &mut Output, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), std::io::Error> {
        if *self.name != TYPE_NAME {
            let field_name = to_snake_case(&self.name);

            writeln!(out, "#[serde(rename = \"{}\")]", &self.name)?;

            // if self.selection_type.is_array() {
            //     writeln!(out, "#[serde(skip_serializing)]")?;
            // }
            writeln!(out, "pub {}_: {}, // T1", field_name, self.selection_type.rust_type(selection_manager, schema, self.force_nonnull))
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
    Scalar(Name),
    Object(Name),
    Interface(Name),
    Union(Name),
    Enum(Name),
    InputObject(Name),
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
    }
    
    pub fn is_optional(&self) -> bool {
        if let Type::Required(_) = self {
            false
        }
        else {
            true
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


pub enum TypeDefinition {
    Scalar(Rc<Scalar>),
    Object(Rc<Object>),
    Interface(Rc<Interface>),
    Union(Rc<Union>),
    Enum(Rc<Enum>),
    InputObject(Rc<Object>),
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

    pub fn rust_name(&self) -> &Name {
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
    pub rust_name: Name,
    pub rust_type: Name,
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
    
    fn new(registry: &mut NameRegistry, parsed: &Rc<parsed_model::Scalar>, types: &HashMap<String, String>) -> TypeDefinition {
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
        writeln!(out, "type {} = {};", &self.rust_name, &self.rust_type)?;
        writeln!(out, "")?;
        Ok(())
    }
}

pub struct Object {
    pub name: Name,
    pub rust_name: Name,
    pub fully_implements: Vec<Name>,
    pub fields: FieldMap,
    pub is_input: bool,
}

impl Context for Object {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
}

impl Object {
    fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, parsed: &Rc<parsed_model::Object>, schema: &Rc<parsed_model::Schema>,) -> Result<Rc<Self>, Error> {
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
                name: parsed.name.clone(),
                rust_name: registry.intern(to_pascal_case(&parsed.name)),
                fully_implements,
                fields: builder.build(),
                is_input: parsed.is_input,
            }))
    }
    

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "validated_model::Object {{")?;
        {
            let mut out = out.indent();

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
}

fn generate_struct(out: &mut Output<'_>, name: &Name, rust_name: &Name, context: &dyn Context, is_input: bool) -> Result<(), Error> {
   

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

#[derive(Debug)]
pub struct Interface {
    pub parsed: Rc<parsed_model::Interface>,
    pub rust_name: Name,
    pub implemented_by: Vec<Name>,
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

    fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, parsed: &Rc<parsed_model::Interface>, schema: &Rc<parsed_model::Schema>,) -> Result<TypeDefinition, Error> {
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
}


pub struct Union {
    pub parsed: Rc<parsed_model::Union>,
    pub rust_name: Name,
    pub fields: FieldMap,
}

impl Context for Union {
    fn fields(&self) -> &FieldMap {
        &self.fields
    }
}

impl Union {
    fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, parsed: &Rc<parsed_model::Union>) -> Result<TypeDefinition, Error> {
        let builder = FieldMap::builder();
        let err = err.child();

        // for type_name in &parsed.types {
        //     if let Some(object) = schema.get_object(&mut err, &parsed.position, type_name) {
        //         for (name, field) in &object.fields {
        //             if let Ok(field) = Field::new(&mut err, field, schema) {
        //                 builder.insert(name.clone(), field);
        //             }
        //         }
        //     }
        //     else {
        //         err.error(BuildError::MissingInterfaceError(parsed.position.clone(), format!("Interface {} Not Found", parsed.name)))
        //     };
        // }

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
}


pub struct Enum {
    pub parsed: Rc<parsed_model::Enum>,
    pub rust_name: Name,
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

    pub fn new(registry: &mut NameRegistry, parsed: &Rc<parsed_model::Enum>) -> TypeDefinition {
        TypeDefinition::Enum(Rc::new(Enum {
            parsed: parsed.clone(),
            rust_name: registry.intern(to_pascal_case(&parsed.name)),
        }))
    }
    
    fn generate(&self, out: &mut Output<'_>) -> Result<(), Error> {
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
    pub defined_types: IndexMap<Name, TypeDefinition>,
    pub query: Name,
    pub mutation: Option<Name>,
    pub subscription: Option<Name>,
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

    // pub fn get_scalar(&self, name: &Name) -> Result<&Rc<Scalar>, Error> {
    //     if let Some(type_definition) = &self.defined_types.get(name) {
    //         if let TypeDefinition::Scalar(type_definition) = type_definition {
    //             Ok(type_definition)
    //         }
    //         else {
    //             Err(Error::BuildFailed(format!("Expected Scalar for \"{}\" but found {}", name, type_definition.type_name())))
    //         }
    //     }
    //     else {
    //         Err(Error::BuildFailed(format!("Failed to find Scalar \"{}\"", name)))
    //     }
    // }

    pub fn get_object_or_interface_fields(&self, name: &Name) -> Result<&FieldMap, Error> {
        if let Some(type_definition) = &self.defined_types.get(name) {
            //if let TypeDefinition::Object(object) = 
            match type_definition {
                TypeDefinition::Object(object) => Ok(&object.fields),
                TypeDefinition::Interface(interface) => Ok(&interface.fields),
                _ => {
                    Err(Error::BuildFailed(format!("Expected Object or Interface for \"{}\" but found {}", name, type_definition.type_name())))
                },
            } 
        }
        else {
            Err(Error::BuildFailed(format!("Failed to find Object or Interface \"{}\"", name)))
        }
    }

    pub fn get_object(&self, name: &Name) -> Result<&Rc<Object>, Error> {
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

    pub fn get_interface(&self, name: &Name) -> Result<&Rc<Interface>, Error> {
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
    
    fn get_union(&self, name: &Name) -> Result<&Rc<Union>, Error> {
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
    
    // fn get_enum(&self, name: &Name) -> Result<&Rc<Enum>, Error> {
    //     if let Some(type_definition) = &self.defined_types.get(name) {
    //         if let TypeDefinition::Enum(type_definition) = type_definition {
    //             Ok(type_definition)
    //         }
    //         else {
    //             Err(Error::BuildFailed(format!("Expected Enum for \"{}\" but found {}", name, type_definition.type_name())))
    //         }
    //     }
    //     else {
    //         Err(Error::BuildFailed(format!("Failed to find Enum \"{}\"", name)))
    //     }
    // }

    pub fn get_input_object(&self, name: &Name) -> Result<&Rc<Object>, Error> {
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

    fn new_get_object(out: &mut ErrorCollector, name: &Option<Name>, position: &Pos, defined_types: &IndexMap<Name, TypeDefinition>) -> Option<Name> {
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

    fn new_get_default_object(out: &mut ErrorCollector, name: Name, defined_types: &IndexMap<Name, TypeDefinition>, missing_error: Option<BuildError>) -> Option<Name> {
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

    pub fn new(err: &mut ErrorCollector, parsed: &Rc<parsed_model::Schema>, registry: &mut NameRegistry, types: &HashMap<String, String>) -> Result<Rc<Self>, Error> {

        let mut defined_types: IndexMap<Name, TypeDefinition> = IndexMap::new();

        for parsed_type in parsed.named_types.values() {
            let defined_type = match parsed_type {
                parsed_model::TypeDefinition::Scalar(type_def) => Scalar::new(registry, &type_def, types),
                parsed_model::TypeDefinition::Object(object) => TypeDefinition::Object(Object::new(err, registry, &object, &parsed)?),
                parsed_model::TypeDefinition::Interface(interface) => Interface::new(err, registry, interface, &parsed)?,
                parsed_model::TypeDefinition::Union(union) => Union::new(err, registry, union)?,
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

#[derive(Debug)]
pub struct SelectionQueryField {
    pub name: Name,
    pub alias: Option<Name>,
    pub position: Pos,
    pub force_nonnull: bool,
    pub arguments: Vec<Rc<parsed_model::Argument>>,
    pub selections: Rc<SelectionQueryList>,
}

impl SelectionQueryField {
    pub fn new(selection_field: &Rc<parsed_model::SelectionField>, registry: &mut NameRegistry, schema: &Rc<Schema>, fields: &FieldMap) -> Result<SelectionQuery, Error> {
println!("SelectionQueryField {}", selection_field.name);

        Ok(SelectionQuery::Field(Rc::new(SelectionQueryField {
            name: selection_field.name.clone(),
            alias: selection_field.alias.clone(),
            position: selection_field.position.clone(),
            force_nonnull: selection_field.force_nonnull,
            arguments: selection_field.arguments.clone(),
            selections: Rc::new(SelectionQueryList::new(&selection_field.selections, registry, schema, fields)?),
        })))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionQueryField {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
            writeln!(out, "force_nonnull:  {}", self.force_nonnull)?;
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

        pub fn generate_query(&self, out: &mut Output, variables: &crate::validated_model::FieldMap, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
            if let Some(alias) = &self.alias {
                writeln!(out, "buf.push_str(\"{}: {}\\n\");", alias, &self.name)?;
            }
            else {
                writeln!(out, "buf.push_str(\"{}\\n\");", &self.name)?;
            }
    
            if ! self.arguments.is_empty() {
                writeln!(out, "buf.push('(');")?;
                {
                    let mut out = out.indent();
    
                
                    for argument in &self.arguments {
                        argument.generate_query(&mut out, variables)?;
                    }
                }
                writeln!(out, "buf.push(')');")?;
            }
    
            self.selections.generate_query(out, variables, fragments)?;
            Ok(())
        }
}

#[derive(Debug)]
pub struct SelectionQueryFragmentSpread {
    pub name: Name,
    pub position: Pos,
}

impl SelectionQueryFragmentSpread {
    pub fn new(parsed: &parsed_model::FragmentSpread) -> SelectionQuery {
        SelectionQuery::FragmentSpread(Rc::new(SelectionQueryFragmentSpread {
            name: parsed.name.clone(),
            position: parsed.position.clone(),
        }))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionQueryFragmentSpread {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name:      {}", self.name)?;
            writeln!(out, "position:  {}", self.position)?;
        }
        writeln!(out, "}}")
    }

    pub fn generate_query(&self, out: &mut Output, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
        fragments.insert(self.name.clone());
        writeln!(out, "buf.push_str(\"...{}\\n\");", &self.name)?;
        Ok(())
    }
}


#[derive(Debug)]
pub struct SelectionQueryInlineFragmentSpread {
    pub position: Pos,
    pub selections: Rc<SelectionQueryList>,
    pub type_condition: Option<Name>,
}

impl SelectionQueryInlineFragmentSpread{
    pub fn new(parsed: &parsed_model::InlineFragmentSpread, registry: &mut NameRegistry, schema: &Rc<Schema>, fields: &FieldMap) -> Result<SelectionQuery, Error> {
        println!("SelectionQueryInlineFragmentSpread {:?}", parsed.type_condition);
        Ok(SelectionQuery::InlineFragment(Rc::new(SelectionQueryInlineFragmentSpread {
            position: parsed.position.clone(),
            selections: Rc::new(SelectionQueryList::new(&parsed.selections, registry, schema, fields)?),
            type_condition: parsed.type_condition.clone(),
        })))
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionQueryInlineFragmentSpread {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "position:        {}", self.position)?;
            writeln!(out, "type_condition:  {:?}", self.type_condition)?;
            self.selections.print(&mut out)?;
        }
        writeln!(out, "}}")
    }

    pub fn generate_query(&self, out: &mut Output, variables: &crate::validated_model::FieldMap, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
        if let Some(cond) = &self.type_condition {
            writeln!(out, "buf.push_str(\"... on {} {{\\n\");", cond)?;
        }
        else {
            writeln!(out, "buf.push_str(\"... {{\\n\");")?;
        }
        {
            let mut out = out.indent();

        
            for selection in &self.selections.selections {
                selection.generate_query(&mut out, variables, fragments)?;
            }
        }
        writeln!(out, "buf.push('}}');")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum SelectionQuery {
    Field(Rc<SelectionQueryField>),
    FragmentSpread(Rc<SelectionQueryFragmentSpread>),
    InlineFragment(Rc<SelectionQueryInlineFragmentSpread>),
}

impl Print for SelectionQuery {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        match self {
            SelectionQuery::Field(content) => content.print(out),
            SelectionQuery::FragmentSpread(content) => content.print(out),
            SelectionQuery::InlineFragment(content) => content.print(out),
        }
    }
}

impl SelectionQuery {
    
    pub fn generate_query(&self, out: &mut Output, variables: &crate::validated_model::FieldMap, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
        match self {
            SelectionQuery::Field(selection_field) => selection_field.generate_query(out, variables, fragments),
            SelectionQuery::FragmentSpread(fragment_spread) => fragment_spread.generate_query(out, fragments),
            SelectionQuery::InlineFragment(inline_fragment_spread) => inline_fragment_spread.generate_query(out, variables, fragments),
        }
    }
}

// Used to generate the query (graphql request)
#[derive(Debug)]
pub struct SelectionQueryList {
    pub selections: Vec<SelectionQuery>,
}

impl Print for SelectionQueryList {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "SelectionQueryList {{")?;
        {
            let mut out = out.indent();

            for selection in &self.selections {
                selection.print(&mut out)?;
            }
        }
        writeln!(out, "}}")
    }
}

impl SelectionQueryList {
    fn new(parsed: &parsed_model::SelectionList, registry: &mut NameRegistry, schema: &Rc<Schema>, fields: &FieldMap) -> Result<Self, Error> {
        let mut selections = Vec::new();
        let mut has_typename = false;
        let mut requires_discriminator = false;

        for parsed_selection in &parsed.selections {
            match parsed_selection {
                parsed_model::Selection::Field(selection_field) => {
                    // println!("field {}", selection_field.name);

                    if *selection_field.name == TYPE_NAME {
                        has_typename = true;
                    }
                    let new_fields = if let Some(field) = fields.get(&selection_field.name) {
                        println!("field={:?}", field);

                        match field.ty.get_scalar() {
                            ScalarType::DefinedType(defined_type) => {
                                match defined_type {
                                    DefinedType::Scalar(_) => fields,
                                    DefinedType::Object(name) => {
                                        let object = schema.get_object(name)?;
                                        &object.fields
                                    },
                                    DefinedType::Interface(name) => {
                                        let interface = schema.get_interface(name)?;
                                        &interface.fields
                                    }
                                    DefinedType::Union(name) => {
                                        let object = schema.get_union(name)?;
                                        &object.fields
                                    },
                                    DefinedType::Enum(_) => fields,
                                    DefinedType::InputObject(name) => {
                                        let object = schema.get_input_object(name)?;
                                        &object.fields
                                    },
                                }
                            },
                            ScalarType::BuiltinType(_) => fields,
                        }
                    }
                    else {
                        fields
                    };
                    selections.push(SelectionQueryField::new(selection_field, registry, schema, new_fields)?);
                },
                parsed_model::Selection::FragmentSpread(fragment_spread) => {
                    requires_discriminator = true;
                    selections.push(SelectionQueryFragmentSpread::new(fragment_spread));
                },
                parsed_model::Selection::InlineFragment(inline_fragment_spread) => {
                    requires_discriminator = true;
                    selections.push(SelectionQueryInlineFragmentSpread::new(inline_fragment_spread, registry, schema, fields)?);
                },
            }
        }
        println!("has_typename {}", has_typename);
        if requires_discriminator && ! has_typename {
            selections.push(SelectionQuery::Field(Rc::new(SelectionQueryField {
                name: registry.intern_str(TYPE_NAME),
                alias: None,
                position: Pos { line: 0, column: 0 },
                force_nonnull: true,
                arguments: Vec::new(),
                selections: Rc::new(SelectionQueryList {
                    selections: Vec::new(),
                }),
            })));


            // println!("Adding __typename");
            // for sq in &selections {
            //     println!(" {:?}", sq)
            // }
            // println!("*******************************************");
        }
        Ok(Self { selections})
    }
    
    pub fn generate_query(&self, out: &mut Output<'_>, variables: &FieldMap, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
        if ! &self.selections.is_empty() {
            writeln!(out, "buf.push('{{');")?;
            {
                let mut out = out.indent();

            
                for selection in &self.selections {
                    selection.generate_query(&mut out, variables, fragments)?;
                }
            }
            writeln!(out, "buf.push('}}');")?;
        }
        Ok(())
    }
}

pub struct GenericOperation {
    pub parsed: Rc<parsed_model::GenericOperation>,
    pub variables: FieldMap,
    // pub response: Rc<Selection>,
    pub selection_query_list: SelectionQueryList,
}

impl GenericOperation {
    fn new(err: &mut ErrorCollector, registry: &mut NameRegistry, selection_manager: &mut NameSpaceManager, parsed: &Rc<parsed_model::GenericOperation>, schema: &Rc<Schema>, fragments: &IndexMap<Name, Rc<parsed_model::FragmentDefinition>>)-> Result<Self, Error> {

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
        let name = registry.intern_str("Response");
        let response = Self::create_selection(err, registry, selection_manager, schema, fragments, 
            name.clone(), name, &parsed.selections, fields, &None)?;

        if response.is_pagination_internal() {
            println!("\n\n UNUSED RESPONSE");

            selection_manager.bruce(response);
        }

        err.ok(GenericOperation {
                parsed: parsed.clone(),
                variables,
                // response,
                selection_query_list: SelectionQueryList::new(&parsed.selections, registry, schema, fields)?
            })
    }

    fn get_fields<'a>(err: &mut ErrorCollector, operation: &OperationType, name: &Name, position: &Pos, schema: &'a Rc<Schema>) -> Result<&'a FieldMap, Error> {
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
    
    fn create_selection(err: &mut ErrorCollector, registry: &mut NameRegistry, selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fragments: &IndexMap<Name, Rc<parsed_model::FragmentDefinition>>,
        preferred_type_name: Name, graphql_type_name: Name, selections: &Rc<parsed_model::SelectionList>, fields: &FieldMap, implemented_by: &Option<Vec<Name>>) -> Result<SelectionFieldType, Error>
    {
        println!("!! GenericOperation {} gather dependencies", &graphql_type_name);
        let mut selection_builder = SelectionBuilder::new(graphql_type_name, preferred_type_name, implemented_by);

        Self::gather_dependencies(err, registry, selection_manager, selections, schema, fragments, None, fields, &mut selection_builder)?;

        Ok(selection_builder.build(selection_manager))
    }

    fn create_selection_field(err: &mut ErrorCollector, registry: &mut NameRegistry, selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fragments: &IndexMap<Name, Rc<parsed_model::FragmentDefinition>>,
        field_name: Name, preferred_type_name: Name, graphql_type_name: Name, selections: &Rc<parsed_model::SelectionList>, force_nonnull: bool, fields: &FieldMap, implemented_by: &Option<Vec<Name>>) -> Result<SelectionField, Error>
    {
        let selection_type = Rc::new(Self::create_selection(err, registry, selection_manager, schema, fragments, preferred_type_name, graphql_type_name, selections, fields, implemented_by)?);
   
        Ok(SelectionField {
            name: field_name,
            selection_type,
            force_nonnull,
        })
    }

    fn new_get_dependencies(selection_manager: &mut NameSpaceManager, schema: &Rc<Schema>, fields: &FieldMap) -> Result<(), Error> {
        for field in fields.values() {
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
                        },
                        DefinedType::Union(name) => {
                            selection_manager.import_schema(name.clone());
                            let object = schema.get_union(name)?;

                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                        }
                        DefinedType::Enum(name) => {
                            selection_manager.import_schema(name.clone());
                        },
                        DefinedType::InputObject(name) => {
                            selection_manager.import_schema(name.clone());
                            let object = schema.get_input_object(name)?;
                            Self::new_get_dependencies(selection_manager, schema, &object.fields)?
                        },
                    }
                },
                ScalarType::BuiltinType(_) => {}
            }
        }
        Ok(())
    }
    
    fn wrap_type(ty: &Type, selection_field: SelectionField) -> SelectionField {
        SelectionField {
            name: selection_field.name,
            selection_type: Self::wrap_type2(ty, selection_field.selection_type),
            force_nonnull: selection_field.force_nonnull,
        }
    }

    fn wrap_type2(ty: &Type, selection_type: Rc<SelectionFieldType>) -> Rc<SelectionFieldType> {
        match ty {
            Type::Required(wrapped) => Rc::new(SelectionFieldType::Required(Self::wrap_type2(wrapped, selection_type))),
            Type::Array(wrapped) => Rc::new(SelectionFieldType::Array(Self::wrap_type2(wrapped, selection_type))),
            Type::Scalar(_) => selection_type,
        }
        
    }
    
    fn gather_dependencies(err: &mut ErrorCollector, registry: &mut NameRegistry, selection_manager: &mut NameSpaceManager, selections: &Rc<parsed_model::SelectionList>, schema: &Rc<Schema>, fragments: &IndexMap<Name, Rc<parsed_model::FragmentDefinition>>,
        variant_id: Option<VariantID>, fields: &FieldMap, selection_builder: &mut SelectionBuilder) -> Result<(), Error>
    {
        println!("!!   gather dependencies variant {:?}", variant_id);
        for selection in &selections.selections {
            match selection {
                parsed_model::Selection::Field(selection_field) => {
                    let field_name  = if let Some(alias) = &selection_field.alias { alias } else {&selection_field.name};
                    println!("!!     field {:?}", field_name);
                    if let Some(field) = get_field(schema, fields, &selection_field.name) {
                        selection_builder.with_field(&variant_id,  &field_name, 
                            Self::wrap_type(&field.ty, 
                                match field.ty.get_scalar() {
                                ScalarType::DefinedType(defined_type) => {
                                    match defined_type {
                                        DefinedType::Scalar(name) => {
                                            selection_manager.import_schema(name.clone());

                                            SelectionField {
                                                name: field_name.clone(), 
                                                selection_type: Rc::new(SelectionFieldType::Scalar(name.clone())),
                                                force_nonnull: selection_field.force_nonnull,
                                            }
                                        },
                                        DefinedType::Object(name) => {
                                            let object = schema.get_object(name)?;
                                            let preferred_type_name = if let Some(alias) = &selection_field.alias {
                                                registry.intern(to_pascal_case(&alias))
                                            }
                                            else {
                                                registry.intern(to_pascal_case(&object.name))
                                                // registry.intern(to_pascal_case(&selection_field.name))
                                            };


                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments,
                                                field_name.clone(), preferred_type_name, name.clone(), &selection_field.selections, selection_field.force_nonnull, &object.fields, &None)?
                                        },
                                        DefinedType::Interface(name) => {
                                            let object = schema.get_interface(name)?;
                                            let preferred_type_name = if let Some(alias) = &selection_field.alias {
                                                registry.intern(to_pascal_case(&alias))
                                            }
                                            else {
                                                registry.intern(to_pascal_case(&object.parsed.name))
                                                // registry.intern(to_pascal_case(&selection_field.name))
                                            };

                                            let interface = Some(object.implemented_by.clone());


                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_name, name.clone(), &selection_field.selections, selection_field.force_nonnull, &object.fields, &interface)?
                                        },
                                        DefinedType::Union(name) => {
                                            let object = schema.get_union(name)?;
                                            
                                            let preferred_type_name = if let Some(alias) = &selection_field.alias {
                                                registry.intern(to_pascal_case(&alias))
                                            }
                                            else {
                                                registry.intern(to_pascal_case(&object.parsed.name))
                                                // registry.intern(to_pascal_case(&selection_field.name))
                                            };

                                            let interface = Some(object.parsed.types.clone());

                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_name, name.clone(), &selection_field.selections, selection_field.force_nonnull, &object.fields, &interface)?
                                        }
                                        DefinedType::Enum(name) => {
                                            selection_manager.import_schema(name.clone());

                                            SelectionField {
                                                name: field_name.clone(), 
                                                selection_type: Rc::new(SelectionFieldType::Enum(name.clone())),
                                                force_nonnull: selection_field.force_nonnull,
                                            }
                                        },
                                        DefinedType::InputObject(name) => {
                                            let object = schema.get_input_object(name)?;
                                            
                                            let preferred_type_name = if let Some(alias) = &selection_field.alias {
                                                registry.intern(to_pascal_case(&alias))
                                            }
                                            else {
                                                registry.intern(to_pascal_case(&object.name))
                                                // registry.intern(to_pascal_case(&selection_field.name))
                                            };


                                            Self::create_selection_field(err, registry, selection_manager, schema, fragments, 
                                                field_name.clone(), preferred_type_name, name.clone(), &selection_field.selections, selection_field.force_nonnull, &object.fields, &None)?
                                        },
                                    }
                                },
                                ScalarType::BuiltinType(builtin_type) => {

                                    SelectionField {
                                        name: field_name.clone(), 
                                        selection_type: Rc::new(SelectionFieldType::BuiltinType(builtin_type.clone())),
                                        force_nonnull: selection_field.force_nonnull,
                                    }
                                }
                        }));
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
                        // let object = schema.get_object(&fragment.type_condition)?;
                        let fields = schema.get_object_or_interface_fields(&fragment.type_condition)?;
                        println!("\n\nSPREAD {}\n\n", &fragment_spread.name);
                        println!("builder graphql_type_name {}", selection_builder.graphql_type_name);
                        println!("builder implemented_by {:?}", selection_builder.implemented_by);

     

                        Self::gather_dependencies(err, registry, selection_manager, &fragment.selections, schema, fragments, 
                            // Some((fragment_spread.name.clone(), fragment.type_condition.clone())), 
                            Some(Self::get_variant_id(&fragment_spread.name, &fragment.type_condition, &selection_builder.implemented_by, schema)?), 
                            fields, selection_builder)?;
                    }
                    else {
                        err.error(BuildError::MissingFragmentError(fragment_spread.position.clone(), fragment_spread.name.to_string()));
                    }
                
                },
                parsed_model::Selection::InlineFragment(inline_fragment_spread) => {

                    if let Some(type_condition) = &inline_fragment_spread.type_condition {
                        let object = schema.get_object(type_condition)?;

                        Self::gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, 
                            // Some((type_condition.clone(), type_condition.clone())),
                            Some(Self::get_variant_id(type_condition, type_condition, &selection_builder.implemented_by, schema)?), 
                            &object.fields, selection_builder)?;
                    }
                    else {
                        // anonymous inline spread is essentially a no op nesting

                        Self::gather_dependencies(err, registry, selection_manager, &inline_fragment_spread.selections, schema, fragments, None,
                            fields, selection_builder)?;
                    }
                },
            };
        }

        
        Ok(())
       
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

            self.selection_query_list.print(&mut out)?;

            // self.response.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
    
    pub fn generate(&self, out: &mut Output, schema: &Rc<Schema>, executable_document: &ExecutableDocument, selection_manager: &NameSpaceManager) -> Result<(), Error> {
        writeln!(out, "pub mod {} {{", to_snake_case(&self.parsed.name))?;
        {
            let mut out = out.indent();
            writeln!(out, r#"
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
use sparko_graphql::{{GraphQLResponse, GraphQLQuery}};
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
            for id in selection_manager.get_context().names.values() {
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

                let rust_name = "Variables";

                writeln!(out, "impl {} {{", rust_name)?;
                {
                    let mut out = out.indent();
        
                    // writeln!(out, "pub fn builder() -> {}Builder {{", rust_name)?;
                    // writeln!(out, "    {}Builder {{", rust_name)?;
                    // for field in self.variables.values() {
                    //     writeln!(out, "        {}_: None,", to_snake_case(&field.parsed.name))?;
                    // }
                    // writeln!(out, "    }}")?;
                    // writeln!(out, "}}")?;


        
                    writeln!(out, "pub fn get_formal_params(&self, buf: &mut String) {{")?;
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
                    writeln!(out, "}}")?;
                }
            
                writeln!(out, "}}")?;
                writeln!(out, "")?;
        
                writeln!(out, "")?;
                // writeln!(out, "pub struct {}Builder {{", rust_name)?;
        
                // for field in self.variables.values() {
                //     writeln!(out, "    {}_: {},", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Optional))?;
                // }
            
                // writeln!(out, "}}")?;
                // writeln!(out, "")?;
        
                // writeln!(out, "impl {}Builder {{", rust_name)?;
                // {
                //     let mut out = out.indent();
        
                //     for field in self.variables.values() {
                //         writeln!(out, "pub fn with_{}(mut self, value: {}) -> Self {{", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Required))?;
                //         {
                //             let mut out = out.indent();
        
                //             writeln!(out, "self.{}_ = Some(value);", to_snake_case(&field.parsed.name))?;
                //             writeln!(out, "self")?;
                //         }
                //         writeln!(out, "}}")?;
                //         writeln!(out, "")?;
                //     }
                //     writeln!(out, "pub fn build(self) -> Result<{}, sparko_graphql::error::Error> {{", rust_name)?;
                //     {
                //         let mut out = out.indent();
        
                //         for field in self.variables.values() {
                //             if let Type::Required(_) = field.ty {
                //                 writeln!(out, "if let None = self.{}_ {{", to_snake_case(&field.parsed.name))?;
                //                 writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", field.parsed.name)?;
                //                 writeln!(out, "}}")?;
                //             }
                //         }
        
                //         writeln!(out, "Ok({} {{", rust_name)?;
                //         {
                //             let mut out = out.indent();
        
                //             for field in self.variables.values() {
                //                 if let Type::Required(_) = field.ty {
                //                     writeln!(out, "{}_: self.{}_.unwrap(),", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
                //                 }
                //                 else {
                //                     writeln!(out, "{}_: self.{}_,", to_snake_case(&field.parsed.name), to_snake_case(&field.parsed.name))?;
                //                 }
                //             }
                //             writeln!(out, "}})")?;
                //         }
                //     }
                //     writeln!(out, "}}")?;
                // }
                // writeln!(out, "}}")?;
                // writeln!(out, "")?;
            }

            writeln!(out, "#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]")?;
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

                    writeln!(out, "pub fn builder() -> {}Builder {{", self.parsed.operation)?;
                    writeln!(out, "    {}Builder {{", self.parsed.operation)?;
                    for field in self.variables.values() {
                        writeln!(out, "        {}_: None,", to_snake_case(&field.parsed.name))?;
                    }
                    writeln!(out, "    }}")?;
                    writeln!(out, "}}")?;
                }
            }
            
            writeln!(out, "}} // End of {}", self.parsed.operation)?;
            writeln!(out, "")?;

            if !self.variables.is_empty() {

        
                writeln!(out, "")?;
                writeln!(out, "pub struct {}Builder {{", self.parsed.operation)?;
        
                for field in self.variables.values() {
                    writeln!(out, "    {}_: {},", to_snake_case(&field.parsed.name), field.ty.rust_type(Optionality::Optional))?;
                }
            
                writeln!(out, "}}")?;
                writeln!(out, "")?;
        
                writeln!(out, "impl {}Builder {{", self.parsed.operation)?;
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
                    writeln!(out, "pub fn build(self) -> Result<{}, sparko_graphql::error::Error> {{", self.parsed.operation)?;
                    {
                        let mut out = out.indent();
        
                        for field in self.variables.values() {
                            if let Type::Required(_) = field.ty {
                                writeln!(out, "if let None = self.{}_ {{", to_snake_case(&field.parsed.name))?;
                                writeln!(out, "    return Err(sparko_graphql::error::Error::MissingRequiredValueError(\"{}\"))", field.parsed.name)?;
                                writeln!(out, "}}")?;
                            }
                        }
        
                        writeln!(out, "Ok({}::from(Variables {{", self.parsed.operation)?;
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
                            writeln!(out, "}}))")?;
                        }
                    }
                    writeln!(out, "}}")?;
                }
                writeln!(out, "}}")?;
                writeln!(out, "")?;
            }

            writeln!(out, "impl GraphQLQuery<Response> for {} {{", self.parsed.operation)?;
            {
                let mut out = out.indent();

                writeln!(out, "fn get_request_name() -> &'static str {{")?;
                writeln!(out.indent(), "Self::REQUEST_NAME")?;
                writeln!(out, "}}")?;

                writeln!(out, "fn get_query(&self) -> String {{")?;
                {
                    let mut out = out.indent();
                    writeln!(out, "let mut buf = String::from(\"{} {}\");", self.parsed.operation.to_lower_case(), self.parsed.name)?;

                    if ! self.variables.is_empty() {

                        writeln!(out, "buf.push('(');")?;
                        writeln!(out, "self.variables.get_formal_params(&mut buf);")?;
                        writeln!(out, "buf.push(')');")?;
                        
                    }

                    let mut done_fragments = IndexSet::new();
                    let mut fragments = IndexSet::new();
                    self.selection_query_list.generate_query(&mut out, &self.variables, &mut fragments)?;

                    while ! fragments.is_empty() {
                        fragments = Self::generate_fragments(&mut out, &self.variables, executable_document, &fragments, &mut done_fragments)?;
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
            writeln!(out, "")?;

            writeln!(out, "impl GraphQLResponse for Response {{")?;
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")?;
        Ok(())
    }
    
    fn generate_fragments(out: &mut Output<'_>, variables: &SharedMap<Field>, executable_document: &ExecutableDocument, fragments: &IndexSet<Name>, done_fragments: &mut IndexSet<Name>) -> Result<IndexSet<Name>, Error> {
        let mut new_fragments = IndexSet::new();
        for fragment_name in fragments {
            if ! done_fragments.contains(fragment_name) {
                let fragment = executable_document.fragments.get(fragment_name).unwrap();

                fragment.generate_query(out, variables, &mut new_fragments)?;
                done_fragments.insert(fragment_name.clone());
            }
        }

        Ok(new_fragments)
    }
    
    fn get_variant_id(name: &Name, type_condition: &Name, implemented_by: &Option<Vec<Name>>, schema: &Rc<Schema>) -> Result<VariantID, Error> {
        if name.as_str() == "meterFields" {
            println!("HERE");
        }
        if let Some(condition_type) = schema.defined_types.get(type_condition) {
            match condition_type {
                TypeDefinition::Object(object) => {
                    Ok(VariantID {
                        fragment_name: name.clone(),
                        type_conditions: vec!(type_condition.clone())
                    })
                },
                TypeDefinition::Interface(interface) => {
                    let mut type_conditions = Vec::new();

                    if let Some(implemented_by) = implemented_by {
                        for object_name in implemented_by {
                            if interface.implemented_by.contains(object_name) {
                                type_conditions.push(object_name.clone());

                                let object = schema.get_object(object_name)?;

                                for also_implements in &object.fully_implements {
                                    println!("{} also_implements {}", object_name, also_implements);
                                }
                            }
                        }

                        println!("!! VariantID name={} type_conditions={:?}", name, type_conditions);
                        Ok(VariantID {
                            fragment_name: name.clone(),
                            type_conditions
                        })
                    }
                    else {
                        Err(Error::BuildFailed(format!("Type condition '{}' is an interface but container is not", type_condition)))
                    }
                },
                _ => Err(Error::BuildFailed(format!("Invalid type condition '{}' is neither interface or object but {}", type_condition, condition_type.type_name()))),
            }
        }
        else {
            Err(Error::BuildFailed(format!("Type condition '{}' not found", type_condition)))
        }
    }
}


#[derive(Debug)]
pub struct VariantID {
    pub fragment_name: Name,
    pub type_conditions: Vec<Name>,
}

fn get_field<'a>(schema: &'a Rc<Schema>, fields: &'a SharedMap<Field>, name: &String) -> Option<&'a Rc<Field>> {
    if name == TYPE_NAME {
        Some(&schema.__typename)
    }
    else {
        fields.get(name)
    }
}

#[derive(Debug)]
pub struct SelectionContext {
    pub imports: HashSet<usize>,
    pub schema_imports: IndexSet<Name>,
    pub names: IndexMap<Name, usize>,
}

impl SelectionContext {
    pub fn new() -> Self {
        SelectionContext {
            imports: HashSet::new(),
            schema_imports: IndexSet::new(),
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
    fn name(&self) -> &Name {
        match self {
            NameSpaceItem::Selection(selection) => &selection.name,
            NameSpaceItem::Variant(variant) => &variant.name,
        }
    }

    fn generate(&self, out: &mut Output<'_>, selection_manager: &NameSpaceManager, schema: &Schema) -> Result<(), Error> {
        match self {
            NameSpaceItem::Selection(selection) => selection.generate(out, selection_manager, schema),
            NameSpaceItem::Variant(_) => Ok(()),
        }
    }
}

#[derive(Debug)]
pub struct NameSpaceManager {
    pub items: Vec<NameSpaceItem>,
    pub all_schema_imports: IndexSet<Name>,
    pub contexts: IndexMap<Name, IndexMap<Name, SelectionContext>>,
    pub names: IndexMap<usize, Name>,
    pub document_name: Option<Name>,
    pub operation_name: Option<Name>,
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
        }
        writeln!(out, "}}")
    }
}

impl NameSpaceManager {
    pub fn new() -> NameSpaceManager {
        NameSpaceManager {
            items: Vec::new(),
            all_schema_imports: IndexSet::new(),
            contexts: IndexMap::new(),
            names: IndexMap::new(),
            document_name: None,
            operation_name: None,
        }
    }

    pub fn allocate_names(&mut self, registry: &mut NameRegistry) {
        println!("Allocate Names");
        for (_document, map) in self.contexts.iter_mut() {
            println!("Document {}", _document);
            for (_operation, context) in map {
                println!("operation {}", _operation);
                for id in &context.imports {
                    if let Some(item) = self.items.get(*id) {
                        let preferred_name = match item {
                            NameSpaceItem::Selection(content ) => &content.name,
                            NameSpaceItem::Variant(content ) => &content.name,
                        };

                        // let preferred_name = selection_names.iter().next().unwrap();
                        let name;

                        if context.names.contains_key(preferred_name) {
                            let mut i = 2;
                            loop {
                                let possible_name = registry.intern(format!("{}{}", preferred_name, i));

                                if ! context.names.contains_key(&possible_name) {
                                    name = possible_name;
                                    break;
                                }
                                i += 1;
                            }
                        }
                        else {
                            name = preferred_name.clone();
                        }

                        // let default_name = Rc::new(format!("XType{}", id));
                        // let mut name = &default_name;
                        // print!("{} {:?} ", name, selection_names);
                        // for posible_name in selection_names {
                        //     print!("{} ", posible_name);
                        //     if ! context.names.contains_key(posible_name) {
                        //         name = posible_name;
                        //         // println!("Thats it");
                        //         break;
                        //     }
                        // }
                        println!("allocate_name({}) = {}", id, name);
                        context.names.insert(name.clone(), *id);
                        println!("context.names = {:?}", &context.names);
                        // self.names.insert(*id, name.clone());
                        self.names.insert(*id, name);
                    }
                    else {
                        panic!("Failed to find selection {}", id);
                    }
                }
            }
       }
    }

    pub fn create_document(&mut self, document_name: Name) {
        self.contexts.insert(document_name.clone(), IndexMap::new());
        self.document_name = Some(document_name);
    }

    pub fn create_operation(&mut self, operation_name: Name) {
        if let Some(document_name) = &self.document_name {
            if let Some(map) = self.contexts.get_mut(document_name) {
                map.insert(operation_name.clone(), SelectionContext::new());
                self.operation_name = Some(operation_name);
                return;
            }
        }
        panic!("Failed to create_operation?");
    }

    pub fn set_document(&mut self, document_name: Name) {
        self.document_name = Some(document_name);
    }

    pub fn set_operation(&mut self, operation_name: Name) {
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
                if let Some(operation_name) = &self.operation_name {
                    if let Some(context) = map.get(operation_name) {
                        return context;
                    }
                }
            }
        }
        panic!("Failed to set_document?");
    }

    fn import_schema(&mut self, name: Name) {
        self.get_context_mut().schema_imports.insert(name.clone());
        self.all_schema_imports.insert(name);
    }

    fn insert(&mut self, graphql_type_name: Name, name: Name, fields: IndexMap<Name, SelectionField>, p_variants: IndexMap<Name, (Name, IndexMap<Name, SelectionField>)>, implemented_by: Option<Vec<Name>>) -> Rc<Selection> {
        
        if "Node" == graphql_type_name.as_str() {
            println!("HERE {}", graphql_type_name);
        }
        println!("insert {} {:?}", self.items.len(), &name);

        let mut variants: Vec<Rc<Variant>> = Vec::new();
        // let mut other_interfce_map = HashMap::new();

        for (type_condition, (name, fields)) in p_variants {
            let index = self.items.len();

            println!("insert push {} {:?}", self.items.len(), &name);
            let variant: Rc<Variant> = Rc::new(Variant{
                index,
                name,
                fields,
                type_condition,
            });

            self.items.push( NameSpaceItem::Variant(variant.clone()));
            self.get_context_mut().imports.insert(index);

            variants.push(variant);


        }


        let selection_index = self.items.len();


        println!("insert push {} {:?}", self.items.len(), &name);

        let content = Rc::new(Selection {
            index: selection_index,
            graphql_type_name,
            name,
            fields,
            variants,
            implemented_by,
        });

        let result = content.clone();

        self.items.push( NameSpaceItem::Selection(content));
        self.get_context_mut().imports.insert(selection_index);
        

        result
    }
    
    fn get_schema_dependencies(&self) -> &IndexSet<Name> {
        &self.get_context().schema_imports
    }
    
    fn get_name(&self, id: &usize) -> &Name {
        println!("get_name({}) = {:?}", id, self.names.get(id));
        self.names.get(id).unwrap()
    }
    
    fn get(&self, id: usize) -> &NameSpaceItem {
        if let Some(item) = self.items.get(id) {
            item
        }
        else {
            panic!("Cant find selection {}", id);
        }
    }
    
    fn bruce(&mut self, response: SelectionFieldType) {
        if response.is_pagination_internal() {
            if let Some((wrapped, selection_creation_params)) = match response {
                SelectionFieldType::EdgeOf(selection_field_type, selection_creation_params) => Some((selection_field_type, selection_creation_params)),
                // SelectionFieldType::PageInfo => todo!(),
                // SelectionFieldType::ForwardPageInfo => todo!(),
                // SelectionFieldType::ReversePageInfo => todo!(),
                _ => None,
            } {
                if let SelectionFieldType::Selection(index) = SelectionFieldType::remove_wrapper(&wrapped).as_ref() {
                    println!("\n\n UNUSED RESPONSE insert {}", index);
                    self.insert(selection_creation_params.graphql_type_name, selection_creation_params.name, selection_creation_params.fields, selection_creation_params.p_variants, selection_creation_params.implemented_by);



                    // self.items.push( NameSpaceItem::Selection(content));
                    // self.get_context_mut().imports.insert(index.clone());
                }
            }
            
        }
    }
}

#[derive(Debug)]
pub struct SelectionCreationParams {
    graphql_type_name: Name, name: Name, fields: IndexMap<Name, SelectionField>, p_variants: IndexMap<Name, (Name, IndexMap<Name, SelectionField>)>, implemented_by: Option<Vec<Name>>,
}

impl SelectionCreationParams {
    fn new(graphql_type_name: Name, name: Name, fields: IndexMap<Name, SelectionField>, p_variants: IndexMap<Name, (Name, IndexMap<Name, SelectionField>)>, implemented_by: Option<Vec<Name>>,) -> SelectionCreationParams {
        SelectionCreationParams{
            graphql_type_name,
            name,
            fields,
            p_variants,
            implemented_by
        }
    }
}

pub struct FragmentDefinition {
    pub name: Name,
    pub selection_query_list: SelectionQueryList,
    pub type_condition: Name,
}

impl Print for FragmentDefinition {
    fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "FragmentDefinition {{")?;
        {
            let mut out = out.indent();

            writeln!(out, "name {}", self.name)?;
            writeln!(out, "type_condition {}", self.type_condition)?;
            self.selection_query_list.print(&mut out)?;
        }
        writeln!(out, "}}")
    }
}

impl FragmentDefinition {
    fn new(registry: &mut NameRegistry, parsed: &Rc<parsed_model::FragmentDefinition>, schema: &Rc<Schema>, fields: &FieldMap)-> Result<Self, Error> {
        Ok(FragmentDefinition {
            name: parsed.name.clone(),
            selection_query_list: SelectionQueryList::new(&parsed.selections, registry, schema, fields)?,
            type_condition: parsed.type_condition.clone(),
        })
    }

    pub fn generate_query(&self, out: &mut Output<'_>, variables: &crate::validated_model::FieldMap, fragments: &mut IndexSet<Name>) -> Result<(), Error> {
        
        writeln!(out, "")?;
        writeln!(out, "buf.push_str(\"fragment {} on {}\\n\");", self.name, self.type_condition)?;
        writeln!(out, "buf.push('{{');")?;
        {
            let mut out = out.indent();

            for selection in &self.selection_query_list.selections {
                selection.generate_query(&mut out, variables, fragments)?;
            }
        }
        writeln!(out, "buf.push('}}');")?;
        
        Ok(())
    }
}

pub struct ExecutableDocument {
    pub parsed: Rc<parsed_model::ExecutableDocument>,
    pub fragments: IndexMap<Name, FragmentDefinition>,
    pub operations: Vec<GenericOperation>,
}

impl ExecutableDocument {
    pub fn new(err: &mut ErrorCollector, registry: &mut NameRegistry,  selection_manager: &mut NameSpaceManager, parsed: &Rc<parsed_model::ExecutableDocument>, schema: &Rc<Schema>) ->  Result<Self, Error> {

        let mut fragments: IndexMap<Rc<String>, FragmentDefinition> = IndexMap::new();

        for fragment_definition in parsed.fragments.values() {
            // let object = schema.get_object(&fragment_definition.type_condition)?;
            // let fields = &object.fields;
            let fields = schema.get_object_or_interface_fields(&fragment_definition.type_condition)?;
            fragments.insert(fragment_definition.name.clone(), FragmentDefinition::new(registry, fragment_definition, schema, fields)?);
        }

        

        let mut operations = Vec::new();

        for query in &parsed.queries {
            selection_manager.create_operation(query.name.clone());
            operations.push(GenericOperation::new(err, registry, selection_manager, query, schema, &parsed.fragments)?);
        }

        for mutation in &parsed.mutations {
            selection_manager.create_operation(mutation.name.clone());
            operations.push(GenericOperation::new(err, registry, selection_manager, mutation, schema, &parsed.fragments)?);
        }

        Ok(ExecutableDocument {
            parsed: parsed.clone(),
            fragments,
            operations,
        })
    }

    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "ExecutableDocument {{")?;
        {
            let mut out = out.indent();

            self.parsed.print(&mut out)?;
            writeln!(out, "fragments {{")?;
            {
                let mut out = out.indent();

                for item in self.fragments.values() {
                    item.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;

            writeln!(out, "operations {{")?;
            {
                let mut out = out.indent();

                for operation in &self.operations {
                    operation.print(&mut out)?;
                }
            }
            writeln!(out, "}}")?;
        }
        writeln!(out, "}}")
    }

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
        }
        writeln!(out, "}} // End of executable_document {}", self.parsed.name)?;
        Ok(())
    }
}