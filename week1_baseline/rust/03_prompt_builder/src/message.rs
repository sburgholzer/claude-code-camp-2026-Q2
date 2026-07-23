use std::fmt;

pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_use_id: Option<String>,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) -> Self {
        Self { role: role.into(), content: content.into(), tool_use_id }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_tag = self
            .tool_use_id
            .as_deref()
            .map(|id| format!(" [{id}]"))
            .unwrap_or_default();
        let content: String = self.content.chars().take(61).collect();
        write!(f, "#<Message role={}{} content={}...>", self.role, id_tag, content)
    }
}
