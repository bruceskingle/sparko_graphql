#![cfg(test)]

use crate::{graph_ql_type, graph_ql_type_params};
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

pub fn test_graph_ql_type(input: TokenStream, expect: &str) {
    test_generation(graph_ql_type::derive_graphql_type2(input).unwrap(), expect);
}

pub fn test_graph_ql_query_params(input: TokenStream, expect: &str) {
    test_generation(graph_ql_type_params::derive_graphql_query_params2(input).unwrap(), expect);
}

#[test]
fn test_entity() {
    test_graph_ql_type(quote! {
#[derive(GraphQLType)]
#[graphql(params = "NoParams")]
#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct StatementTotalType {
    pub net_total: Int,
    pub tax_total: Int,
    pub gross_total: Int,
}
    },
r#"impl sparko_graphql::GraphQLType<NoParams> for StatementTotalType {
    fn get_query_attributes(params: &NoParams, prefix: &str) -> String {
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
        params.push_formal(prefix, "APIKey", "String");
        params.push_formal(prefix, "email", "String");
        params.push_formal(prefix, "organizationSecretKey", "String");
        params.push_formal(prefix, "password", "String");
        params.push_formal(prefix, "preSignedKey", "String");
        params.push_formal(prefix, "refreshToken", "String");
    }
    fn get_actual_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
        params.push_actual(prefix, "APIKey");
        params.push_actual(prefix, "email");
        params.push_actual(prefix, "organizationSecretKey");
        params.push_actual(prefix, "password");
        params.push_actual(prefix, "preSignedKey");
        params.push_actual(prefix, "refreshToken");
    }
    fn get_variables_part(
        &self,
        super_variables: &mut serde_json::Map<String, serde_json::Value>,
        prefix: &str,
    ) -> Result<(), serde_json::Error> {
        let variables = super_variables;
        if let Some(_value) = &self.api_key {
            variables
                .insert(
                    format!("{}{}", prefix, "APIKey"),
                    serde_json::to_value(&self.api_key)?,
                );
        }
        if let Some(_value) = &self.email {
            variables
                .insert(
                    format!("{}{}", prefix, "email"),
                    serde_json::to_value(&self.email)?,
                );
        }
        if let Some(_value) = &self.organization_secret_key {
            variables
                .insert(
                    format!("{}{}", prefix, "organizationSecretKey"),
                    serde_json::to_value(&self.organization_secret_key)?,
                );
        }
        if let Some(_value) = &self.password {
            variables
                .insert(
                    format!("{}{}", prefix, "password"),
                    serde_json::to_value(&self.password)?,
                );
        }
        if let Some(_value) = &self.pre_signed_key {
            variables
                .insert(
                    format!("{}{}", prefix, "preSignedKey"),
                    serde_json::to_value(&self.pre_signed_key)?,
                );
        }
        if let Some(_value) = &self.refresh_token {
            variables
                .insert(
                    format!("{}{}", prefix, "refreshToken"),
                    serde_json::to_value(&self.refresh_token)?,
                );
        }
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
r#"impl sparko_graphql::GraphQLQueryParams for ObtainJSONWebTokenInput {
    fn get_formal_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
        params.push_formal(prefix, "input", "ObtainJSONWebTokenInput");
    }
    fn get_actual_part(&self, params: &mut sparko_graphql::ParamBuffer, prefix: &str) {
        params.push_actual(prefix, "input");
    }
    fn get_variables_part(
        &self,
        super_variables: &mut serde_json::Map<String, serde_json::Value>,
        prefix: &str,
    ) -> Result<(), serde_json::Error> {
        let mut variables = serde_json::Map::<String, serde_json::Value>::new();
        if let Some(_value) = &self.api_key {
            variables
                .insert(
                    format!("{}{}", prefix, "APIKey"),
                    serde_json::to_value(&self.api_key)?,
                );
        }
        if let Some(_value) = &self.email {
            variables
                .insert(
                    format!("{}{}", prefix, "email"),
                    serde_json::to_value(&self.email)?,
                );
        }
        if let Some(_value) = &self.organization_secret_key {
            variables
                .insert(
                    format!("{}{}", prefix, "organizationSecretKey"),
                    serde_json::to_value(&self.organization_secret_key)?,
                );
        }
        if let Some(_value) = &self.password {
            variables
                .insert(
                    format!("{}{}", prefix, "password"),
                    serde_json::to_value(&self.password)?,
                );
        }
        if let Some(_value) = &self.pre_signed_key {
            variables
                .insert(
                    format!("{}{}", prefix, "preSignedKey"),
                    serde_json::to_value(&self.pre_signed_key)?,
                );
        }
        if let Some(_value) = &self.refresh_token {
            variables
                .insert(
                    format!("{}{}", prefix, "refreshToken"),
                    serde_json::to_value(&self.refresh_token)?,
                );
        }
        super_variables
            .insert(
                format!("{}{}", prefix, "input"),
                serde_json::Value::Object(variables),
            );
        Ok(())
    }
}
"#);
}