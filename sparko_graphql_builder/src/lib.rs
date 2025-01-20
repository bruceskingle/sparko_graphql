use graphql_parser::{parse_query, Pos};
use graphql_parser::schema::parse_schema;
use inflections::case::to_snake_case;
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
// mod error;


#[derive(Debug)]
pub enum Error {
    InternalError(Box<dyn std::error::Error>),
    BuildFailed(String),
    BuildErrors{errors: u32, warnings: u32}
}

impl std::error::Error for Error {
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::InternalError(Box::new(err))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum BuildError {
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
    NoOperations,
    ValidationError,
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum BuildWarning {
    UnsupportedFeature(Pos, &'static str),
    NameCollision(Pos, Pos, String),
}

impl Display for BuildWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}

pub struct BaseErrorCollector {
    errors: Vec<BuildError>,
    warnings: Vec<BuildWarning>,
}

impl Display for BaseErrorCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    Errors")?;
        for item in &self.errors {
            writeln!(f, "        {}", item)?;
        }

        writeln!(f, "    Warnings")?;
        for item in &self.warnings {
            writeln!(f, "        {}", item)?;
        }
        Ok(())
    }
}

impl BaseErrorCollector {
    pub fn new() -> BaseErrorCollector {
        BaseErrorCollector {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn new_error_collector(&mut self) -> ErrorCollector {
        ErrorCollector::new(self)
    }
    
    // fn err(&self) -> Result<(), Error> {
    //     Err(Error::BuildErrors { errors: self.errors.len() as u32, warnings: self.warnings.len() as u32 })
    // }

    pub fn expect_ok(&self) {
        if self.warnings.len() != 0 || self.errors.len() != 0 {
            println!("{}", self);
            panic!("Expected OK");
        }
    }

    pub fn expect_one_error(&self) -> Option<&BuildError> {
        if self.warnings.len() == 0 && self.errors.len() == 1 {
            self.errors.get(0)
        }
        else {
            None
        }
    }
}

pub struct ErrorCollector<'a> {
    base: &'a mut BaseErrorCollector,
    errors: u32,
    warnings: u32,
}

impl ErrorCollector<'_> {
    pub fn new<'a>(base: &'a mut BaseErrorCollector) -> ErrorCollector<'a> {
        ErrorCollector {
            base,
            errors: 0,
            warnings: 0,
        }
    }

    pub fn child(&mut self) -> ErrorCollector<'_> {
        ErrorCollector {
            base: self.base,
            errors: 0,
            warnings: 0,
        }
    }

    pub fn fail<T>(&mut self, error: BuildError) -> Result<T, Error> {
        let msg = format!("{}", error);
        self.error(error);
        Err(Error::BuildFailed(msg))
    }

    pub fn error(&mut self, error: BuildError) {
        self.base.errors.push(error);
        self.errors += 1;
    }

    pub fn warning(&mut self, warning: BuildWarning) {
        self.base.warnings.push(warning);
        self.warnings += 1;
    }

    pub fn ok_then<T>(&self, ok: &dyn Fn() -> T) -> Result<T, Error> {
        if self.errors == 0 {
            Ok(ok())
        }
        else {
            Err(Error::BuildErrors { errors: self.errors, warnings: self.warnings })
        }
    } 

    pub fn ok<T>(&self, ok: T) -> Result<T, Error> {
        if self.errors == 0 {
            Ok(ok)
        }
        else {
            Err(Error::BuildErrors { errors: self.errors, warnings: self.warnings })
        }
    } 
}

// fn test() -> Result<String, Error>{
//     let mut base = BaseErrorCollector::new();
//     let mut err = ErrorCollector::new(&mut base);

//     err.error(BuildError::NoQueryDefinition);

//     err.ok_then(&||{String::new()})
// }

// fn test2() -> Result<String, Error>{
//     let mut base = BaseErrorCollector::new();
//     let mut err = ErrorCollector::new(&mut base);

//     err.error(BuildError::NoQueryDefinition);

//     err.ok(String::new())
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

    // pub fn error(&mut self, error: BuildError) {
    //     self.base.error(error);
    // }

    // pub fn warning(&mut self, warning: BuildWarning) {
    //     self.base.warning(warning);
    // }

    // pub fn new_error_collector(&mut self) -> ErrorCollector {
    //     self.base.new_error_collector()
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

