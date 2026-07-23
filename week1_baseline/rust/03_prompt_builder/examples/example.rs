use std::env;
use std::path::Path;

use serde_yaml_ng::Value;

use boukensha_03_prompt_builder::backends::{Anthropic, Gemini, Ollama, OllamaCloud, OpenAI, PromptBackend};
use boukensha_03_prompt_builder::{Config, Context, Player, PromptBuilder, Registry, Task};

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

    let look_params: Value = serde_yaml_ng::from_str("{}").expect("valid parameters yaml");
    registry.tool(
        "look",
        "Look around the current room for details",
        look_params,
        |_args| "A damp stone corridor stretches north. Torches flicker on the walls.".to_string(),
    );

    let move_params: Value = serde_yaml_ng::from_str(
        "direction:\n  type: string\n  description: The direction to move\n",
    )
    .expect("valid parameters yaml");
    registry.tool(
        "move",
        "Move the player in a direction (north, south, east, west, up, down)",
        move_params,
        |args| {
            let direction = args.get("direction").cloned().unwrap_or_default();
            format!("You move {direction} into a torch-lit corridor.")
        },
    );

    registry.context.add_message(
        "user",
        "I just arrived in the dungeon. What's around me, and can you move north?",
        None,
    );
    registry.context.add_message("assistant", "Let me take a look around first.", None);
    registry.context.add_message(
        "tool_result",
        "A damp stone corridor stretches north. Torches flicker on the walls.",
        Some("toolu_01X".to_string()),
    );

    println!("=== BOUKENSHA Step 3: Prompt Builder ===");

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

    println!();
    println!("Config: {config}");
    println!("Provider: {provider}");
    println!("Model: {model}");
    println!(
        "{}",
        serde_json::to_string_pretty(&builder.to_api_payload(1024)).expect("payload must serialize")
    );
}
