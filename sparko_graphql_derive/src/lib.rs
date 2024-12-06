/*****************************************************************************
 MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/



use std::collections::HashMap;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};
use sparko_graphql_core::{graph_ql_entity, graph_ql_type, graph_ql_type_params, graph_ql_variables};
use syn::{DeriveInput, Data, Fields, Type};
use syn::spanned::Spanned;
use inflections::case::to_camel_case;

// mod parsed_type;
// mod graph_ql_entity;
// mod graph_ql_variables;
// mod graph_ql_query;
// mod graph_ql_type;
// mod graph_ql_type_params;



// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(serde))]
// struct SerdeDeriveParams {
//     rename_all: Option<String>,
//     rename_all_fields: Option<String>,
//     deny_unknown_fields: Option<String>,
//     tag: Option<String>,
//     content: Option<String>,
//     untagged: Option<bool>,
//     bound: Option<String>,
//     default: Option<bool>,
//     remote: Option<String>,
//     transparent: Option<bool>,
//     from: Option<String>,
//     try_from: Option<String>,
//     into: Option<String>,
//     // r#crate: Option<String>,
//     expecting: Option<String>,
// }

#[proc_macro_derive(GraphQLEntity, attributes(graphql))]
pub fn derive_graphql_entity(item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    graph_ql_entity::derive_graphql_entity2(item.into()).unwrap().into()
}

#[proc_macro_derive(GraphQLVariables, attributes(graphql))]
pub fn derive_graphql_variables(item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    graph_ql_variables::derive_graphql_query2(item.into()).unwrap().into()
}

#[proc_macro_derive(GraphQLType, attributes(graphql))]
pub fn derive_graphql_type(item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    graph_ql_type::derive_graphql_type2(item.into()).unwrap().into()
}



#[proc_macro_derive(GraphQLQueryParams, attributes(graphql))]
pub fn derive_graphql_query_params(item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    graph_ql_type_params::derive_graphql_query_params2(item.into()).unwrap().into()
}








// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(graphql))]
// struct GraphQLDeriveQueryParams {
//     #[deluxe(default = false)]
//     as_object: bool,
// }




// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(graphql))]
// struct GraphQLDeriveParams {
//     params: String,
//     super_type: Option<Vec<String>>
// }

// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(graphql))]
// struct GraphQLDeriveAttributeParams {
//     #[deluxe(default = false)]
//     required: bool,

//     #[deluxe(default = false)]
//     scalar: bool,

//     #[deluxe(default = false)]
//     no_params: bool,

//     rename: Option<String>
// }

// #[derive(deluxe::ExtractAttributes)]
// #[deluxe(attributes(serde))]
// struct SerdeDeriveAttributeParams {
//     #[deluxe(default = false)]
//     flatten: bool,
//     rename: Option<String>,
//     alias: Option<String>,
//     #[deluxe(default = false)]
//     default: bool,
//     #[deluxe(default = false)]
//     skip: bool,
//     #[deluxe(default = false)]
//     skip_serializing: bool,
//     #[deluxe(default = false)]
//     skip_deserializing: bool,
//     skip_serializing_if: Option<String>,
//     serialize_with: Option<String>,
//     deserialize_with: Option<String>,
//     with: Option<String>,
//     borrow: Option<String>,
//     bound: Option<String>,
//     getter: Option<String>,
// }



