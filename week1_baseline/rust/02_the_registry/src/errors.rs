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
