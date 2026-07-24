use std::collections::HashMap;

use serde_yaml_ng::Value;

use crate::context::Context;
use crate::errors::{DispatchError, ToolError, UnknownToolError};
use crate::tasks::Task;
use crate::tool::Tool;

pub struct Registry<T> {
    pub context: Context<T>,
}

impl<T: Task> Registry<T> {
    pub fn new(context: Context<T>) -> Self {
        Self { context }
    }

    pub fn tool(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
        block: impl Fn(&HashMap<String, String>) -> Result<String, String> + 'static,
    ) {
        self.context.register_tool(Tool::new(name, description, parameters, block));
    }

    pub fn dispatch(&self, name: &str, args: &HashMap<String, String>) -> Result<String, DispatchError> {
        let tool = self
            .context
            .tools
            .get(name)
            .ok_or_else(|| DispatchError::UnknownTool(UnknownToolError { name: name.to_string() }))?;
        (tool.block)(args).map_err(|msg| DispatchError::ToolFailed(ToolError(msg)))
    }
}
