use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::message::Message;
use crate::tasks::Task;
use crate::tool::Tool;

use super::base::{schema_parts, Backend, CostPerMillion, ModelInfo, ParsedResponse, PromptBackend, StopReason};

const BASE_URL: &str = "https://api.anthropic.com/v1/messages";

pub struct Anthropic {
    api_key: String,
    model: String,
    info: ModelInfo,
}

impl Anthropic {
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
                if msg.role == "tool_result" {
                    json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": msg.tool_use_id,
                            "content": msg.content,
                        }],
                    })
                } else if let Some(blocks) = &msg.content_blocks {
                    json!({"role": msg.role, "content": blocks})
                } else {
                    json!({"role": msg.role, "content": msg.content})
                }
            })
            .collect()
    }

    fn to_tools(&self, tools: &IndexMap<String, Tool>) -> Vec<serde_json::Value> {
        tools
            .values()
            .map(|tool| {
                let (properties, required) = schema_parts(&tool.parameters);
                json!({
                    "name": tool.name,
                    "description": tool.description,
                    "input_schema": {
                        "type": "object",
                        "properties": properties,
                        "required": required,
                    },
                })
            })
            .collect()
    }
}

impl Backend for Anthropic {
    fn backend_name() -> &'static str {
        "Anthropic"
    }

    fn models() -> HashMap<&'static str, ModelInfo> {
        HashMap::from([
            (
                "claude-haiku-4-5",
                ModelInfo {
                    context_window: 200_000,
                    cost_per_million: CostPerMillion { input: Some(1.0), output: Some(5.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "claude-haiku-4-5-20251001",
                ModelInfo {
                    context_window: 200_000,
                    cost_per_million: CostPerMillion { input: Some(1.0), output: Some(5.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "claude-sonnet-4-6",
                ModelInfo {
                    context_window: 1_000_000,
                    cost_per_million: CostPerMillion { input: Some(3.0), output: Some(15.0) },
                    usage_unit: "tokens",
                    usage_level: None,
                },
            ),
            (
                "claude-opus-4-8",
                ModelInfo {
                    context_window: 1_000_000,
                    cost_per_million: CostPerMillion { input: Some(5.0), output: Some(25.0) },
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

impl<T: Task> PromptBackend<T> for Anthropic {
    fn to_payload(
        &self,
        context: &Context<T>,
        max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> serde_json::Value {
        json!({
            "model": self.model,
            "system": context.system,
            "max_tokens": max_output_tokens,
            "tools": tools.map(|t| t.to_vec()).unwrap_or_else(|| self.to_tools(&context.tools)),
            "messages": self.to_messages(&context.messages),
        })
    }

    // Normalizes an Anthropic Messages API response into the common shape.
    fn parse_response(&self, response: &serde_json::Value) -> ParsedResponse {
        let stop_reason = if response["stop_reason"] == "tool_use" { StopReason::ToolUse } else { StopReason::EndTurn };
        let content = response["content"].as_array().cloned().unwrap_or_default();
        ParsedResponse { stop_reason, content }
    }

    fn headers(&self) -> IndexMap<String, String> {
        IndexMap::from([
            ("Content-Type".to_string(), "application/json".to_string()),
            ("x-api-key".to_string(), self.api_key.clone()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ])
    }

    fn url(&self) -> String {
        BASE_URL.to_string()
    }
}
