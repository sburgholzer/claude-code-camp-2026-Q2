use std::collections::HashMap;
use std::fmt;

use serde_yaml_ng::Value;

pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub block: Box<dyn Fn(&HashMap<String, String>) -> String>,
}

impl Tool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
        block: impl Fn(&HashMap<String, String>) -> String + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            block: Box::new(block),
        }
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = self.description.chars().take(41).collect();
        let keys: Vec<&str> = self
            .parameters
            .as_mapping()
            .map(|m| m.keys().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        write!(f, "#<Tool name={} description={} params={:?}>", self.name, desc, keys)
    }
}
