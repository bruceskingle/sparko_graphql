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

use std::sync::Arc;


use display_json::DisplayAsJsonPretty;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{Error, GraphQLEntity,  GraphQLResponse, GraphQLVariables};

#[derive(Serialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct Request<'a>
{
    query:          String,
    variables:      String,
    operation_name:  &'a str,
}

// #[derive(Debug)]
pub struct RequestManager {
    reqwest_client: reqwest::Client,
    url: String,
}

impl RequestManager {
    pub fn new(url: String) -> Result<RequestManager, Box<dyn std::error::Error>> {
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
        })
    }


    // pub async fn call<V: GraphQLVariables, E: GraphQLEntity<V>>(&self, query: GraphQLQuery<V,E>) 
    // -> Result<E, Box<dyn std::error::Error>> {
    //     self.do_call::<V,E>(operation_name, variables, None).await
    // }

    pub async fn do_call<V: GraphQLVariables, E: GraphQLEntity<V>>(&self, operation_name: &str, variables: V, token: Option<&Arc<String>>) 
    -> Result<E, Box<dyn std::error::Error>> {

        let query = format!(r#"
            query {}{}
                #T::get_query_part(&params, "")
                {}"#, 
            operation_name,
            variables.get_formal(),
            // query_name,
            // params.get_actual(""),
            E::get_query_part(&variables, "")
        );


        println!("NEW query {}", &query);

        let payload = Request {
            query: query,
            variables: variables.get_variables()?,
            operation_name,
        };
        let serialized = serde_json::to_string(&payload).unwrap();

        println!("NEW payload {}", &serialized);
        println!("NEW variables {}", variables.get_variables()?);
               

        let mut request = self.reqwest_client.post(&self.url.clone());

        if let Some(token) = token {
            request = request.header(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(token)?);
        }

        let response = request
            .body(serialized)
            .send().await?;

        println!("\nStatus:   {:?}", &response.status());

        if &response.status() != &StatusCode::OK {
            let status = response.status();
            let text = &(response).text().await;
            println!("ERROR {}", text.as_ref().expect("No Response Body"));
            return Err(Box::new(Error::HttpError(status)));
        }

        let response_json: serde_json::Value = response.json().await?;

        println!("response {}", serde_json::to_string_pretty(&response_json)?);

        let mut graphql_response: GraphQLResponse = serde_json::from_value(response_json)?;





        // let response_json = response.json().await?;

        // println!("response {:?}", response_json);

        // let graphql_response:  GraphQLResponse = response_json;

        if let Some(errors) = graphql_response.errors {
            
            println!("\nerrors:   {:?}", serde_json::to_string_pretty(&errors)?);

            return Err(Box::new(Error::GraphQLError(errors)));
        }
        
        if let Some(response) = graphql_response.data.remove("query_name") {
            let object: E = serde_json::from_value(response)?;
            Ok(object)
        }
        else {
            return Err(Box::new(Error::InternalError(format!("No response found"))))
        }


    }
}