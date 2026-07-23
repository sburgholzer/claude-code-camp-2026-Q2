# Rust Port Plan — 00 Config

## Goal

Port the behavior of `week1_baseline/python/00_config/` (itself a behavior
port of `week1_baseline/ruby/00_config/`) to `week1_baseline/rust/00_config/`
(directory already exists, currently empty). End state: a runnable Rust
example that resolves the same `.boukensha/` config directory, loads the
same `settings.yaml` / `.env`, and exposes task-settings lookups +
system-prompt resolution with the same behavior as the Ruby and Python
versions, against the **same** `.boukensha/` fixture at the repo root so
all three languages can be verified against one shared config.

This is a behavior port, not a redesign. Python is the more direct
reference (closer language shape — dynamic dict + dotted lookups), but
Ruby's README remains the ultimate spec where the two disagree.

## Source files to port (read these to know what to build)

| File | Role |
|---|---|
| `week1_baseline/python/00_config/README.md` | Design spec ported from Ruby: dir resolution order, config schema, task/prompt resolution rules, expected example output |
| `week1_baseline/python/00_config/lib/boukensha/config.py` | `Config` — dir resolution, `.env` loading, `settings.yaml` loading, `tasks`, `mud_*`, `dig`, `__repr__` |
| `week1_baseline/python/00_config/lib/boukensha/tasks/base.py` | `Base` — abstract, stateless classmethod API: `task_name`, `provider`, `model`, `prompt_override`, `prompt`/`system_prompt`, private `_fetch`/`_read_user_prompt`/`_read_default_prompt`/`_read_file` |
| `week1_baseline/python/00_config/lib/boukensha/tasks/player.py` | `Player(Base)` — concrete task, just sets `task_name = "player"` |
| `week1_baseline/python/00_config/prompts/system.md` | Default system prompt shipped with the library (fallback when no task override) |
| `week1_baseline/python/00_config/examples/example.py` | Runnable smoke test — the Rust port should produce the same fields in the same order |
| `week1_baseline/python/00_config/requirements.txt` | Declares the accepted third-party deps (`python-dotenv`, `PyYAML`) — Rust equivalents chosen below |
| `week1_baseline/bin/python/00_config` | Launcher shape to mirror: `cd` into the step dir, invoke via the shared toolchain, run the example |
| `week1_baseline/ruby/00_config/README.md` | Original spec, in case Python's port introduced any drift worth catching |

## Runtime fixture to reuse (do not duplicate)

| Path | Role |
|---|---|
| `.boukensha/settings.yaml` | Real settings fixture at repo root — `tasks.player.{provider,model,prompt_override.system}`, `mud.{host,port,username,password}` |
| `.boukensha/.env` | Real secrets fixture (`ANTHROPIC_API_KEY`) — gitignored, not committed |
| `.boukensha/prompts/player/system.md` | Per-task prompt override fixture, exercises the `prompt_override.system: true` path |

Both other ports point `BOUKENSHA_DIR` at this same directory via a
relative path from their example file. The Rust example does the same.

## Decisions (confirmed)

1. **Workspace structure** — a single Cargo workspace rooted at
   **`week1_baseline/Cargo.toml`** (new file) — **not** the true repo
   root. `.venv` lives at the repo root, but it's gitignored and
   contributes zero tracked files there — `git ls-files` shows the
   *only* tracked things sitting loose at repo root today are
   `.gitignore` and `README.md`; everything else is a directory. A
   workspace `Cargo.toml` is a real committed manifest (like
   `Gemfile`/`requirements.txt`), not a build artifact like `.venv/`
   itself, so putting it at true repo root would add a new category of
   permanent top-level clutter that nothing else in this repo does.
   `week0_explore/` also sets no repo-root precedent for shared
   tooling — `week0_explore/circlemud-world-parser/` has its own
   self-contained `pyproject.toml` and `.venv/`, not anything shared
   from above. So the workspace file is scoped to `week1_baseline/`,
   keeping each week self-contained and leaving the repo root's
   tracked-file list unchanged:
   ```toml
   # week1_baseline/Cargo.toml
   [workspace]
   resolver = "2"
   members = ["rust/00_config"]
   ```
   Every future `rust/NN_*` step gets appended to `members` using a
   `rust/NN_*` path relative to `week1_baseline/`. One shared
   `Cargo.lock` and `target/` dir at `week1_baseline/` — mirrors the
   Python port's shared-venv decision (one place to manage deps/build
   cache instead of N) at the same directory level `.venv` conceptually
   serves, rather than Ruby's per-step Gemfile isolation. Later steps
   that need the same deps (serde, serde_yaml_ng, dotenvy, dirs) don't
   recompile them from scratch.

   Practically this needs `week1_baseline/target/` ignored — either a
   new `week1_baseline/.gitignore` entry or a `target/` line in the
   root `.gitignore` (which already covers `.venv/` the same way,
   matching by name anywhere under the tree).

   One nice property: `cargo` locates the workspace root by walking up
   parent directories looking for a `Cargo.toml` with a `[workspace]`
   table, so running `cargo run --example example` from *inside*
   `week1_baseline/rust/00_config/` still transparently resolves
   against `week1_baseline/Cargo.lock`/`target/` — the launcher script
   below needs no path gymnastics despite the workspace file living
   two directories up.

