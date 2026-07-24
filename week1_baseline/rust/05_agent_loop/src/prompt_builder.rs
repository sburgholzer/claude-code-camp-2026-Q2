use indexmap::IndexMap;

use crate::backends::{ParsedResponse, PromptBackend};
use crate::context::Context;
use crate::tasks::Task;

pub struct PromptBuilder<T: Task> {
    backend: Box<dyn PromptBackend<T>>,
}

impl<T: Task> PromptBuilder<T> {
    pub fn new(backend: Box<dyn PromptBackend<T>>) -> Self {
        Self { backend }
    }

    pub fn to_api_payload(
        &self,
        context: &Context<T>,
        max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> serde_json::Value {
        self.backend.to_payload(context, max_output_tokens, tools)
    }

    pub fn parse_response(&self, response: &serde_json::Value) -> ParsedResponse {
        self.backend.parse_response(response)
    }

    pub fn headers(&self) -> IndexMap<String, String> {
        self.backend.headers()
    }

    pub fn url(&self) -> String {
        self.backend.url()
    }
}
