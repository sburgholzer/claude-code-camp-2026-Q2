use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::tasks::Task;

use super::base::{chat_style_messages, function_wrapped_tools, Backend, CostPerMillion, ModelInfo, PromptBackend};

const BASE_URL: &str = "https://ollama.com";

pub struct OllamaCloud {
    api_key: String,
    model: String,
    info: ModelInfo,
}

impl OllamaCloud {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, UnsupportedModelError> {
        let model = model.into();
        let validated = Self::validate_model(&model)?;
        let info = Self::models().get(validated.as_str()).cloned().expect("validated model must exist in table");
        Ok(Self { api_key: api_key.into(), model: validated, info })
    }
}

impl Backend for OllamaCloud {
    fn backend_name() -> &'static str {
        "OllamaCloud"
    }

    fn models() -> HashMap<&'static str, ModelInfo> {
        HashMap::from([
            (
                "gemma4:31b-cloud",
                ModelInfo {
                    context_window: 256_000,
                    cost_per_million: CostPerMillion { input: None, output: None },
                    usage_unit: "ollama_cloud_usage",
                    usage_level: Some("medium"),
                },
            ),
            (
                "minimax-m3:cloud",
                ModelInfo {
                    context_window: 512_000,
                    cost_per_million: CostPerMillion { input: None, output: None },
                    usage_unit: "ollama_cloud_usage",
                    usage_level: Some("high"),
                },
            ),
            (
                "kimi-k2.5:cloud",
                ModelInfo {
                    context_window: 256_000,
                    cost_per_million: CostPerMillion { input: None, output: None },
                    usage_unit: "ollama_cloud_usage",
                    usage_level: Some("high"),
                },
            ),
        ])
    }

    fn info(&self) -> &ModelInfo {
        &self.info
    }
}

impl<T: Task> PromptBackend<T> for OllamaCloud {
    fn to_payload(&self, context: &Context<T>, _max_output_tokens: u32) -> serde_json::Value {
        json!({
            "model": self.model,
            "stream": false,
            "messages": chat_style_messages(&context.system, &context.messages, "tool_name"),
            "tools": function_wrapped_tools(&context.tools),
        })
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), format!("Bearer {}", self.api_key)),
        ])
    }

    fn url(&self) -> String {
        format!("{}/api/chat", BASE_URL)
    }
}
