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


use std::collections::HashMap;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};
use serde_json::Error;

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

pub trait GraphQLQueryParams {
    fn get_formal_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_actual_part(&self, params: &mut ParamBuffer, prefix: &str);
    fn get_variables_part(&self, variables: &mut VariableBuffer, prefix: &str) -> Result<(), Error>;


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
        let mut variables = VariableBuffer::new();
        self.get_variables_part(&mut variables, "")?;

        variables.to_string()
    }

    fn get_variable_map(&self) -> Result<HashMap<String, serde_json::Value>, Error>  {
        let mut variables = VariableBuffer::new();
        self.get_variables_part(&mut variables, "")?;

        Ok(variables.map)
    }

    
}

#[derive(Serialize, Deserialize, Debug, DisplayAsJsonPretty)]
#[serde(rename_all = "camelCase")]
pub struct NoParams;

impl GraphQLQueryParams for NoParams {
    fn get_formal_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_actual_part(&self, _params: &mut ParamBuffer, _prefix: &str) {
    }

    fn get_variables_part(&self, _variables: &mut VariableBuffer, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }
}

// pub trait GraphQLComponent<Q: GraphQLQueryParams> {
//     fn get_query_part(params: &Q, prefix: &str) -> String;
//     // fn get_params(&self) -> Q;
// }

// pub trait GraphQLElement {
//     fn get_field_names() -> &'static str;
//     // fn get_params(&self) -> Q;
// }

pub trait GraphQLQuery<Q: GraphQLQueryParams> {
    fn get_query(request_name: &str, params: &Q) -> String;
    //  {
    //     format!(r#"
    //         query {}{} {{
    //             {}
    //         }}
    //     "#, request_name, params.get_formal(),
    //         Self::get_query_part(params, "")
    //     )
    // }

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