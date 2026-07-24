use std::env;
use std::fs;
use std::path::Path;

use serde_yaml_ng::Value;

use boukensha_06_the_logger::backends::{Anthropic, Gemini, Ollama, OllamaCloud, OpenAI, PromptBackend};
use boukensha_06_the_logger::{Agent, Client, Config, Context, Logger, Player, PromptBuilder, Registry, Task};

fn main() {
    // Override the config directory so the example works from the repo
    // root. In real usage a user's ~/.boukensha is picked up automatically.
    if env::var("BOUKENSHA_DIR").is_err() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("could not resolve repo root");
        env::set_var("BOUKENSHA_DIR", repo_root.join(".boukensha"));
    }

    let config = Config::new();
    let player_settings = config.tasks(Some("player")).unwrap_or_default();
    let system_prompt = Player::system_prompt(
        &player_settings,
        Some(&config.user_prompts_dir()),
        Some(Path::new(Config::PROMPTS_DIR)),
    );
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let ctx: Context<Player> = Context::new(system_prompt);
    let mut registry = Registry::new(ctx);

    // Tools and the seed message are registered before `Agent::new` takes an
    // exclusive borrow of `registry` for the rest of the run (Ruby/Python
    // register tools after constructing the agent — harmless there since
    // neither language enforces borrow exclusivity).
    let read_file_params: Value = serde_yaml_ng::from_str(
        "path:\n  type: string\n  description: The file path to read\n",
    )
    .expect("valid parameters yaml");
    registry.tool("read_file", "Read the contents of a file from disk", read_file_params, {
        let base_dir = base_dir.to_path_buf();
        move |args| {
            let path = base_dir.join(args.get("path").cloned().unwrap_or_default());
            fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))
        }
    });

    let list_directory_params: Value = serde_yaml_ng::from_str(
        "path:\n  type: string\n  description: The directory path to list\n",
    )
    .expect("valid parameters yaml");
    registry.tool("list_directory", "List the files in a directory", list_directory_params, {
        let base_dir = base_dir.to_path_buf();
        move |args| {
            let path = base_dir.join(args.get("path").cloned().unwrap_or_default());
            let entries = fs::read_dir(&path).map_err(|e| format!("failed to list {}: {e}", path.display()))?;
            Ok(entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| !name.starts_with('.'))
                .collect::<Vec<_>>()
                .join(", "))
        }
    });

    registry.context.add_message(
        "user",
        "Read the README.md file and summarise what this MUD player assistant framework can do.",
        None,
    );

    println!("=== BOUKENSHA Step 6: The Logger ===");

    let provider = Player::provider(&player_settings).expect("provider is required");
    let model = Player::model(&player_settings).expect("model is required");

    let backend: Box<dyn PromptBackend<Player>> = match provider.as_str() {
        "anthropic" => Box::new(
            Anthropic::new(env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set"), &model)
                .expect("supported model"),
        ),
        "ollama" => Box::new(Ollama::new(&model, None).expect("supported model")),
        "ollama_cloud" => Box::new(
            OllamaCloud::new(env::var("OLLAMA_API_KEY").expect("OLLAMA_API_KEY must be set"), &model)
                .expect("supported model"),
        ),
        "openai" => Box::new(
            OpenAI::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"), &model)
                .expect("supported model"),
        ),
        "gemini" => Box::new(
            Gemini::new(env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"), &model)
                .expect("supported model"),
        ),
        other => panic!("Unsupported provider for player task: {other}"),
    };

    let builder = PromptBuilder::new(backend);
    let client = Client::new(&builder);

    // Writes structured JSONL events to .boukensha/sessions/<session-id>.jsonl.
    // Call boukensha_06_the_logger::debug() before running the agent to
    // include the full raw API response in those lines.
    let mut logger = Logger::new(None, None, None, None);

    println!();
    println!("Config: {config}");
    println!("Provider: {provider}");
    println!("Model: {model}");
    println!("Max iterations: {}", Player::max_iterations(&player_settings));
    println!("Max output tokens: {}", Player::max_output_tokens(&player_settings));
    println!();

    let mut agent = Agent::new(&mut registry, &builder, &client, &mut logger, Some(&player_settings), None, None);
    let result = agent.run().expect("agent run must succeed");

    println!();
    println!("=== FINAL RESPONSE ===");
    println!("{result}");
}
