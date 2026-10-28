/*****************************************************************************
MIT License

Copyright (c) 2024 Bruce Skingle

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
******************************************************************************/

pub mod date;
pub use date::Date;

pub mod date_range;
pub use date_range::DateRange;

pub mod datetime;
pub use datetime::DateTime;

pub mod currency;
pub use currency::Currency;

pub mod boolean;
pub use boolean::Boolean;

pub mod id;
pub use id::ID;

pub mod int;
pub use int::Int;

pub mod float;
pub use float::Float;

mod page_info;
pub use page_info::*;

/// Reads a list from a response without any nulls. A list the schema allows to be null is read as an empty list if it is
/// null, and nulls in a list are left out, with a message saying so. Generated code uses this for every list field,
/// which is a Vec of values rather than an Option<Vec> of Options.
pub fn list_without_nulls<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let items = <Option<Vec<Option<T>>> as serde::Deserialize>::deserialize(deserializer)?.unwrap_or_default();
    let count = items.len();
    let items: Vec<T> = items.into_iter().flatten().collect();
    if items.len() < count {
        println!("Left out {} null items of a list of {} in a response", count - items.len(), count);
    }
    Ok(items)
}

#[cfg(test)]
mod list_without_nulls_tests {
    #[derive(serde::Deserialize)]
    struct List {
        #[serde(default, deserialize_with = "super::list_without_nulls")]
        items: Vec<i32>,
    }

    #[test]
    fn null_and_missing_lists_are_empty_and_nulls_in_lists_are_left_out() {
        let items = |json: &str| serde_json::from_str::<List>(json).unwrap().items;
        assert_eq!(items(r#"{"items": [1, 2]}"#), [1, 2]);
        assert!(items(r#"{"items": null}"#).is_empty());
        assert!(items(r#"{}"#).is_empty());
        assert_eq!(items(r#"{"items": [1, null, 2, null]}"#), [1, 2]);
        assert!(items(r#"{"items": [null]}"#).is_empty());
    }
}
