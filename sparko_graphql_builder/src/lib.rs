use error::GraphQLError;
use graphql_parser::parse_query;
use graphql_parser::schema::parse_schema;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::{env, fs};
use std::io::Write;

mod model;
mod error;

use model::{GraphQLQuerySet, GraphQlModel};

#[cfg(test)]
struct BufOutput {
    buf: Vec<u8>,
    errors: Vec<GraphQLError>,
    warnings: Vec<GraphQLError>,
}

struct FileOutput {
    buf: BufWriter<File>,
    errors: Vec<GraphQLError>,
    warnings: Vec<GraphQLError>,
}

enum Output {
    File(FileOutput),
    #[cfg(test)]
    Buffer(BufOutput),
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        fn fmt_errors(f: &mut std::fmt::Formatter<'_>, name: &str, errors: &Vec<GraphQLError>)  -> std::fmt::Result {
            writeln!(f, "    {}", name)?;
            for item in errors {
                writeln!(f, "        {}", item)?;
            }
            Ok(())
        }

        writeln!(f, "Output")?;
        
        match self {
            Output::File(output) => {
                fmt_errors(f, "Errors", &output.errors)?;
                fmt_errors(f, "Warnings", &output.warnings)?;
            }
            #[cfg(test)]
            Output::Buffer(output) => {
                fmt_errors(f, "Errors", &output.errors)?;
                fmt_errors(f, "Warnings", &output.warnings)?;
                writeln!(f, "    Output")?;
                let buf = &output.buf;
                for item in std::str::from_utf8(buf).unwrap().lines() {
                    writeln!(f, "        {}", item)?;
                }
            },
        }

        
        Ok(())
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Output::File(output) => output.buf.write(buf),
            #[cfg(test)]
            Output::Buffer(output) => output.buf.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        
        match self {
            Output::File(output) => output.buf.flush(),
            #[cfg(test)]
            Output::Buffer(output) => output.buf.flush(),
        }
    }
}

