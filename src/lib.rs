pub mod error;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

pub use error::{Error, GraphQLJsonError};


pub mod types;
mod traits;
pub use traits::{ParamBuffer,GraphQLQueryParams,GraphQLType,TokenManager, GraphQL, NoParams};

mod request_manager;
pub use request_manager::RequestManager;
mod authenticated_request_manager;
pub use authenticated_request_manager::AuthenticatedRequestManager;

pub trait GraphQLQuery<R: GraphQLResponse> {
    fn get_query(&self) -> String;
    fn get_request_name() -> &'static str;
    fn get_variables(&self) -> Result<std::string::String, serde_json::Error>;
}

pub trait GraphQLResponse: serde::de::DeserializeOwned + Serialize {

}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
struct GraphQLResponseStructure {
   errors: Option<Vec<GraphQLJsonError>>,
   data:   serde_json::Value,
}