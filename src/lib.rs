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

pub mod error;

use std::collections::HashMap;

use display_json::DisplayAsJsonPretty;
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use error::{Error, GraphQLJsonError};


pub mod types;
mod traits;
pub use traits::{ParamBuffer,VariableBuffer,GraphQLQueryParams,GraphQLType, GraphQLQuery, GraphQL, NoParams};


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct Request<'a, T>
    where T: Serialize
{
    query:          &'a str,
    variables:      T,
    operation_name:  &'a str,
}



#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct GraphQLResponse {
   errors: Option<Vec<GraphQLJsonError>>,
   data:   HashMap<String, serde_json::Value>,
}


// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// struct NewGraphQLResponse {
//    errors: Option<Vec<GraphQLJsonError>>,
//    data:   serde_json::Value,
// }



// #[derive(Deserialize, Debug)]
// #[serde(rename_all = "camelCase")]
// struct NewGraphQLResponse<'a,T,Q>
// where T: GraphQLType<Q> + Deserialize, Q: GraphQLQueryParams + Deserialize {
//    errors: Option<Vec<GraphQLJsonError>>,
//    data:   T,
//    query: Option<Q>
// }

#[derive(Debug)]
pub struct Client {
    reqwest_client: reqwest::Client,
    url: String,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn new(url: String) -> Client {
        Client {
            reqwest_client: reqwest::Client::new(),
            url,
        }
    }

    /*
    
    {
      "account_bills_transactions_first": 100,
      "account_bills_first": 1,
      "account_bills_onlyCurrentEmail": false,
      "account_bills_fromDate": null,
      "account_bills_issuedToDate": null,
      "account_accountNumber": "A-B3D8B29D",
      "account_bills_transactions_last": null,
      "account_bills_toDate": null,
      "account_bills_transactions_after": null,
      "account_bills_includeBillsWithoutPDF": false,
      "account_bills_includeHistoricStatements": true,
      "account_bills_after": null,
      "account_bills_includeOpenStatements": false,
      "account_bills_includeHeldStatements": false,
      "account_bills_issuedFromDate": null,
      "account_bills_transactions_before": null,
      "account_bills_last": null,
      "account_bills_before": null,
      "account_bills_offset": null
    }
    
     */

    pub async fn new_call<'h, T: GraphQLType<Q> + DeserializeOwned, Q: GraphQLQueryParams>(&self, request_name: &str, query_name: &str, params: Q, headers: Option<&'h HashMap<&'h str, &String>>) -> Result<T, Error> {
        
        
        let query = //T::get_query(request_name, &params);

        format!(r#"
            query {}{} {{
                {}{} {}
            }}
        "#, 
            request_name,
            params.get_formal(),
            query_name,
            params.get_actual(""),
            T::get_query_part(&params, "")
        );

        let variables = params.get_variables()?;

        let payload = Request {
            query: &query,
            variables: &variables,
            operation_name: request_name,
        };

        let serialized = serde_json::to_string(&payload).unwrap();

        println!("NEW payload {}", &serialized);
        println!("NEW query {}", &query);
        println!("NEW variables {}", &variables);

        let mut request = self.reqwest_client.post(&self.url)
            .header("Content-Type", "application/json");

        if let Some(map) = headers {
            
            for (key, value) in map {
                request = request.header(*key, *value);
            }
        }
        
        let response = request
            .body(serialized)
            .send()
            .await?;

        println!("\nStatus:   {:?}", &response.status());

        if &response.status() != &StatusCode::OK {
            let status = response.status();
            let text = &(response).text().await;
            println!("ERROR {}", text.as_ref().expect("No Response Body"));
            return Err(Error::HttpError(status));
        }

        let response_json: serde_json::Value = response.json().await?;

        println!("response {}", serde_json::to_string_pretty(&response_json)?);

        let mut graphql_response: GraphQLResponse = serde_json::from_value(response_json)?;





        // let response_json = response.json().await?;

        // println!("response {:?}", response_json);

        // let graphql_response:  GraphQLResponse = response_json;

        if let Some(errors) = graphql_response.errors {
            
            println!("\nerrors:   {:?}", serde_json::to_string_pretty(&errors)?);

            return Err(Error::GraphQLError(errors));
        }
        
        if let Some(response) = graphql_response.data.remove(query_name) {
            let object: T = serde_json::from_value(response)?;
            Ok(object)
        }
        else {
            return Err(Error::InternalError(format!("No response found")))
        }
    }

    pub async fn call<'h, T>(&self, operation_name: &str, query: &str, variables: &T, headers: Option<&'h HashMap<&'h str, &String>>) -> Result<HashMap<String, serde_json::Value>, Error>
    where T: Serialize
    {
        let payload = Request {
            query,
            variables,
            operation_name,
        };

        let serialized = serde_json::to_string(&payload).unwrap();

        println!("payload {}", &serialized);

        let mut request = self.reqwest_client.post(&self.url)
            .header("Content-Type", "application/json");

        if let Some(map) = headers {
            
            for (key, value) in map {
                request = request.header(*key, *value);
            }
        }
        
        let response = request
            .body(serialized)
            .send()
            .await?;

        println!("\nStatus:   {:?}", &response.status());

        if &response.status() != &StatusCode::OK {
            let status = response.status();
            let text = &(response).text().await;
            println!("ERROR {}", text.as_ref().expect("No Response Body"));
            return Err(Error::HttpError(status));
        }

        let response_json: serde_json::Value = response.json().await?;

        println!("response {}", serde_json::to_string_pretty(&response_json)?);

        let graphql_response: GraphQLResponse = serde_json::from_value(response_json)?;





        // let response_json = response.json().await?;

        // println!("response {:?}", response_json);

        // let graphql_response:  GraphQLResponse = response_json;

        if let Some(errors) = graphql_response.errors {
            
            println!("\nerrors:   {:?}", serde_json::to_string_pretty(&errors)?);

            return Err(Error::GraphQLError(errors));
        }
        
        
        Ok(graphql_response.data)               
    }
}

#[derive(Debug)]
pub struct ClientBuilder {
    url:                Option<String>
}

impl ClientBuilder {

    pub fn new() -> ClientBuilder {
        ClientBuilder {
            url: None,
        }
    }
    pub fn with_url(mut self, url: String) -> Result<ClientBuilder, Error> {
        self.url = Some(url);
        Ok(self)
    }
    
    pub fn with_url_if_not_set(mut self, url: String) -> Result<ClientBuilder, Error> {
        if self.url == None {
            self.url = Some(url);
        }
        Ok(self)
    }

    pub fn build(self) -> Result<Client, Error> {
        Ok(Client::new(self.url.unwrap()))
    }
}