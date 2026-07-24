use std::collections::HashMap;

use crate::backends::StopReason;
use crate::client::{Client, DEFAULT_MAX_OUTPUT_TOKENS};
use crate::config::Settings;
use crate::errors::ApiError;
use crate::prompt_builder::PromptBuilder;
use crate::registry::Registry;
use crate::tasks::Task;

// Default iteration ceiling, used only when no task_settings are supplied at
// all. When task_settings are present, Task::max_iterations already supplies
// this same 25-default internally if the key is absent — see
// docs/plans/rust_port/05_agent_loop.md Decision 7.
const MAX_ITERATIONS: u32 = 25;

// The wind-down call is deliberately short and cheap.
const WRAP_UP_OUTPUT_TOKENS: u32 = 400;
const WRAP_UP_DIRECTIVE: &str = "You have reached your action limit for this turn. Do not call any more tools.\n\
Briefly summarize what you accomplished, what is still unfinished, and the\n\
single next action you would take.";

pub struct Agent<'a, T: Task> {
    registry: &'a mut Registry<T>,
    builder: &'a PromptBuilder<T>,
    client: &'a Client<'a, T>,
    max_iterations: u32,
    max_output_tokens: Option<u32>,
    iteration: u32,
}

impl<'a, T: Task> Agent<'a, T> {
    pub fn new(
        registry: &'a mut Registry<T>,
        builder: &'a PromptBuilder<T>,
        client: &'a Client<'a, T>,
        task_settings: Option<&Settings>,
        max_iterations: Option<u32>,
        max_output_tokens: Option<u32>,
    ) -> Self {
        let resolved_max_iterations =
            max_iterations.unwrap_or_else(|| task_settings.map(T::max_iterations).unwrap_or(MAX_ITERATIONS));
        let resolved_max_output_tokens = max_output_tokens.or_else(|| task_settings.map(T::max_output_tokens));

        Self {
            registry,
            builder,
            client,
            max_iterations: resolved_max_iterations,
            max_output_tokens: resolved_max_output_tokens,
            iteration: 0,
        }
    }

    pub fn run(&mut self) -> Result<String, ApiError> {
        loop {
            // Limits are *trigger thresholds*, not hard caps: once we reach one
            // we stop starting new work iterations and make exactly one
            // terminal wind-down call instead of failing.
            if self.iteration_limit_reached() {
                return Ok(self.wrap_up("max_iterations"));
            }

            self.iteration += 1;
            println!("[iteration {}/{}]", self.iteration, self.max_iterations);

            let max_output_tokens = self.max_output_tokens.unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);
            let response = self.client.call(&self.registry.context, max_output_tokens, None)?;
            let parsed = self.builder.parse_response(&response);

            match parsed.stop_reason {
                StopReason::ToolUse => self.handle_tool_calls(parsed.content),
                StopReason::EndTurn => return Ok(Self::extract_text(&parsed.content)),
            }
        }
    }

    fn iteration_limit_reached(&self) -> bool {
        self.max_iterations > 0 && self.iteration >= self.max_iterations
    }

    // One final, tools-disabled model call so the agent ends the turn in
    // character rather than aborting. Runs *outside* the counted loop: it
    // never re-checks the limits (so it cannot re-trigger) and does not
    // increment `iteration`. Falls back to a deterministic message if the
    // call fails.
    fn wrap_up(&mut self, reason: &str) -> String {
        self.registry.context.add_message("user", WRAP_UP_DIRECTIVE, None);

        match self.client.call(&self.registry.context, WRAP_UP_OUTPUT_TOKENS, Some(&[])) {
            Ok(response) => {
                let parsed = self.builder.parse_response(&response);
                let text = Self::extract_text(&parsed.content);
                if text.trim().is_empty() { self.fallback_message(reason) } else { text }
            }
            Err(_) => self.fallback_message(reason),
        }
    }

    fn fallback_message(&self, reason: &str) -> String {
        format!(
            "I reached my {}-action limit for this turn before finishing ({reason}). Ask me to continue and I'll pick up from here.",
            self.max_iterations
        )
    }

    fn extract_text(content: &[serde_json::Value]) -> String {
        content
            .iter()
            .filter(|b| b["type"] == "text")
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("")
    }

    fn handle_tool_calls(&mut self, content: Vec<serde_json::Value>) {
        let text = Self::extract_text(&content);
        self.registry.context.add_assistant_message(text, content.clone());

        for block in &content {
            if block["type"] != "tool_use" {
                continue;
            }

            let name = block["name"].as_str().unwrap_or_default();
            let input = &block["input"];
            let use_id = block["id"].as_str().map(str::to_string);

            println!("  tool call → {name}({input})");
            let args = json_object_to_string_map(input);
            let result = self.registry.dispatch(name, &args).expect("tool must be registered");
            let preview: String = result.chars().take(61).collect();
            println!("  tool result → {preview}");

            self.registry.context.add_message("tool_result", result, use_id);
        }
    }
}

/// Converts a tool_use block's JSON `input` object into the `HashMap<String,
/// String>` shape `Registry::dispatch`'s block signature expects (fixed since
/// `02_the_registry`, unchanged by this step). String values pass through
/// as-is; any other JSON value is stringified as compact JSON.
fn json_object_to_string_map(value: &serde_json::Value) -> HashMap<String, String> {
    value
        .as_object()
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| (k.clone(), v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string())))
                .collect()
        })
        .unwrap_or_default()
}
