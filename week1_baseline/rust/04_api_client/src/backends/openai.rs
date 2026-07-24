use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::tasks::Task;

use super::base::{chat_style_messages, function_wrapped_tools, Backend, CostPerMillion, ModelInfo, PromptBackend};

const BASE_URL: &str = "https://api.openai.com/v1/chat/completions";

pub struct OpenAI {
    api_key: String,
    model: String,
    info: ModelInfo,
}

impl OpenAI {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, UnsupportedModelError> {
        let model = model.into();
        let validated = Self::validate_model(&model)?;
        let info = Self::models().get(validated.as_str()).cloned().expect("validated model must exist in table");
        Ok(Self { api_key: api_key.into(), model: validated, info })
    }
}

impl Backend for OpenAI {
    fn backend_name() -> &'static str {
        "OpenAI"
    }

    fn models() -> HashMap<&'static str, ModelInfo> {
        HashMap::from([
            (
                "gpt-5.5",
                ModelInfo {
                    context_window: 1_000_000,
                    cost_per_million: CostPerMillion { input: Some(5.0), output: Some(30.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gpt-5.4",
                ModelInfo {
                    context_window: 1_000_000,
                    cost_per_million: CostPerMillion { input: Some(2.5), output: Some(15.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gpt-5.4-mini",
                ModelInfo {
                    context_window: 400_000,
                    cost_per_million: CostPerMillion { input: Some(0.75), output: Some(4.5) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
        ])
    }

    fn info(&self) -> &ModelInfo {
        &self.info
    }
}

impl<T: Task> PromptBackend<T> for OpenAI {
    fn to_payload(&self, context: &Context<T>, max_output_tokens: u32) -> serde_json::Value {
        json!({
            "model": self.model,
            "messages": chat_style_messages(&context.system, &context.messages, "tool_call_id"),
            "tools": function_wrapped_tools(&context.tools),
            "max_completion_tokens": max_output_tokens,
        })
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
        ])
    }

    fn url(&self) -> String {
        BASE_URL.to_string()
    }
}
