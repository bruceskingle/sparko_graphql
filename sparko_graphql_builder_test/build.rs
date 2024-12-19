use std::error::Error;

// Example custom build script.
fn main()  -> Result<(), Box<dyn Error>> {
   
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=graphql_builder/src/lib.rs");

    sparko_graphql_builder::builder("example")
        .with_schema("graphql/schema/example.graphql")
        .build();


        sparko_graphql_builder::builder("swapi")
        .with_schema("graphql/schema/swapi.graphql")
        .with_query("graphql/query/swapi/get-luke.graphql")
        .build();

    // panic!("Panic test!"); 
    Ok(())
}
