

mod example {
    include!(concat!(env!("OUT_DIR"), "/example.rs"));
}

mod swapi {
    include!(concat!(env!("OUT_DIR"), "/swapi.rs"));
}


fn main() {
    println!("Hello, world");

    let x: OneOfEverything;

    //let x = swapi::Film { character_connection_: todo!(), created_: todo!(), director_: todo!(), edited_: todo!(), episode_id_: todo!(), id_: todo!(), opening_crawl_: todo!(), planet_connection_: todo!(), producers_: todo!(), release_date_: todo!(), species_connection_: todo!(), starship_connection_: todo!(), title_: todo!(), vehicle_connection_: todo!() };
}
