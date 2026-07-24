use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

pub mod agent;
pub mod backends;
pub mod client;
pub mod config;
pub mod context;
pub mod errors;
pub mod logger;
pub mod message;
pub mod prompt_builder;
pub mod registry;
pub mod tasks;
pub mod tool;

pub use agent::Agent;
pub use client::Client;
pub use config::Config;
pub use context::Context;
pub use errors::{ApiError, DispatchError, ToolError, UnknownToolError, UnsupportedModelError};
pub use logger::Logger;
pub use message::Message;
pub use prompt_builder::PromptBuilder;
pub use registry::Registry;
pub use tasks::{Player, Task};
pub use tool::Tool;

// Rust's analog of Ruby's `module Boukensha` self-methods (`quiet!`/
// `loud!`/`quiet?`, `debug!`/`debug?`, memoized `config`) / Python's
// `__init__.py` module-level functions — `std`-only state, no new
// dependency.
static QUIET: AtomicBool = AtomicBool::new(false);
static DEBUG: AtomicBool = AtomicBool::new(false);
static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(Config::new)
}

pub fn quiet() {
    QUIET.store(true, Ordering::Relaxed);
}

pub fn loud() {
    QUIET.store(false, Ordering::Relaxed);
}

pub fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

pub fn debug() {
    DEBUG.store(true, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}
