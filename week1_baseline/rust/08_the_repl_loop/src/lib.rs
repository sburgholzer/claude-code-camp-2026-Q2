use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use backends::{Anthropic, Gemini, Ollama, OllamaCloud, OpenAI, PromptBackend};

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
pub mod repl;
pub mod run_dsl;
pub mod tasks;
pub mod tool;
pub mod version;

pub use agent::Agent;
pub use client::Client;
pub use config::Config;
pub use context::Context;
pub use errors::{ApiError, DispatchError, LoopError, RunError, ToolError, UnknownToolError, UnsupportedModelError};
pub use logger::Logger;
pub use message::Message;
pub use prompt_builder::PromptBuilder;
pub use registry::Registry;
pub use repl::Repl;
pub use run_dsl::RunDsl;
pub use tasks::{Player, Task};
pub use tool::Tool;
pub use version::VERSION;

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

// The top-level entry point. Wires together every primitive so the caller
// only has to describe *what* to do, not *how* to plumb it.
//
//   let result = run::<Player>(
//       "Summarise lib/boukensha",
//       None, None, None, None, None, None, None,
//       Some(|dsl: &mut RunDsl<Player>| {
//           dsl.tool("read_file", "Read a file from disk", params, |args| {
//               std::fs::read_to_string(&args["path"]).map_err(|e| e.to_string())
//           });
//       }),
//   );
//
// Arguments (positional, matching Ruby/Python's `run(task:, ...)` kwarg
// order — Rust has no named-argument syntax):
//   task:               The user message to hand the agent.
//   system:             System prompt. Defaults to the task's configured prompt.
//   model:              Model name. Defaults to the task's configured model.
//   backend:            "anthropic" (default), "openai", "gemini", "ollama", or "ollama_cloud".
//   api_key:            API key for the chosen backend. Defaults to the matching
//                       ANTHROPIC_API_KEY / OPENAI_API_KEY / GEMINI_API_KEY / OLLAMA_API_KEY
//                       env var (loaded from .boukensha/.env). Not needed for "ollama".
//   ollama_host:        Ollama base URL. `None` falls back to Ollama::new's own default.
//   log:                Optional JSONL path override. Defaults to .boukensha/sessions/<session-id>.jsonl.
//   max_output_tokens:  Per-reply output cap. Defaults to the task's configured value.
//   register:           Optional closure receiving a RunDsl to register tools on.
pub fn run<T: Task>(
    task: impl Into<String>,
    system: Option<String>,
    model: Option<String>,
    backend: Option<String>,
    api_key: Option<String>,
    ollama_host: Option<String>,
    log: Option<PathBuf>,
    max_output_tokens: Option<u32>,
    register: Option<impl FnOnce(&mut RunDsl<T>)>,
) -> Result<String, RunError> {
    let cfg = config(); // loads .env; populates std::env
    let task_settings = cfg.tasks(Some(T::task_name())).unwrap_or_default();

    let system = system.or_else(|| {
        T::system_prompt(&task_settings, Some(&cfg.user_prompts_dir()), Some(Path::new(Config::PROMPTS_DIR)))
    });
    let model = match model {
        Some(m) => m,
        None => T::model(&task_settings).map_err(RunError::Config)?,
    };
    let backend_name = match backend {
        Some(b) => b,
        None => T::provider(&task_settings).map_err(RunError::Config)?,
    };

    let api_key = api_key.or_else(|| match backend_name.as_str() {
        "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
        "openai" => env::var("OPENAI_API_KEY").ok(),
        "gemini" => env::var("GEMINI_API_KEY").ok(),
        "ollama_cloud" => env::var("OLLAMA_API_KEY").ok(),
        _ => None,
    });

    let ctx: Context<T> = Context::new(system);
    let mut registry = Registry::new(ctx);

    if let Some(register) = register {
        let mut dsl = RunDsl::new(&mut registry);
        register(&mut dsl);
    }

    let backend_impl: Box<dyn PromptBackend<T>> = match backend_name.as_str() {
        "anthropic" => {
            Box::new(Anthropic::new(api_key.unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?)
        }
        "openai" => Box::new(OpenAI::new(api_key.unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?),
        "gemini" => Box::new(Gemini::new(api_key.unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?),
        "ollama" => Box::new(Ollama::new(&model, ollama_host).map_err(RunError::UnsupportedModel)?),
        "ollama_cloud" => {
            Box::new(OllamaCloud::new(api_key.unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?)
        }
        other => return Err(RunError::UnknownBackend(other.to_string())),
    };

    let builder = PromptBuilder::new(backend_impl);
    let client = Client::new(&builder);

    let effective_max_iterations = T::max_iterations(&task_settings);
    let effective_max_output_tokens = max_output_tokens.unwrap_or_else(|| T::max_output_tokens(&task_settings));

    let snapshot = serde_json::json!({
        "task": T::task_name(),
        "max_iterations": effective_max_iterations,
        "max_output_tokens": effective_max_output_tokens,
        "model": model,
        "provider": backend_name,
    });
    let mut logger = Logger::new(None, None, log, Some(snapshot));

    registry.context.add_message("user", task.into(), None);

    let mut agent = Agent::new(
        &mut registry,
        &builder,
        &client,
        &mut logger,
        Some(&task_settings),
        Some(effective_max_iterations),
        Some(effective_max_output_tokens),
    );

    let result = agent.run().map_err(RunError::Api);
    logger.close();
    result
}

// Interactive REPL: register tools once, then loop — reading tasks from
// stdin, running the agent, and printing replies — until the user types
// exit or sends EOF.
//
// Conversation history accumulates across every turn so the agent always
// sees the full transcript.
//
// Arguments are the same as `run()`, minus `task` (the user supplies tasks
// interactively). system/model/backend/api_key all default to config
// values.
pub fn repl<T: Task>(
    system: Option<String>,
    model: Option<String>,
    backend: Option<String>,
    api_key: Option<String>,
    ollama_host: Option<String>,
    log: Option<PathBuf>,
    max_output_tokens: Option<u32>,
    register: Option<impl FnOnce(&mut RunDsl<T>)>,
) -> Result<(), RunError> {
    let cfg = config(); // loads .env; populates std::env
    let task_settings = cfg.tasks(Some(T::task_name())).unwrap_or_default();

    let system = system.or_else(|| {
        T::system_prompt(&task_settings, Some(&cfg.user_prompts_dir()), Some(Path::new(Config::PROMPTS_DIR)))
    });
    let model = match model {
        Some(m) => m,
        None => T::model(&task_settings).map_err(RunError::Config)?,
    };
    let backend_name = match backend {
        Some(b) => b,
        None => T::provider(&task_settings).map_err(RunError::Config)?,
    };

    let api_key = api_key.or_else(|| match backend_name.as_str() {
        "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
        "openai" => env::var("OPENAI_API_KEY").ok(),
        "gemini" => env::var("GEMINI_API_KEY").ok(),
        "ollama_cloud" => env::var("OLLAMA_API_KEY").ok(),
        _ => None,
    });

    let ctx: Context<T> = Context::new(system);
    let mut registry = Registry::new(ctx);

    if let Some(register) = register {
        let mut dsl = RunDsl::new(&mut registry);
        register(&mut dsl);
    }

    let backend_impl: Box<dyn PromptBackend<T>> = match backend_name.as_str() {
        "anthropic" => Box::new(
            Anthropic::new(api_key.clone().unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?,
        ),
        "openai" => {
            Box::new(OpenAI::new(api_key.clone().unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?)
        }
        "gemini" => {
            Box::new(Gemini::new(api_key.clone().unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?)
        }
        "ollama" => Box::new(Ollama::new(&model, ollama_host).map_err(RunError::UnsupportedModel)?),
        "ollama_cloud" => Box::new(
            OllamaCloud::new(api_key.clone().unwrap_or_default(), &model).map_err(RunError::UnsupportedModel)?,
        ),
        other => return Err(RunError::UnknownBackend(other.to_string())),
    };

    let builder = PromptBuilder::new(backend_impl);
    let client = Client::new(&builder);

    let effective_max_iterations = T::max_iterations(&task_settings);
    let effective_max_output_tokens = max_output_tokens.unwrap_or_else(|| T::max_output_tokens(&task_settings));

    let snapshot = serde_json::json!({
        "task": T::task_name(),
        "max_iterations": effective_max_iterations,
        "max_output_tokens": effective_max_output_tokens,
        "model": model.clone(),
        "provider": backend_name.clone(),
    });
    let mut logger = Logger::new(None, None, log, Some(snapshot));

    // Ctrl-C has no std-only cross-platform handling; installed once per
    // process (see docs/plans/rust_port/08_the_repl_loop.md Decision 3).
    // The handler runs on its own signal-handling thread with no safe way
    // to reach this function's locals (registry/logger aren't shareable
    // without extra Arc<Mutex<_>> machinery this step doesn't otherwise
    // need) — and Logger::write_log already flushes after every event, so
    // Logger::close() is redundant to run from the handler. Printing the
    // message and exiting directly is enough.
    let _ = ctrlc::set_handler(|| {
        println!("\nInterrupted.");
        let _ = io::stdout().flush();
        std::process::exit(0);
    });

    Repl::new(
        &mut registry,
        &builder,
        &client,
        &mut logger,
        Some(cfg.dir.clone()),
        Some(backend_name),
        Some(model),
        Some(VERSION),
        api_key,
        Some(&task_settings),
        Some(effective_max_iterations),
        Some(effective_max_output_tokens),
    )
    .start();

    logger.close();
    Ok(())
}
