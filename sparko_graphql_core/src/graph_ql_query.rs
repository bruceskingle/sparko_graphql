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

use crate::parsed_type::ParsedType;
use crate::{GraphQLDeriveAttributeParams, SerdeDeriveAttributeParams, SerdeDeriveParams};


#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(graphql))]
struct GraphQLEntityDeriveParams {
    // params: String,
    super_type: Option<Vec<String>>
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(graphql))]
struct GraphQLEntityDeriveAttributeParams {
    #[deluxe(default = false)]
    no_params: bool,
    query: Option<String>,
    // rename: Option<String>,
}



pub fn derive_graphql_query2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract the attributes!
    // let GraphQLDeriveParams { params } = deluxe::extract_attributes(&mut ast)?;
    // let params_ident = Ident::new(&params, ast.span());

    // define impl variables

    let ident = ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // generate

    
    let (formal, actual, variable) = impl_graphql_query(&mut ast.data)?;

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics sparko_graphql::GraphQLVariables for #ident #ty_generics #where_clause {
            

            fn get_formal_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
                #formal
            }

            fn get_actual_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str){
                #actual
                
            }
        
            fn get_variables_part(&self, variables: &mut sparko_graphql::VariableBuffer, prefix: &str) -> Result<(), serde_json::Error> {
                #variable
                Ok(())
            }
        }
    };



    eprintln!("\n\n\n\n// GENERATED START GraphQLVariables\n{}\n// GENERATED END\n\n\n\n", expanded);
    
    
    
        // Hand the output tokens back to the compiler.
       Ok(expanded)
}

fn get_graphql_query_enum_parts(variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>)  -> 
    deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    let mut formal: Vec<TokenStream> = Vec::new();
    let mut actual: Vec<TokenStream> = Vec::new();
    let mut variable: Vec<TokenStream> = Vec::new();

    Ok((quote!{#(#formal;)*}, quote!{#(#actual;)*}, quote!{#(#variable?;)*}))
}

fn get_graphql_query_parts(fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) -> 
    deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    let mut formal: Vec<TokenStream> = Vec::new();
    let mut actual: Vec<TokenStream> = Vec::new();
    let mut variable: Vec<TokenStream> = Vec::new();

    for f in fields.iter_mut() {
        let attrs: GraphQLDeriveAttributeParams = deluxe::extract_attributes(f)?;
        let serde_attrs: SerdeDeriveAttributeParams = deluxe::extract_attributes(f)?;
        // let rename = if serde_attrs.rename.is_some() {
        //     &attrs.rename
        // }
        // else {
        //     &serde_attrs.rename
        // };

        
        let ident = &f.ident;

        let ident = if let Some(ident) = ident {
            ident
        }
        else {
            panic!("Un identified type");
        };

        let name = ident.to_string();

        //let field_name: Literal = Literal::string(&name);

        let parsed_type = ParsedType::parse(&f.ty);
        if let Some(parsed_type) = parsed_type {
            if parsed_type.scalar {
                let mut type_name_string = parsed_type.type_name;

                if attrs.required {
                    type_name_string.push('!');
                }
                let type_name = Literal::string(&&type_name_string);
                let camel_name = if let Some(rename) = serde_attrs.rename {
                    Literal::string(&rename)
                }
                else {
                    Literal::string(&to_camel_case(&name))
                };


                
                actual.push(quote_spanned! {f.span()=>
                    params.push_actual(prefix, #camel_name)
                    // Actual
                });

                formal.push(quote_spanned! {f.span()=>
                    params.push_formal(prefix, #camel_name, #type_name)
                    // Formal
                });

                variable.push(quote_spanned! {f.span()=>
                    variables.push_variable(prefix, #camel_name, &self.#ident)
                    // Variable
                });
            }
            else {
                actual.push(quote_spanned! {f.span()=>
                    // // params.push_actual(#camel_name, &GraphQL::prefix(prefix, #name));
                    // // self.properties.get_formal_part(params, Self::prefix(prefix, #name))
                    // self.#ident.get_actual_part(params, &GraphQL::prefix(prefix, #name))
                });

                if parsed_type.is_option {
                    formal.push(quote_spanned! {f.span()=>

                        foo("HERE 1");
                        if let Some(field) = &self.#ident {
                            field.get_formal_part(params, &sparko_graphql::GraphQL::prefix(prefix, #name))
                        }                    
                    });

                    variable.push(quote_spanned! {f.span()=>

                        foo("HERE 2");
                        if let Some(field) = &self.#ident {
                            field.get_variables_part(variables, &sparko_graphql::GraphQL::prefix(prefix, #name))?
                        }
                        // self.properties.get_formal_part(params, Self::prefix(prefix, #name))
                    });
                }
                else {
                    formal.push(quote_spanned! {f.span()=>

                        foo("HERE 1b");
                        &self.#ident.get_formal_part(params, &sparko_graphql::GraphQL::prefix(prefix, #name))
                    });

                    variable.push(quote_spanned! {f.span()=>

                        foo("HERE 2b");
                        &self.#ident.get_variables_part(variables, &sparko_graphql::GraphQL::prefix(prefix, #name))?
                        // self.properties.get_formal_part(params, Self::prefix(prefix, #name))
                    });
                }
            }
        }
        else {
            panic!("Unrecognised type {:?}", &f.ty);
        }
    }
    
    Ok((quote!{#(#formal;)*}, quote!{#(#actual;)*}, quote!{#(#variable;)*}))
}



fn impl_graphql_query(data: &mut Data) -> deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    match data {
        Data::Struct(data) => {
            match &mut data.fields {
                Fields::Named(fields) => {

                    get_graphql_query_parts(&mut fields.named)
                }
                Fields::Unnamed(ref _fields) => {
                    panic!("Unnamed query param struct fields not supported");
                }
                Fields::Unit => {
                    panic!("Unit query params struct fields not supported");
                }
            }
        }
        Data::Enum(data_enum) => {
            get_graphql_query_enum_parts(&data_enum.variants)
        },
        Data::Union(_) => panic!("Union queries struct fields not supported"),
    }
}
