use std::collections::hash_map::RandomState;
use std::fs::{self, File, OpenOptions};
use std::hash::{BuildHasher, Hasher};
use std::io::Write;
use std::path::PathBuf;

use indexmap::IndexMap;
use serde_json::json;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::backends::Backend;
use crate::message::Message;
use crate::tool::Tool;

const DEFAULT_SESSION_DIR: &str = "sessions";

pub struct Logger {
    pub session_id: String,
    pub path: PathBuf,
    log_io: File,
}

impl Logger {
    pub fn new(
        session_id: Option<String>,
        dir: Option<PathBuf>,
        log: Option<PathBuf>,
        snapshot: Option<serde_json::Value>,
    ) -> Self {
        let session_id = session_id.unwrap_or_else(Self::generate_session_id);
        let path = log.unwrap_or_else(|| dir.unwrap_or_else(Self::default_dir).join(format!("{session_id}.jsonl")));

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("could not create session log directory");
        }
        let log_io = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|e| panic!("failed to open session log {}: {e}", path.display()));

        let mut logger = Self { session_id, path, log_io };

        let mut start_event = serde_json::Map::new();
        start_event.insert("phase".to_string(), json!("session_start"));
        if let Some(serde_json::Value::Object(snap)) = snapshot {
            for (k, v) in snap {
                start_event.insert(k, v);
            }
        }
        logger.write_log(serde_json::Value::Object(start_event));

        logger
    }

    pub fn iteration(&mut self, n: u32, max: u32) {
        self.write_log(json!({"phase": "iteration", "n": n, "max": max}));
    }

    pub fn limit_reached(&mut self, kind: &str, n: u32, max: u32) {
        self.write_log(json!({"phase": "limit_reached", "kind": kind, "n": n, "max": max}));
    }

    pub fn turn_end(&mut self, reason: &str, iterations: u32, tokens: Option<&serde_json::Value>) {
        self.write_log(json!({"phase": "turn_end", "reason": reason, "iterations": iterations, "tokens": tokens}));
    }

    pub fn prompt(&mut self, messages: &[Message], tools: &IndexMap<String, Tool>) {
        let serialized: Vec<serde_json::Value> = messages.iter().map(Self::serialize_message).collect();
        let tool_names: Vec<&str> = tools.keys().map(String::as_str).collect();
        self.write_log(json!({
            "phase": "prompt",
            "message_count": messages.len(),
            "messages": serialized,
            "tool_count": tools.len(),
            "tools": tool_names,
        }));
    }

    pub fn tool_call(&mut self, name: &str, args: &serde_json::Value) {
        self.write_log(json!({"phase": "tool_call", "name": name, "args": args}));
    }

    pub fn tool_result(&mut self, name: &str, result: &str, ok: bool, error: Option<&str>) {
        self.write_log(json!({"phase": "tool_result", "name": name, "result": result, "ok": ok, "error": error}));
    }

    pub fn response(
        &mut self,
        text: &str,
        usage: Option<&serde_json::Value>,
        stop_reason: Option<&str>,
        task: Option<&str>,
        backend: Option<&dyn Backend>,
    ) {
        let mut event = serde_json::Map::new();
        event.insert("phase".to_string(), json!("response"));
        event.insert("text".to_string(), json!(text.trim()));
        event.insert("usage".to_string(), usage.cloned().unwrap_or(serde_json::Value::Null));
        event.insert("stop_reason".to_string(), json!(stop_reason));
        for (k, v) in Self::execution_metadata(task, backend, usage) {
            event.insert(k, v);
        }
        self.write_log(serde_json::Value::Object(event));
    }

    pub fn raw(&mut self, data: &serde_json::Value) {
        if !crate::is_debug() {
            return;
        }
        self.write_log(json!({"phase": "raw", "data": data}));
    }

    pub fn close(&mut self) {
        let _ = self.log_io.flush();
    }

    fn default_dir() -> PathBuf {
        crate::config().dir.join(DEFAULT_SESSION_DIR)
    }

    fn write_log(&mut self, event: serde_json::Value) {
        let mut map = match event {
            serde_json::Value::Object(m) => m,
            _ => unreachable!("log events are always objects"),
        };
        map.insert("session_id".to_string(), json!(self.session_id));
        map.insert("at".to_string(), json!(Self::now_iso8601()));
        let line = serde_json::Value::Object(map);
        writeln!(self.log_io, "{line}").expect("failed to write log line");
        self.log_io.flush().expect("failed to flush log line");
    }

    fn now_iso8601() -> String {
        OffsetDateTime::now_utc().format(&Rfc3339).expect("valid RFC3339 timestamp")
    }

    fn generate_session_id() -> String {
        let now = OffsetDateTime::now_utc();
        let timestamp = format!(
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            now.year(),
            u8::from(now.month()),
            now.day(),
            now.hour(),
            now.minute(),
            now.second()
        );
        format!("{timestamp}-{}", Self::random_hex(4))
    }

    /// `std` has no public RNG, but `RandomState::new()` already draws
    /// fresh OS-seeded entropy for HashDoS protection on every call;
    /// hashing with no `.write()` input still finalizes against those
    /// random keys, giving an effectively-random `u64` with no new
    /// dependency. Good enough for a unique session-id suffix (not a
    /// security boundary), matching `SecureRandom.hex(4)`/
    /// `secrets.token_hex(4)`'s 8-lowercase-hex-char shape.
    fn random_hex(nbytes: usize) -> String {
        let mut out = String::with_capacity(nbytes * 2);
        while out.len() < nbytes * 2 {
            let value = RandomState::new().build_hasher().finish();
            out.push_str(&format!("{value:016x}"));
        }
        out.truncate(nbytes * 2);
        out
    }

    fn serialize_message(msg: &Message) -> serde_json::Value {
        let content = match &msg.content_blocks {
            Some(blocks) => serde_json::Value::Array(blocks.clone()),
            None => serde_json::Value::String(msg.content.clone()),
        };
        json!({"role": msg.role, "content": content})
    }

    fn execution_metadata(
        task: Option<&str>,
        backend: Option<&dyn Backend>,
        usage: Option<&serde_json::Value>,
    ) -> serde_json::Map<String, serde_json::Value> {
        let mut metadata = serde_json::Map::new();

        let usage_present = usage.map(|u| !matches!(u, serde_json::Value::Object(m) if m.is_empty())).unwrap_or(false);
        if task.is_none() && backend.is_none() && !usage_present {
            return metadata;
        }

        if let Some(t) = task {
            metadata.insert("task".to_string(), json!(t));
        }

        let (input, output) = Self::usage_tokens(usage);

        if let Some(b) = backend {
            metadata.insert("provider".to_string(), json!(Self::snake_case(b.name())));
            metadata.insert("model".to_string(), json!(b.model()));
            metadata.insert("usage_unit".to_string(), json!(b.usage_unit()));
            if let Some(level) = b.usage_level() {
                metadata.insert("usage_level".to_string(), json!(level));
            }
        }

        if let Some(i) = input {
            metadata.insert("input_tokens".to_string(), json!(i));
        }
        if let Some(o) = output {
            metadata.insert("output_tokens".to_string(), json!(o));
        }

        if let (Some(b), Some(i), Some(o)) = (backend, input, output) {
            if let Some(cost) = b.estimate_cost(i as u64, o as u64) {
                metadata.insert("cost_usd".to_string(), json!(cost));
            }
        }

        metadata
    }

    fn usage_tokens(usage: Option<&serde_json::Value>) -> (Option<i64>, Option<i64>) {
        let input = Self::first_integer(
            usage,
            &["input_tokens", "prompt_tokens", "promptTokenCount", "prompt_eval_count"],
        );
        let output = Self::first_integer(
            usage,
            &["output_tokens", "completion_tokens", "candidatesTokenCount", "eval_count"],
        );
        (input, output)
    }

    fn first_integer(usage: Option<&serde_json::Value>, keys: &[&str]) -> Option<i64> {
        let usage = usage?;
        for key in keys {
            let Some(value) = usage.get(*key) else { continue };
            if value.is_null() {
                continue;
            }
            if let Some(i) = value.as_i64() {
                return Some(i);
            }
            if let Some(f) = value.as_f64() {
                return Some(f as i64);
            }
            if let Some(s) = value.as_str() {
                return s.parse().ok();
            }
            return None;
        }
        None
    }

    /// Mirrors Ruby's `backend.class.name.split("::").last.gsub(/([a-z\d])([A-Z])/,
    /// '\1_\2').downcase` / Python's `re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", ...)`:
    /// insert `_` before an uppercase letter that immediately follows a
    /// lowercase letter or digit, then lowercase everything.
    /// `"OllamaCloud"` -> `"ollama_cloud"`, `"OpenAI"` -> `"open_ai"`.
    fn snake_case(name: &str) -> String {
        let mut out = String::with_capacity(name.len() + 4);
        let mut prev_lower_or_digit = false;
        for ch in name.chars() {
            if ch.is_ascii_uppercase() && prev_lower_or_digit {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            prev_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
        out
    }
}
