use std::fmt;
use std::marker::PhantomData;

use indexmap::IndexMap;

use crate::message::Message;
use crate::tasks::Task;
use crate::tool::Tool;

pub struct Context<T> {
    task: PhantomData<T>,
    pub system: Option<String>,
    pub messages: Vec<Message>,
    pub tools: IndexMap<String, Tool>,
}

impl<T: Task> Context<T> {
    pub fn new(system: Option<String>) -> Self {
        Self {
            task: PhantomData,
            system,
            messages: Vec::new(),
            tools: IndexMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) {
        self.messages.push(Message::new(role, content, tool_use_id));
    }

    pub fn add_assistant_message(&mut self, content: impl Into<String>, blocks: Vec<serde_json::Value>) {
        self.messages.push(Message::assistant(content, blocks));
    }

    /// Drop all conversation history, keeping tools and system prompt
    /// intact. Used by the REPL's `/clear` command.
    pub fn clear_messages(&mut self) {
        self.messages.clear();
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
