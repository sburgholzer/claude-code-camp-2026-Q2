use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::message::Message;
use crate::tasks::Task;
use crate::tool::Tool;

use super::base::{schema_parts, Backend, CostPerMillion, ModelInfo, PromptBackend};

const BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

pub struct Gemini {
    api_key: String,
    model: String,
    info: ModelInfo,
}

impl Gemini {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, UnsupportedModelError> {
        let model = model.into();
        let validated = Self::validate_model(&model)?;
        let info = Self::models().get(validated.as_str()).cloned().expect("validated model must exist in table");
        Ok(Self { api_key: api_key.into(), model: validated, info })
    }

    fn to_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|msg| {
                if msg.role == "assistant" {
                    json!({"role": "model", "parts": [{"text": msg.content}]})
                } else if msg.role == "tool_result" {
                    json!({
                        "role": "user",
                        "parts": [{
                            "functionResponse": {
                                "name": msg.tool_use_id,
                                "response": {"content": msg.content},
                            },
                        }],
                    })
                } else {
                    json!({"role": msg.role, "parts": [{"text": msg.content}]})
                }
            })
            .collect()
    }

    fn to_tools(&self, tools: &IndexMap<String, Tool>) -> Vec<serde_json::Value> {
        if tools.is_empty() {
            return Vec::new();
        }

        let declarations: Vec<serde_json::Value> = tools
            .values()
            .map(|tool| {
                let (properties, required) = schema_parts(&tool.parameters);
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": {
                        "type": "object",
                        "properties": properties,
                        "required": required,
                    },
                })
            })
            .collect();

        vec![json!({"functionDeclarations": declarations})]
    }
}

impl Backend for Gemini {
    fn backend_name() -> &'static str {
        "Gemini"
    }

    fn models() -> HashMap<&'static str, ModelInfo> {
        HashMap::from([
            (
                "gemini-3.5-flash",
                ModelInfo {
                    context_window: 1_048_576,
                    cost_per_million: CostPerMillion { input: Some(1.5), output: Some(9.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gemini-3.1-flash-lite",
                ModelInfo {
                    context_window: 1_048_576,
                    cost_per_million: CostPerMillion { input: Some(0.25), output: Some(1.5) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gemini-2.5-pro",
                ModelInfo {
                    context_window: 1_048_576,
                    cost_per_million: CostPerMillion { input: Some(1.25), output: Some(10.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gemini-2.5-flash",
                ModelInfo {
                    context_window: 1_048_576,
                    cost_per_million: CostPerMillion { input: Some(0.30), output: Some(2.50) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "gemini-2.5-flash-lite",
                ModelInfo {
                    context_window: 1_048_576,
                    cost_per_million: CostPerMillion { input: Some(0.10), output: Some(0.40) },
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

impl<T: Task> PromptBackend<T> for Gemini {
    fn to_payload(&self, context: &Context<T>, max_output_tokens: u32) -> serde_json::Value {
        json!({
            "systemInstruction": {"parts": [{"text": context.system}]},
            "contents": self.to_messages(&context.messages),
            "tools": self.to_tools(&context.tools),
            "generationConfig": {"maxOutputTokens": max_output_tokens},
        })
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
            ("x-goog-api-key".to_string(), self.api_key.clone()),
        ])
    }

    fn url(&self) -> String {
        format!("{}/{}:generateContent", BASE_URL, self.model)
    }
}
