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
use syn::{DeriveInput, Data, Fields, Type};
use syn::spanned::Spanned;
use inflections::case::to_camel_case;

use crate::{SerdeDeriveAttributeParams, SerdeDeriveParams};



pub struct ParsedType {
    pub type_name: String, 
    pub scalar: bool,
    pub page_forward: bool,
    pub page_reverse: bool,
    pub is_option: bool,
}

impl ParsedType {

    fn is_graphql_simple_type_name(name: &str) -> bool {
        match name {
            "ID" => true,
            "String" => true,
            "Int" => true,
            "i32" => true,
            "i16" => true,
            "i8" => true,
            "Float" => true,
            "f64" => true,
            "f32" => true,
            "Boolean" => true,
            "bool" => true,
            "Decimal" => true,
            "Date" => true,
            "DateTime" => true,
            _ => false,
        }
    }

    pub fn parse(ty: &Type) -> Option<ParsedType> {
        if let Type::Path(path_type) = ty {
            if path_type.qself.is_some() {
                return None
            }

            let path = &path_type.path;

            if path.leading_colon.is_some() {
                return None
            }

            if path.segments.len() != 1 {
                return None
            }

            let segment = &path.segments.iter().next().unwrap();

            let name = segment.ident.to_string();

            match &segment.arguments {
                syn::PathArguments::None => {},
                syn::PathArguments::AngleBracketed(args) => {
                    if args.args.len() != 1 {
                        return None
                    }
                    let arg = args.args.iter().next().unwrap();

                    if let syn::GenericArgument::Type(item) = arg {
                        if let Type::Path(path_item) = item {
                            if path_item.path.segments.len() != 1 {
                                return None
                            }
                    
                            let segment = &path_item.path.segments.iter().next().unwrap();
                    
                            let parameter_name = segment.ident.to_string();

                            return Some(match name.as_str() {
                                "Option" =>  ParsedType{
                                    scalar: Self::is_graphql_simple_type_name(&parameter_name),
                                    type_name: parameter_name,
                                    page_forward: false,
                                    page_reverse: false,
                                    is_option: true,
                                },
                                "Vec" =>  ParsedType{
                                    scalar: Self::is_graphql_simple_type_name(&parameter_name),
                                    type_name: parameter_name,
                                    page_forward: false,
                                    page_reverse: false,
                                    is_option: false,
                                },
                                "ForwardPageOf" => ParsedType{
                                    type_name: parameter_name,
                                    scalar: false,
                                    page_forward: true,
                                    page_reverse: false,
                                    is_option: false,
                                },
                                "ReversePageOf" => ParsedType{
                                    type_name: parameter_name,
                                    scalar: false,
                                    page_forward: false,
                                    page_reverse: true,
                                    is_option: false,
                                },
                                "PageOf" => ParsedType{
                                    type_name: parameter_name,
                                    scalar: false,
                                    page_forward: true,
                                    page_reverse: true,
                                    is_option: false,
                                },
                                _ => ParsedType{
                                    type_name: format!("{}<{}>", name, parameter_name),
                                    scalar: false,
                                    page_forward: false,
                                    page_reverse: false,
                                    is_option: false,
                                },
                            });
                        }
                        else {
                            return None
                        }
                    }
                    else {
                        return None
                    }
                },
                syn::PathArguments::Parenthesized(_) => {
                    return None
                },
            }

            return Some(ParsedType{
                scalar: Self::is_graphql_simple_type_name(&name),
                type_name: name,
                page_forward: false,
                page_reverse: false,
                is_option: false,
            });

        }
        else {
            // Not interesting to us
            return None
        }
    }
}