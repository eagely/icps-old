use std::collections::HashMap;
use crate::token::Value;

struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    fn new() -> Self {
        Environment { values: HashMap::new() }
    }

    fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

}
