#![cfg(test)]

use crate::{graph_ql_entity, graph_ql_type_params};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_file;


pub fn test_generation(output: TokenStream, expect: &str) {
    let formatted = prettyplease::unparse(&parse_file(&output.to_string()).unwrap());

    if !(formatted == *expect) {
        eprintln!("EXPECTED>{}<EXPECTED", format!("{}", expect));

        eprintln!("output>{}<output", format!("{}", formatted));

        panic!("Unexpected output");
    }
}

pub fn test_graph_ql_entity(input: TokenStream, expect: &str) {
    test_generation(graph_ql_entity::derive_graphql_entity2(input).unwrap(), expect);
}

pub fn test_graph_ql_query_params(input: TokenStream, expect: &str) {
    test_generation(graph_ql_type_params::derive_graphql_query_params2(input).unwrap(), expect);
}

#[test]
fn test_entity() {
    test_graph_ql_entity(quote! {
        pub struct StatementTotalType {
            pub net_total: Int,
            pub tax_total: Int,
            pub gross_total: Int,
        }
    },
r#"type StatementTotalTypeVariables = NoVariables;
impl sparko_graphql::GraphQLEntity<StatementTotalTypeVariables> for StatementTotalType {
    fn get_query_attributes(
        params: &StatementTotalTypeVariables,
        prefix: &str,
    ) -> String {
        format!("netTotal\ntaxTotal\ngrossTotal\n",)
    }
}
"#);
}




#[test]
fn test_param() {
    test_graph_ql_query_params(quote! {
#[derive(GraphQLQueryParams)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObtainJSONWebTokenInput {
    // "API key of the account user. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "APIKey")]
    api_key: Option<String>,
    // "Email address of the account user. Use with 'password' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    // // "Live secret key of an third-party organization. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    organization_secret_key: Option<String>,
    // // "Password of the account user. Use with 'email' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    // // "Short-lived, temporary key (that's pre-signed). Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    pre_signed_key: Option<String>,
    // // "The refresh token that can be used to extend the expiry claim of a Kraken token. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}
    },
r#"impl sparko_graphql::GraphQLQueryParams for ObtainJSONWebTokenInput {
    fn get_formal_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
        params.push_formal(prefix, "apiKey", "String");
        params.push_formal(prefix, "email", "String");
        params.push_formal(prefix, "organizationSecretKey", "String");
        params.push_formal(prefix, "password", "String");
        params.push_formal(prefix, "preSignedKey", "String");
        params.push_formal(prefix, "refreshToken", "String");
    }
    fn get_actual_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
        params.push_actual(prefix, "apiKey");
        params.push_actual(prefix, "email");
        params.push_actual(prefix, "organizationSecretKey");
        params.push_actual(prefix, "password");
        params.push_actual(prefix, "preSignedKey");
        params.push_actual(prefix, "refreshToken");
    }
    fn get_variables_part(
        &self,
        variables: &mut sparko_graphql::VariableBuffer,
        prefix: &str,
    ) -> Result<(), serde_json::Error> {
        variables.push_variable(prefix, "apiKey", &self.api_key)?;
        variables.push_variable(prefix, "email", &self.email)?;
        variables
            .push_variable(
                prefix,
                "organizationSecretKey",
                &self.organization_secret_key,
            )?;
        variables.push_variable(prefix, "password", &self.password)?;
        variables.push_variable(prefix, "preSignedKey", &self.pre_signed_key)?;
        variables.push_variable(prefix, "refreshToken", &self.refresh_token)?;
        Ok(())
    }
}
"#);
}

#[test]
fn test_object_param() {
    test_graph_ql_query_params(quote! {
#[derive(GraphQLQueryParams)]
#[graphql(as_object)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ObtainJSONWebTokenInput {
    // "API key of the account user. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "APIKey")]
    api_key: Option<String>,
    // "Email address of the account user. Use with 'password' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    // // "Live secret key of an third-party organization. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    organization_secret_key: Option<String>,
    // // "Password of the account user. Use with 'email' field."
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,
    // // "Short-lived, temporary key (that's pre-signed). Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    pre_signed_key: Option<String>,
    // // "The refresh token that can be used to extend the expiry claim of a Kraken token. Use standalone, don't provide a second input field."
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
}
    },
r#"type StatementTotalTypeVariables = NoVariables;
impl sparko_graphql::GraphQLEntity<StatementTotalTypeVariables> for StatementTotalType {
    fn get_query_attributes(
        params: &StatementTotalTypeVariables,
        prefix: &str,
    ) -> String {
        format!("netTotal\ntaxTotal\ngrossTotal\n",)
    }
}
"#);
}