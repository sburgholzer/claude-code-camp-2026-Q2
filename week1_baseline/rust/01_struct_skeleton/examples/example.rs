use std::env;
use std::path::Path;

use boukensha_01_struct_skeleton::{Config, Context, Player, Task, Tool};

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
        None,
    );

    let mut ctx: Context<Player> = Context::new(system_prompt);

    let parameters = serde_yaml_ng::from_str(
        "direction:\n  type: string\n  description: The direction to move\n",
    )
    .expect("valid parameters yaml");

    ctx.register_tool(Tool::new(
        "move",
        "Move the player in a direction (north, south, east, west, up, down)",
        parameters,
        |direction| format!("You move {direction} into a torch-lit corridor."),
    ));

    ctx.add_message("user", "Explore north and tell me what you find.", None);
    ctx.add_message("assistant", "Sure, let me head north and take a look.", None);

    println!("=== Boukensha Step 1: Struct Skeleton ===");
    println!();
    println!("Config:   {config}");
    println!("Context:  {ctx}");
    println!("Tool:     {}", ctx.tools["move"]);
    println!("Messages:");
    for m in &ctx.messages {
        println!("  {m}");
    }
}