// impl Display for Output<'_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.base.fmt(f)
//     }
// }

// #[cfg(test)]
// struct TestOutput {
//     buf: Vec<u8>,
//     errors: Vec<GraphQLError>,
//     warnings: Vec<GraphQLError>,
// }

// struct FileOutput {
//     buf: BufWriter<File>,
//     errors: Vec<GraphQLError>,
//     warnings: Vec<GraphQLError>,
// }

enum BaseOutput {
    File(BufWriter<File>),
    #[cfg(test)]
    Buffer(Vec<u8>),
}

// impl Display for BaseOutput {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

//         fn fmt_errors(f: &mut std::fmt::Formatter<'_>, errors: &Vec<BuildError>)  -> std::fmt::Result {
//             writeln!(f, "    Errors")?;
//             for item in errors {
//                 writeln!(f, "        {}", item)?;
//             }
//             Ok(())
//         }

//         fn fmt_warnings(f: &mut std::fmt::Formatter<'_>, warnings: &Vec<BuildWarning>)  -> std::fmt::Result {
//             writeln!(f, "    Warnings")?;
//             for item in warnings {
//                 writeln!(f, "        {}", item)?;
//             }
//             Ok(())
//         }

//         writeln!(f, "Output")?;
//         match self {
//             BaseOutput::File(out) => {
//                 fmt_errors(f, &err.errors)?;
//                 fmt_warnings(f, &err.warnings)?;
//             }
//             #[cfg(test)]
//             BaseOutput::Buffer{err, out} => {
//                 fmt_errors(f, &err.errors)?;
//                 fmt_warnings(f, &err.warnings)?;
//                 writeln!(f, "    Output")?;
//                 let buf = &out;
//                 for item in std::str::from_utf8(buf).unwrap().lines() {
//                     writeln!(f, "        {}", item)?;
//                 }
//             },
//         }

        
//         Ok(())
//     }
// }

impl Write for BaseOutput {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            BaseOutput::File(out) => out.write(buf),
            #[cfg(test)]
            BaseOutput::Buffer(out) => out.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        
        match self {
            BaseOutput::File(out) => out.flush(),
            #[cfg(test)]
            BaseOutput::Buffer(out) => out.flush(),
        }
    }
}

// impl From<File> for BaseOutput {
//     fn from(value: File) -> Self {
//         BaseOutput::File{
//             err: BaseErrorCollector::new(),
//             out: BufWriter::new(value),
//         }
//     }
// }

impl BaseOutput {

    pub fn from_file(file: File) -> BaseOutput {
        BaseOutput::File(BufWriter::new(file))
    }



    // pub fn new_error_collector(&mut self) -> ErrorCollector {
    //     match self{
    //         BaseOutput::File { err, out: _ } => err.new_error_collector(),
    //         #[cfg(test)]
    //         BaseOutput::Buffer { err, out: _ } => err.new_error_collector(),
    //     }
    // }

    // pub fn has_errors(&self) -> bool {
    //     match self {
    //         BaseOutput::File{err, out} => err.errors.len()>0,
    //         #[cfg(test)]
    //         BaseOutput::Buffer{err, out} => err.errors.len()>0,
    //     }
    // }

    // pub fn errors(&self) -> &Vec<BuildError> {
    //     match self {
    //         BaseOutput::File{err, out} => &err.errors,
    //         #[cfg(test)]
    //         BaseOutput::Buffer{err, out} => &err.errors,
    //     }
    // }

    // pub fn warnings(&self) -> &Vec<BuildWarning> {
    //     match self {
    //         BaseOutput::File{err, out} => &err.warnings,
    //         #[cfg(test)]
    //         BaseOutput::Buffer{err, out} => &err.warnings,
    //     }
    // }

    // pub fn error(&mut self, error: BuildError) {
    //     if let Err(_) = writeln!(self, "ERROR //{}", &error) {
    //         // oh dear.....
    //     };
    //     let errors = match self {
    //         BaseOutput::File{err, out} => &mut err.errors,
    //         #[cfg(test)]
    //         BaseOutput::Buffer{err, out} => &mut err.errors,
    //     };

