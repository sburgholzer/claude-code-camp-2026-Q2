use std::collections::HashMap;

use serde_yaml_ng::Value;

use crate::registry::Registry;
use crate::tasks::Task;

/// Passed to `run`'s `register` callback. Exposes only `tool`, keeping the
/// DSL surface intentionally small.
pub struct RunDsl<'a, T: Task> {
    registry: &'a mut Registry<T>,
}

impl<'a, T: Task> RunDsl<'a, T> {
    pub fn new(registry: &'a mut Registry<T>) -> Self {
        Self { registry }
    }

    pub fn tool(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
        block: impl Fn(&HashMap<String, String>) -> Result<String, String> + 'static,
    ) {
        self.registry.tool(name, description, parameters, block);
    }
}
