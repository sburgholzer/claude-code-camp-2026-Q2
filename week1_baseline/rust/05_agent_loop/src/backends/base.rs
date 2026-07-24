use std::collections::HashMap;

use indexmap::IndexMap;
use serde_yaml_ng::Value;

use crate::context::Context;
use crate::errors::UnsupportedModelError;
use crate::message::Message;
use crate::tasks::Task;

#[derive(Debug, Clone)]
pub struct CostPerMillion {
    pub input: Option<f64>,
    pub output: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub context_window: u64,
    pub cost_per_million: CostPerMillion,
    pub usage_unit: &'static str,
    pub usage_level: Option<&'static str>,
}

pub trait Backend {
    fn backend_name() -> &'static str
    where
        Self: Sized;

    fn models() -> HashMap<&'static str, ModelInfo>
    where
        Self: Sized;

    fn info(&self) -> &ModelInfo;

    fn validate_model(model: &str) -> Result<String, UnsupportedModelError>
    where
        Self: Sized,
    {
        let table = Self::models();
        if table.contains_key(model) {
            return Ok(model.to_string());
        }
        let mut supported: Vec<&str> = table.keys().copied().collect();
        supported.sort();
        Err(UnsupportedModelError(format!(
            "{} does not support model '{}'. Supported models: {}",
            Self::backend_name(),
            model,
            supported.join(", ")
        )))
    }

    fn context_window(&self) -> u64 {
        self.info().context_window
    }

    fn input_token_cost_per_million(&self) -> Option<f64> {
        self.info().cost_per_million.input
    }

    fn output_token_cost_per_million(&self) -> Option<f64> {
        self.info().cost_per_million.output
    }

    fn usage_unit(&self) -> &'static str {
        self.info().usage_unit
    }

    fn usage_level(&self) -> Option<&'static str> {
        self.info().usage_level
    }

    fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> Option<f64> {
        let input_cost = self.input_token_cost_per_million()?;
        let output_cost = self.output_token_cost_per_million()?;
        Some(((input_tokens as f64 * input_cost) + (output_tokens as f64 * output_cost)) / 1_000_000.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    ToolUse,
    EndTurn,
}

/// The common shape every backend normalizes its raw response into:
///   { stop_reason: "tool_use" | "end_turn", content: [ {"type": "text", "text": ...} | {"type": "tool_use", "id": ..., "name": ..., "input": ...} ] }
#[derive(Debug, Clone)]
pub struct ParsedResponse {
    pub stop_reason: StopReason,
    pub content: Vec<serde_json::Value>,
}

pub trait PromptBackend<T: Task> {
    fn to_payload(
        &self,
        context: &Context<T>,
        max_output_tokens: u32,
        tools: Option<&[serde_json::Value]>,
    ) -> serde_json::Value;
    fn parse_response(&self, response: &serde_json::Value) -> ParsedResponse;
    fn headers(&self) -> IndexMap<String, String>;
    fn url(&self) -> String;
}

/// Converts a tool's YAML-authored parameter schema into
/// (`properties`, `required`) for a JSON Schema-shaped `parameters`/
/// `input_schema` object.
pub fn schema_parts(parameters: &Value) -> (serde_json::Value, Vec<String>) {
    let properties = serde_json::to_value(parameters).expect("parameters must serialize to JSON");
    let required = properties
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    (properties, required)
}

pub fn function_wrapped_tools(tools: &IndexMap<String, crate::tool::Tool>) -> Vec<serde_json::Value> {
    tools
        .values()
        .map(|tool| {
            let (properties, required) = schema_parts(&tool.parameters);
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": {
                        "type": "object",
                        "properties": properties,
                        "required": required,
                    },
                },
            })
        })
        .collect()
}

pub fn chat_style_messages(
    system: &Option<String>,
    messages: &[Message],
    tool_id_field: &str,
    assistant_message: impl Fn(&[serde_json::Value]) -> serde_json::Value,
) -> Vec<serde_json::Value> {
    let mut result = vec![serde_json::json!({"role": "system", "content": system})];
    result.extend(messages.iter().map(|msg| {
        if msg.role == "tool_result" {
            serde_json::json!({
                "role": "tool",
                tool_id_field: msg.tool_use_id,
                "content": msg.content,
            })
        } else if msg.role == "assistant" {
            match &msg.content_blocks {
                Some(blocks) => assistant_message(blocks),
                None => serde_json::json!({"role": "assistant", "content": msg.content}),
            }
        } else {
            serde_json::json!({"role": msg.role, "content": msg.content})
        }
    }));
    result
}

pub fn extract_text(blocks: &[serde_json::Value]) -> String {
    blocks
        .iter()
        .filter(|b| b["type"] == "text")
        .filter_map(|b| b["text"].as_str())
        .collect::<Vec<_>>()
        .join("")
}

/// Rebuilds an Ollama/OllamaCloud assistant message from normalized content
/// blocks (the inverse of their shared `parse_response` shape). Neither
/// provider assigns call ids, so `tool_calls` carries no `id` field.
pub fn tool_call_assistant_message(blocks: &[serde_json::Value]) -> serde_json::Value {
    let tool_calls: Vec<serde_json::Value> = blocks
        .iter()
        .filter(|b| b["type"] == "tool_use")
        .map(|b| serde_json::json!({"function": {"name": b["name"], "arguments": b["input"]}}))
        .collect();

    let mut message = serde_json::json!({"role": "assistant", "content": extract_text(blocks)});
    if !tool_calls.is_empty() {
        message["tool_calls"] = serde_json::json!(tool_calls);
    }
    message
}
