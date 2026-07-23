# Rust Port Plan — 01 Struct Skeleton

## Goal

Port the behavior of `week1_baseline/python/01_struct_skeleton/` (itself a
behavior port of `week1_baseline/ruby/01_struct_skeleton/`) to
`week1_baseline/rust/01_struct_skeleton/` (directory already exists,
currently empty). End state: a runnable Rust example that defines the three
data structures the rest of the port passes around — `Tool`, `Message`,
`Context` — carries forward `Config`/`Task`/`Player` unchanged from
`00_config`, and produces the same fields as the Ruby/Python examples,
against the **same** `.boukensha/` fixture at the repo root.

This is a behavior port, not a redesign. Python is the more direct
reference (closer language shape), but Ruby's actual `context.rb`/`tool.rb`/
`message.rb` code — not its README prose — remains the ultimate spec where
the two disagree (see Decision 5 below on why the README overstates what's
implemented).

## Source files to port (read these to know what to build)

| File | Role |
|---|---|
| `week1_baseline/python/01_struct_skeleton/README.md` | Design spec ported from Ruby: three data structures, field tables, expected example output |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/tool.py` | `Tool` dataclass — `name`, `description`, `parameters`, `block`; `__repr__` truncates description to 41 chars, lists `parameters` keys |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/message.py` | `Message` dataclass — `role`, `content`, `tool_use_id=None`; `__repr__` truncates content to 61 chars, appends `[id]` tag when `tool_use_id` is set |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/context.py` | `Context` — keyword-only `task`/`system`, `messages: []`, `tools: {}`; `register_tool`, `add_message`, `tool_count`, `turn_count` properties, `__repr__` |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/config.py` | `Config`, carried forward from `00_config` **minus `PROMPTS_DIR`** — this step ships no `prompts/` dir |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/tasks/base.py`, `tasks/player.py` | Byte-for-byte unchanged from `00_config` (confirmed via `diff`) |
| `week1_baseline/python/01_struct_skeleton/lib/boukensha/__init__.py` | Top-level exports: `Config`, `Context`, `Message`, `Player`, `Tool` |
| `week1_baseline/python/01_struct_skeleton/examples/example.py` | Runnable smoke test — the Rust port should produce the same fields in the same order |
| `week1_baseline/python/01_struct_skeleton/requirements.txt` | Same deps as `00_config` (`python-dotenv`, `PyYAML`) — no new ones for this step |
| `week1_baseline/bin/python/01_struct_skeleton` | Launcher shape to mirror |
| `week1_baseline/ruby/01_struct_skeleton/README.md` | Original spec — **note:** documents a `token_budget` field and a richer multi-line `Context#to_s` (with `description`/`tools` listing) that the actual `ruby/01_struct_skeleton/lib/boukensha/context.rb` code does **not** implement. Follow the real Ruby/Python code (single-line `to_s`, no `token_budget`), not the aspirational README prose — same code-over-README precedent as the `00_config` plan. |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/{tool,message,context}.rb` | Ground truth for actual (as opposed to documented) behavior |

## Runtime fixture to reuse (do not duplicate)

Same as `00_config` — `.boukensha/settings.yaml`, `.boukensha/.env`,
`.boukensha/prompts/player/system.md` at the repo root, pointed at via
`BOUKENSHA_DIR` set relative to the example file, same as the other two
language ports and as `rust/00_config`'s own example.

## Decisions (confirmed)

1. **Workspace membership** — append to the existing workspace member list
   rather than creating a new workspace:
   ```toml
   # week1_baseline/Cargo.toml
   [workspace]
   resolver = "2"
   members = ["rust/00_config", "rust/01_struct_skeleton"]
   ```
   Same shared `Cargo.lock`/`target/` as `00_config`, per the workspace
   decision already made and documented in `rust/README.md`.

