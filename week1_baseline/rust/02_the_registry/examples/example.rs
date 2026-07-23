use std::collections::HashMap;
use std::env;
use std::path::Path;

use serde_yaml_ng::Value;

use boukensha_02_the_registry::{Config, Context, Player, Registry, Task};

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

    let ctx: Context<Player> = Context::new(system_prompt);
    let mut registry = Registry::new(ctx);

    // Notice that we now register the tools through the registry instead of
    // directly on the context in the previous step. They will still be
    // attached to context which is why we pass it into our registry when we
    // initialize it.

    let move_params: Value = serde_yaml_ng::from_str("direction:\n  type: string\n")
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

    let shout_params: Value = serde_yaml_ng::from_str("message:\n  type: string\n")
        .expect("valid parameters yaml");
    registry.tool(
        "shout",
        "Shout a message so everyone in the zone can hear it",
        shout_params,
        |args| args.get("message").cloned().unwrap_or_default().to_uppercase(),
    );

    println!("=== BOUKENSHA Step 2: Tool Registry ===");
    println!();
    println!("Config:  {config}");
    println!("Context: {}", registry.context);
    println!("Tools:");
    for t in registry.context.tools.values() {
        println!("  {t}");
    }
    println!();

    // Here we are mimicking what the agent would do when it needs to call a
    // tool from the registry. We are still missing the actual code that
    // would decide when to call the registry for a tool.
    println!("Dispatching 'shout' with message='dragon spotted'...");
    let result = registry
        .dispatch(
            "shout",
            &HashMap::from([("message".to_string(), "dragon spotted".to_string())]),
        )
        .unwrap();
    println!("Result: {result}");
    println!();

    println!("Dispatching 'move' with direction='north'...");
    let result = registry
        .dispatch(
            "move",
            &HashMap::from([("direction".to_string(), "north".to_string())]),
        )
        .unwrap();
    println!("Result: {result}");
    println!();

    match registry.dispatch("flee", &HashMap::new()) {
        Ok(_) => unreachable!("flee should not be registered"),
        Err(e) => println!("UnknownToolError caught: {e}"),
    }
}
