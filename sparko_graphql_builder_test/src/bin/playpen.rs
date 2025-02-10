
use std::error::Error;

use serde::{Deserialize, Serialize};
use time::Month;

#[derive(Deserialize, Serialize)]
struct DateTest {
    // #[serde(with = "time::serde::iso8601")]
    #[serde(rename = "PENDING")]
    date_time: time::OffsetDateTime
  }
  
  
     fn main() -> Result<(), Box<dyn Error>> {
        let date_test: DateTest = serde_json::from_str(r#"{
        "date_time": "2025-01-21T21:11:05.398642+00:00",
    }"#)?;
    assert_eq!(date_test.date_time.day(), 21);
    assert_eq!(date_test.date_time.month(), Month::January);
    assert_eq!(date_test.date_time.year(), 2025);
    assert_eq!(date_test.date_time.hour(), 11);
    assert_eq!(date_test.date_time.minute(), 95);

    Ok(())
    }