2. **Per-step duplication, not a crate dependency on `boukensha_00_config`.**
   Ruby and Python both give every step folder its own full copy of
   `lib/boukensha/` — confirmed by `diff`, `python/01_struct_skeleton`'s
   `config.py`/`tasks/base.py`/`tasks/player.py` are copies of `00_config`'s
   (with one line removed from `config.py`), not imports from a shared
   package. The Rust port matches this: `rust/01_struct_skeleton` gets its
   own `config.rs`/`tasks/base.rs`/`tasks/player.rs`, copied forward from
   `rust/00_config` rather than declaring `boukensha_00_config` as a
   `[dependencies]` entry. This keeps every step folder self-contained and
   independently readable — the point of the step-by-step port — instead of
   turning `00_config` into a shared-library step that all later ones
   import. The **only** edit to the carried-forward files is deleting the
   `Config::PROMPTS_DIR` const (see Decision 6) to match Python's drop.
   `tasks/base.rs` and `tasks/player.rs` carry forward byte-for-byte.

3. **Package naming — `boukensha_01_struct_skeleton`.** Same convention
   established in the `00_config` plan and already previewed in
   `rust/README.md`.

4. **`Tool.parameters` stays untyped via `serde_yaml_ng::Value`.** Ruby's
   hash literal (`{ direction: { type: "string", description: "..." } }`)
   and Python's dict literal are both open-ended, unschema'd nested maps —
   the same "no fixed schema" shape `Config::dig` already handles for
   `settings.yaml`. Reusing `serde_yaml_ng::Value::Mapping` (rather than
   inventing a `ParamSpec` struct) keeps this step's `Tool` exactly as loose
   as the Ruby/Python versions, and — importantly — `serde_yaml_ng::Mapping`
   preserves insertion order (it's backed by an ordered map internally), so
   `params=[...]` prints keys in the same order they were inserted, matching
   Ruby `Hash`/Python `dict` iteration order. `Tool::parameters` is typed
   `serde_yaml_ng::Value`, constructed the same slightly-verbose way
   `settings.yaml` values are already handled in this port.

5. **`Tool.block` — `Box<dyn Fn(&str) -> String>`, not invoked this step.**
   Neither `examples/example.py` nor `examples/example.rb` ever calls the
   registered `move` block — this step only constructs and displays a
   `Tool`, so the block's signature is never exercised. The one concrete
   closure literal that exists (`lambda direction: f"..."` /
   `->(direction) { "..." }`) takes a single string argument and returns a
   string, so `Box<dyn Fn(&str) -> String>` is the narrowest signature that
   matches today's actual usage. Boxing (vs. a generic `Tool<F: Fn...>`
   type param) is required because `Context.tools` is a
   `HashMap<String, Tool>` holding heterogeneous closures — a generic
   `Tool<F>` couldn't be stored uniformly in one map, mirroring why Ruby/
   Python can freely mix different lambdas/callables in one hash/dict but
   Rust needs type erasure to do the same. **Flagged for revisiting**: once
   a later step actually dispatches tool calls, the single-`&str` signature
   will likely need to generalize (e.g. to a parsed-args map) — this step
   only needs it to compile and display, not to be final.

6. **No `PROMPTS_DIR`, no shipped `prompts/` dir — matches Python's drop.**
   `python/01_struct_skeleton/config.py` is `00_config`'s `config.py` with
   the `PROMPTS_DIR` class attribute deleted (confirmed via `diff`); this
   step's README states outright "no `PROMPTS_DIR` — this step ships no
   `prompts/` dir." `examples/example.py` calls `Player.system_prompt` with
   only `user_prompts_dir` (the per-task override path), never a
   `default_prompts_dir`. The Rust port mirrors this exactly: carried-
   forward `config.rs` has no `PROMPTS_DIR` const, `rust/01_struct_skeleton`
   ships no `prompts/` directory, and `examples/example.rs` calls
   `Player::system_prompt(&player_settings, Some(&config.user_prompts_dir()), None)`
   — `None` for `default_prompts_dir` where `00_config`'s example passed
   `Some(Path::new(Config::PROMPTS_DIR))`. This is fine because the shared
   `.boukensha/prompts/player/system.md` fixture always exists and
   `prompt_override.system: true` is set for `player` in the fixture
   settings — the default-prompt fallback path is simply never reached in
   this step's example, same as in Ruby/Python.

