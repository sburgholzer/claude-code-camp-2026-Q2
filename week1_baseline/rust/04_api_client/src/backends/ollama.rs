use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::tasks::Task;

use super::base::{chat_style_messages, function_wrapped_tools, Backend, CostPerMillion, ModelInfo, PromptBackend};

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
}

impl<T: Task> PromptBackend<T> for Ollama {
    fn to_payload(&self, context: &Context<T>, _max_output_tokens: u32) -> serde_json::Value {
        json!({
            "model": self.model,
            "stream": false,
            "messages": chat_style_messages(&context.system, &context.messages, "tool_name"),
            "tools": function_wrapped_tools(&context.tools),
        })
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([("Content-Type".to_string(), "application/json".to_string())])
    }

    fn url(&self) -> String {
        format!("{}/api/chat", self.host)
    }
}
