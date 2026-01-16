use std::sync::Arc;

use crate::{Error, GraphQLQuery, GraphQLResponse, TokenManager};

use crate::RequestManager;

pub struct AuthenticatedRequestManager<M: TokenManager> {
    request_manager: Arc<RequestManager>,
    token_manager: Arc<M>,
}

impl<M: TokenManager> AuthenticatedRequestManager<M> {
    pub fn new(request_manager: Arc<RequestManager>, token_manager: Arc<M>) -> Result<AuthenticatedRequestManager<M>, Error> {
        Ok(AuthenticatedRequestManager {
            request_manager,
            token_manager,
        })
    }

    pub async fn call<Q: GraphQLQuery<R>, R: GraphQLResponse>(&self, query: &Q) 
    -> Result<R, Error> {
        let token = &self.token_manager.get_authenticator(false).await?;

        self.request_manager.call(query, Some(token)).await
    }

    pub fn get_request_manager(&self) -> Arc<RequestManager> {
        self.request_manager.clone()
    }
}