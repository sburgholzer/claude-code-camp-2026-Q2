use std::env;
use std::fs;
use std::path::Path;

use serde_yaml_ng::Value;

use boukensha_04_api_client::backends::{Anthropic, Gemini, Ollama, OllamaCloud, OpenAI, PromptBackend};
use boukensha_04_api_client::{Client, Config, Context, Player, PromptBuilder, Registry, Task};

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

    let ctx: Context<Player> = Context::new(system_prompt);
    let mut registry = Registry::new(ctx);

    let read_file_params: Value = serde_yaml_ng::from_str(
        "path:\n  type: string\n  description: The file path to read\n",
    )
    .expect("valid parameters yaml");
    registry.tool("read_file", "Read the contents of a file from disk", read_file_params, |args| {
        let path = args.get("path").cloned().unwrap_or_default();
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"))
    });

    let list_directory_params: Value = serde_yaml_ng::from_str(
        "path:\n  type: string\n  description: The directory path to list\n",
    )
    .expect("valid parameters yaml");
    registry.tool("list_directory", "List files in a directory", list_directory_params, |args| {
        let path = args.get("path").cloned().unwrap_or_default();
        let entries = fs::read_dir(&path).unwrap_or_else(|e| panic!("failed to list {path}: {e}"));
        entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('.'))
            .collect::<Vec<_>>()
            .join("\n")
    });

    registry.context.add_message("user", "What files are in the current directory?", None);

    println!("=== BOUKENSHA Step 4: API Client ===");

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

    let builder = PromptBuilder::new(&registry.context, backend);
    let client = Client::new(&builder);

    println!();
    println!("Config: {config}");
    println!("Provider: {provider}");
    println!("Model: {model}");
    println!("Sending request to {}...", builder.url());
    println!();

    let response = client.call(1024).expect("API request must succeed");
    println!("Raw response:");
    println!("{}", serde_json::to_string_pretty(&response).expect("response must serialize"));
}