2. **Settings representation** — untyped, matching Ruby's `Hash#dig` and
   Python's plain-`dict` `dig(*keys)`. `settings.yaml` deserializes via
   `serde_yaml_ng` into `serde_yaml::Value` (or a thin
   `type Settings = serde_yaml::Value` alias for readability at call
   sites). `Config::dig` walks the Value tree by string key the same way
   `config.py`'s `dig` walks the dict — no fixed schema, so a later step
   adding e.g. `tasks.player.max_iterations` needs zero struct changes
   here.

3. **Third-party crates** — the Rust port's equivalent of the
   Ruby-`dotenv`-gem / Python-`PyYAML`+`python-dotenv` carve-out:
   - `serde` (`derive` feature only where actually needed — likely just
     for any small typed pieces, not for `Settings` itself since that
     stays untyped `Value`)
   - `serde_yaml_ng` — actively-maintained fork; the original
     `serde_yaml` crate is deprecated upstream. Drop-in-compatible API
     (`serde_yaml_ng::from_str::<Value>(...)`).
   - `dotenvy` — actively-maintained fork of the abandoned `dotenv`
     crate. `dotenvy::from_path(env_file).ok()` mirrors
     `Dotenv.load(env_file) if File.exist?(env_file)` /
     `dotenv.load_dotenv(env_file)`.
   - `dirs` — resolved by decision 5 below; a fourth, narrowly-scoped
     exception for home-directory lookup.

   No other third-party deps for this step (no `thiserror`, no
   `anyhow`) — a hand-rolled error type keeps parity with the
   stdlib-first preference in `ITERATIONS.md` beyond the accepted
   exceptions.

4. **Shipped-prompts-dir resolution — `CARGO_MANIFEST_DIR`.** Python
   resolves the default-prompt directory via `__file__`, a *runtime*
   lookup of "where does the currently-executing source file live on
   disk" — recomputed fresh every time the program starts. Rust has no
   runtime equivalent, since compiling produces a standalone binary
   with no source files left to ask. Chosen fix:
   `concat!(env!("CARGO_MANIFEST_DIR"), "/prompts")` — a *compile-time*
   macro that bakes the absolute path to `00_config/` (on whatever
   machine did the `cargo build`) into the binary as a constant. At
   runtime, `Config::PROMPTS_DIR` is just that baked-in string, and
   reading `prompts/system.md` from it is a normal file read — so the
   file stays editable and reloadable without recompiling, same as
   Ruby/Python. The tradeoff (rejected as out of scope for now): this
   path is frozen to the build machine's filesystem layout, so the
   compiled binary would break if moved elsewhere or built on one
   machine and run on another. The alternative considered and turned
   down was `include_str!`, which embeds the prompt *text itself* into
   the binary at compile time (no file read, no path, ever) but would
   require a recompile any time the shipped prompt wording changes.
   Since this project only ever builds and runs from the same repo
   checkout on the same machine, that portability risk doesn't apply
   here — but **the ported `README.md` (see Target files) must state
   this explicitly**: name both options, and say plainly that
   `CARGO_MANIFEST_DIR` was chosen *for now* because there's no
   cross-machine deployment at this stage, so a future step that adds
   packaging/distribution should revisit it.

5. **Home-directory resolution for `DEFAULT_DIR` — `dirs` crate.**
   Ruby's `Dir.home` and Python's `Path.home()` are both stdlib
   functions that resolve the home directory correctly on every OS.
   Rust's `std::env::home_dir()` has a rockier history — it was
   deprecated for years over a Windows-specific bug (it read the wrong
   environment variable there) and, while since un-deprecated, its docs
   still carry caveats. The ecosystem's standard fix is the `dirs`
   crate — confirmed via `cargo search` as "a tiny low-level library"
   with a minimal footprint, matching the "small, narrowly-scoped
   exception" bar set for the other three crates. Chosen over
   hand-rolling `std::env::var("HOME")` (which is Unix-only and would
   silently break on Windows) so the Rust port isn't the one language
   port that fails to run on Windows. `Config::DEFAULT_DIR` becomes
   `dirs::home_dir().expect(...).join(".boukensha")`, called once at
   `Config` construction the same way Python computes
   `Config.DEFAULT_DIR` at class-definition time via `Path.home()`.

