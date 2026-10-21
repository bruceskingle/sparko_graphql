use std::error::Error;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};
use sparko_graphql::GraphQLResponse;

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
struct Foo {}

impl GraphQLResponse for Foo {
}

mod example {
    include!(concat!(env!("OUT_DIR"), "/example.rs"));
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world");

    // let query = example::luke::Query {

    // };
    // let x: swapi::get_luke::Response;

    // let request_manager = RequestManager::new("https://swapi-graphql.eskerda.vercel.app/".to_string())?;

    // let response = request_manager.new_call("Xquery", "GetLuke", "GetLuke", query, None).await?;
    // //let x = swapi::Film { character_connection_: todo!(), created_: todo!(), director_: todo!(), edited_: todo!(), episode_id_: todo!(), id_: todo!(), opening_crawl_: todo!(), planet_connection_: todo!(), producers_: todo!(), release_date_: todo!(), species_connection_: todo!(), starship_connection_: todo!(), title_: todo!(), vehicle_connection_: todo!() };

    // println!("Result {}", serde_json::to_string_pretty(&response)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use sparko_graphql::types::Currency;

    use super::example::example::get_example::get_luke::Response;

    #[test]
    fn test_currency_field() {
        let response: Response = serde_json::from_str(r#"{
            "oneOfEverything": {
                "string": "s",
                "optionalAmount": null,
                "optionalString": null,
                "currency": "GBP",
                "optionalCurrency": null
            }
        }"#).unwrap();
        let one_of_everything = response.one_of_everything_.unwrap();

        assert_eq!(one_of_everything.currency_, "GBP".parse::<Currency>().unwrap());
        assert_eq!(one_of_everything.currency_.symbol(), "£");
        assert_eq!(one_of_everything.optional_currency_, None);

        let serialized = serde_json::to_string(&one_of_everything).unwrap();
        assert!(serialized.contains(r#""currency":"GBP""#), "{}", serialized);
    }

    #[test]
    fn test_invalid_currency_field() {
        let result: Result<Response, serde_json::Error> = serde_json::from_str(r#"{
            "oneOfEverything": {
                "string": "s",
                "currency": "Pounds"
            }
        }"#);

        assert!(result.is_err());
    }
}