    //     // if let GraphQLError::MultipleErrors(multiple) = error {
    //     //     for error in multiple {
    //     //         errors.push(error);
    //     //     }
    //     // }
    //     // else {
    //         errors.push(error);
    //     // }
    // }

    // pub fn warning(&mut self, warning: BuildWarning) {
    //     if let Err(_) = writeln!(self, "// WARNING {}", &warning) {
    //         // oh dear.....
    //     };
    //     let warnings = match self {
    //         BaseOutput::File{err, out: _} => &mut err.warnings,
    //         #[cfg(test)]
    //         BaseOutput::Buffer{err, out: _} => &mut err.warnings,
    //     };

    //     warnings.push(warning);
    // }

    pub fn indent(&mut self) -> Output {
        Output {
            base: self,
            indent: 0,
            start_of_line: true,
        }
    }

    // pub fn new_error_collector(&mut self) -> ErrorCollector {
    //     match self {
    //         BaseOutput::File { err, out: _ } => ErrorCollector::new(err),
    //         #[cfg(test)]
    //         BaseOutput::Buffer { err, out: _ } => ErrorCollector::new(err),
    //     }
    // }
}

#[cfg(test)]

impl BaseOutput {
    pub fn new() -> BaseOutput {
        BaseOutput::Buffer(Vec::new())
    }
}

pub struct Builder {
    model_name: String,
    schema: Option<String>,
    queries: Vec<(String, String)>,
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

    pub fn with_query(&mut self, file_name: &str, model_name: &str) -> &mut Builder {
        self.queries.push((file_name.to_string(), model_name.to_string()));

        self
    }
    
    // pub fn build(&mut self) {
    //     match self.do_build() {
    //         Ok(_) => (),
    //         Err(error) => panic!("GraohQL Generation failed: {}", error),
    //     }
    // }
    
    pub fn build(&mut self) -> Result<String, Error> {
            
        let mut base = BaseErrorCollector::new();
        let mut err = ErrorCollector::new(&mut base);

        let schema: String;
        let schema = if let Some(file_name) = &self.schema {
            schema = fs::read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            
            self.do_build_schema(&mut err, &schema)?
        }
        else {
            return Err(Error::BuildFailed(format!("No schema defined")));
        };

        let mut queries = Vec::new();

        for (file_name, model_name) in &self.queries {
            let query: String = fs::read_to_string(file_name)?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", file_name);
            
            if let Ok(query) = self.do_build_query(&mut err, &schema, &query, model_name) {
                queries.push(query);
            }
        }
        let out_dir = env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join(format!("{}.rs", self.model_name));
        let dest_path_string = format!("{}", dest_path.to_string_lossy());

        if queries.is_empty() {
            err.error(BuildError::NoOperations);
        }
        else {
            let file = File::create(dest_path)?;
            let mut base_out: BaseOutput = BaseOutput::from_file(file);
            let mut out = base_out.indent();
            writeln!(out, 
                r#"
pub mod {} {{
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
"#, to_snake_case(&self.model_name)
            )?;

            for query in queries {
                query.generate(&mut out, &schema)?;
            }




            writeln!(out, 
                r#"
    }} // End model {}
    "#, to_snake_case(&self.model_name)
            )?;
        }


        if !base.errors.is_empty() {
            for error in  &base.errors {
                // println!("cargo::error={}", error);
                println!("cargo::error={}", error);
            }
            // panic!("GraohQL Generation completed with errors: {}", base_out);
            println!("cargo::warning=Build failed with {} errors and {} warnings", &base.errors.len(), &base.warnings.len());
            Err(Error::BuildFailed(format!("Build failed with {} errors and {} warnings", &base.errors.len(), &base.warnings.len())))
        }
        else {
            Ok(dest_path_string)
        }
    }

    fn do_build_schema<'p>(&'p self, err: &mut ErrorCollector, schema: &'p str) -> Result<validated_model::Schema, Error> {
    //     match self.do_build_schema2(err, schema) {
    //         Ok(schema) => Ok(schema),
    //         Err(error) => {
    //             let result = GraphQLError::BuildFailed(format!("{}", &error));
    //             err.error(error);
    //             Err(result)
    //         },
    //     }
    // }

