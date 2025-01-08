use error::GraphQLError;
use graphql_parser::parse_query;
use graphql_parser::schema::parse_schema;
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::rc::Rc;
use std::{env, fs};
use std::io::Write;

mod utils;
mod parsed_model;
mod validated_model;
mod error;



// mod validated_model {
//     pub struct Schema {}

//     impl Schema {
//         pub(crate) fn new(model: crate::parsed_model::Schema, out: &mut crate::Output<'_>) -> Result<Self, crate::error::GraphQLError> {
//             Ok(Self {  })
//         }
        
//         pub(crate) fn print(&self, out: &mut crate::Output<'_>) -> Result<(), crate::error::GraphQLError> {
//             Ok(())
//         }
        
//         pub(crate) fn generate(&self, out: &mut crate::Output<'_>) -> Result<(), crate::error::GraphQLError> {
//             Ok(())
//         }
//     }

//     pub struct Operations {}

//     impl Operations {

        
//         pub(crate) fn print(&self, out: &mut crate::Output<'_>) -> Result<(), crate::error::GraphQLError> {
//             Ok(())
//         }
        
//         pub(crate) fn generate(&self, out: &mut crate::Output<'_>) -> Result<(), crate::error::GraphQLError> {
//             Ok(())
//         }
        
//         pub(crate) fn new(model: crate::parsed_model::Operations, out: &mut crate::Output<'_>, schema: &Schema) -> Result<Self, crate::error::GraphQLError> {
//             Ok(Self {  })
//         }
//     }
// }

pub struct Output<'a> {
    base: &'a mut BaseOutput,
    indent: usize,
    start_of_line: bool,
}

impl Output<'_> {
    pub fn indent(&mut self) -> Output {
        Output {
            base: self.base,
            indent: self.indent + 1,
            start_of_line: true,
        }
    }

    pub fn error(&mut self, error: GraphQLError) {
        self.base.error(error);
    }

    // pub fn warning(&mut self, error: GraphQLError) {
    //     self.base.warning(error);
    // }
}

impl Write for Output<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut nbytes: usize = 0;
        let mut s = 0;

        while s< buf.len() {
            if self.start_of_line {
                let mut t: usize = 0;

                while t < self.indent {
                    self.base.write(b"    ")?;
                    t += 1;
                }

                self.start_of_line = false;
            }
            let mut e = s;
            while e < buf.len() && buf[e] != b'\n' {
                e += 1;
            }

            if e < buf.len() {
                e += 1;
                nbytes += self.base.write(&buf[s..e])?;

                self.start_of_line = true;
            }
            else {
                nbytes += self.base.write(&buf[s..e])?;
            }
            s = e;
        }
        Ok(nbytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.base.flush()
    }
}

impl Display for Output<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base.fmt(f)
    }
}

#[cfg(test)]
struct TestOutput {
    buf: Vec<u8>,
    errors: Vec<GraphQLError>,
    warnings: Vec<GraphQLError>,
}

struct FileOutput {
    buf: BufWriter<File>,
    errors: Vec<GraphQLError>,
    warnings: Vec<GraphQLError>,
}

enum BaseOutput {
    File(FileOutput),
    #[cfg(test)]
    Buffer(TestOutput),
}

impl Display for BaseOutput {
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
            BaseOutput::File(output) => {
                fmt_errors(f, "Errors", &output.errors)?;
                fmt_errors(f, "Warnings", &output.warnings)?;
            }
            #[cfg(test)]
            BaseOutput::Buffer(output) => {
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

impl Write for BaseOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            BaseOutput::File(output) => output.buf.write(buf),
            #[cfg(test)]
            BaseOutput::Buffer(output) => output.buf.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        
        match self {
            BaseOutput::File(output) => output.buf.flush(),
            #[cfg(test)]
            BaseOutput::Buffer(output) => output.buf.flush(),
        }
    }
}

