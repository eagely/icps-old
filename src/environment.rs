use std::collections::HashMap;
use crate::icps::Error;
use crate::scanner::LocToken;
use crate::token::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Environment { values: HashMap::new(), enclosing: None }
    }
    pub(crate) fn new_local(enclosing: Environment) -> Self {
        Environment { values: HashMap::new(), enclosing: Some(Box::new(enclosing)) }
    }

    pub fn define(&mut self, name: LocToken, value: Value) {
        self.values.insert(name.token.to_string(), value);
    }

    pub fn assign(&mut self, name: LocToken, value: Value) -> Result<(), Error> {
        if self.values.contains_key(name.token.to_string().as_str()) {
            self.values.insert(name.token.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        }
        else {
            Err(Error::new(name.loc, format!("Undefined variable '{}'.", name.token.to_string()).as_str()))
        }
    }

    pub fn get(&self, name: LocToken) -> Result<Value, Error> {
        if self.values.contains_key(&name.token.to_string()) {
            Ok(self.values.get(&name.token.to_string()).unwrap().clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(Error::new(name.loc, "Environment Error: Undefined variable."))
        }
    }
}
