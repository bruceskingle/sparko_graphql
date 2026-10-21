use std::fmt::{self, Display};
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};

use crate::Error;

/// An ISO 4217 currency, serialized as its alphabetic code (e.g. "GBP")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Currency(pub(crate) iso_currency::Currency);

impl Currency {
  /// The ISO 4217 code for "No currency" (XXX)
  pub const NONE: Currency = Currency(iso_currency::Currency::XXX);

  pub fn from_code(code: &str) -> Option<Currency> {
    iso_currency::Currency::from_code(code).map(Currency)
  }

  /// The ISO 4217 alphabetic code, e.g. "GBP"
  pub fn code(&self) -> &'static str {
    self.0.code()
  }

  /// The English name, e.g. "Pound Sterling"
  pub fn name(&self) -> &str {
    self.0.name()
  }

  /// The major unit symbol, e.g. "£"
  pub fn symbol(&self) -> String {
    self.0.symbol().symbol
  }

  /// The minor unit symbol, if there is one, e.g. "p"
  pub fn subunit_symbol(&self) -> Option<String> {
    self.0.symbol().subunit_symbol
  }

  /// The number of decimal places between the minor and major units, e.g. 2 for GBP, 0 for JPY.
  /// Currencies with no minor unit defined (e.g. XAU) return 0.
  pub fn minor_units(&self) -> u32 {
    self.0.exponent().unwrap_or(0) as u32
  }
}

impl Deref for Currency {
    type Target = iso_currency::Currency;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.pad(self.0.code())
    }
}

impl FromStr for Currency {
    type Err = Error;

    fn from_str(str: &str) -> Result<Currency, Self::Err> {
      Ok(Currency(iso_currency::Currency::from_str(str)?))
    }
}

struct CurrencyVisitor;

impl<'de> Visitor<'de> for CurrencyVisitor {
    type Value = Currency;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an ISO 4217 currency code")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
      match Currency::from_str(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(E::custom(format!("Invalid Currency value '{}': {}", value, error)))
      }
    }
}

impl<'de> serde::Deserialize<'de> for Currency {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    deserializer.deserialize_str(CurrencyVisitor)
  }
}

impl Serialize for Currency {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    serializer.serialize_str(self.0.code())
  }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct MyStruct {
        value: Currency,
    }

    #[test]
    fn test_from_str() {
      let value = Currency::from_str("GBP").unwrap();

      assert_eq!(value.0, iso_currency::Currency::GBP);
      assert!(Currency::from_str("XYZ").is_err());
      assert!(Currency::from_str("").is_err());
    }

    #[test]
    fn test_display() {
      assert_eq!(format!("{}", Currency::from_str("GBP").unwrap()), "GBP");
      assert_eq!(format!("{:>5}", Currency::from_str("USD").unwrap()), "  USD");
    }

    #[test]
    fn test_attributes() {
      let gbp = Currency::from_str("GBP").unwrap();

      assert_eq!(gbp.code(), "GBP");
      assert_eq!(gbp.symbol(), "£");
      assert_eq!(gbp.subunit_symbol().as_deref(), Some("p"));
      assert_eq!(gbp.minor_units(), 2);
      assert_eq!(Currency::from_str("JPY").unwrap().minor_units(), 0);
      assert_eq!(Currency::from_str("XAU").unwrap().minor_units(), 0);
    }

    #[test]
    fn test_none() {
      assert_eq!(Currency::from_str("XXX").unwrap(), Currency::NONE);
      assert_eq!(format!("{}", Currency::NONE), "XXX");
      assert_eq!(Currency::NONE.minor_units(), 0);
    }

    #[test]
    fn test_parse() {
      let my_struct: MyStruct = serde_json::from_str(r#"{ "value": "EUR" }"#).unwrap();
      assert_eq!(my_struct.value.0, iso_currency::Currency::EUR);

      assert!(serde_json::from_str::<MyStruct>(r#"{ "value": "Pounds" }"#).is_err());
      assert!(serde_json::from_str::<MyStruct>(r#"{ "value": 826 }"#).is_err());
    }

    #[test]
    fn test_serialize() {
      let value = Currency::from_str("GBP").unwrap();
      assert_eq!(serde_json::to_string(&MyStruct { value }).unwrap(), r#"{"value":"GBP"}"#);
    }
}
