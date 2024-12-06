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


use std::{collections::HashMap, sync::Arc};

use display_json::DisplayAsJsonPretty;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Error;

use crate::{AuthenticatedRequestManager, Client};

pub struct ParamBuffer {
    buf: String
}

impl ParamBuffer {
    pub fn new() -> ParamBuffer {
        ParamBuffer {
            buf: String::new()
        }
    }

    pub fn push(&mut self, s: &str) {
        self.buf.push_str(if self.buf.len() == 0 {
            "("
        }
        else {
            ", "
        });

        self.buf.push_str(s);
    }

    pub fn consume(mut self) -> String {
        if self.buf.len() > 0 {
            self.buf.push(')');
        }
        self.buf
    }

    pub fn push_formal(&mut self, prefix: &str, param_name: &str, param_type: &str) {
        self.push("$");
        self.buf.push_str(prefix);
        self.buf.push_str(param_name);
        self.buf.push_str(": ");
        self.buf.push_str(param_type);
    }

    pub fn push_actual(&mut self, prefix: &str, param_name: &str) {
        self.push(param_name);
        self.buf.push_str(": $");
        self.buf.push_str(prefix);
        self.buf.push_str(param_name);
    }
}

// pub struct VariableBuffer {
//     pub map: serde_json::Map<String, serde_json::Value>
// }

// impl VariableBuffer {
//     pub fn new() -> VariableBuffer {
//         VariableBuffer {
//             map: serde_json::Map::new()
//         }
//     }

//     pub fn push_variable<T: Serialize>(&mut self, prefix: &str, name: &str, value: &T) -> Result<(), Error> {
//        self.map.insert(format!("{}{}", prefix, name), serde_json::to_value(value)?);
//        Ok(())
//     }

//     pub fn to_string(self) -> Result<String, Error> {
//         serde_json::to_string_pretty(&self.map)
//     }
// }

const EMPTY_STRING: String = String::new();

pub struct GraphQL;

impl GraphQL {
    pub fn prefix(a: &str, b: &str) -> String {
        if b.len() == 0 {
            a.to_string()
        }
        else {
            if a.len() == 0 {
                format!("{}_", b)
            }
            else {
                format!("{}{}_", a, b)
            }
        }
    }
}


/*
pub struct VariableBuffer {
    map: HashMap<String, serde_json::Value>
}

impl VariableBuffer {
    pub fn new() -> VariableBuffer {
        VariableBuffer {
            map: HashMap::new()
        }
    }

    pub fn push_variable<T: Serialize>(&mut self, prefix: &str, name: &str, value: &T) -> Result<(), Error> {
       self.map.insert(format!("{}{}", prefix, name), serde_json::to_value(value)?);
       Ok(())
    }

    pub fn to_string(self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.map)
    }
}

const EMPTY_STRING: String = String::new();

pub struct GraphQL;

impl GraphQL {
    pub fn prefix(a: &str, b: &str) -> String {
        if b.len() == 0 {
            a.to_string()
        }
        else {
            if a.len() == 0 {
                format!("{}_", b)
            }
            else {
                format!("{}{}_", a, b)
            }
        }
    }
}

*/

/*

pub struct VariableBuffer<'a> {
    // map: HashMap<String, serde_json::Value>,
    root: serde_json::Map<String, serde_json::Value>,
    stack: Vec<&'a mut serde_json::Map<String, serde_json::Value>>,
}

impl<'a> VariableBuffer<'a> {
    pub fn new() -> VariableBuffer<'a> {
        let mut buf = VariableBuffer {
            // map: HashMap::new(),
            root: serde_json::Map::new(),
            stack: Vec::new(),
        };
        buf.stack.push(&mut buf.root);

        buf
    }

    pub fn push_variable<T: Serialize>(&mut self, prefix: &str, name: &str, value: &T) -> Result<(), Error> {
       let mut top = self.stack.pop().unwrap();
       top.insert(format!("{}{}", prefix, name), serde_json::to_value(value)?);
       self.stack.push(top);
       
    //    let x = &self.stack;
    //    let y = x.last().unwrap();
       
    //    x.last().unwrap()
    //    //self.map
    //    .insert(format!("{}{}", prefix, name), serde_json::to_value(value)?);
       Ok(())
    }

    pub fn push_object(&mut self, prefix: &str, name: &str) {
        let mut new = serde_json::Map::new();
        let mut top = self.stack.pop().unwrap();
        top.insert(format!("{}{}", prefix, name), serde_json::Value::Object(new));
        self.stack.push(top);
        self.stack.push(&mut new);
    }

    pub fn to_string(self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self
            // .map
            .stack.last().unwrap()
        )
    }
}

*/

