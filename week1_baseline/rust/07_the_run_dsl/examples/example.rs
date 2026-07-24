use std::env;
use std::fs;
use std::path::Path;

use serde_yaml_ng::Value;

use boukensha_07_the_run_dsl::{config, run, Player, RunDsl};

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

    // Config is loaded automatically inside run() — system prompt, model,
    // and backend all come from ~/.boukensha (or BOUKENSHA_DIR) by default.
    // You can still override any of them via run()'s parameters.

    println!("=== BOUKENSHA Step 7: The Boukensha.run DSL ===");
    println!();
    println!("Config: {}", config());
    println!();

    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let result = run::<Player>(
        "Read the README.md file and summarise what this MUD player assistant framework can do.",
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(|dsl: &mut RunDsl<Player>| {
            let read_file_params: Value = serde_yaml_ng::from_str(
                "path:\n  type: string\n  description: The file path to read\n",
            )
            .expect("valid parameters yaml");
            dsl.tool("read_file", "Read the contents of a file from disk", read_file_params, {
                let base_dir = base_dir.clone();
                move |args| {
                    let path = base_dir.join(args.get("path").cloned().unwrap_or_default());
                    fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))
                }
            });

            let list_directory_params: Value = serde_yaml_ng::from_str(
                "path:\n  type: string\n  description: The directory path to list\n",
            )
            .expect("valid parameters yaml");
            dsl.tool("list_directory", "List the files in a directory", list_directory_params, {
                let base_dir = base_dir.clone();
                move |args| {
                    let path = base_dir.join(args.get("path").cloned().unwrap_or_default());
                    let entries =
                        fs::read_dir(&path).map_err(|e| format!("failed to list {}: {e}", path.display()))?;
                    Ok(entries
                        .filter_map(|entry| entry.ok())
                        .map(|entry| entry.file_name().to_string_lossy().into_owned())
                        .filter(|name| !name.starts_with('.'))
                        .collect::<Vec<_>>()
                        .join(", "))
                }
            });
        }),
    )
    .expect("agent run must succeed");

    println!();
    println!("=== FINAL RESPONSE ===");
    println!("{result}");
}
