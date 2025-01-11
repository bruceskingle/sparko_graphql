use std::{error::Error, process};

// fn get_current_working_dir() -> String {
//     let res = std::env::current_dir();
//     match res {
//         Ok(path) => path.into_os_string().into_string().unwrap(),
//         Err(_) => "FAILED".to_string()
//     }
// }
fn main() {
    if let Err(_) = build() {
        process::exit(1);
        // panic!("Build failed {}", error);
    }
}

// Example custom build script.
fn build()  -> Result<(), Box<dyn Error>> {
    // panic!("Panic cwd={}", get_current_working_dir()); 

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=graphql_builder/src/lib.rs");

    sparko_graphql_builder::builder("example")
        .with_schema("graphql/schema/example.graphql")
        .with_query("graphql/query/example/get-luke.graphql", "get-luke")
        .build()?;


        sparko_graphql_builder::builder("swapi")
        .with_schema("graphql/schema/swapi.graphql")
        .with_query("graphql/query/swapi/get-luke.graphql", "luke")
        .with_query("graphql/query/swapi/get-person.graphql", "person")
        .with_query("graphql/query/swapi/get-luke-strikes-back.graphql", "strikes")
        .build()?;

    // panic!("Panic test!"); 
    Ok(())
}
