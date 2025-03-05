use std::sync::Arc;

use display_json::DisplayAsJsonPretty;
use serde::{Deserialize, Serialize};

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
    fn get_variables_part(&self, variables: &mut serde_json::Map<String, serde_json::Value>, prefix: &str) -> Result<(), serde_json::Error>;


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

    fn get_variables(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.get_variable_map()?)
    }

    fn get_variable_map(&self) -> Result<serde_json::Map<String, serde_json::Value>, serde_json::Error>  {
        let mut variables: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        self.get_variables_part(&mut variables, "")?;

        Ok(variables)
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

    fn get_variables_part(&self, _variables: &mut serde_json::Map<String, serde_json::Value>, _prefix: &str) -> Result<(), serde_json::Error> {
        Ok(())
    }
}


pub trait GraphQLType<Q: GraphQLQueryParams> {
    fn get_query_part(params: &Q, prefix: &str) -> String {
        format!("{{ #get_query_part\n  {}\n}} #/get_query_part\n", Self::get_query_attributes(params, prefix))
    }

    fn get_query_attributes(params: &Q, prefix: &str) -> String;
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
    // Returns a bearer token, which may be cached if refresh is false, otherwise a new token will be obtained.
    // async fn get_authenticator(&mut self) -> Result<Arc<String>, Box<dyn StdError>>;
    fn get_authenticator(&self, refresh: bool) -> impl std::future::Future<Output = Result<Arc<String>, crate::Error>> + Send;

    // // Returns a fresh bearer token forcing a reauthentication
    // // async fn authenticate(&mut self) -> Result<Arc<String>, Box<dyn StdError>>;
    // fn authenticate(&self) -> impl std::future::Future<Output = Result<Arc<String>, Error>> + Send;
}