// pub trait GraphQLRoot<V: GraphQLVariables, E: GraphQLEntity<Q>> {

//     async fn query(client: &crate::Client, params: Q) -> Result<E, crate::Error>;
// }

pub trait GraphQLVariables {
    fn get_formal_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_actual_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_variables_part(&self, variables: &mut serde_json::Map<String, serde_json::Value>, prefix: &str) -> Result<(), Error>;


    fn get_formal(&self) -> String {
        let mut params = ParamBuffer::new();
        self.get_formal_part(&mut params, "");

        params.consume()
    }

    fn get_actual(&self, prefix: &str) -> String {
        let mut params = ParamBuffer::new();
        self.get_actual_part(&mut params, prefix);

        params.consume()
    }

    fn get_variables(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.get_variable_map()?)
    }

    fn get_variable_map(&self) -> Result<serde_json::Map<String, serde_json::Value>, Error>  {
        let mut variables: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        self.get_variables_part(&mut variables, "")?;

        Ok(variables)
    }

    
}

pub trait GraphQLQueryBuilder <V: GraphQLVariables> 
{
    fn build(self) -> V;
}

// #[derive(Serialize)]
pub struct NoVariables;

impl GraphQLVariables for NoVariables {

    fn get_formal_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_actual_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_variables_part(&self, _variables: &mut serde_json::Map<String, serde_json::Value>, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }
}

pub struct NoQueryBuilder;

impl GraphQLQueryBuilder<NoVariables> for NoQueryBuilder {
    fn build(self) -> NoVariables {
        NoVariables {}
    }
}

pub trait GraphQLQueryParams {
    fn get_formal_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_actual_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_variables_part(&self, variables: &mut serde_json::Map<String, serde_json::Value>, prefix: &str) -> Result<(), Error>;


    fn get_formal(&self) -> String {
        let mut params = ParamBuffer::new();
        self.get_formal_part(&mut params, "");

        params.consume()
    }

    fn get_actual(&self, prefix: &str) -> String {
        let mut params = ParamBuffer::new();
        self.get_actual_part(&mut params, prefix);

        params.consume()
    }

    fn get_variables(&self) -> Result<String, Error> {
        serde_json::to_string_pretty(&self.get_variable_map()?)
    }

    fn get_variable_map(&self) -> Result<serde_json::Map<String, serde_json::Value>, Error>  {
        let mut variables: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        self.get_variables_part(&mut variables, "")?;

        Ok(variables)
    }

    // fn get_variables(&self) -> Result<String, Error> {
    //     let mut variables: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    //     self.get_variables_part(&mut variables, "")?;

    //     variables.to_string()
    // }

    // fn get_variable_map(&self) -> Result<serde_json::Map<String, serde_json::Value>, Error>  {
    //     let mut variables: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    //     self.get_variables_part(&mut variables, "")?;

    //     Ok(variables.map)
    // }

    
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct NoParams;

impl GraphQLQueryParams for NoParams {
    fn get_formal_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_actual_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_variables_part(&self, _variables: &mut serde_json::Map<String, serde_json::Value>, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }
}


pub trait GraphQLType<Q: GraphQLQueryParams> {
    fn get_query_part(params: &Q, prefix: &str) -> String {
        format!("{{ #get_query_part\n  {}\n}} #/get_query_part\n", Self::get_query_attributes(params, prefix))
    }

