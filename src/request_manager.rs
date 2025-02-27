use std::sync::Arc;


use display_json::DisplayAsJsonPretty;
use reqwest::StatusCode;
use serde::Serialize;

use crate::{Error, GraphQLResponseStructure, GraphQLQuery, GraphQLResponse};

pub const DASHES: &str = "=======================================================================================================================";

#[derive(Serialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct NewRequest<'a>
{
    query:          &'a str,
    variables:      String,
    operation_name:  &'a str,
}

// #[derive(Debug)]
pub struct RequestManager {
    reqwest_client: reqwest::Client,
    url: String,
    verbose: bool,
}

impl RequestManager {
    pub fn new(url: String, verbose: bool) -> Result<RequestManager, Box<dyn std::error::Error>> {
        Ok(RequestManager {
            reqwest_client: reqwest::Client::builder()
                .user_agent("sparko_graphql/0.0.1")
                .default_headers(
                    std::iter::once((
                        reqwest::header::CONTENT_TYPE,
                        reqwest::header::HeaderValue::from_str("application/json")?
                    ))
                    .collect(),
                )
                .build()?,
            url,
            verbose,
        })
    }

    fn report_error(message: &str) {

        println!("\n\n\n{} {}", message, DASHES);
    }

    pub async fn call<Q: GraphQLQuery<R>, R: GraphQLResponse>(&self, query: &Q, token: Option<&Arc<String>>) 
    -> Result<R, Box<dyn std::error::Error>> {

        // println!("NEW query {}", &query);

        let payload = NewRequest {
            query: &query.get_query(),
            variables: query.get_variables()?,
            operation_name: Q::get_request_name(),
        };

        if self.verbose {
            println!("\n{}\rQuery ", DASHES);
            println!("{}", &payload.query);
            println!("\n{}\rVariables ", DASHES);
            println!("{}", &payload.variables);
            println!("{}", DASHES);
        }
        // let serialized = serde_json::to_string(&payload).unwrap();

        // println!("NEW payload {}", &serialized);
        // panic!("Don't send");
        // println!("NEW variables {}", params.get_variables()?);
               

        let mut request = self.reqwest_client.post(&self.url.clone());

        if let Some(token) = token {
            request = request.header(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(token)?);
        }

        let response = request
            .body(serde_json::to_string(&payload).unwrap())
            .send().await?;

        if &response.status() != &StatusCode::OK {
            let status = response.status();
            Self::report_error("ERROR Request Failed");
            println!("HTTP status {}", status);
            
            Self::report_error("Query");
            println!("{}", &query.get_query());
            
            Self::report_error("Variables");
            println!("{}", query.get_variables()?);
            
            Self::report_error("Payload");
            println!("{}",  &serde_json::to_string(&payload).unwrap());

            let text = &(response).text().await;
            println!("ERROR {}", text.as_ref().expect("No Response Body"));
            return Err(Box::new(Error::HttpError(status)));
        }

        let response_json: serde_json::Value = response.json().await?;

        if self.verbose {
            println!("\n{}\rResponse ", DASHES);
            println!("{}", serde_json::to_string_pretty(&response_json)?);
            println!("{}", DASHES);
        }

        // println!("response_json {}", serde_json::to_string_pretty(&response_json)?);

        let graphql_response: GraphQLResponseStructure = serde_json::from_value(response_json)?;

        // println!("graphql_response {}", serde_json::to_string_pretty(&graphql_response)?);



        if let Some(errors) = graphql_response.errors {
            
            Self::report_error("GraphQL Errors");
            println!("{:?}", serde_json::to_string_pretty(&errors)?);

            return Err(Box::new(Error::GraphQLError(errors)));
        }
        // println!("query_name {}", &query_name);
        // if let Some(response) = graphql_response.data {
        //     println!("response {}", serde_json::to_string_pretty(&response)?);

        
            let json = graphql_response.data.clone();
            let object: R = match serde_json::from_value(graphql_response.data) {
                Ok(object) => object,
                Err(error) => {
                    // println!("Deserialization error {}", error);
                    // println!("response_json {}", serde_json::to_string_pretty(&json)?);
                    return Err(Box::new(Error::InvalidResponseError{json, error}))
                },
            };
            Ok(object)
        // }
        // else {
        //     return Err(Box::new(Error::InternalError(format!("No response found"))))
        // }


    }
}