use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::agent::Agent;
use crate::client::Client;
use crate::config::Settings;
use crate::logger::Logger;
use crate::prompt_builder::PromptBuilder;
use crate::registry::Registry;
use crate::tasks::Task;

/// The interactive session loop.
///
/// It wraps the same primitives as a single `run()` call, but instead of
/// running once it stays alive: it reads a task from the user, runs the
/// agent, prints the reply, and loops back to the prompt.
///
/// The `Context` (owned by `registry`) is shared across every turn so
/// conversation history accumulates naturally — the agent sees the full
/// transcript each time it is called.
///
/// Built-in commands (not sent to the agent):
///   /help    print the command list
///   /quiet   suppress detailed logging
///   /loud    re-enable logging
///   /clear   wipe conversation history (tools stay registered)
///   /exit    leave the REPL
///   /quit    alias for /exit
pub struct Repl<'a, T: Task> {
    registry: &'a mut Registry<T>,
    builder: &'a PromptBuilder<T>,
    client: &'a Client<'a, T>,
    logger: &'a mut Logger,
    config_dir: Option<PathBuf>,
    provider: Option<String>,
    model: Option<String>,
    version: Option<&'static str>,
    // Accepted for parity with `new`'s Ruby/Python-mirrored signature,
    // but no longer read anywhere — `banner()` dropped its API-key-
    // status line this step, exactly like `repl.rb`/`repl.py`, whose
    // `@api_key`/`self.api_key` are equally unused dead state now.
    #[allow(dead_code)]
    api_key: Option<String>,
    task_settings: Option<&'a Settings>,
    max_iterations: Option<u32>,
    max_output_tokens: Option<u32>,
    turn: u32,
}

impl<'a, T: Task> Repl<'a, T> {
    const PROMPT: &'static str = "boukensha> ";

    const HELP: &'static str = "Commands:\n  /quiet   suppress logging output\n  /loud    re-enable logging output\n  /clear   wipe conversation history (tools stay)\n  /exit    leave the REPL\n  /help    show this message\n";

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        registry: &'a mut Registry<T>,
        builder: &'a PromptBuilder<T>,
        client: &'a Client<'a, T>,
        logger: &'a mut Logger,
        config_dir: Option<PathBuf>,
        provider: Option<String>,
        model: Option<String>,
        version: Option<&'static str>,
        api_key: Option<String>,
        task_settings: Option<&'a Settings>,
        max_iterations: Option<u32>,
        max_output_tokens: Option<u32>,
    ) -> Self {
        Self {
            registry,
            builder,
            client,
            logger,
            config_dir,
            provider,
            model,
            version,
            api_key,
            task_settings,
            max_iterations,
            max_output_tokens,
            turn: 0,
        }
    }

    pub fn start(&mut self) {
        println!("{}", self.banner());

        let stdin = io::stdin();
        let mut line = String::new();

        loop {
            print!("{}", Self::PROMPT);
            let _ = io::stdout().flush();

            line.clear();
            let bytes_read = stdin.lock().read_line(&mut line).unwrap_or(0);
            if bytes_read == 0 {
                break; // EOF / Ctrl-D
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            match trimmed {
                "/exit" | "/quit" => {
                    println!("Goodbye.");
                    break;
                }
                "/help" => {
                    println!("{}", Self::HELP);
                    continue;
                }
                "/quiet" => {
                    crate::quiet();
                    println!("(logging suppressed — type /loud to re-enable)");
                    continue;
                }
                "/loud" => {
                    crate::loud();
                    println!("(logging enabled)");
                    continue;
                }
                "/clear" => {
                    self.registry.context.clear_messages();
                    self.turn = 0;
                    println!("(conversation history cleared)");
                    continue;
                }
                _ => {}
            }

            self.run_turn(trimmed);
        }
    }

    fn banner(&self) -> String {
        let ver = self.version.unwrap_or("?.?.?");
        let padding = " ".repeat(9usize.saturating_sub(ver.len()));

        let config_line = self.config_dir.as_ref().map(|d| d.display().to_string()).unwrap_or_else(|| "(default)".to_string());

        let lines = [
            String::new(),
            "╔══════════════════════════════════════╗".to_string(),
            format!("║  BOUKENSHA MUD Assistant (v{ver}){padding}║"),
            "╚══════════════════════════════════════╝".to_string(),
            format!("  config:        {config_line}"),
            format!("  provider:      {}", self.provider.as_deref().unwrap_or("(default)")),
            format!("  model:         {}", self.model.as_deref().unwrap_or("(default)")),
            String::new(),
            "  /quiet or /loud   toggle logging".to_string(),
            "  /clear           reset conversation history".to_string(),
            "  /exit or /quit    leave the REPL".to_string(),
            String::new(),
        ];
        lines.join("\n")
    }

    fn run_turn(&mut self, input: &str) {
        self.turn += 1;
        self.logger.turn(self.turn);

        self.registry.context.add_message("user", input, None);

        let mut agent = Agent::new(
            &mut *self.registry,
            self.builder,
            self.client,
            &mut *self.logger,
            self.task_settings,
            self.max_iterations,
            self.max_output_tokens,
        );

        // Only `ApiError` can come back here — `Agent::run`'s Rust
        // signature is `Result<String, ApiError>`, so unlike Ruby's/
        // Python's two-clause rescue (LoopError/ApiError), there is no
        // second error branch to write: LoopError is never constructed
        // anywhere in this port, matching Ruby/Python never raising it
        // either (see docs/plans/rust_port/08_the_repl_loop.md Decision 6).
        match agent.run() {
            Ok(result) => {
                println!();
                println!("{result}");
            }
            Err(e) => {
                println!("\n[error] API call failed: {e}");
            }
        }
    }
}
