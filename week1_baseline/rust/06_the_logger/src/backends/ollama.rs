use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::tasks::Task;

use super::base::{
    chat_style_messages, function_wrapped_tools, tool_call_assistant_message, Backend, CostPerMillion, ModelInfo,
    ParsedResponse, PromptBackend, StopReason,
};

const DEFAULT_HOST: &str = "http://localhost:11434";

pub struct Ollama {
    host: String,
    model: String,
    info: ModelInfo,
}

impl Ollama {
    pub fn new(model: impl Into<String>, host: Option<String>) -> Result<Self, UnsupportedModelError> {
        let model = model.into();
        let validated = Self::validate_model(&model)?;
        let info = Self::models().get(validated.as_str()).cloned().expect("validated model must exist in table");
        Ok(Self { host: host.unwrap_or_else(|| DEFAULT_HOST.to_string()), model: validated, info })
    }
}

impl Backend for Ollama {
    fn backend_name() -> &'static str {
        "Ollama"
    }

    fn models() -> HashMap<&'static str, ModelInfo> {
        let local = |context_window: u64| ModelInfo {
            context_window,
            cost_per_million: CostPerMillion { input: Some(0.0), output: Some(0.0) },
            usage_unit: "local_compute",
            usage_level: None,
        };

        HashMap::from([
            ("gemma4", local(128_000)),
            ("gemma4:e2b", local(128_000)),
            ("gemma4:e4b", local(128_000)),
            ("gemma4:12b", local(256_000)),
            ("gemma4:26b", local(256_000)),
            ("gemma4:31b", local(256_000)),
            ("qwen3:30b", local(256_000)),
            ("qwen3:8b", local(40_000)),
            ("deepseek-r1:8b", local(128_000)),
        ])
    }

    fn info(&self) -> &ModelInfo {
        &self.info
    }

    fn name(&self) -> &'static str {
        Self::backend_name()
    }

    fn model(&self) -> &str {
        &self.model
    }
}

impl<T: Task> PromptBackend<T> for Ollama {
    fn to_payload(
        &self,
        context: &Context<T>,
        _max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> serde_json::Value {
        json!({
            "model": self.model,
            "stream": false,
            "messages": chat_style_messages(&context.system, &context.messages, "tool_name", tool_call_assistant_message),
            "tools": tools.map(|t| t.to_vec()).unwrap_or_else(|| function_wrapped_tools(&context.tools)),
        })
    }

    // Normalizes an Ollama /api/chat response into the common shape.
    // Ollama doesn't assign call ids, so the function name is reused as the
    // id (Ollama also matches tool results back to a call by name).
    fn parse_response(&self, response: &serde_json::Value) -> ParsedResponse {
        let message = &response["message"];
        let tool_calls = message["tool_calls"].as_array().cloned().unwrap_or_default();

        let mut content = Vec::new();
        if let Some(text) = message["content"].as_str() {
            if !text.is_empty() {
                content.push(json!({"type": "text", "text": text}));
            }
        }

        for tc in &tool_calls {
            let name = &tc["function"]["name"];
            let input = if tc["function"]["arguments"].is_null() { json!({}) } else { tc["function"]["arguments"].clone() };
            content.push(json!({"type": "tool_use", "id": name, "name": name, "input": input}));
        }

        ParsedResponse {
            stop_reason: if tool_calls.is_empty() { StopReason::EndTurn } else { StopReason::ToolUse },
            content,
        }
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([("Content-Type".to_string(), "application/json".to_string())])
    }

    fn url(&self) -> String {
        format!("{}/api/chat", self.host)
    }
}
