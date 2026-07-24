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