6. **Package naming — fully-qualified, `boukensha_00_config`.** Cargo
   *requires* every package to have a name (unlike Ruby's un-gemified
   per-step layout or Python's no-package-declared step folders), and
   that name can't start with a digit — confirmed by running
   `cargo init --name 00_config` directly, which fails hard:
   `error: invalid character '0' in package name: '00_config', the
   name cannot start with a digit`. So the package name can never be
   literally the same string as the step directory (`00_config`); it
   has to diverge somehow, at every step, forever — not just this one.
   Given this workspace will eventually hold ~12 members (steps 00
   through the full agent, per `ITERATIONS.md`), a short generic name
   like `config` risks collisions or ambiguity down the line (a later
   step wanting `registry` or `agent` hits the exact same problem, and
   `cargo build -p <name>` across a dozen members reads better fully
   qualified). Convention: prefix every step's package name with
   `boukensha_` and snake_case the step folder name — this step is
   `boukensha_00_config`; a future `01_struct_skeleton` step becomes
   `boukensha_01_struct_skeleton`, `02_the_registry` becomes
   `boukensha_02_the_registry`, and so on. Using underscores (not
   hyphens) means the package name *is* already a valid Rust
   identifier — no Cargo-side hyphen-to-underscore conversion to think
   about when the crate is referenced via `use` elsewhere. This applies
   to every future Rust step, not just this one, so it belongs here as
   a workspace-wide convention rather than a one-off choice.
   `src/`-level `mod`/`struct` names (`Config`, `Task`, `Player`) are
   unaffected either way — the package name only governs the
   `Cargo.toml` manifest and cross-crate references, never what you
   type inside the code.

## Target files to create (Rust)

```
week1_baseline/
  Cargo.toml                        # new: workspace root, members = ["rust/00_config"]
  rust/
    00_config/
      Cargo.toml                    # package "boukensha_00_config", deps: serde, serde_yaml_ng, dotenvy, dirs
      README.md                     # ported from python/00_config/README.md
      src/
        lib.rs                      # top-level module wiring (`pub mod config; pub mod tasks;`)
        config.rs                   # Config struct + impl
        tasks/
          mod.rs                    # `pub mod base; pub mod player;` + re-exports
          base.rs                   # Task trait with default methods
          player.rs                 # Player marker type implementing Task
      examples/
        example.rs                  # Cargo's built-in examples/ convention — runs via `cargo run --example example`
      prompts/
        system.md                   # copy of the shipped default system prompt, verbatim
```

Plus one line added to the root `.gitignore`: `target/` (a bare
`target/` pattern with no leading slash matches at any depth, so it
covers `week1_baseline/target/` the same way the existing `.venv/`
entry there already matches regardless of which directory it's in).

Plus a launcher at `week1_baseline/bin/rust/00_config` (the placeholder
file mentioned in the Python plan as "ignore for now" — this step fills
it in), following the repo's `bin/<language>/<step>` convention:

```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../rust/00_config"
cargo run --quiet --example example
```

No separate "install deps" step is needed the way Python's venv needed
one — `cargo run` resolves and builds dependencies from `Cargo.lock` on
first invocation, the same way `bundle exec` does for Ruby. So unlike
`python/README.md`, no port-wide `rust/README.md` setup doc is required
for dependency install — though a short `rust/README.md` noting the
workspace-per-line convention (mirroring the "one venv / one workspace"
framing) is still worth adding for discoverability.

`00_config/README.md` (ported from `python/00_config/README.md`) must
include a "Design Considerations" note on the shipped-prompt path
question (decision 4 above): name both `CARGO_MANIFEST_DIR` and
`include_str!` as the two options considered, and state that
`CARGO_MANIFEST_DIR` was picked *for now* specifically because this
step never builds on one machine and runs on another — flagged as
worth revisiting if a later step adds real packaging/distribution.

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`Tasks::Base` / `Base` (abstract, stateless, classmethod-only) →
  `trait Task` with default method bodies.** None of the trait methods
  take `&self` — they're associated functions, called as
  `Player::provider(&settings)`, matching the Ruby/Python call sites
  (`Player.provider(config.tasks("player"))`) more closely than an
  instance-method trait would:
  ```rust
  pub trait Task {
      fn task_name() -> &'static str;

      fn provider(settings: &Settings) -> Result<String, ConfigError> { .. }
      fn model(settings: &Settings) -> Result<String, ConfigError> { .. }
      fn prompt_override(settings: &Settings, prompt: &str) -> bool { .. }
      fn prompt(settings: &Settings, name: &str, user_prompts_dir: Option<&Path>, default_prompts_dir: Option<&Path>) -> Option<String> { .. }
      fn system_prompt(settings: &Settings, user_prompts_dir: Option<&Path>, default_prompts_dir: Option<&Path>) -> Option<String> { .. }
  }

  pub struct Player;
  impl Task for Player {
      fn task_name() -> &'static str { "player" }
  }
  ```
  `task_name` has no default body (every concrete task must define it) —
  the closest Rust equivalent to Python's `NotImplementedError` /
  Ruby's abstract-method-raises-if-uncalled pattern, enforced at compile
  time instead of at call time.

- **Missing `provider`/`model` (Python raises `ValueError`, Ruby raises
  `ArgumentError`) → a small `ConfigError` enum** implementing
  `std::error::Error` + `Display` by hand (no `thiserror`, per the
  stdlib-first carve-out above):
  ```rust
  pub enum ConfigError {
      MissingSetting { task: &'static str, key: &'static str },
  }
  ```
  `provider`/`model` return `Result<String, ConfigError>`; the example
  unwraps/`expect`s, same as the Ruby/Python examples not
  catching the raise.

- **`Config#dig(*keys)` → `Config::dig(&self, keys: &[&str]) -> Option<&serde_yaml::Value>`.**
  Walks `self.settings` (a `serde_yaml::Value::Mapping`) the same way
  Python's `dig` walks a `dict`, returning `None` on any missed key or
  non-mapping node — no string/symbol dual-key concern (Rust/serde_yaml
  keys are always `Value::String` here, same simplification Python's
  plan already made vs. Ruby).

- **`Config#to_s`/`#inspect` → `impl fmt::Display for Config` /
  `impl fmt::Debug for Config`** (or just `Display`, used for both print
  sites the way Python aliased `__str__ = __repr__`).

- **`File.exist? && File.read(...).strip` /
  `path.read_text().strip() if path.exists()` →
  `std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())`.**

- **Cargo's built-in `examples/` directory** is a closer parity to
  Ruby's `examples/example.rb` / Python's `examples/example.py` than
  anything hand-rolled — `cargo run --example example` compiles
  `examples/example.rs` against the crate's `lib.rs` exports, no
  separate binary target or `sys.path`-style import hack needed the way
  Python's launcher required.

## Behavior parity checklist (from the Ruby/Python spec)

- [ ] Config dir resolution order: `BOUKENSHA_DIR` env var, else
      `~/.boukensha` (`Config::DEFAULT_DIR` / equivalent, via `std::env::home_dir`-free
      approach — see open question below)
- [ ] Loads `.env` from the resolved dir if present (via `dotenvy`),
      before reading settings
- [ ] Loads `settings.yaml` from the resolved dir if present via
      `serde_yaml_ng`, else an empty mapping
- [ ] `Config::tasks(name: Option<&str>)` with `None` returns the full
      `tasks:` map; with `Some(name)` returns that task's settings
      (or `None`)
- [ ] `Config::user_prompts_dir()` = `<dir>/prompts`
- [ ] `Config::mud_host()` / `mud_port()` / `mud_username()` /
      `mud_password()` with the same defaults (`"localhost"`, `4000`)
- [ ] `Task` trait stays stateless/associated-function-only;
      `task_name()` has no default (compile error if a concrete type
      omits it); `provider`/`model` return `Err(ConfigError::..)` if
      missing from settings
- [ ] `prompt_override(settings, prompt = "system")` reads
      `settings["prompt_override"][prompt]`, defaults `false`
- [ ] System prompt resolution order: per-task override file
      (`<user_prompts_dir>/<task_name>/system.md`) if
      `prompt_override` is true and the file exists, else the default
      `prompts/system.md` shipped with the crate
- [ ] `Player::task_name() == "player"`, no other behavior
- [ ] `examples/example.rs` produces the same fields as
      `examples/example.py` in the same order (config dir, task list,
      provider, model, prompt-override flag, truncated system prompt,
      mud host/port/user, whether the API key env var is set, and a
      `Display`/`Debug` of the config)

## Open questions

None outstanding — all decided above.
