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
