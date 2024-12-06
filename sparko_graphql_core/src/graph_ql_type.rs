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
use crate::parsed_type::ParsedType;


pub fn derive_graphql_type2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract the attributes!
    let derive_params: crate::GraphQLDeriveParams = deluxe::extract_attributes(&mut ast)?;
    let serde_derive_params: crate::SerdeDeriveParams = deluxe::extract_attributes(&mut ast)?;

    let params_ident = Ident::new(&derive_params.params, ast.span());
    // define impl variables

    let ident = ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();


    let (names, args) = impl_graphql_type_params(&mut ast.data, &ident, &derive_params, &serde_derive_params)?;


    let lit = proc_macro2::Literal::string(&names);
    let expanded = quote! {
        impl #impl_generics sparko_graphql::GraphQLType<#params_ident> for #ident #ty_generics #where_clause {
            fn get_query_attributes(params: &#params_ident, prefix: &str) -> String {
                // #lit.to_string(#args)
                format!(#lit, #args)
            }
        }};




        eprintln!("\n\n\n\nTYPE GENERATED_TOKENS\n{}\n\n\n\n\n", expanded);
    
    
    
        // Hand the output tokens back to the compiler.
       Ok(expanded)
}

fn get_graphql_type_parts(fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, struct_name: &Ident) -> 
deluxe::Result<(String, TokenStream)> {
    let mut names = String::new();
    let mut args: Vec<TokenStream> = Vec::new();
    let mut has_fields = false;
    let mut has_flattened = false;

         
    // names.push_str("{{\n");
    for f in fields {
        let attrs: crate::GraphQLDeriveAttributeParams = deluxe::extract_attributes(f)?;
        let serde_attrs: crate::SerdeDeriveAttributeParams = deluxe::extract_attributes(f)?;

       

        let ident = &f.ident;

        let ident = if let Some(ident) = ident {
            ident
        }
        else {
            panic!("Unnamed struct field");
        };

        let mut name = ident.to_string();

        if !name.starts_with("__") {
            name = to_camel_case(&name);
        }
        let field_name: Literal = Literal::string(&name);
        let camel_name = Literal::string(&to_camel_case(&name));

        let parsed_type = ParsedType::parse(&f.ty);
        if let Some(parsed_type) = parsed_type {
            if serde_attrs.flatten {
                has_flattened = true;
                names.push_str(&format!("# flattened {}\n", name))
            }
            else {
                has_fields = true;
                if attrs.scalar || parsed_type.scalar {
                    names.push_str(&name);
                    names.push('\n');
                }
                else {
                    let type_name = Ident::new(&parsed_type.type_name, f.span().clone());

                    names.push_str(&name);
                    // names.push_str("{}{{\n");
                    names.push_str("{}\n");

                    if parsed_type.page_forward || parsed_type.page_reverse {
                        names.push_str(&format!("  # pageOf {}\n", camel_name));
                        names.push_str("  {{ # pageOf\n");
                        names.push_str("    pageInfo {{\n");
                        if parsed_type.page_forward {
                            names.push_str("        startCursor\n");
                            names.push_str("        hasNextPage\n");
                        }
                        if parsed_type.page_reverse {

                            names.push_str("        endCursor\n");
                            names.push_str("        hasPreviousPage\n");
                        }
                        names.push_str("    }}\n");
                        names.push_str("    edges {{ # pageOf.edges\n");

                        names.push_str(&format!("  # pageOf.node {}\n", camel_name));
                        names.push_str("        node {}\n");

                        names.push_str(&format!("  # /pageOf.node {}\n", camel_name));
                        names.push_str("    }} # /pageOf.edges\n");
                        names.push_str("  }} # /pageOf\n");

                        names.push_str(&format!("  # /pageOf {}\n", camel_name));
                    }
                    else {          
                        names.push_str(&format!("  # object {}\n", camel_name));     
                        // names.push_str("  {{ # object\n");     
                        names.push_str("    {}\n");
                        // names.push_str("  }} # /object\n");
                            
                        names.push_str(&format!("  # /object {}\n", camel_name));  
                    }
                    // names.push_str("}}\n");
        
                    let arg = if attrs.no_params {
                        quote_spanned! {f.span()=>
                            "",
                            #type_name::get_query_part(&NoParams, &sparko_graphql::GraphQL::prefix(prefix, #field_name))
                        }

                    } 
                    else { 
                        quote_spanned! {f.span()=>
                            params.#ident.get_actual(&sparko_graphql::GraphQL::prefix(prefix, #field_name)),
                            #type_name::get_query_part(&params.#ident, &sparko_graphql::GraphQL::prefix(prefix, #field_name))
                        }
                    };
                
                    args.push(arg);
                }
            }
        }
        else {
            panic!("Unrecognised type {:?}", &f.ty);
        }
    }
    // names.push_str("}}\n");

    if has_fields && has_flattened {
        names = format!("...on {} {{{{\n{}\n}}}}\n", struct_name.to_string(), names);
    }
    
    Ok((names, quote!{#(#args,)*}))
}

fn get_graphql_type_enum_parts(variants: &mut syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>, struct_name: &Ident, derive_params: &crate::GraphQLDeriveParams, serde_derive_params: &SerdeDeriveParams) -> 
deluxe::Result<(String, TokenStream)> {
    let mut names = String::new();
    let mut args: Vec<TokenStream> = Vec::new();

    if let Some(tag) = &serde_derive_params.tag {
        names.push_str(&tag);
        names.push('\n');
    }
    // names.push_str(&format!("/* super {:?}*/", derive_params.super_type));
    if let Some(super_types) = &derive_params.super_type {
        for super_type in super_types {
            names.push_str("{}\n");

            let super_type_name = Ident::new(&super_type, variants.span().clone());
            
            
            let arg = 
                quote_spanned! {variants.span()=>
                    #super_type_name::get_query_attributes(params, prefix) //&sparko_graphql::GraphQL::prefix(prefix, #variant_name))
                };
        
            args.push(arg);
        }
    }
         
    // names.push_str("{{\n");
    for v in variants {
        let attrs: crate::GraphQLDeriveAttributeParams = deluxe::extract_attributes(v)?;
       

        let ident = &v.ident;

        let mut name = ident.to_string();

        if !name.starts_with("__") {
            name = to_camel_case(&name);
        }


        match &mut v.fields {
            Fields::Named(fields) => {
                panic!("Named fields in enum not supported");
            }
            Fields::Unnamed(ref fields) => {

                let mut i = 0;
                for f in &fields.unnamed {
                    if i == 0 {
                        i = 1;
                    }
                    else {
                        panic!("Multiple fields in enum not supported");
                    }
                }

                let the_field = fields.unnamed.iter().next().unwrap();

                

                let parsed_type = ParsedType::parse(&the_field.ty);
                if let Some(parsed_type) = parsed_type {
                    if attrs.scalar || parsed_type.scalar {
                        panic!("Scalar filed in enum not supported");

                        // maybe this would work:
                        // names.push_str(&name);
                        // names.push('\n');
                    }
                    else {
                        let type_name = Ident::new(&parsed_type.type_name, v.span().clone());

                        names.push_str(&format!("  # enum variant {}\n", name));
                        names.push_str("  {}\n");


                        names.push_str(&format!("  # /enum variant {}\n", name));
            
                        let arg = 
                            quote_spanned! {v.span()=>
                                #type_name::get_query_attributes(params, prefix) //&sparko_graphql::GraphQL::prefix(prefix, #variant_name))
                            };
                    
                        args.push(arg);
                    }
                }
                else {
                    panic!("Unrecognised type {:?}", &the_field.ty);
                }
            }
            Fields::Unit => {
                panic!("Unit field in enum not supported");
            }
        }
        
    }
    
    Ok((names, quote!{#(#args,)*}))
}



fn impl_graphql_type_params(data: &mut Data, struct_name: &Ident, derive_params: &crate::GraphQLDeriveParams, serde_derive_params: &SerdeDeriveParams) -> deluxe::Result<(String, TokenStream)> {
    match data {
        Data::Struct(data) => {
            match &mut data.fields {
                Fields::Named(fields) => {
                    get_graphql_type_parts(&mut fields.named, struct_name)
                }
                Fields::Unnamed(ref _fields) => {
                    panic!("Unnamed struct fields not supported");
                }
                Fields::Unit => {
                    panic!("Unit struct fields not supported");
                }
            }
        },
        Data::Enum(data) => {
            get_graphql_type_enum_parts(&mut data.variants, struct_name, derive_params, serde_derive_params)
        },
        Data::Union(_) => panic!("Unions not supported"),
    }
}