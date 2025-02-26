use std::fmt::{self, Display};
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserializer, Serialize};
use serde::de::{self, Visitor};

use crate::Error;

/// A GraphQL Float value
#[derive(Serialize, Debug)]
pub struct Float(f64);

impl Float {
  pub fn new(s: f64) -> Float {
    Float(s)
  }
}

impl Deref for Float {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}


impl Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.fmt(f)
    }
  }
  
  impl FromStr for Float {
      type Err = Error;
  
      fn from_str(str: &str) -> Result<Float, Self::Err> {
        Ok(Float(str.parse::<f64>()?))
      }
  }
  
  
  struct FloatVisitor;
  
  impl<'de> Visitor<'de> for FloatVisitor {
      type Value = Float;
  
      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
          formatter.write_str("an f64 value")
      }
  
      fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
      where
          E: de::Error,
      {
        Ok(Float::new(value))
      }
  }
  
  
  impl<'de> serde::Deserialize<'de> for Float {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
      deserializer.deserialize_f64(FloatVisitor)
      }
  }
  
  #[cfg(test)]
  mod tests {
      use display_json::DisplayAsJsonPretty;
    use serde::Deserialize;

    use super::*;
  
  
      #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
      struct MyStruct {
          value: Float,
      }
  
      #[test]
      fn test_from_str() {
        let expected = 3.14159;
        let value = Float::from_str("3.14159").unwrap();
  
        assert_eq!(value.0, expected);
      }
  
      fn expect_parse(s: &str, expect: f64) {
        let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
        if let Ok(my_struct) = result {
          assert_eq!(my_struct.value.0, expect);
        }
        else {
          panic!("Expecting {:?} for {}", expect, s);
        }
      }
  
      fn expect_parse_error(s: &str) {
        let result: Result<MyStruct, serde_json::Error> = serde_json::from_str(s);
        if let Ok(_) = result {
          panic!("Expecting error for {}", s);
        }
      }
  
      #[test]
      fn test_parse() {
        
        expect_parse(r#"{ "value": 3.14159 }"#, 3.14159);
        
        expect_parse_error(r#"{ "value": 123 }"#);
        expect_parse_error(r#"{ "value": [1,2,3]] }"#);
        expect_parse_error(r#"{ "value": {} }"#);
      }
  
      #[test]
      fn test_serialize() {
        assert_eq!(serde_json::to_string(&MyStruct {
          value: Float(3.14159)
        }).unwrap(), "{\"value\":3.14159}");
      }
  }