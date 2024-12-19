

mod example {
    include!(concat!(env!("OUT_DIR"), "/example.rs"));
}

mod swapi {
    include!(concat!(env!("OUT_DIR"), "/swapi.rs"));
}


fn main() {
    println!("Hello, world");
}
