use std::path::Path;

use serde_yaml_ng::Value;

use crate::config::{ConfigError, Settings};

/// Abstract, stateless task. Concrete types implement `task_name`.
///
/// All behavior is expressed as associated functions that accept a
/// `settings` value — no instances are created, matching the Ruby/Python
/// classmethod-only `Base`.
pub trait Task {
    fn task_name() -> &'static str;

    fn provider(settings: &Settings) -> Result<String, ConfigError> {
        Self::fetch(settings, "provider")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or(ConfigError::MissingSetting { task: Self::task_name(), key: "provider" })
    }

    fn model(settings: &Settings) -> Result<String, ConfigError> {
        Self::fetch(settings, "model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or(ConfigError::MissingSetting { task: Self::task_name(), key: "model" })
    }

    fn prompt_override(settings: &Settings, prompt: &str) -> bool {
        Self::fetch(settings, "prompt_override")
            .and_then(Value::as_mapping)
            .and_then(|m| m.get(prompt))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    fn prompt(
        settings: &Settings,
        name: &str,
        user_prompts_dir: Option<&Path>,
        default_prompts_dir: Option<&Path>,
    ) -> Option<String> {
        if Self::prompt_override(settings, name) {
            if let Some(text) = Self::read_user_prompt(name, user_prompts_dir) {
                return Some(text);
            }
        }

        Self::read_default_prompt(name, default_prompts_dir)
    }

    fn system_prompt(
        settings: &Settings,
        user_prompts_dir: Option<&Path>,
        default_prompts_dir: Option<&Path>,
    ) -> Option<String> {
        Self::prompt(settings, "system", user_prompts_dir, default_prompts_dir)
    }

    fn fetch<'a>(settings: &'a Settings, key: &str) -> Option<&'a Value> {
        settings.as_mapping().and_then(|m| m.get(key))
    }

    fn read_user_prompt(prompt_name: &str, user_prompts_dir: Option<&Path>) -> Option<String> {
        let dir = user_prompts_dir?;
        Self::read_file(&dir.join(Self::task_name()).join(format!("{prompt_name}.md")))
    }

    fn read_default_prompt(prompt_name: &str, default_prompts_dir: Option<&Path>) -> Option<String> {
        let dir = default_prompts_dir?;
        Self::read_file(&dir.join(format!("{prompt_name}.md")))
    }

    fn read_file(path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    }
}
