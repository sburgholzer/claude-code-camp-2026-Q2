use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml_ng::Value;

/// Untyped settings tree, matching Ruby's `Hash#dig` / Python's dict `dig`.
pub type Settings = Value;

#[derive(Debug)]
pub enum ConfigError {
    MissingSetting { task: &'static str, key: &'static str },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingSetting { task, key } => {
                write!(f, "tasks.{task}.{key} is required in settings.yml")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct Config {
    pub dir: PathBuf,
    settings: Settings,
}

impl Config {
    pub fn new() -> Self {
        let dir = Self::resolve_dir();
        Self::load_env(&dir);
        let settings = Self::load_settings(&dir);
        Self { dir, settings }
    }

    // ---------- tasks --------------------------------------------------

    /// `None` returns the full `tasks:` map; `Some(name)` returns that
    /// task's settings, or `None` if the task isn't configured.
    pub fn tasks(&self, name: Option<&str>) -> Option<Value> {
        let all = self
            .dig(&["tasks"])
            .cloned()
            .unwrap_or_else(|| Value::Mapping(Default::default()));

        match name {
            Some(n) => all.as_mapping().and_then(|m| m.get(n)).cloned(),
            None => Some(all),
        }
    }

    pub fn task_names(&self) -> Vec<String> {
        self.tasks(None)
            .and_then(|v| v.as_mapping().cloned())
            .map(|m| {
                m.keys()
                    .filter_map(|k| k.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn user_prompts_dir(&self) -> PathBuf {
        self.dir.join("prompts")
    }

    // ---------- MUD connection ------------------------------------------

    pub fn mud_host(&self) -> String {
        self.dig(&["mud", "host"])
            .and_then(|v| v.as_str())
            .unwrap_or("localhost")
            .to_string()
    }

    pub fn mud_port(&self) -> i64 {
        self.dig(&["mud", "port"]).and_then(|v| v.as_i64()).unwrap_or(4000)
    }

    pub fn mud_username(&self) -> Option<String> {
        self.dig(&["mud", "username"]).and_then(|v| v.as_str()).map(str::to_string)
    }

    pub fn mud_password(&self) -> Option<String> {
        self.dig(&["mud", "password"]).and_then(|v| v.as_str()).map(str::to_string)
    }

    // ---------- low-level helpers ----------------------------------------

    pub fn dig(&self, keys: &[&str]) -> Option<&Value> {
        let mut node = &self.settings;
        for key in keys {
            node = node.as_mapping()?.get(*key)?;
        }
        Some(node)
    }

    fn default_dir() -> PathBuf {
        dirs::home_dir()
            .expect("could not resolve home directory")
            .join(".boukensha")
    }

    fn resolve_dir() -> PathBuf {
        let raw = env::var("BOUKENSHA_DIR")
            .unwrap_or_else(|_| Self::default_dir().to_string_lossy().into_owned());
        let expanded = Self::expand_tilde(&raw);
        std::path::absolute(&expanded).unwrap_or(expanded)
    }

    fn expand_tilde(raw: &str) -> PathBuf {
        if raw == "~" {
            return dirs::home_dir().expect("could not resolve home directory");
        }
        if let Some(rest) = raw.strip_prefix("~/") {
            return dirs::home_dir()
                .expect("could not resolve home directory")
                .join(rest);
        }
        PathBuf::from(raw)
    }

    fn load_env(dir: &Path) {
        let env_file = dir.join(".env");
        if env_file.exists() {
            let _ = dotenvy::from_path(&env_file);
        }
    }

    fn load_settings(dir: &Path) -> Value {
        let settings_file = dir.join("settings.yaml");
        if !settings_file.exists() {
            return Value::Mapping(Default::default());
        }

        let text = fs::read_to_string(&settings_file).unwrap_or_default();
        match serde_yaml_ng::from_str(&text).unwrap_or(Value::Null) {
            Value::Null => Value::Mapping(Default::default()),
            value => value,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "#<Boukensha::Config dir={} tasks={}>",
            self.dir.display(),
            self.task_names().join(",")
        )
    }
}