    fn get_query_attributes(params: &Q, prefix: &str) -> String;

    // fn get_request_name(&self) -> &'static str;
    // fn get_query(&self) -> String ;
    // fn get_query(&self) -> String {
    //     format!(r#"
    //     query {}{} {{
    //         account{} {{
    //             id
    //             properties{} {{
    //                 {}
    //             }}
    //         }}
    //     }}
    //     "#, self.get_request_name(), self.get_params().get_formal(),
    //         self.get_params().get_actual(""),
    //         self.get_params().properties.get_actual("properties_"),
    //         PropertySimpleView::get_query_part()
    // )
    // }
}


pub trait GraphQLEntity<V: GraphQLVariables>: DeserializeOwned {
    fn get_query_part(params: &V, prefix: &str) -> String {
        format!("{{ #get_query_part\n  {}\n}} #/get_query_part\n", Self::get_query_attributes(params, prefix))
    }

    fn get_query_attributes(params: &V, prefix: &str) -> String;

    // fn get_request_name(&self) -> &'static str;
    // fn get_query(&self) -> String ;
    // fn get_query(&self) -> String {
    //     format!(r#"
    //     query {}{} {{
    //         account{} {{
    //             id
    //             properties{} {{
    //                 {}
    //             }}
    //         }}
    //     }}
    //     "#, self.get_request_name(), self.get_params().get_formal(),
    //         self.get_params().get_actual(""),
    //         self.get_params().properties.get_actual("properties_"),
    //         PropertySimpleView::get_query_part()
    // )
    // }
}


pub trait TokenManager {
    /*
    use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
   --> src/lib.rs:547:5
    |
547 |     async fn get_authenticator(&mut self) -> Result<Arc<String>, Box<dyn StdError>>;
    |     ^^^^^
    |
    = note: you can suppress this lint if you plan to use the trait only in your own code, or do not care about auto traits like `Send` on the `Future`
    = note: `#[warn(async_fn_in_trait)]` on by default
help: you can alternatively desugar to a normal `fn` that returns `impl Future` and add any desired bounds such as `Send`, but these cannot be relaxed without a breaking API change
     */
    // Returns a bearer token, which may be cached.
    // async fn get_authenticator(&mut self) -> Result<Arc<String>, Box<dyn StdError>>;
    fn get_authenticator(&mut self) -> impl std::future::Future<Output = Result<Arc<String>, Box<dyn std::error::Error>>> + Send;

    // Returns a fresh bearer token forcing a reauthentication
    // async fn authenticate(&mut self) -> Result<Arc<String>, Box<dyn StdError>>;
    fn authenticate(&mut self) -> impl std::future::Future<Output = Result<Arc<String>, Box<dyn std::error::Error>>> + Send;
}

// pub struct GraphQLQuery<S: GraphQLVariables, V: GraphQLVariables> {
//     pub operation_name: String,
//     pub query_name: String,
//     pub selector: S,
//     pub variables: V,
// }

// impl<S: GraphQLVariables, V: GraphQLVariables, E: GraphQLEntity<V>> GraphQLQuery<S,V> {
//     async fn query(&self, request_manager: &crate::RequestManager) -> Result<E, Error> {
//         let query = format!(r#"
//             query {}{}
//                 #T::get_query_part(&params, "")
//                 {}"#, 
//             self.operation_name,
//             self.selector.get_formal(),
//             // query_name,
//             // params.get_actual(""),
//             E::get_query_part(&variables, "")
//         );
//     }
// }

// pub trait GraphQLQuery<S: GraphQLVariables, V: GraphQLVariables, E: GraphQLEntity<V>> {
//     fn query(&self, request_manager: &crate::RequestManager) -> impl std::future::Future<Output = Result<E, Error>> + Send;
// }

// pub trait TRequestManager {
//     fn query<V: GraphQLVariables, E: GraphQLEntity<V>>(&self, operation_name: &str, variables: V) -> impl std::future::Future<Output = Result<E, Error>> + Send; 
// }