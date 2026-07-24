// The `boukensha` console script — Rust's analog of Ruby's `bin/boukensha`
// + `boukensha_loader.rb` and Python's `boukensha_loader.py`. Installed via
// `cargo install --path .`, producing a real ~/.cargo/bin/boukensha binary.
//
// Unlike Ruby/Python, this binary can't dynamically load a *different*
// step's code at runtime (Rust binaries are compiled ahead-of-time; there
// is no `require`/`import`-by-path equivalent short of unsafe dynamic
// library loading, which this project doesn't take on for a teaching
// convenience that already has a working alternative). BOUKENSHA_PATH is
// still read — solely to print a note pointing at the per-step
// bin/rust/<step> launchers — but this binary always runs its own bundled
// (current) step. BOUKENSHA_DIR / ~/.boukensharc's config-dir resolution
// is fully portable and works exactly like Ruby/Python (see config.rs).
use std::env;

use boukensha_09_global_executable::{rc, repl, Player, RunDsl, VERSION};

fn main() {
    warn_if_boukensha_path_set();

    if env::var("BOUKENSHA_DEBUG").is_ok() {
        println!("[boukensha] loading from: bundled (v{VERSION})");
    }

    // No tools registered, matching Ruby's `Boukensha.repl` / Python's
    // `boukensha.repl()` being called with no block/register callback
    // from their own loaders.
    if let Err(e) = repl::<Player>(None, None, None, None, None, None, None, None::<fn(&mut RunDsl<Player>)>) {
        eprintln!("boukensha: {e}");
        std::process::exit(1);
    }
}

fn warn_if_boukensha_path_set() {
    let path = env::var("BOUKENSHA_PATH").ok().or_else(|| rc::read().get("BOUKENSHA_PATH").cloned());

    let Some(path) = path else { return };

    eprintln!(
        "[boukensha] BOUKENSHA_PATH={path} is set, but this Rust build can't switch which \
         step's code runs at runtime (Rust binaries are compiled ahead-of-time, unlike \
         Ruby/Python's dynamic require/import)."
    );
    eprintln!(
        "[boukensha] Running the bundled step (v{VERSION}) instead. To run an older step \
         directly, use its own launcher, e.g. bin/rust/<step>, or `cargo run --example \
         example` inside rust/<step>."
    );
}
