use std::error::Error;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};
use sparko_graphql::{NewGraphQLResponse, RequestManager};

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
struct Foo {}

impl NewGraphQLResponse for Foo {
}

mod example {
    include!(concat!(env!("OUT_DIR"), "/example.rs"));
}

mod swapi {
    include!(concat!(env!("OUT_DIR"), "/swapi.rs"));
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world");

    // let query = swapi::get_luke::Query {

    // };
    // let x: swapi::get_luke::Response;

    // let request_manager = RequestManager::new("https://swapi-graphql.eskerda.vercel.app/".to_string())?;

    // let response = request_manager.new_call("Xquery", "GetLuke", "GetLuke", query, None).await?;
    // //let x = swapi::Film { character_connection_: todo!(), created_: todo!(), director_: todo!(), edited_: todo!(), episode_id_: todo!(), id_: todo!(), opening_crawl_: todo!(), planet_connection_: todo!(), producers_: todo!(), release_date_: todo!(), species_connection_: todo!(), starship_connection_: todo!(), title_: todo!(), vehicle_connection_: todo!() };

    // println!("Result {}", serde_json::to_string_pretty(&response)?);

    Ok(())
}
