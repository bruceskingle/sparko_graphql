use std::error::Error;
// use sparko_graphql::{NewGraphQLQuery, NewGraphQLResponse, RequestManager};


include!(concat!(env!("OUT_DIR"), "/example.rs"));

// async fn test<Q: NewGraphQLQuery<R>, R: NewGraphQLResponse>(query: &Q, request_manager: Option<&RequestManager>) -> Result<(), Box<dyn Error>> {
//     {
//         // println!(r#"query: {},
//         //     variables: {},
//         //     operation_name: {},"#,
//         //     &query.get_query(),
//         //     query.get_variables().unwrap(),
//         //     Q::get_request_name(),
//         // );

//         if let Some(request_manager) = request_manager {
//             let response = request_manager.call(query, None).await?;
//             println!("Result {}", serde_json::to_string_pretty(&response)?);
//         }

//         Ok(())
//     }
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world");

    // let the_request_manager: RequestManager = RequestManager::new("https://swapi-graphql.eskerda.vercel.app/".to_string(), true)?;
    // let request_manager: Option<&RequestManager> = 
    //     Some(&the_request_manager);
    //     // None;


    let query = example::get_example::get_luke::Query::new();

    

    // test(&query, request_manager).await?;

    

    // let query = swapi::strikes::get_two_films::Query::new();

    // test(&query, request_manager).await?;


    // let query = swapi::person::get_person::Query::new("cGVvcGxlOjE=".to_string());

    // test(&query, request_manager).await?;

    

    // if let Some(request_manager) = request_manager {
    //     {
    //         let query = swapi::pagination::get_fim_people::Query::new();
    //         let response = request_manager.call(&query, None).await?;
    //         println!("Result {}", serde_json::to_string_pretty(&response)?);

    //         for edge in &response.all_films_.edges_ {

    //             println!("{}", &edge.node.title_);
    //             println!("{}", serde_json::to_string_pretty(&edge.node)?);
    //             println!();

    //             for edge in &edge.node.character_connection_.edges_ {
    //                 println!("{}", &edge.node.name_);
    //                 println!("{}", serde_json::to_string_pretty(&edge.node)?);
    //                 println!();
    //             }
    //         }
    //     }

    //     {
    //         let query = swapi::pagination::get_pagable_fim_people::Query::new();
    //         let response = request_manager.call(&query, None).await?;
    //         println!("Result {}", serde_json::to_string_pretty(&response)?);

    //         for film in &response.all_films_ {

    //             println!("{}", &film.title_);
    //             println!("{}", serde_json::to_string_pretty(&film)?);
    //             println!();

    //             for person in &film.character_connection_ {
    //                 println!("{}", &person.name_);
    //                 println!("{}", serde_json::to_string_pretty(&person)?);
    //                 println!();
    //             }
    //         }
    //     }
    // }

    Ok(())
}
