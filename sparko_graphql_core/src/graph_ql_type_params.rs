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

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Data, Fields};
use syn::spanned::Spanned;
use inflections::case::to_camel_case;

use crate::parsed_type::ParsedType;




pub fn derive_graphql_query_params2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract the attributes!
    let derive_params: crate::GraphQLDeriveQueryParams = deluxe::extract_attributes(&mut ast)?;


    // define impl variables
   

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // generate

    
    let (formal, actual, variable) = impl_graphql_query_params(&ast.ident, &mut ast.data, &derive_params)?;

    let ident = &ast.ident;

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics sparko_graphql::GraphQLQueryParams for #ident #ty_generics #where_clause {
            

            fn get_formal_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
                #formal
            }

            fn get_actual_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str){
                #actual
                
            }
        
            fn get_variables_part(&self, super_variables: &mut serde_json::Map<String, serde_json::Value>, prefix: &str) -> Result<(), serde_json::Error> {
                #variable
                Ok(())
            }
        }
    };



    eprintln!("\n\n\n\n// GENERATED START GraphQLQueryParams\n{}\n// GENERATED END\n\n\n\n", expanded);
    
    
    
        // Hand the output tokens back to the compiler.
       Ok(expanded)
}



fn get_graphql_query_params_enum_parts(_variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>)  -> 
    deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    let formal: Vec<TokenStream> = Vec::new();
    let actual: Vec<TokenStream> = Vec::new();
    let variable: Vec<TokenStream> = Vec::new();

    Ok((quote!{#(#formal;)*}, quote!{#(#actual;)*}, quote!{#(#variable?;)*}))
}

fn get_graphql_query_params_parts(struct_ident: &Ident, fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, derive_params: &crate::GraphQLDeriveQueryParams) -> 
    deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    let mut formal: Vec<TokenStream> = Vec::new();
    let mut actual: Vec<TokenStream> = Vec::new();
    let mut variable: Vec<TokenStream> = Vec::new();

    let mut type_name_string = struct_ident.to_string();
    if derive_params.required {
        type_name_string.push('!');
    }

    let type_name = Literal::string(&type_name_string);
    let camel_name = Literal::string("input");
    
    if derive_params.as_object {

        
        
        actual.push(quote_spanned! {fields.span()=>
            params.push_actual(prefix, #camel_name)
            // Actual
        });

        formal.push(quote_spanned! {fields.span()=>
            params.push_formal(prefix, #camel_name, #type_name)
            // Formal
        });

        variable.push(quote_spanned! {fields.span()=>
            let mut variables = serde_json::Map::<String, serde_json::Value>::new();
        });
    }
    else {

        variable.push(quote_spanned! {fields.span()=>
            let variables = super_variables;
        });
    }
    for f in fields.iter_mut() {
        let attrs: crate::GraphQLDeriveAttributeParams = deluxe::extract_attributes(f)?;
        let serde_attrs: crate::SerdeDeriveAttributeParams = deluxe::extract_attributes(f)?;
        
        let ident = &f.ident;

        let ident = if let Some(ident) = ident {
            ident
        }
        else {
            panic!("Un identified type");
        };

        let name = ident.to_string();

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
                if !derive_params.as_object {
                    actual.push(quote_spanned! {f.span()=>
                        params.push_actual(prefix, #camel_name)
                        // Actual
                    });
    
                    formal.push(quote_spanned! {f.span()=>
                        params.push_formal(prefix, #camel_name, #type_name)
                        // Formal
                    });
                }

                let skip = if let Some(cond) = serde_attrs.skip_serializing_if {
                    cond == "Option::is_none"
                }
                else {
                    false
                };

                if skip {
                    variable.push(quote_spanned! {f.span()=>
                        if let Some(_value) = &self.#ident {
                            variables.insert(format!("{}{}", prefix, #camel_name), serde_json::to_value(&self.#ident)?);
                        }
                    });
                }
                else {
                    variable.push(quote_spanned! {f.span()=>
                        variables.insert(format!("{}{}", prefix, #camel_name), serde_json::to_value(&self.#ident)?);
                    });
                }
                
            }
            else {

                if !derive_params.as_object {
                    actual.push(quote_spanned! {f.span()=>
                    });

                    formal.push(quote_spanned! {f.span()=>
                        self.#ident.get_formal_part(params, &sparko_graphql::GraphQL::prefix(prefix, #name))
                    });
                }
                
                variable.push(quote_spanned! {f.span()=>
                    self.#ident.get_variables_part(variables, &sparko_graphql::GraphQL::prefix(prefix, #name))
                });
            }
        }
        else {
            panic!("Unrecognised type {:?}", &f.ty);
        }
    }

    if derive_params.as_object {

        variable.push(quote_spanned! {fields.span()=>
            super_variables.insert(format! ("{}{}", prefix, #camel_name), serde_json::Value::Object(variables))
        });
    }
    
    Ok((quote!{#(#formal;)*}, quote!{#(#actual;)*}, quote!{#(#variable;)*}))
}


fn impl_graphql_query_params(ident: &Ident, data: &mut Data, derive_params: &crate::GraphQLDeriveQueryParams) -> deluxe::Result<(TokenStream, TokenStream, TokenStream)> {
    match data {
        Data::Struct(data) => {
            match &mut data.fields {
                Fields::Named(fields) => {
                    get_graphql_query_params_parts(ident, &mut fields.named, derive_params)
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
            get_graphql_query_params_enum_parts(&data_enum.variants)
        },
        Data::Union(_) => panic!("Union queries struct fields not supported"),
    }
}


// END