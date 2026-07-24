use std::fmt;

use crate::config::ConfigError;

#[derive(Debug)]
pub struct UnknownToolError {
    pub name: String,
}

impl fmt::Display for UnknownToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No tool registered as '{}'", self.name)
    }
}

impl std::error::Error for UnknownToolError {}

#[derive(Debug)]
pub struct UnsupportedModelError(pub String);

impl fmt::Display for UnsupportedModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for UnsupportedModelError {}

#[derive(Debug)]
pub struct ApiError(pub String);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ApiError {}

#[derive(Debug)]
pub struct ToolError(pub String);

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ToolError {}

#[derive(Debug)]
pub struct LoopError(pub String);

impl fmt::Display for LoopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for LoopError {}

/// `Registry::dispatch`'s error type — either the tool name wasn't
/// registered, or the tool's own block returned `Err`. Both are caught
/// by `Agent::handle_tool_calls` and logged instead of propagating.
#[derive(Debug)]
pub enum DispatchError {
    UnknownTool(UnknownToolError),
    ToolFailed(ToolError),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DispatchError::UnknownTool(e) => write!(f, "UnknownToolError: {e}"),
            DispatchError::ToolFailed(e) => write!(f, "ToolError: {e}"),
        }
    }
}

impl std::error::Error for DispatchError {}

/// `boukensha::run`'s error type — aggregates every fallible step between
/// resolving task settings and running the agent loop.
#[derive(Debug)]
pub enum RunError {
    Config(ConfigError),
    UnsupportedModel(UnsupportedModelError),
    UnknownBackend(String),
    Api(ApiError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Config(e) => write!(f, "{e}"),
            RunError::UnsupportedModel(e) => write!(f, "{e}"),
            RunError::UnknownBackend(name) => write!(
                f,
                "Unknown backend '{name}'. Use 'anthropic', 'openai', 'gemini', 'ollama', or 'ollama_cloud'."
            ),
            RunError::Api(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RunError {}
