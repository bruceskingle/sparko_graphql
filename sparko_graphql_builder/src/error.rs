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

use std::error::Error as StdError;
use std::fmt::{self, Display};

use graphql_parser::Pos;

#[derive(Debug)]
pub enum GraphQLError {
    InternalError(Box<dyn std::error::Error>),
    SchemaSyntaxError(graphql_parser::schema::ParseError),
    QuerySyntaxError(graphql_parser::query::ParseError),
    UndefinedTypeError(Pos, String),
    TypeMismatchError(Pos, String),
    MissingInterfaceError(Pos, String),
    MissingFieldError(Pos, String),
    OptionalNonNullFieldError(Pos, String),
    MissingObjectError(Pos, String),
    InvalidQueryError(Pos, String),
    UnsupportedError(Pos, String),
    DuplicateName(Pos, Pos, String),
    NoQueryDefinition,
    ValidationError,
}

impl Display for GraphQLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

impl StdError for GraphQLError {

}

impl From<std::io::Error> for GraphQLError {
    fn from(err: std::io::Error) -> Self {
        GraphQLError::InternalError(Box::new(err))
    }
}

impl From<graphql_parser::schema::ParseError> for GraphQLError {
    fn from(err: graphql_parser::schema::ParseError) -> GraphQLError {
        GraphQLError::SchemaSyntaxError(err)
    }
}

impl From<graphql_parser::query::ParseError> for GraphQLError {
    fn from(err: graphql_parser::query::ParseError) -> GraphQLError {
        GraphQLError::QuerySyntaxError(err)
    }
}