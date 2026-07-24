# Rust Port

One Cargo workspace for the whole port line — rooted at
`week1_baseline/Cargo.toml`, one directory up from here — not per-step, so
each new step only adds its own path to `members` on top of it.

```toml
# week1_baseline/Cargo.toml
[workspace]
resolver = "2"
members = ["rust/00_config"]
```

Unlike the Python port's venv, no separate "install deps" step is needed —
`cargo run`/`cargo build` resolve and build dependencies from
`week1_baseline/Cargo.lock` on first invocation, sharing one `target/` build
cache across every step. Each step's `bin/rust/<step>` launcher just `cd`s
into the step directory and runs `cargo run --example example`; Cargo finds
the workspace root by walking up parent directories, so this works
regardless of how deep the step directory sits.

Cargo package names can't start with a digit, so every step's
`Cargo.toml` package name is prefixed `boukensha_` and the step folder name
snake_cased — `00_config` becomes `boukensha_00_config`,
`01_struct_skeleton` becomes `boukensha_01_struct_skeleton`, and so on. This
only affects the `Cargo.toml` manifest and cross-crate `use` paths; `mod`/
`struct` names inside the code are unaffected.

## 09_global_executable: a real `boukensha` binary

Unlike every prior step, `09_global_executable` isn't run through a
`bin/rust/<step>` launcher — it's the official global executable, installed
via:

```bash
cargo install --path rust/09_global_executable
```

This produces a real `~/.cargo/bin/boukensha` binary, runnable from any
directory once `~/.cargo/bin` is on `PATH`. See
`rust/09_global_executable/README.md` for `BOUKENSHA_DIR`/`~/.boukensharc`
usage and the documented `BOUKENSHA_PATH` capability gap (Rust binaries
can't dynamically load another step's code at runtime the way Ruby/Python
can — use that step's own `bin/rust/<step>` launcher instead).
