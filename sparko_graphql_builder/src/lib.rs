use graphql_parser::{parse_query, Pos};
use graphql_parser::schema::parse_schema;
use indexmap::IndexMap;
use inflections::case::to_snake_case;
use validated_model::NameSpaceManager;
use std::collections::{HashMap, HashSet};
// use model::{ExecutableDocument, Schema};
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::rc::Rc;
use std::{env, fs};
use std::io::Write;

mod utils;
// mod model;
mod parsed_model;
mod validated_model;
// mod error;

const STARS: &str = "***************************************************************************************************************************************************";
const TYPE_NAME: &str = "__typename";

pub trait Isomorphic {
    fn is_isomorphic(&self, other: &Self) -> bool;
}

pub trait Print {
    fn print(&self, out: &mut Output) -> std::io::Result<()>;
}

pub struct BagOfNamed<T: Print> {
    map: IndexMap<Atom, Vec<T>>,
}

impl<T: Print> BagOfNamed<T> {
    pub fn new() -> BagOfNamed<T> {
        BagOfNamed {
            map: IndexMap::new(),
        }
    }
    
    pub fn insert(&mut self, name: &Atom, value: T) {
        let list = if let Some(existing) = self.map.get_mut(name) {
            existing
        }
        else {
            self.map.insert(name.clone(), Vec::new());
            self.map.get_mut(name).unwrap()
        };
        list.push(value);
    }
    
    pub fn print(&self, out: &mut Output) -> std::io::Result<()> {
        writeln!(out, "{{")?;
        {
            let mut out = out.indent();

            for (name, list) in &self.map {
                writeln!(out, "name {} {{", name)?;
                {
                    let mut out = out.indent();

                    for item in list {
                        item.print(&mut out)?;
                    }
                }
                writeln!(out, "}}")?;
            }
        }
        writeln!(out, "}}")
    }
}

type Atom = Rc<String>;

#[derive(Debug)]
pub struct Registry {
    pub atoms: HashSet<Atom>,
}

impl Registry {
    pub fn new() -> Registry {
        Registry {
            atoms: HashSet::new(),
        }
    }

    pub fn intern(&mut self, value: String) -> Atom {
        if let Some(atom) = self.atoms.get(&value) {
            atom.clone()
        }
        else {
            let new_value = Rc::new(value);
            self.atoms.insert(new_value.clone());
            new_value
        }
    }

    pub fn intern_str(&mut self, value: &str) -> Atom {
        self.intern(String::from(value))
    }

    pub fn intern_option(&mut self, value: Option<String>) -> Option<Atom> {
        match value {
            Some(value) => Some(self.intern(value)),
            None => None,
        }
    }
    
    fn intern_vec(&mut self, input: Vec<String>) -> Vec<Rc<String>> {
        let mut result = Vec::new();
    
        for value in input {
            result.push(self.intern(value));
        }
    
        result
    }
}

// pub fn intern(value: String) -> Atom {
//     Rc::new(value)
// }

// pub fn intern_option(value: Option<String>) -> Option<Atom> {
//     match value {
//         Some(value) => Some( Rc::new(value)),
//         None => None,
//     }
// }

// fn intern_vec(input: Vec<String>) -> Vec<Rc<String>> {
//     let mut result = Vec::new();

//     for value in input {
//         result.push(intern(value));
//     }

//     result
// }

