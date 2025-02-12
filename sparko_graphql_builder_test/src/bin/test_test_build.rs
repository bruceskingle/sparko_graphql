use std::{error::Error, sync::Arc};
use std::io::Write;
use sparko_graphql::{NewGraphQLQuery, NewGraphQLResponse, RequestManager};


// include!("/private/tmp/octopus.rs");

async fn test<Q: NewGraphQLQuery<R>, R: NewGraphQLResponse>(query: &Q, request_manager: Option<&RequestManager>, token: Option<&Arc<String>>) -> Result<Option<R>, Box<dyn Error>> {
    {
        println!(r#"query: {},
            variables: {},
            operation_name: {},"#,
            &query.get_query(),
            query.get_variables().unwrap(),
            Q::get_request_name(),
        );

        if let Some(request_manager) = request_manager {
            let response = request_manager.call(query, token).await?;
            println!("Result {}", serde_json::to_string_pretty(&response)?);
            Ok(Some(response))
        }
        else {
            Ok(None)
        }
        
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