    // fn do_build_schema2<'p>(&'p self, err: &mut ErrorCollector, schema: &'p str) -> Result<validated_model::Schema, Error> {
        // let mut base = BaseErrorCollector::new();
        // let mut err = ErrorCollector::new(&mut base);

        // let mut err = out.new_error_collector();

        let ast = match parse_schema::<'p, String>(schema) {
            Ok(ast) => ast,
            Err(parse_error) => {
                let msg = format!("Schema syntax error: {}", parse_error);
                err.error(BuildError::SchemaSyntaxError(parse_error));
                return Err(Error::BuildFailed(msg))
            },
        };
        // let ast = match parse_schema::<'p, String>(schema) {
        //     Ok(ast) => ast,
        //     Err(error) => {
        //         writeln!(out, "ERROR: {}", error)?;
        //         return Err(Box::new(error));
        //     },
        // };


        let model = parsed_model::Schema::new(err, ast)?;
        
        // writeln!(out, "/* Parsed Model *********************************************************************************************")?;
        // model.print(out)?;
        // writeln!(out, " * *********************************************************************************************/")?;




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



        let validated_model = validated_model::Schema::new(err, model)?;
        // model.validate(out)?;

        // writeln!(out, "/* Validated Model *********************************************************************************************")?;
        // validated_model.print(out)?;
        // writeln!(out, " * *********************************************************************************************/")?;


        // validated_model.generate(out)?;

        Ok(validated_model)
    }

    fn do_build_query<'a>(&self, err: &mut ErrorCollector,  schema: &'a validated_model::Schema, query: &str, model_name: &str) -> Result<validated_model::Operations<'a>, Error> {
        // let ast = parse_query::<String>(query)?.to_owned();

        let ast = match parse_query::<String>(query) {
            Ok(ast) => ast,
            Err(parse_error) => {
                let msg = format!("Query syntax error: {}", parse_error);
                err.error(BuildError::QuerySyntaxError(parse_error));
                return Err(Error::BuildFailed(msg))
            },
        };

        let model = parsed_model::Operations::new(err, schema, ast.definitions, model_name);
        
        // writeln!(out, "/* *********************************************************************************************")?;
        // model.print(out)?;
        // writeln!(out, " * *********************************************************************************************/")?;

         let validated_model = validated_model::Operations::new(err, model, schema)?;



        // writeln!(out, "/* Validated Model *********************************************************************************************")?;
        // validated_model.print(out)?;
        // writeln!(out, " * *********************************************************************************************/")?;

        // validated_model.generate(err, schema)?;
        
        Ok(validated_model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_schema(schema: &str) -> BaseErrorCollector {

        let mut base = BaseErrorCollector::new();
        let mut err = ErrorCollector::new(&mut base);

        let builder = builder("test");
        let _ = builder.do_build_schema(&mut err, schema);
        
        base
    }

    fn get_schema(schema: &str) -> validated_model::Schema {

        let mut base = BaseErrorCollector::new();
        let mut err = ErrorCollector::new(&mut base);
        let builder = builder("test");
        let schema = builder.do_build_schema(&mut err, schema);
        
        schema.unwrap()
    }

    fn test_query(schema: &str, query: &str) -> BaseErrorCollector {

        let mut base = BaseErrorCollector::new();
        let mut err = ErrorCollector::new(&mut base);
        let builder = builder("test");
        let schema = builder.do_build_schema(&mut err, schema).unwrap();
        let _q = builder.do_build_query(&mut err, &schema, query, "test_query");
        
        base
    }

    #[test]
    fn test_missing_interface() {
        let out = test_schema(r#"
type Query implements Foo {
    name: String,
}
"#);
                
        if let Some(BuildError::MissingInterfaceError(..)) = out.expect_one_error() {
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
        if let Some(BuildError::NoQueryDefinition) = out.expect_one_error() {
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
        if let Some(BuildError::MissingObjectError(..)) = out.expect_one_error() {
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
        if let Some(BuildError::MissingObjectError(..)) = out.expect_one_error() {
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
        
        
        if let Some(BuildError::MissingFieldError(..)) = out.expect_one_error() {
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
        
        
        if let Some(BuildError::MissingFieldError(..)) = out.expect_one_error() {
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