impl From<File> for Output {
    fn from(value: File) -> Self {
        Output::File(FileOutput {
            buf: BufWriter::new(value),
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

impl Output {
    pub fn has_errors(&self) -> bool {
        match self {
            Output::File(output) => output.errors.len()>0,
            #[cfg(test)]
            Output::Buffer(output) => output.errors.len()>0,
        }
    }
    

    pub fn error(&mut self, error: GraphQLError) {
        match self {
            Output::File(output) => output.errors.push(error),
            #[cfg(test)]
            Output::Buffer(output) => output.errors.push(error),
        }
    }

    // pub fn warning(&mut self, error: GraphQLError) {
    //     match self {
    //         Output::File(output) => output.warnings.push(error),
    //         #[cfg(test)]
    //         Output::Buffer(output) => output.warnings.push(error),
    //     }
    // }
}

#[cfg(test)]

impl Output {
    pub fn new() -> Output {
        Output::Buffer(BufOutput {
            buf: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }

    pub fn expect_one_error(&self) -> &GraphQLError {
        self.expect_one(true)
    }

    // pub fn expect_one_warning(&self) -> &GraphQLError {
    //     self.expect_one(false)
    // }

    fn expect_one(&self, error: bool) -> &GraphQLError {
        match self {
            Output::File(output) => Self::do_check_one(error, &output.errors, &output.warnings),
            Output::Buffer(output) => Self::do_check_one(error, &output.errors, &output.warnings),
        }
    }

    fn do_check_one<'a>(error: bool, errors: &'a Vec<GraphQLError>,
        warnings: &'a Vec<GraphQLError>) -> &'a GraphQLError {
        if error {
            if warnings.len() == 0 && errors.len() == 1 {
                errors.get(0).unwrap()
            }
            else {
                panic!("Expected 1 error")
            }
        }
        else {

            if warnings.len() == 1 && errors.len() == 0 {
                warnings.get(0).unwrap()
            }
            else {
                panic!("Expected 1 warning")
            }
        }
    }
}

pub struct Builder {
    model_name: String,
    schemas: Vec<String>,
    queries: Vec<String>,
}

pub fn builder(model_name: impl Into<String>) -> Builder {
    Builder {
        model_name: model_name.into(),
        schemas: Vec::new(),
        queries: Vec::new(),
    }
}

impl Builder {
    pub fn with_schema(&mut self, file_name: &str) -> &mut Builder {
        self.schemas.push(file_name.to_string());

        self
    }

    pub fn with_query(&mut self, file_name: &str) -> &mut Builder {
        self.queries.push(file_name.to_string());

        self
    }
    
    pub fn build(&mut self) {
        match self.do_build() {
            Ok(_) => (),
            Err(error) => panic!("GraohQL Generation failed: {}", error),
        }
    }
    
    fn do_build(&mut self) -> Result<(), Box<dyn Error>> {
            

        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join(format!("{}.rs", self.model_name));

        let file = File::create(dest_path)?;
        let mut out: Output = Output::from(file);

        writeln!(out, 
            r#"
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
"#
        )?;

        
        for file_name in &self.schemas {
            let schema: String = read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            writeln!(out, "// cargo::rerun-if-changed={}", file_name)?;
            
            self.do_build_schema(&mut out, &schema)?;
        }

        for file_name in &self.queries {
            let query: String = read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            writeln!(out, "// cargo::rerun-if-changed={}", file_name)?;
            
            self.do_build_query(&mut out, &query)?;
        }

        if out.has_errors() {
            panic!("GraohQL Generation completed with errors: {}", out);
        }
        Ok(())
    }

    fn do_build_schema(&self, out: &mut Output, schema: &str) -> Result<(), Box<dyn Error>> {
        let ast = parse_schema::<String>(schema)?.to_owned();

        let mut model = GraphQlModel::new(out, ast.definitions)?;
        
        model.validate(out)?;
        model.generate(out)?;

        Ok(())
    }

    fn do_build_query(&self, out: &mut Output, query: &str) -> Result<(), Box<dyn Error>> {
        let ast = parse_query::<String>(query)?.to_owned();

        let mut model = GraphQLQuerySet::new(out, ast.definitions)?;
        
        // model.validate(out)?;
        // model.generate(out)?;

        Ok(())
    }
}

fn read_to_string(file_name: &str) -> Result<String, std::io::Error> {
    let result = fs::read_to_string(file_name);

    if let Err(error) = &result {
        eprintln!("Unable to open file \"{}\" ({})", file_name, error);
    }
    result
}

// fn visit_fields(out: &mut BufWriter<&File>, 
//     object_model: &mut ObjectModel,
//     fields: Vec<graphql_parser::schema::Field<'_, String>>) -> Result<(), Box<dyn Error>> {
//     for f in fields {
//         writeln!(out, "//  field {} type {}", f.name,  visit_type(&f.field_type, false)?)?;

//         object_model.fields.push(FieldModel {
//             name: f.name,
//             ty: visit_type(&f.field_type, false)?,
//         });
        
//     }
//     Ok(())
// }

// fn visit_type(field_type: &graphql_parser::query::Type<'_, String>, non_null: bool) -> Result<String, Box<dyn Error>> {
   
//    Ok( match field_type {
//         graphql_parser::query::Type::NamedType(v) => {
//             let t = match v as &str {
//                 "Boolean" => String::from("bool"),
//                 "Date" => String::from("time::Date"),
//                 "DateTime" => String::from("time::OffsetDateTime"),
//                 "Float" => String::from("f64"),
//                 "ID" => String::from("String"),
//                 "Int" => String::from("i32"),
//                 "String" => { String::from("String")},

//                 _ => { String::from(v)},
//             };

//             if non_null {
//                 t
//             }
//             else {
//                 format!("Option<{}>", t)
//             }
//         },
//         graphql_parser::query::Type::ListType(t) => {
//             format!("Vec<{}>", visit_type(&*t, non_null)?)
//         },
//         graphql_parser::query::Type::NonNullType(t) => {
//             visit_type(&*t, true)?
//         },
//     })
// }

// fn visit_directives(out: &mut BufWriter<&File>, directives: Vec<graphql_parser::query::Directive<'_, String>>) -> Result<(), Box<dyn Error>> {
//     for directive in directives {
//         writeln!(out, "//Directive {}", directive.name)?;

//         for arg in directive.arguments {
//             writeln!(out, "//  Arg {} {}", arg.0, arg.1)?;
//         }
//     }
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema(schema: &str) -> Result<Output, Box<dyn Error>> {

        let mut out = Output::new();
        let builder = builder("test");
        builder.do_build_schema(&mut out, schema)?;
        // let string = out.to_string();

        Ok(out)
    }

    #[test]
    fn test_missing_interface() {
        match test_schema(r#"
type Query implements Foo {
    name: String,
}
"#) 
        {
            Ok(out)  => {
                println!("{}", out);
                if let GraphQLError::MissingInterfaceError(..) = out.expect_one_error() {
                    return;
                }
            },
            Err(error) => {
                println!("{}", error);
            },
        }
        panic!("Expected missing interface");
    }

    #[test]
    fn test_missing_query() {
        match test_schema(r#"
type Object {
    name: String,
}
"#) 
        {
            Ok(out)  => {
                println!("{}", out);
                if let GraphQLError::NoQueryDefinition = out.expect_one_error() {
                    return;
                }
            },
            Err(error) => {
                println!("{}", error);
            },
        }
        panic!("Expected missing interface");
    }

    #[test]
    fn test_missing_mutation() {
        match test_schema(r#"
schema {
    query: Object,
    mutation: Missing
}
type Object {
    name: String,
}
"#) 
        {
            Ok(out)  => {
                println!("{}", out);
                if let GraphQLError::MissingObjectError(..) = out.expect_one_error() {
                    return;
                }
            },
            Err(error) => {
                println!("{}", error);
            },
        }
        panic!("Expected missing interface");
    }

    #[test]
    fn test_missing_subscription() {
        match test_schema(r#"
schema {
    query: Object,
    subscription: Missing
}
type Object {
    name: String,
}
"#) 
        {
            Ok(out)  => {
                println!("{}", out);
                if let GraphQLError::MissingObjectError(..) = out.expect_one_error() {
                    return;
                }
            },
            Err(error) => {
                println!("{}", error);
            },
        }
        panic!("Expected missing interface");
    }

//     #[test]
//     fn test_overlapping_interface() {
//         match test_schema(r#"
// interface Foo {
//     name: String,
// }

// interface Bar {
//     name: String,
// }

// type Object implements Foo & Bar {
//     name: String,
// }
// "#) {
//             Ok(out)  => {
//                 println!("{}", out);
//                 if let GraphQLError::OverlappingInterfaceError(..) = out.expect_one_warning() {
//                     return;
//                 }
//             },
//             Err(error) => {
//                 println!("{}", error);
//             },
//         }
    
//     panic!("Expected OverlappingInterfaceError");
//     }

//     #[test]
//     fn test_incompatible_interface() {
//         match test_schema(r#"
// interface Foo {
//     name: Int,
// }

// type Object implements Foo {
//     name: String,
// }
// "#) {
//             Ok(out)  => {
//                 println!("{}", out);
//                 if let GraphQLError::IncompatibleInterfaceError(..) = out.expect_one_error() {
//                     return;
//                 }
//             },
//             Err(error) => {
//                 println!("{}", error);
//             },
//         }
    
//     panic!("Expected IncompatibleInterfaceError");
//     }
}