impl From<File> for BaseOutput {
    fn from(value: File) -> Self {
        BaseOutput::File(FileOutput {
            buf: BufWriter::new(value),
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }
}

impl BaseOutput {
    pub fn has_errors(&self) -> bool {
        match self {
            BaseOutput::File(output) => output.errors.len()>0,
            #[cfg(test)]
            BaseOutput::Buffer(output) => output.errors.len()>0,
        }
    }
    

    pub fn error(&mut self, error: GraphQLError) {
        match self {
            BaseOutput::File(output) => output.errors.push(error),
            #[cfg(test)]
            BaseOutput::Buffer(output) => output.errors.push(error),
        }
    }

    pub fn indent(&mut self) -> Output {
        Output {
            base: self,
            indent: 0,
            start_of_line: true,
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

impl BaseOutput {
    pub fn new() -> BaseOutput {
        BaseOutput::Buffer(TestOutput {
            buf: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        })
    }

    pub fn expect_ok(&self) {
        match self {
            BaseOutput::File(output) => Self::do_check_ok(self, &output.errors, &output.warnings),
            BaseOutput::Buffer(output) => Self::do_check_ok(self, &output.errors, &output.warnings),
        }
    }
    
    fn do_check_ok<'a>(self: &BaseOutput, errors: &'a Vec<GraphQLError>,
        warnings: &'a Vec<GraphQLError>) {
        if warnings.len() != 0 || errors.len() != 0 {
            println!("{}", self);
            panic!("Expected OK");
        }
    }

    pub fn expect_one_error(&self) -> Option<&GraphQLError> {
        self.expect_one(true)
    }

    // pub fn expect_one_warning(&self) -> &GraphQLError {
    //     self.expect_one(false)
    // }

    fn expect_one(&self, error: bool) -> Option<&GraphQLError> {
        match self {
            BaseOutput::File(output) => Self::do_check_one(error, &output.errors, &output.warnings),
            BaseOutput::Buffer(output) => Self::do_check_one(error, &output.errors, &output.warnings),
        }
    }

    fn do_check_one<'a>(error: bool, errors: &'a Vec<GraphQLError>,
        warnings: &'a Vec<GraphQLError>) -> Option<&'a GraphQLError> {
        if error {
            if warnings.len() == 0 && errors.len() == 1 {
                errors.get(0)
            }
            else {
                None
            }
        }
        else {

            if warnings.len() == 1 && errors.len() == 0 {
                warnings.get(0)
            }
            else {
                None
            }
        }
    }
}

pub struct Builder {
    model_name: String,
    schema: Option<String>,
    queries: Vec<String>,
}

pub fn builder(model_name: impl Into<String>) -> Builder {
    Builder {
        model_name: model_name.into(),
        schema: None,
        queries: Vec::new(),
    }
}

impl Builder {
    pub fn with_schema(&mut self, file_name: &str) -> &mut Builder {
        if let Some(schema) = &self.schema {
            panic!("Multiple schemas defined ({} and {})", schema, file_name);
        }
        self.schema = Some(file_name.to_string());

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
    
    fn do_build(&mut self) -> Result<(), GraphQLError> {
            

        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join(format!("{}.rs", self.model_name));

        let file = File::create(dest_path)?;
        let mut base_out: BaseOutput = BaseOutput::from(file);
        let mut out = base_out.indent();
        writeln!(out, 
            r#"
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
"#
        )?;

        let schema: String;
        let schema = if let Some(file_name) = &self.schema {
            schema = read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            writeln!(out, "// cargo::rerun-if-changed={}", file_name)?;
            
                self.do_build_schema(&mut out, &schema)?
        }
        else {
            panic!("No schema defined");
        };

        for file_name in &self.queries {
            let query: String = read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            writeln!(out, "// cargo::rerun-if-changed={}", file_name)?;
            
            self.do_build_query(&mut out, &schema, &query)?;
        }

        if base_out.has_errors() {
            panic!("GraohQL Generation completed with errors: {}", base_out);
        }
        Ok(())
    }

    fn do_build_schema<'p>(&'p self, out: &mut Output, schema: &'p str) -> Result<Rc<validated_model::Schema>, GraphQLError> {
        let ast = parse_schema::<'p, String>(schema)?;
        // let ast = match parse_schema::<'p, String>(schema) {
        //     Ok(ast) => ast,
        //     Err(error) => {
        //         writeln!(out, "ERROR: {}", error)?;
        //         return Err(Box::new(error));
        //     },
        // };


        let model = parsed_model::Schema::new(out, ast)?;
        
        writeln!(out, "/* Parsed Model *********************************************************************************************")?;
        model.print(out)?;
        writeln!(out, " * *********************************************************************************************/")?;




//         writeln!(out, "/* TEST *********************************************************************************************")?;

//         let rc = std::rc::Rc::new(model);
//         let mut names = Vec::new();
        

//         for defined_type in rc.named_types.values() {
//             if let parsed_model::TypeDefinition::Interface(interface) = defined_type {
//                 names.push(interface.name.clone());
//             }
//         }

//         let list = parsed_model::InterfaceReferenceList {
//             schema: rc.clone(),
//             names,
//         };

//         for interface in &list {
//             interface.print(out)?;
//         }

//         writeln!(out, " * *********************************************************************************************/")?;


// panic!("TEST");



        let validated_model = validated_model::Schema::new(model, out)?;
        // model.validate(out)?;

        writeln!(out, "/* Validated Model *********************************************************************************************")?;
        validated_model.print(out)?;
        writeln!(out, " * *********************************************************************************************/")?;


        validated_model.generate(out)?;

        Ok(validated_model)
    }

    fn do_build_query(&self, out: &mut Output,  schema: &validated_model::Schema, query: &str) -> Result<(), GraphQLError> {
        let ast = parse_query::<String>(query)?.to_owned();

        let model = parsed_model::Operations::new(out, schema, ast.definitions);
        
        writeln!(out, "/* *********************************************************************************************")?;
        model.print(out)?;
        writeln!(out, " * *********************************************************************************************/")?;

         let validated_model = validated_model::Operations::new(model, out, schema)?;



        writeln!(out, "/* Validated Model *********************************************************************************************")?;
        validated_model.print(out)?;
        writeln!(out, " * *********************************************************************************************/")?;

        validated_model.generate(out)?;
        
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

    fn test_schema(schema: &str) -> BaseOutput {

        let mut base_out = BaseOutput::new();
        let mut out = base_out.indent();
        let builder = builder("test");
        let _ = builder.do_build_schema(&mut out, schema);
        
        base_out
        // let string = out.to_string();

        
    }

    fn test_query(schema: &str, query: &str) -> BaseOutput {

        let mut base_out = BaseOutput::new();
        let mut out = base_out.indent();
        let builder = builder("test");
        let schema = builder.do_build_schema(&mut out, schema).unwrap();
        let _q = builder.do_build_query(&mut out, &schema, query);
        
        base_out
    }

    #[test]
    fn test_missing_interface() {
        let out = test_schema(r#"
type Query implements Foo {
    name: String,
}
"#);
                
        if let Some(GraphQLError::MissingInterfaceError(..)) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected missing interface");
    }

    #[test]
    fn test_missing_query() {
        let out = test_schema(r#"
type Object {
    name: String,
}
"#);
        if let Some(GraphQLError::NoQueryDefinition) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected NoQueryDefinition");
    }

    #[test]
    fn test_missing_mutation() {
        let out = test_schema(r#"
schema {
    query: Object,
    mutation: Missing
}
type Object {
    name: String,
}
"#);
        if let Some(GraphQLError::MissingObjectError(..)) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected MissingObjectError");
    }

    #[test]
    fn test_missing_subscription() {
        let out = test_schema(r#"
schema {
    query: Object,
    subscription: Missing
}
type Object {
    name: String,
}
"#);
        if let Some(GraphQLError::MissingObjectError(..)) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected MissingObjectError");
    }

    const PERSON_SCHEMA: &'static str = r#"
type Query {
    person: Person
}
type Person {
    name: String!
    dateOfBirth: String
}
"#;

    #[test]
    fn test_simple_query() {
        let out = test_query(PERSON_SCHEMA, 
r#"query GetPerson {
  person {
    name
  }
}"#);
        out.expect_ok();
    }

    #[test]
    fn test_missing_attribute() {
        let out = test_query(PERSON_SCHEMA, 
r#"query GetPerson {
  xperson {
    name?
    dateOfBirth?
  }
}"#);
        
        
        if let Some(GraphQLError::MissingFieldError(..)) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected MissingFieldError");
    }

    #[test]
    fn test_missing_attribute2() {
        let out = test_query(PERSON_SCHEMA, 
r#"query GetPerson {
  person {
    xname
  }
}"#);
        
        
        if let Some(GraphQLError::MissingFieldError(..)) = out.expect_one_error() {
            return;
        }
        println!("{}", out);
        panic!("Expected MissingFieldError");
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