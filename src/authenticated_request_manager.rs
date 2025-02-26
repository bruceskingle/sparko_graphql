use std::sync::Arc;

use crate::{NewGraphQLQuery, NewGraphQLResponse, TokenManager};

use crate::RequestManager;

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

    pub async fn call<Q: NewGraphQLQuery<R>, R: NewGraphQLResponse>(&self, query: &Q) 
    -> Result<R, Box<dyn std::error::Error>> {
        let token = &self.token_manager.get_authenticator(false).await?;

        self.request_manager.call(query, Some(token)).await
    }
}