7. **`Message.role` stays `String`, not an enum.** Ruby/Python pass raw
   symbols/strings (`:user`, `"user"`) with no validation at this layer —
   `Message` doesn't constrain which roles are legal. Introducing a Rust
   `enum Role { User, Assistant, ToolResult }` now would be adding
   structure the source doesn't have yet, contrary to the "behavior port,
   not a redesign" ground rule. `Message::role: String` matches today's
   actual looseness; a later step that starts pattern-matching on role can
   introduce an enum then.

8. **`Context<T: Task>` replaces the runtime "task class" reference with a
   compile-time type parameter.** Ruby stores the literal class
   (`Boukensha::Tasks::Player`) as a value in `@task` and calls
   `task&.task_name` on it at display time; Python does the same with
   `self.task` holding the `Player` class object and calling
   `self.task.task_name()`. Rust's `Task` trait (established in `00_config`)
   already made every method an associated function with no `&self` —
   there's no instance to box as `dyn Task` and no need for one, since which
   task a `Context` is built for is known at compile time at every call
   site in this step. `Context<T: Task>` holds `task: PhantomData<T>`
   (zero-sized — costs nothing at runtime) and reads `T::task_name()`
   wherever Ruby/Python would call `self.task.task_name()`. The trait bound
   `T: Task` is placed only on the `impl` blocks that need it (`Display`),
   not on the struct definition itself, so `Context` doesn't over-constrain
   callers who never need to print it. Call sites read
   `Context::<Player>::new(Some(system_prompt))`, mirroring
   `Context.new(task: Player, system: system_prompt)`.

9. **Truncation via `.chars().take(n)`, not byte slicing.** Same reasoning
   already applied in `rust/00_config/examples/example.rs`'s
   `system_prompt.chars().take(60)`: Ruby's `str[0..40]` / Python's `[:41]`
   are character-index slices, and Rust byte-slicing (`&s[..41]`) panics on
   a non-ASCII multibyte boundary. `Tool`'s Display truncates `description`
   to 41 chars, `Message`'s Display truncates `content` to 61 chars, both
   via `.chars().take(n).collect::<String>()`.

## Target files to create (Rust)

```
week1_baseline/
  Cargo.toml                        # edit: members += "rust/01_struct_skeleton"
  rust/
    01_struct_skeleton/
      Cargo.toml                    # package "boukensha_01_struct_skeleton", same deps as 00_config
      README.md                     # ported from python/01_struct_skeleton/README.md
      src/
        lib.rs                      # `pub mod {config, tasks, tool, message, context};` + re-exports
        config.rs                   # copied from rust/00_config, PROMPTS_DIR const removed
        tasks/
          mod.rs                    # copied unchanged from rust/00_config
          base.rs                   # copied unchanged from rust/00_config
          player.rs                 # copied unchanged from rust/00_config
        tool.rs                     # new: Tool struct + Display
        message.rs                  # new: Message struct + Display
        context.rs                  # new: Context<T: Task> struct + Display
      examples/
        example.rs                  # `cargo run --example example`
```

No `prompts/` directory this step (Decision 6). No new crates — `Cargo.toml`
deps are the same four as `00_config` (`serde`, `serde_yaml_ng`, `dotenvy`,
`dirs`); `serde`'s derive feature stays declared-but-unused, matching
`00_config`'s own current state (nothing in either step derives
`Serialize`/`Deserialize` yet — both keep `Settings` as untyped `Value`).

Plus a launcher at `week1_baseline/bin/rust/01_struct_skeleton`, matching
`bin/rust/00_config`'s shape:

```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../rust/01_struct_skeleton"
cargo run --quiet --example example
```

