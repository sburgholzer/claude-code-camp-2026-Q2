use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use crate::message::Message;
use crate::tasks::Task;
use crate::tool::Tool;

pub struct Context<T> {
    task: PhantomData<T>,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: HashMap<String, Tool>,
}

impl<T: Task> Context<T> {
    pub fn new(system: Option<String>) -> Self {
        Self {
            task: PhantomData,
            system,
            messages: Vec::new(),
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) {
        self.messages.push(Message::new(role, content, tool_use_id));
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn turn_count(&self) -> usize {
        self.messages.len()
    }
}

impl<T: Task> fmt::Display for Context<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#<Context task={} turns={} tools={}>",
            T::task_name(),
            self.turn_count(),
            self.tool_count()
        )
    }
}
