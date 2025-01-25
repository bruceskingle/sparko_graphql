use std::error::Error;

// Example custom build script.
fn main()  -> Result<(), Box<dyn Error>> {
   
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=graphql_builder/src/lib.rs");


        // sparko_graphql_builder::builder("swapi")
        // .with_schema("../sparko_graphql_builder_test/graphql/schema/swapi.graphql")
        // .with_query("../sparko_graphql_builder_test/graphql/query/swapi/get-luke.graphql", "luke")
        // // .with_query("../sparko_graphql_builder_test/graphql/query/swapi/fragment.graphql", "fragment")
        // .build()?;

        sparko_graphql_builder::builder("example")
        .with_schema("../sparko_graphql_builder_test/graphql/schema/example.graphql")
        .with_query("../sparko_graphql_builder_test/graphql/query/example/get-luke.graphql", "get-luke")
        .with_query("../sparko_graphql_builder_test/graphql/query/example/get-logged-in-user.graphql", "get-logged-in-user")
        .build()?;


    sparko_graphql_builder::builder("swapi")
        .with_schema("../sparko_graphql_builder_test/graphql/schema/swapi.graphql")
        .with_query("../sparko_graphql_builder_test/graphql/query/swapi/get-luke.graphql", "luke")
        .with_query("../sparko_graphql_builder_test/graphql/query/swapi/get-person.graphql", "person")
        .with_query("../sparko_graphql_builder_test/graphql/query/swapi/get-luke-strikes-back.graphql", "strikes")
        // .with_query("../sparko_graphql_builder_test/graphql/query/swapi/fragment.graphql", "fragment")
        .build()?;

    sparko_graphql_builder::builder("octopus")
        .with_schema("../sparko_graphql_builder_test/graphql/octopus/schema.graphql")
        .with_query("../sparko_graphql_builder_test/graphql/octopus/main.graphql", "main")
        .build()?;
    Ok(())
}