`01_struct_skeleton/README.md` (ported from `python/01_struct_skeleton/README.md`)
keeps the same three data-structure field tables, and adds a "Porting notes"
section (mirroring Python's) covering:
- Ruby symbols (`:direction`) vs. Rust's `Vec<String>`-backed key list —
  `params=["direction"]` (double-quoted) differs from both Ruby's
  `[:direction]` and Python's `['direction']` (single-quoted); same category
  of divergence Python's README already flagged for Ruby.
- `Context<T: Task>`'s `PhantomData<T>` type-parameter substitution for
  Ruby/Python's runtime class reference (Decision 8).
- `Tool.block`'s `Box<dyn Fn(&str) -> String>` signature and that it's
  unexercised this step (Decision 5).
- No `PROMPTS_DIR` / no shipped `prompts/` dir this step (Decision 6),
  worded the same way Python's README states it.

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`Tool` (Ruby `Struct`, Python `@dataclass`) → plain struct + `Display`:**
  ```rust
  pub struct Tool {
      pub name: String,
      pub description: String,
      pub parameters: serde_yaml_ng::Value,
      pub block: Box<dyn Fn(&str) -> String>,
  }

  impl fmt::Display for Tool {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          let desc: String = self.description.chars().take(41).collect();
          let keys: Vec<&str> = self.parameters.as_mapping()
              .map(|m| m.keys().filter_map(|k| k.as_str()).collect())
              .unwrap_or_default();
          write!(f, "#<Tool name={} description={} params={:?}>", self.name, desc, keys)
      }
  }
  ```

- **`Message` (Ruby `Struct`, Python `@dataclass`) → plain struct + `Display`:**
  ```rust
  pub struct Message {
      pub role: String,
      pub content: String,
      pub tool_use_id: Option<String>,
  }

  impl Message {
      pub fn new(role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) -> Self { .. }
  }

  impl fmt::Display for Message {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          let id_tag = self.tool_use_id.as_deref().map(|id| format!(" [{id}]")).unwrap_or_default();
          let content: String = self.content.chars().take(61).collect();
          write!(f, "#<Message role={}{} content={}...>", self.role, id_tag, content)
      }
  }
  ```

- **`Context` (task stored as a runtime class reference) → `Context<T: Task>`
  with `PhantomData<T>`** (Decision 8):
  ```rust
  pub struct Context<T> {
      task: std::marker::PhantomData<T>,
      pub system: Option<String>,
      pub messages: Vec<Message>,
      pub tools: std::collections::HashMap<String, Tool>,
  }

  impl<T: Task> Context<T> {
      pub fn new(system: Option<String>) -> Self {
          Self { task: std::marker::PhantomData, system, messages: Vec::new(), tools: std::collections::HashMap::new() }
      }

      pub fn register_tool(&mut self, tool: Tool) {
          self.tools.insert(tool.name.clone(), tool);
      }

      pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>, tool_use_id: Option<String>) {
          self.messages.push(Message::new(role, content, tool_use_id));
      }

      pub fn tool_count(&self) -> usize { self.tools.len() }
      pub fn turn_count(&self) -> usize { self.messages.len() }
  }

  impl<T: Task> fmt::Display for Context<T> {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          write!(f, "#<Context task={} turns={} tools={}>", T::task_name(), self.turn_count(), self.tool_count())
      }
  }
  ```

- **`Config`, `Task` trait, `Player`** — carried forward from `00_config`
  unchanged except deleting `Config::PROMPTS_DIR` (Decision 6); everything
  else (dir resolution, `.env`/`settings.yaml` loading, `dig`, `tasks`,
  `mud_*`, `ConfigError`, the `Task` trait's default method bodies) is a
  straight copy.

## Behavior parity checklist (from the Ruby/Python spec)

- [ ] `Tool { name, description, parameters, block }`; `Display` truncates
      `description` to 41 chars and lists `parameters` keys
- [ ] `Message { role, content, tool_use_id }`; `Display` truncates
      `content` to 61 chars, prepends ` [id]` tag when `tool_use_id` is set
- [ ] `Context<T: Task> { system, messages, tools }`; `register_tool`,
      `add_message`, `tool_count()`, `turn_count()`; `Display` shows
      `T::task_name()`, `turn_count()`, `tool_count()`
- [ ] `Config`/`Task`/`Player` carried forward from `00_config` with
      `PROMPTS_DIR` removed; no `prompts/` dir shipped this step
- [ ] `examples/example.rs` builds a `Context::<Player>::new(...)`,
      registers one `move` tool, adds a user + assistant message, and
      prints the same fields in the same order as `examples/example.py`:
      `Config` Display, `Context` Display, the `move` `Tool` Display, then
      each `Message` Display on its own line

## Open questions

None outstanding — all decided above.
