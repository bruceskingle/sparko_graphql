use std::sync::Arc;

use crate::{Error, GraphQLCaller, GraphQLQuery, GraphQLResponse, TokenManager};

use crate::RequestManager;

/// A GraphQLCaller which adds a bearer token from a TokenManager to every request made through an inner caller.
pub struct AuthenticatedRequestManager<M: TokenManager, C: GraphQLCaller = RequestManager> {
    request_manager: Arc<C>,
    token_manager: Arc<M>,
}

impl<M: TokenManager, C: GraphQLCaller> AuthenticatedRequestManager<M, C> {
    pub fn new(request_manager: Arc<C>, token_manager: Arc<M>) -> Result<AuthenticatedRequestManager<M, C>, Error> {
        Ok(AuthenticatedRequestManager {
            request_manager,
            token_manager,
        })
    }

    pub fn get_request_manager(&self) -> Arc<C> {
        self.request_manager.clone()
    }
}

impl<M: TokenManager, C: GraphQLCaller> GraphQLCaller for AuthenticatedRequestManager<M, C> {
    /// A token passed in is used as given, otherwise one is obtained from the token manager.
    fn call2<Q: GraphQLQuery<R>, R: GraphQLResponse>(&self, query: &Q, token: Option<&Arc<String>>)
        -> impl std::future::Future<Output = Result<R, Error>> {
        async move {
            let token = match token {
                Some(token) => token.clone(),
                None => self.token_manager.get_authenticator(false).await?,
            };

            self.request_manager.call2(query, Some(&token)).await
        }
    }
}
