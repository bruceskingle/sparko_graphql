use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::num::{ParseFloatError, ParseIntError};
use std::sync::Arc;
use display_json::DisplayAsJsonPretty;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

#[derive(Debug, Clone)]
pub enum Error {
    GraphQLError(Arc<Vec<GraphQLJsonError>>),
    IOError(Arc<reqwest::Error>),
    JsonError(Arc<serde_json::Error>),
    /// The status and a readable summary of the response body, which may be empty, see response_summary.
    HttpError(StatusCode, String),
    InvalidInputError(Arc<Box<dyn StdError + Sync + Send>>),
    InvalidResponseError{json: serde_json::Value, error: Arc<serde_json::Error>},
    InternalError(String),
    MissingRequiredValueError(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            
            Error::GraphQLError(err_list) => {
                
                f.write_str("GraphQLError[\n")?;

                match serde_json::to_string_pretty(&err_list.as_ref()) {
                    Ok(json) => {
                        f.write_str(&json)?;
                    },
                    Err(_) => {
                        for err in err_list.as_ref() {
                            f.write_fmt(format_args!(" [{}]\n", err))?;
                        }
                    },
                };
                
                f.write_str("]\n")
            },
            Error::IOError(err) => f.write_fmt(format_args!("IOError({})", err)),
            Error::JsonError(err) => f.write_fmt(format_args!("JsonError({})", err)),
            Error::HttpError(status, message) if message.is_empty() => write!(f, "HttpError({})", status),
            Error::HttpError(status, message) => write!(f, "HttpError({}: {})", status, message),
            Error::InvalidInputError(err) => f.write_fmt(format_args!("InvalidInputError({})", err)),
            Error::InvalidResponseError{json, error} => write!(f, "InvalidResponseError({} {})", 
            if let Ok(str) = serde_json::to_string_pretty(&json) {
                str
            } else {
                json.to_string()
            }, error),
            Error::InternalError(err) => f.write_fmt(format_args!("InternalError({})", err)),
            Error::MissingRequiredValueError(field) => write!(f, "MissingRequiredValueError({})", field),
        }
    }
}

impl StdError for Error {}

/// The longest summary of a response body, in characters.
const MAX_SUMMARY_LENGTH: usize = 1000;

/// A readable summary of the body of an error response for display, without the tags if it is HTML (e.g. a CDN's
/// error page) and with runs of white space replaced by a single space.
pub fn response_summary(body: &str) -> String {
    let text = if body.trim_start().starts_with('<') {
        let mut text = String::new();
        let mut in_tag = false;
        for c in body.chars() {
            match c {
                '<' => {
                    in_tag = true;
                    // Tags separate words, e.g. <BR> and </H1>
                    text.push(' ');
                },
                '>' => in_tag = false,
                c if !in_tag => text.push(c),
                _ => {},
            }
        }
        text.replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&")
    }
    else {
        body.to_string()
    };

    let summary = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if summary.chars().count() > MAX_SUMMARY_LENGTH {
        format!("{}...", summary.chars().take(MAX_SUMMARY_LENGTH).collect::<String>())
    }
    else {
        summary
    }
}

 

impl From<std::str::ParseBoolError> for Error {
    fn from(err: std::str::ParseBoolError) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<ParseFloatError> for Error {
    fn from(err: ParseFloatError) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<time::error::ComponentRange> for Error {
    fn from(err: time::error::ComponentRange) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<time::error::Parse> for Error {
    fn from(err: time::error::Parse) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<iso_currency::ParseCurrencyError> for Error {
    fn from(err: iso_currency::ParseCurrencyError) -> Error {
        Error::InvalidInputError(Arc::new(Box::new(err)))
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::IOError(Arc::new(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JsonError(Arc::new(err))
    }
}


#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub line: i32,
    pub column: i32,
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
       pub message: String,
       pub input_path: Vec<String>
}

// #[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
// #[serde(rename_all = "camelCase")]
// pub struct Extensions {
//     pub error_type: Option<String>,
//     pub error_code: Option<String>,
//     pub error_description: Option<String>,
//     pub error_class: Option<String>,
//     pub validation_errors: Option<Vec<ValidationError>>
// }

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct GraphQLJsonError {
    pub message: Option<String>,
    pub locations: Vec<Location>,
    pub path: Vec<String>,
    pub extensions: IndexMap<String, serde_json::Value>,
}
#[cfg(test)]
mod tests {
    use super::response_summary;

    #[test]
    fn html_error_pages_are_reduced_to_their_text() {
        let page = "<!DOCTYPE HTML>\n<HTML><HEAD><TITLE>ERROR: The request could not be satisfied</TITLE></HEAD><BODY>\n<H1>403 ERROR</H1>Request blocked.<BR clear=\"all\">We can&#39;t connect &amp; more\n</BODY></HTML>";
        assert_eq!(response_summary(page), "ERROR: The request could not be satisfied 403 ERROR Request blocked. We can't connect & more");
    }

    #[test]
    fn other_bodies_have_their_white_space_collapsed_and_long_ones_are_shortened() {
        assert_eq!(response_summary("  {\"error\":\n  \"bad\"}  "), "{\"error\": \"bad\"}");
        assert_eq!(response_summary(""), "");
        assert_eq!(response_summary(&"x".repeat(1001)), format!("{}...", "x".repeat(1000)));
    }
}
