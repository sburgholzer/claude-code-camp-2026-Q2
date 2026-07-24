use std::collections::HashMap;

use indexmap::IndexMap;
use serde_json::json;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::message::Message;
use crate::tasks::Task;
use crate::tool::Tool;

use super::base::{schema_parts, Backend, CostPerMillion, ModelInfo, ParsedResponse, PromptBackend, StopReason};

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
                    let parts = match &msg.content_blocks {
                        Some(blocks) => Self::assistant_parts(blocks),
                        None => vec![json!({"text": msg.content})],
                    };
                    json!({"role": "model", "parts": parts})
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

    // Rebuilds Gemini "model" parts from normalized content blocks
    // (the inverse of parse_response).
    fn assistant_parts(blocks: &[serde_json::Value]) -> Vec<serde_json::Value> {
        blocks
            .iter()
            .map(|b| {
                if b["type"] == "tool_use" {
                    json!({"functionCall": {"name": b["name"], "args": b["input"]}})
                } else {
                    json!({"text": b["text"]})
                }
            })
            .collect()
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
    fn to_payload(
        &self,
        context: &Context<T>,
        max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> serde_json::Value {
        json!({
            "systemInstruction": {"parts": [{"text": context.system}]},
            "contents": self.to_messages(&context.messages),
            "tools": tools.map(|t| t.to_vec()).unwrap_or_else(|| self.to_tools(&context.tools)),
            "generationConfig": {"maxOutputTokens": max_output_tokens},
        })
    }

    // Normalizes a Gemini generateContent response into the common shape.
    // Gemini doesn't assign call ids, so the function name is reused as the
    // id (Gemini also matches functionResponse back to a call by name).
    fn parse_response(&self, response: &serde_json::Value) -> ParsedResponse {
        let parts = response["candidates"][0]["content"]["parts"].as_array().cloned().unwrap_or_default();

        let mut content = Vec::new();
        let mut tool_used = false;

        for part in &parts {
            if !part["functionCall"].is_null() {
                let fc = &part["functionCall"];
                let args = if fc["args"].is_null() { json!({}) } else { fc["args"].clone() };
                content.push(json!({"type": "tool_use", "id": fc["name"], "name": fc["name"], "input": args}));
                tool_used = true;
            } else if let Some(text) = part["text"].as_str() {
                content.push(json!({"type": "text", "text": text}));
            }
        }

        ParsedResponse { stop_reason: if tool_used { StopReason::ToolUse } else { StopReason::EndTurn }, content }
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