#[derive(Debug)]
pub enum Error {
    InternalError(Box<dyn std::error::Error>),
    FatalBuildError(&'static str),
    BuildFailed(String),
    BuildErrors{errors: u32, warnings: u32},
    FileReadError{file_name: String, reason: std::io::Error},
    FileWriteError{file_name: String, reason: std::io::Error},
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

#[derive(Clone, Debug)]
pub enum BuildError {
    Unimplemented(Pos, &'static str),
    SchemaSyntaxError(String),
    QuerySyntaxError(String),
    UndefinedTypeError(Pos, String),
    TypeMismatchError(Pos, String),
    MissingScalarError(Pos, String),
    MissingObjectError(Pos, String),
    MissingInterfaceError(Pos, String),
    MissingUnionError(Pos, String),
    MissingEnumError(Pos, String),
    MissingInputObjectError(Pos, String),
    MissingFieldError(Pos, String),
    SelectionOnNonObjectError(Pos, Atom),
    OptionalNonNullFieldError(Pos, String),
    MissingFragmentError(Pos, String),
    InvalidQueryError(Pos, String),
    UnsupportedError(Pos, String),
    DuplicateName(Pos, Pos, String),
    NoQueryDefinition,
    NoOperations,
    ValidationError,
}

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum BuildWarning {
    UnsupportedFeature(Pos, &'static str),
    NameCollision(Pos, Pos, String),
}

impl Display for BuildWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct BaseErrorCollector {
    file_name: String,
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
    pub fn new(file_name: String) -> BaseErrorCollector {
        BaseErrorCollector {
            file_name,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn new_test() -> BaseErrorCollector {
        Self::new(String::from("Test"))
    }

    pub fn new_error_collector(&mut self) -> ErrorCollector {
        ErrorCollector::new(self)
    }

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


    fn report_errors(&self) {
        if !self.errors.is_empty() {
            println!("Errors");
            for item in &self.errors {
                println!("    {}", item);
            }
        }

        if ! self.warnings.is_empty() {
            println!("Warnings");
            for item in &self.warnings {
                println!("    {}", item);
            }
        }
    }
    
    fn report(&self, f: &mut dyn Write) -> std::io::Result<()> {
        if self.errors.is_empty() {
            writeln!(f, "//{} built OK", self.file_name)?;
        }
        else {
            // writeln!(f, "compile_warning!(\"{} built with Errors\");", self.file_name)?;
            writeln!(f, "compile_error!(\"{} built with Errors\");", self.file_name)?;
            writeln!(f, "/{}", STARS)?;
            for item in &self.errors {
                writeln!(f, "    {}", item)?;
                println!("cargo::warning={}", item);
            }
            writeln!(f, "{}/", STARS)?;
        }

        if ! self.warnings.is_empty() {
            writeln!(f, "/{}", STARS)?;
            writeln!(f, "Warnings")?;
            for item in &self.warnings {
                writeln!(f, "    {}", item)?;
                println!("cargo::warning={}", item);
            }
            writeln!(f, "{}/", STARS)?;
        }

        Ok(())
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

enum BaseOutput {
    File(BufWriter<File>),
    Stdout,

#[cfg(test)]
    Buffer(Vec<u8>),
}

impl Display for BaseOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        writeln!(f, "Output")?;
        match self {
            BaseOutput::File(_) => {},
            BaseOutput::Stdout => {},
            #[cfg(test)]
            BaseOutput::Buffer(out) => {
                writeln!(f, "    Output")?;
                let buf = &out;
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
            BaseOutput::File(out) => out.write(buf),
            BaseOutput::Stdout => std::io::stdout().write(buf),
            #[cfg(test)]
            BaseOutput::Buffer(out) => out.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        
        match self {
            BaseOutput::File(out) => out.flush(),
            BaseOutput::Stdout => std::io::stdout().flush(),
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

    pub fn from_stdout() -> BaseOutput {
        BaseOutput::Stdout
    }

    pub fn indent(&mut self) -> Output {
        Output {
            base: self,
            indent: 0,
            start_of_line: true,
        }
    }
}

#[cfg(test)]
impl BaseOutput {
    pub fn new() -> BaseOutput {
        BaseOutput::Buffer(Vec::new())
    }
}

pub struct Builder {
    model_name: String,
    schema_file_name: Option<String>,
    query_file_names: Vec<(String, String)>,
    types: HashMap<String, String>,
    print: bool,
    generate: bool,
}

pub fn builder(model_name: impl Into<String>) -> Builder {
    Builder {
        model_name: model_name.into(),
        schema_file_name: None,
        query_file_names: Vec::new(),
        types: HashMap::new(),
        print: false,
        generate: true,
    }
}

impl Builder {
    pub fn with_schema(&mut self, file_name: &str) -> &mut Builder {
        if let Some(schema) = &self.schema_file_name {
            panic!("Multiple schemas defined ({} and {})", schema, file_name);
        }
        self.schema_file_name = Some(file_name.to_string());

        self
    }

    pub fn with_query(&mut self, file_name: &str, model_name: &str) -> &mut Builder {
        self.query_file_names.push((file_name.to_string(), model_name.to_string()));

        self
    }
    
    pub fn with_type(&mut self, name: &str, fully_qualified_type: &str) -> &mut Builder {
        self.types.insert(name.into(), fully_qualified_type.into());

        self
    }

    pub fn with_print(&mut self, value: bool) -> &mut Builder {
        self.print = value;

        self
    }

    pub fn with_generate(&mut self, value: bool) -> &mut Builder {
        self.generate = value;

        self
    }
    
    pub fn build(&mut self) -> Result<String, Error> {
        let out_dir = if let Some(dir) = env::var_os("OUT_DIR") {
            dir
        }
        else {
            std::ffi::OsString::from("/tmp")
        };
        let dest_path = Path::new(&out_dir).join(format!("{}.rs", self.model_name));
        let dest_path_string = format!("{}", dest_path.to_string_lossy());

        let file = File::create(dest_path)?;
        let mut base_out: BaseOutput = if self.generate {
            BaseOutput::from_file(file)
        }
        else {
            BaseOutput::from_stdout()
        };

        let mut out = base_out.indent();

        if let Err(error) = self.do_build(&mut out) {
            writeln!(out, "compile_warning!(\"Fatal build error {:?}\");", error)?;
            println!("cargo-error: {:?}", error);
        }


        Ok(dest_path_string)
    }

    fn do_build(&mut self, out: &mut Output<'_>) -> Result<(), Error> {

        if let Some(schema_file_name) = &self.schema_file_name {
            let schema_string = match fs::read_to_string(schema_file_name) {
                Ok(str) => Ok(str),
                Err(err) => {
                    Err(Error::FileReadError { file_name: schema_file_name.clone(), reason: err })
                },
            }?;
                    
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo::rerun-if-changed={}", schema_file_name);

            let mut base = BaseErrorCollector::new(schema_file_name.clone());
            let mut err = ErrorCollector::new(&mut base);

            // let schema = self.do_build_schema(&mut err, &schema_string)?;

            let ast = match parse_schema::<'_, String>(&schema_string) {
                Ok(ast) => ast,
                Err(parse_error) => {
                    // let msg = format!("Schema syntax error: {}", parse_error);
                    return err.fail(BuildError::SchemaSyntaxError(parse_error.to_string()));
                    // return Err(Error::BuildFailed(msg))
                },
            };
            // let mut model = Schema::new(err, ast)?;
            // model.validate(err);

            let mut registry = Registry::new();
    
            let parsed_schema = match parsed_model::Schema::new(&mut err, &mut registry, ast) {
                Ok(result) => result,
                Err(error) => {
                    base.report_errors();
                    return Err(error);
                },
            };

            if self.print {
                writeln!(out, "/* Parsed Schema *********************************************************************************************")?;
                parsed_schema.print(out)?;
                writeln!(out, " * *********************************************************************************************/")?;
            }

            let validated_schema = match validated_model::Schema::new(&mut err, &parsed_schema, &mut registry, &self.types) {
                Ok(result) => result,
                Err(error) => {
                    base.report_errors();
                    return Err(error);
                },
            };


            if self.print {
                writeln!(out, "/* Validated Schema *********************************************************************************************")?;
                validated_schema.print(out)?;
                writeln!(out, " * *********************************************************************************************/")?;
            }


            if self.print || self.generate {
                base.report(out);
            }



            // let mut dependencies: IndexMap<&str, (validated_model::ExecutableDocument, IndexMap<Atom, HashSet<Atom>>)> = IndexMap::new();

            let mut documents = Vec::new();
            let mut selection_manager = NameSpaceManager::new();

            for (query_file_name, query_model_name) in &self.query_file_names 
            {
                let query_string = match fs::read_to_string(query_file_name) {
                    Ok(str) => Ok(str),
                    Err(err) => {
                        Err(Error::FileReadError { file_name: query_file_name.clone(), reason: err })
                    },
                }?;
                        
                // Tell Cargo that if the given file changes, to rerun this build script.
                println!("cargo::rerun-if-changed={}", query_file_name);
                
                selection_manager.create_document(registry.intern(to_snake_case(query_model_name)));
                let mut base = BaseErrorCollector::new(query_file_name.clone());
                let mut err = ErrorCollector::new(&mut base);
                
                let parse_result = parse_query::<String>(&query_string);

                if self.print {
                    writeln!(out, "/* Query *********************************************************************************************")?;
                    writeln!(out, "{}", query_string)?;
                    writeln!(out, " * *********************************************************************************************/")?;
                }

                match parse_result {
                    Ok(query_ast) => {
                        let query_parsed_model = parsed_model::ExecutableDocument::new(&mut err, &mut registry, query_ast.definitions, query_model_name);
                
                        if self.print {
                            writeln!(out, "/* Parsed ExecutableDocument *********************************************************************************************")?;
                            query_parsed_model.print(out)?;
                            writeln!(out, " * *********************************************************************************************/")?;
                        }
                
                        // let validated_executable = validated_model::ExecutableDocument::new(&mut err, &query_parsed_model, &validated_schema)?;
                            
                        // validated_executable.generate(out, &validated_schema)?;
                
                        if let Ok(validated_executable) = validated_model::ExecutableDocument::new(&mut err, &mut registry, &mut selection_manager, &query_parsed_model, &validated_schema) {
                            
                            if self.print {
                                writeln!(out, "/* Validated ExecutableDocument *********************************************************************************************")?;
                                validated_executable.print(out)?;
                                writeln!(out, " * *********************************************************************************************/")?;
                            }

                            // if self.generate {
                            //     let d = validated_executable.gather_dependencies(&validated_schema)?;
                            //     dependencies.insert(query_model_name, (validated_executable, d));
                            // }
                            documents.push(validated_executable);
                        }
                    },
                    Err(parse_error) => {
                        // let msg = format!("Query syntax error: {}", parse_error);
                        err.error(BuildError::QuerySyntaxError(parse_error.to_string()));
                        // return Err(Error::BuildFailed(msg))
                    },
                };
    
                base.report(out)?;
            }

            if self.print {
                writeln!(out, "/* Selection Manager *********************************************************************************************")?;
                selection_manager.print(out)?;
                writeln!(out, " * *********************************************************************************************/")?;
            }

            if self.generate {

                selection_manager.allocate_names();
                // //propogate dependencies

                // let mut schema_types = HashSet::new();

                // for document in &documents {
                //     println!("Document {}", document.parsed.name);
                //     for operation in &document.operations {

                //         println!("    Operation {}", operation.parsed.name);
                //         operation.response.print(out)?;
                //         for name in &operation.schema_dependencies {
                //             println!("        Type {}", name);
                //             schema_types.insert(name);
                //         }
                //     }
                // }



                writeln!(out, 
                    r#"
// Generated by sparko_graphql
pub mod {} {{
use display_json::DisplayAsJsonPretty;
use serde::{{Deserialize, Serialize}};
"#, to_snake_case(&self.model_name)
                )?;

                for name in &selection_manager.all_schema_imports {
                    if let Some(object) = validated_schema.defined_types.get(name) {
                        object.generate(out)?;
                    }
                    else {
                        println!("Failed to find {}", *name);
                    }
                }

                // println!("selection_manager {:?}", &selection_manager);

                // writeln!(out, "/* selection manager");
                // selection_manager.print(out)?;

                // selection_manager.set_document(Rc::new(String::from("luke")));
                // selection_manager.set_operation(Rc::new(String::from("GetLuke")));
                // let ctx = selection_manager.get_context();

                // ctx.print(out);

                // writeln!(out, " selection manager */");

                for document in &documents {
                    selection_manager.set_document(document.parsed.name.clone());
                    document.generate(out, &validated_schema, &mut selection_manager)?;
                }


                // // writeln!(out, "// Dependencies {:?}", dependencies)?;

                // let mut all_dependencies: HashSet<Atom> = HashSet::new();

                // for (_name, (_validated_executable, d1)) in &dependencies {
                //     writeln!(out, "// Dependency name {:?}", _name)?;
                //     for (_name, d2) in d1 {

                //     writeln!(out, "//   d2 name {:?}", _name)?;
                //         for d3 in d2 {
                //             writeln!(out, "// d3 {:?}", d3)?;
                //             all_dependencies.insert(d3.clone());
                //         }
                //     }
                // }
                // writeln!(out, "// All Dependencies {:?}", all_dependencies)?;

                // for name in all_dependencies {
                //     if let Some(defined_type) = validated_schema.defined_types.get(&name) {
                //         defined_type.generate(out, &None)?;
                //     }
                // }

                // writeln!(out, "// Executable docs")?;

                // for (_name, (validated_executable, dependencies)) in &dependencies {
                //     validated_executable.generate(out, &validated_schema, dependencies)?;
                // }

                for document in documents {

                }


                writeln!(out, 
                    r#"
}} // End model {}
"#, to_snake_case(&self.model_name)
                )?; 
            }
        }
        else {
            return Err(Error::BuildFailed(format!("No schema defined")));
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // fn test_schema(schema: &str) -> BaseErrorCollector {

    //     let mut base = BaseErrorCollector::new_test();
    //     let mut err = ErrorCollector::new(&mut base);

    //     let builder = builder("test");
    //     let _ = builder.do_build_schema(&mut err, schema);
        
    //     base
    // }

    // fn get_schema(schema: &str) -> validated_model::Schema {

    //     let mut base = BaseErrorCollector::new_test();
    //     let mut err = ErrorCollector::new(&mut base);
    //     let builder = builder("test");
    //     let schema = builder.do_build_schema(&mut err, schema);
        
    //     schema.unwrap()
    // }

    pub fn test_schema(schema_string: &str) -> BaseErrorCollector {
        let mut base = BaseErrorCollector::new_test();
        let mut err: ErrorCollector<'_> = ErrorCollector::new(&mut base);

        // let x = do_test_query(schema_string, query, &mut err);

        match do_test_schema(schema_string, &mut err) {
            Ok(_) => (),
            Err(_) => {
                // let mut stderr = std::io::stderr();
                // base.report(&mut stderr).unwrap();
            },
        }

        base
    }

    fn do_test_schema(schema_string: &str, mut err:  &mut ErrorCollector<'_>) -> Result<(), Error> {
        let types = HashMap::new();

         match parse_schema::<'_, String>(&schema_string) {
            Ok(ast) => {
                let mut registry = Registry::new();
                let parsed_schema = parsed_model::Schema::new(&mut err, &mut registry, ast)?;

                validated_model::Schema::new(&mut err, &parsed_schema, &mut registry, &types)?;
            },
            Err(parse_error) => {
                err.error(BuildError::SchemaSyntaxError(parse_error.to_string()));
            },
        };

        Ok(())
    }

    pub fn test_query(schema_string: &str, query: &str) -> BaseErrorCollector {
        let mut base = BaseErrorCollector::new_test();
        let mut err: ErrorCollector<'_> = ErrorCollector::new(&mut base);

        // let x = do_test_query(schema_string, query, &mut err);

        match do_test_query(schema_string, query, &mut err) {
            Ok(_) => (),
            Err(_) => {
                // let mut stderr = std::io::stderr();
                // base.report(&mut stderr).unwrap();
            },
        }

        base
    }

    fn do_test_query(schema_string: &str, query: &str, mut err:  &mut ErrorCollector<'_>) -> Result<(), Error> {
        let types = HashMap::new();
        

        match parse_schema::<'_, String>(&schema_string) {
            Ok(ast) => {
                let mut registry = Registry::new();
                let parsed_schema = parsed_model::Schema::new(&mut err, &mut registry, ast)?;
                let validated_schema = validated_model::Schema::new(&mut err, &parsed_schema, &mut registry, &types)?;
                let mut selection_manager = NameSpaceManager::new();
                selection_manager.create_document(registry.intern_str("query_model_name"));
                match parse_query::<String>(&query) {
                    Ok(ast) => {
                        let parsed_model = parsed_model::ExecutableDocument::new(&mut err, &mut registry, ast.definitions, "test");
                        validated_model::ExecutableDocument::new(&mut err, &mut registry, &mut selection_manager, &parsed_model, &validated_schema)?;
                    },
                    Err(parse_error) => {
                        // let msg = format!("Query syntax error: {}", parse_error);
                        err.error(BuildError::QuerySyntaxError(parse_error.to_string()));
                        // return Err(Error::BuildFailed(msg))
                    },
                };
            },
            Err(parse_error) => {
                err.error(BuildError::SchemaSyntaxError(parse_error.to_string()));
            },
        };

        Ok(())
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
        out.report_errors();
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
       out.report_errors();
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
       out.report_errors();
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
       out.report_errors();
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
       out.report_errors();
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
       out.report_errors();
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
//                out.report_test();
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
//                out.report_test();
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