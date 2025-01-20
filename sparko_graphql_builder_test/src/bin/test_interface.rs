use std::error::Error;
use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
struct Film {
    id: String,
    title: String,
    episode_id: i32,
    release_date: String,
    director: String,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
struct FilmSummary<'a> {
    id: &'a str,
    title: &'a str,
}

impl Film {
    pub fn as_film_summary<'a>(&'a self) -> FilmSummary<'a> {
        FilmSummary {
            id: &self.id,
            title: &self.title,
        }

    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let film = Film {
        id: "ZmlsbXM6Mg==".to_string(),
        title: "The Empire Strikes Back".to_string(),
        episode_id: 5,
        release_date: "1980-05-17".to_string(),
        director: "Irvin Kershner".to_string()
    };

    println!("Film {}", serde_json::to_string_pretty(&film)?);
    println!("Summary {}", serde_json::to_string_pretty(&film.as_film_summary())?);

    Ok(())
}