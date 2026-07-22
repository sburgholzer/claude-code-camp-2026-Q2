use std::env;
use std::path::Path;

use boukensha_00_config::{Config, Player, Task};

fn rb_bool(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

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

    println!("=== Boukensha Step 0: Configuration ===");
    println!();
    println!("Config dir:     {}", config.dir.display());
    println!("Tasks:          {}", config.task_names().join(", "));
    println!();
    println!("-- player task --");
    println!("Provider:       {}", Player::provider(&player_settings).expect("provider"));
    println!("Model:          {}", Player::model(&player_settings).expect("model"));
    println!(
        "Prompt override?{}",
        rb_bool(Player::prompt_override(&player_settings, "system"))
    );

    let system_prompt = Player::system_prompt(
        &player_settings,
        Some(&config.user_prompts_dir()),
        Some(Path::new(Config::PROMPTS_DIR)),
    )
    .expect("system prompt");
    let truncated: String = system_prompt.chars().take(60).collect();
    println!("System prompt:  {truncated}...");
    println!();
    println!("MUD host:       {}:{}", config.mud_host(), config.mud_port());
    println!("MUD user:       {}", config.mud_username().unwrap_or_default());
    println!();
    println!("API key set?    {}", rb_bool(env::var("ANTHROPIC_API_KEY").is_ok()));
    println!();
    println!("{config}");
}
