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
use crate::{SerdeDeriveAttributeParams, SerdeDeriveParams};


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

struct GraphQLEntityProps {
    names: String,
    name_args: TokenStream,
    params: TokenStream,
}


pub fn derive_graphql_entity2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // Extract the attributes!
    let derive_params: GraphQLEntityDeriveParams = deluxe::extract_attributes(&mut ast)?;
    let serde_derive_params: SerdeDeriveParams = deluxe::extract_attributes(&mut ast)?;

    // let params_ident = Ident::new(&derive_params.params, ast.span());
    // define impl variables

    let ident = ast.ident;
    

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();


    let GraphQLEntityProps {
        names,
        name_args,
        params,
    } = impl_graphql_entity(&mut ast.data, &ident, &derive_params, &serde_derive_params)?;



    // eprintln!("TOKENS NAMES = <{}>", names);
    let lit = proc_macro2::Literal::string(&names);
    let mut expanded: TokenStream = TokenStream::new();

    let params_ident = Ident::new(&format!("{}{}", ident.to_string(), "Variables"), ident.span());
    
    if params.is_empty() {
        expanded.extend(quote! {
            type #params_ident = NoVariables;
        });
    }
    else {
        expanded.extend(quote! {
            #[derive(GraphQLVariables)]
            struct #params_ident {
                #params
            }});
    };


   
    
    
    
    
    
    expanded.extend(quote! {
        impl #impl_generics sparko_graphql::GraphQLEntity<#params_ident> for #ident #ty_generics #where_clause {
            fn get_query_attributes(params: &#params_ident, prefix: &str) -> String {
                // #lit.to_string(#args)
                format!(#lit, #name_args)
            }
        }
    });




        eprintln!("\n\n\n\n// GENERATED START GraphQLEntity\n{}\n// GENERATED END GraphQLEntity\n\n\n\n", expanded);
    
    
    
        // Hand the output tokens back to the compiler.
       Ok(expanded)
}


