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


mod parsed_type;
pub mod graph_ql_type;
pub mod graph_ql_type_params;
mod tests;


#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(serde))]
#[allow(dead_code)]
struct SerdeDeriveParams {
    rename_all: Option<String>,
    rename_all_fields: Option<String>,
    deny_unknown_fields: Option<String>,
    tag: Option<String>,
    content: Option<String>,
    untagged: Option<bool>,
    bound: Option<String>,
    default: Option<bool>,
    remote: Option<String>,
    transparent: Option<bool>,
    from: Option<String>,
    try_from: Option<String>,
    into: Option<String>,
    // r#crate: Option<String>,
    expecting: Option<String>,
}




#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(graphql))]
struct GraphQLDeriveQueryParams {
    #[deluxe(default = false)]
    as_object: bool,
    #[deluxe(default = false)]
    required: bool,
}




#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(graphql))]
struct GraphQLDeriveParams {
    params: String,
    super_type: Option<Vec<String>>,
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(graphql))]
struct GraphQLDeriveAttributeParams {
    #[deluxe(default = false)]
    required: bool,

    #[deluxe(default = false)]
    scalar: bool,

    #[deluxe(default = false)]
    no_params: bool,

    // rename: Option<String>
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(serde))]
#[allow(dead_code)]
struct SerdeDeriveAttributeParams {
    #[deluxe(default = false)]
    flatten: bool,
    rename: Option<String>,
    alias: Option<String>,
    #[deluxe(default = false)]
    default: bool,
    #[deluxe(default = false)]
    skip: bool,
    #[deluxe(default = false)]
    skip_serializing: bool,
    #[deluxe(default = false)]
    skip_deserializing: bool,
    skip_serializing_if: Option<String>,
    serialize_with: Option<String>,
    deserialize_with: Option<String>,
    with: Option<String>,
    borrow: Option<String>,
    bound: Option<String>,
    getter: Option<String>,
}



