use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::icps::Error;
use crate::scanner::LocToken;
use crate::token::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment { values: HashMap::new(), enclosing: None }
    }

    pub fn new_local(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { values: HashMap::new(), enclosing: Some(enclosing) }
    }

    pub fn define(&mut self, name: &LocToken, value: Value) {
        self.values.insert(name.token.to_string(), value);
    }

    pub fn assign(&mut self, name: LocToken, value: Value) -> Result<(), Error> {
        if let Some(v) = self.values.get_mut(&name.token.to_string()) {
            *v = value;
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)
        } else {
            Err(Error::new(name.loc, format!("Undefined variable '{}'.", name.token).as_str()))
        }
    }

    pub fn get(&self, name: &LocToken) -> Result<Value, Error> {
        match self.values.get(&name.token.to_string()) {
            Some(value) => Ok(value.clone()),
            None => {
                match &self.enclosing {
                    Some(enclosing) => enclosing.borrow().get(name),
                    None => Err(Error::new(name.loc, format!("Undefined variable '{}'.", name.token).as_str()))
                }
            }
        }
    }
}
