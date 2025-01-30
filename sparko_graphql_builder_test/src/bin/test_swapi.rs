use std::error::Error;
use sparko_graphql::{NewGraphQLQuery, NewGraphQLResponse, RequestManager};


include!(concat!(env!("OUT_DIR"), "/swapi.rs"));

async fn test<Q: NewGraphQLQuery<R>, R: NewGraphQLResponse>(query: &Q, request_manager: Option<&RequestManager>) -> Result<(), Box<dyn Error>> {
    {
        println!(r#"query: {},
            variables: {},
            operation_name: {},"#,
            Q::get_query(),
            query.get_variables().unwrap(),
            Q::get_request_name(),
        );

        if let Some(request_manager) = request_manager {
            let response = request_manager.call(query, None).await?;
            println!("Result {}", serde_json::to_string_pretty(&response)?);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world");

    let the_request_manager: RequestManager = RequestManager::new("https://swapi-graphql.eskerda.vercel.app/".to_string())?;
    let request_manager: Option<&RequestManager> = 
        Some(&the_request_manager);
        // None;


    let query = swapi::luke::get_luke::Query::new();

    

    test(&query, request_manager).await?;

    

    let query = swapi::strikes::get_two_films::Query::new();

    test(&query, request_manager).await?;


    let query = swapi::person::get_person::Query::new("cGVvcGxlOjE=".to_string());

    test(&query, request_manager).await?;

    Ok(())
}
