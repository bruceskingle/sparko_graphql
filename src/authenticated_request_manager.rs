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

use serde::de::DeserializeOwned;

use crate::{GraphQLQueryParams, GraphQLType, NewGraphQLQuery, NewGraphQLResponse, TokenManager};

use crate::RequestManager;


// #[derive(Debug)]
pub struct AuthenticatedRequestManager<M: TokenManager> {
    request_manager: Arc<RequestManager>,
    token_manager: M,
}

impl<M: TokenManager> AuthenticatedRequestManager<M> {
    pub fn new(request_manager: Arc<RequestManager>, token_manager: M) -> Result<AuthenticatedRequestManager<M>, Box<dyn std::error::Error>> {
        Ok(AuthenticatedRequestManager {
            request_manager,
            token_manager,
        })
    }

    pub async fn call<Q: NewGraphQLQuery<R>, R: NewGraphQLResponse>(&mut self, query: &Q) 
    -> Result<R, Box<dyn std::error::Error>> {
        let token = &self.token_manager.get_authenticator().await?;

        self.request_manager.call(query, Some(token)).await
    }

    pub async fn query<P: GraphQLQueryParams, T: GraphQLType<P> + DeserializeOwned>(&mut self, request_name: &str, query_name: &str, params: P) 
    -> Result<T, Box<dyn std::error::Error>> {
        // let x = self.request_manager.call::<P,T>(request_name, query_name, params, Some(&self.token_manager.get_authenticator().await?)).await?;

        let token = &self.token_manager.get_authenticator().await?;

        // eprintln!("AuthenticatedRequestManager token=<{}>", token);

        let result = self.request_manager.do_query::<P,T>(request_name, query_name, params, Some(token)).await;
        
        
        //.call::<P,T>(operation_name, variables, Some(token)).await;

        // if let Err(e) = &result {
        //     eprintln!("Result {:?}", e);
        // }

        // if let Ok(v) = &result {
        //     eprintln!("Result {:?}", v);
        // }
        
        result
    }

    pub async fn mutation<P: GraphQLQueryParams, T: GraphQLType<P> + DeserializeOwned>(&mut self, request_name: &str, query_name: &str, params: P) 
    -> Result<T, Box<dyn std::error::Error>> {
        // let x = self.request_manager.call::<P,T>(request_name, query_name, params, Some(&self.token_manager.get_authenticator().await?)).await?;

        let token = &self.token_manager.get_authenticator().await?;

        eprintln!("AuthenticatedRequestManager token=<{}>", token);

        // let request_manager = *self.request_manager;

        let result = self.request_manager.do_mutation::<P,T>(request_name, query_name, params, Some(token)).await;
        
        
        //.call::<P,T>(operation_name, variables, Some(token)).await;

        if let Err(e) = &result {
            eprintln!("Result {:?}", e);
        }

        // if let Ok(v) = &result {
        //     eprintln!("Result {:?}", v);
        // }
        
        result
    }

    // pub async fn mutation<P: GraphQLQueryParams, T: GraphQLType<P> + DeserializeOwned>(&self, request_name: &str, query_name: &str, params: P, token: Option<&Arc<String>>) 
    // -> Result<T, Box<dyn std::error::Error>> {
    //     self.request_manager.mutation("mutation", request_name, query_name, params, token)
    // }

    // async fn call<P: GraphQLQueryParams, T: GraphQLType<P> + DeserializeOwned>(&self, request_type: &str, request_name: &str, query_name: &str, params: P, token: Option<&Arc<String>>) 
    // -> Result<T, Box<dyn std::error::Error>> {
    //     let token = &self.token_manager.get_authenticator().await?;

    //     eprintln!("AuthenticatedRequestManager token=<{}>", token);
    //     let result = self.request_manager.do_call::<V,E>(operation_name, variables, Some(token)).await;

    //     if let Err(e) = &result {
    //         eprintln!("Result {:?}", e);
    //     }

    //     // if let Ok(v) = &result {
    //     //     eprintln!("Result {:?}", v);
    //     // }
        
    //     result

    //     // if let Some(data) = response.data {
    //     //     Ok(data)
    //     // }
    //     // else {
    //     //     if let Some(errors) = response.errors {
    //     //         Err(Box::new(Error::UserError(serde_json::to_string(&errors)?)))
    //     //     }
    //     //     else {
    //     //         Err(Box::new(Error::InternalError(format!("No result found"))))
    //     //     }
    //     // }
    // }
}