use std::fmt;

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
