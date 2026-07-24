use indexmap::IndexMap;

use crate::backends::PromptBackend;
use crate::context::Context;
use crate::tasks::Task;

pub struct PromptBuilder<'a, T: Task> {
    context: &'a Context<T>,
    backend: Box<dyn PromptBackend<T>>,
}

impl<'a, T: Task> PromptBuilder<'a, T> {
    pub fn new(context: &'a Context<T>, backend: Box<dyn PromptBackend<T>>) -> Self {
        Self { context, backend }
    }

    pub fn to_api_payload(&self, max_output_tokens: u32) -> serde_json::Value {
        self.backend.to_payload(self.context, max_output_tokens)
    }

    pub fn headers(&self) -> IndexMap<String, String> {
        self.backend.headers()
    }

    pub fn url(&self) -> String {
        self.backend.url()
    }
}
