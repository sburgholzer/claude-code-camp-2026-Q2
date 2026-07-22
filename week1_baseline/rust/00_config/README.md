# 00 · Configuration (Rust port)

Behavior port of `ruby/00_config` / `python/00_config` — same `.boukensha/`
config directory, same `settings.yaml` / `.env`, same task-settings lookups
and system-prompt resolution. See `../README.md` for the one-time workspace
setup this step depends on.

We want to be able to manage all configuration from an external file eg.
`~/.boukensha/settings.yaml`. We want a dedicated struct to handle
configuration: `boukensha_00_config::config::Config`. As we add configuration
in each iteration we will be updating the configuration schema and struct.
We can hardcode defaults but we should not hardcode configurable values.

Configuration is organised by **task** — a role in the agentic loop bound to
its own LLM. week1_baseline only drives a single `player` task (the main
loop), but a more advanced loop will assign different LLMs to different
tasks. A task is either a "single-task" or a "multi-task" — the latter
being a full agent.

## Design Considerations

We want to use the standard library as much as possible, avoiding external
crates. Rust's stdlib has no YAML parser, so `settings.yaml` is parsed with
`serde_yaml_ng` (an actively-maintained fork; the original `serde_yaml` is
deprecated upstream). We also need `dotenvy` (an actively-maintained fork of
the abandoned `dotenv` crate) to load `.env` files, and `dirs` to resolve the
home directory — `std::env::home_dir()` carries long-standing
Windows-specific caveats in its docs, and `dirs` is the ecosystem's standard,
minimal-footprint fix.

**Shipped-prompts-dir resolution.** Ruby's `__dir__` and Python's `__file__`
are *runtime* lookups of "where does the currently-executing source file
live on disk," recomputed fresh every process start. Rust compiles to a
standalone binary with no source files left to ask, so there's no runtime
equivalent. Two options were considered:

1. **`concat!(env!("CARGO_MANIFEST_DIR"), "/prompts")`** (chosen) — a
   *compile-time* macro baking the absolute path to this crate's `prompts/`
   directory (on whatever machine ran `cargo build`) into the binary as a
   constant. At runtime, reading `prompts/system.md` from that path is a
   normal file read, so the shipped prompt file stays editable and
   reloadable without recompiling — matching Ruby/Python behavior.
2. **`include_str!`** — embeds the prompt *text itself* into the binary at
   compile time. No file path or file read at runtime, ever, but any change
   to the shipped prompt wording requires a recompile.

`CARGO_MANIFEST_DIR` was picked *for now* specifically because this project
only ever builds and runs from the same repo checkout on the same
machine — there's no cross-machine deployment at this stage. The tradeoff:
the compiled binary's shipped-prompt path is frozen to the build machine's
filesystem layout and would break if the binary were moved elsewhere or
built on one machine and run on another. A future step that adds real
packaging/distribution should revisit this choice.

## Code Changes

| File | Purpose |
|------|---------|
| `src/config.rs` | `Config` struct |
| `src/tasks/base.rs` | `Task` trait (provider/model + prompt resolution) |
| `src/tasks/player.rs` | concrete `Player` (the main loop) |
| `src/lib.rs` | top-level module wiring |
| `prompts/system.md` | default system prompt shipped with the library |
| `examples/example.rs` | runnable smoke-test |

---

## Config directory resolution

The struct looks for a `.boukensha/` directory in this order:

1. **`BOUKENSHA_DIR` env var** — set this to point at any directory you like.
2. **`~/.boukensha`** — the default location for a real install.

## Config directory structure

The struct expects the following:

```
.boukensha/
  .env                 # stores credentials eg. LLMs APIs (never committed to repo)
  settings.yaml        # all non-secret settings
  prompts/
    <task>/
      system.md        # per-task override for the default system prompt (optional)
```

---

## Tasks

`boukensha_00_config::tasks::base::Task` is a stateless trait. All behaviour
is expressed as associated functions that accept a `settings` value — no
instances are created. Concrete types implement `task_name()`. For now only
`boukensha_00_config::tasks::player::Player` exists; future steps add
per-turn ceilings (`max_iterations`, `max_turn_tokens`, `max_output_tokens`,
`compaction_threshold`) — these are **not** read yet.

`Config::tasks(name)` returns the raw value from `settings.yaml` under
`tasks:`. Pass `Some(name)` to look up a specific task's settings, then pass
it to the stateless trait:

```rust
Player::provider(&config.tasks(Some("player")).unwrap())?;
Player::system_prompt(
    &config.tasks(Some("player")).unwrap(),
    Some(&config.user_prompts_dir()),
    Some(Path::new(Config::PROMPTS_DIR)),
);
```

## System prompt resolution

Per task, `Player::system_prompt` is resolved in this order:

1. **`.boukensha/prompts/<task>/system.md`** — used when the task's
   `prompt_override.system` is `true` and the file exists.
2. **`prompts/system.md`** — the default system prompt shipped with the library.

(There is no top-level `system.override`; override is per-task via
`prompt_override.system`.)

## Configuration Schema

The following properties so far:
- `tasks`: a map of task name → task config (provider, model, prompt_override).
- `tasks.<name>.prompt_override.system`: when `true`, the task's
  `.boukensha/prompts/<name>/system.md` overrides the default system prompt.
- `mud`: MUD connection information for the main player.

```yaml
tasks:
  player:
    provider: anthropic        # provider name (string)
    model: claude-haiku-4-5
    prompt_override:
      system: true
mud:
  host: localhost
  port: 4000
  username: dummy
  password: helloworld
```

## Run Example

```bash
./week1_baseline/bin/rust/00_config
```

Expected output (values from your `.boukensha/`):

```
=== Boukensha Step 0: Configuration ===

Config dir:     /home/andrew/Sites/Claude-Code-Camp/.boukensha
Tasks:          player

-- player task --
Provider:       anthropic
Model:          claude-haiku-4-5
Prompt override?true
System prompt:  You are a MUD player assistant. Use the tools available to y...

MUD host:       localhost:4000
MUD user:       dummy

API key set?    true

#<Boukensha::Config dir=/home/andrew/Sites/Claude-Code-Camp/.boukensha tasks=player>
```

## Considerations
These are things we observed but we do not want fixed since future steps will break them again.
- We have a default prompt e.g. prompts/system.md. It is supposed to be scoped on task e.g. prompts/<task>/system.md
- Our settings file should accept .yml or .yaml, right now it only takes .yaml