fn impl_graphql_entity(data: &mut Data, struct_name: &Ident, derive_params: &GraphQLEntityDeriveParams, serde_derive_params: &SerdeDeriveParams) -> deluxe::Result<GraphQLEntityProps> {
    match data {
        Data::Struct(data) => {
            match &mut data.fields {
                Fields::Named(fields) => {
                    get_parts(&mut fields.named, struct_name)
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
            panic!("Unions not supported");
            // get_graphql_type_enum_parts(&mut data.variants, struct_name, derive_params, serde_derive_params)
        },
        Data::Union(_) => panic!("Unions not supported"),
    }
}

fn get_parts(fields: &mut syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, struct_name: &Ident) -> 
deluxe::Result<GraphQLEntityProps> {
    let mut names = String::new();
    let mut args: Vec<TokenStream> = Vec::new();
    let mut param_tokens: Vec<TokenStream> = Vec::new();
    let mut has_fields = false; // not sure we need this
    let mut has_flattened = false;

    for f in fields {


        let attrs: GraphQLEntityDeriveAttributeParams = deluxe::extract_attributes(f)?;
        let serde_attrs: SerdeDeriveAttributeParams = deluxe::extract_attributes(f)?;



        let ident = if let Some(ident) = &f.ident {
            ident
        }
        else {
            panic!("Unnamed struct field");
        };

        let mut name = ident.to_string();

// println!("bruce> field={} attr={:?} serde_attr={:?}", &ident.to_string(), &attrs.rename, &serde_attrs.rename);

//         let rename = if attrs.rename.is_some() {
//             attrs.rename
//         }
//         else {
//             serde_attrs.rename
//         };

// println!("bruce> rename={:?}", &rename);

        // if !name.starts_with("__") {
        //     name = to_camel_case(&name);
        // }
        // let field_name: Literal = Literal::string(&name);
        
        if let Some(rename) = serde_attrs.rename {
            name = rename;
        }
        else if ! name.starts_with("__") {
            name = to_camel_case(&name);
        }
        
        // let camel_name = if name.starts_with("__") {
        //     &name
        // }
        // else if let Some(rename) = rename {
        //     &rename
        // }
        // else {
        //     let n = &name;
        //     &to_camel_case(&name)
        // };
        let field_name = Literal::string(&name);


println!("bruce2> result={}", &name);

        let parsed_type = ParsedType::parse(&f.ty);
        if let Some(parsed_type) = parsed_type {
            if serde_attrs.flatten {
                has_flattened = true;
                names.push_str(&format!("# flattened {}\n", name))
            }
            else {
                has_fields = true;

                if attrs.query.is_some() || attrs.no_params {
                    let type_name = Ident::new(&parsed_type.type_name, f.span().clone());
                    // let field_params_ident = Ident::new(&format!("params.{}", &params), f.span());

                    names.push_str(&format!("# object {}\n", name));
                    names.push_str("{}\n");

                    if let Some(params) = &attrs.query {
                        let field_params_type_ident = Ident::new(&params, f.span());
                        
                        args.push(quote_spanned! {f.span() =>
                            if let Some(params) = &params.#ident {
                                format!("{} {}\n  {}\n", #field_name,
                                 params.get_actual(&sparko_graphql::GraphQL::prefix(prefix, #field_name)),
                                 #type_name::get_query_part(&params.#ident, &sparko_graphql::GraphQL::prefix(prefix, #field_name)))
                            } else {
                                String::new()
                            }
                        });

                        
                        let param = quote_spanned! {f.span() =>
                            #ident: Option<#field_params_type_ident>
                        };

                        param_tokens.push(param);
                    }
                    else {
                        args.push(quote_spanned! {f.span() =>
                            if &params.#ident {
                                format!("{} {{\n  {}\n}}", #field_name,
                                 #type_name::get_query_part(&NoParams, &sparko_graphql::GraphQL::prefix(prefix, #field_name)))
                            } else {
                                String::new()
                            }
                        });

                        let bool_ident = Ident::new("bool", f.span());
                        let param = quote_spanned! {f.span() =>
                            #ident: #bool_ident
                        };

                        param_tokens.push(param);
                    }












                    // names.push_str(&name);
                    // names.push_str("{}\n");

                    // if parsed_type.page_forward || parsed_type.page_reverse {
                    //     names.push_str(&format!("  # pageOf {}\n", camel_name));
                    //     names.push_str("  {{ # pageOf\n");
                    //     names.push_str("    pageInfo {{\n");
                    //     if parsed_type.page_forward {
                    //         names.push_str("        startCursor\n");
                    //         names.push_str("        hasNextPage\n");
                    //     }
                    //     if parsed_type.page_reverse {

                    //         names.push_str("        endCursor\n");
                    //         names.push_str("        hasPreviousPage\n");
                    //     }
                    //     names.push_str("    }}\n");
                    //     names.push_str("    edges {{ # pageOf.edges\n");

                    //     names.push_str(&format!("  # pageOf.node {}\n", camel_name));
                    //     names.push_str("        node {}\n");

                    //     names.push_str(&format!("  # /pageOf.node {}\n", camel_name));
                    //     names.push_str("    }} # /pageOf.edges\n");
                    //     names.push_str("  }} # /pageOf\n");

                    //     names.push_str(&format!("  # /pageOf {}\n", camel_name));
                    // }
                    // else {          
                    //     names.push_str(&format!("  # object {}\n", camel_name));     
                    //     // names.push_str("  {{ # object\n");     
                    //     names.push_str("    {}\n");
                    //     // names.push_str("  }} # /object\n");
                            
                    //     names.push_str(&format!("  # /object {}\n", camel_name));  
                    // }
                    // // names.push_str("}}\n");
        




                    // if let Some(params) = &attrs.params {
                    //     let params_ident = Ident::new(&params, f.span());

                    //     let param = quote_spanned! {f.span() =>
                    //         #field_name: Option<#params_ident>,
                    //     };

                    //     param_tokens.push(param);

                    //     args.push(quote_spanned! 
                    //         (
                    //             f.span()=> if let Some(params) = #params_ident {params.get_actual(&sparko_graphql::GraphQL::prefix(prefix, #field_name))
                    //             } else {
                    //                 ""
                    //             }
                    //         )
                    //     );
                    //     args.push(quote_spanned! 
                    //         (
                    //             f.span()=> if let Some(params) = #params_ident {#type_name::get_query_part(&params.#ident, &sparko_graphql::GraphQL::prefix(prefix, #field_name))
                    //             } else {
                    //                 ""
                    //             }
                    //         ));
                    // }
                    // else {
                    //     let bool_ident = Ident::new("bool", f.span());
                    //     let param = quote_spanned! {f.span() =>
                    //         #field_name: #bool_ident,
                    //     };

                    //     param_tokens.push(param);

                    //     quote_spanned! {f.span()=>
                    //         "",
                    //         #type_name::get_query_part(&NoParams, &sparko_graphql::GraphQL::prefix(prefix, #field_name))
                    //     }

                    // }


                    
                }
                else {
                    names.push_str(&name);
                    names.push('\n');
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
    
    Ok(
        GraphQLEntityProps {
            names,
            name_args: quote!{#(#args,)*},
            params: quote!{#(#param_tokens,)*},
        }
    )
        
        
}