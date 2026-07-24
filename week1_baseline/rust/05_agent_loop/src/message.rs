use std::fmt;

pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_use_id: Option<String>,
    pub content_blocks: Option<Vec<serde_json::Value>>,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) -> Self {
        Self { role: role.into(), content: content.into(), tool_use_id, content_blocks: None }
    }

    /// An assistant turn that included tool calls: `content` is the
    /// extracted text (for `Display`), `blocks` is the normalized
    /// `{"type": ..., ...}` array a backend needs to rebuild its
    /// wire-format assistant message.
    pub fn assistant(content: impl Into<String>, blocks: Vec<serde_json::Value>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
            tool_use_id: None,
            content_blocks: Some(blocks),
        }
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
