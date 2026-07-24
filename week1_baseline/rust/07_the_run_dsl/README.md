# 07 · The Boukensha.run DSL (Rust port)

Behavior port of `ruby/07_the_run_dsl` / `python/07_the_run_dsl` — a
single top-level entry point, `run::<T>`, that wires together
`Context`, `Registry`, a backend, `PromptBuilder`, `Client`, `Logger`,
and `Agent` from just a `task` string and an optional closure that
registers tools, collapsing every previous step's manual wiring (see
`examples/example.rs` in `06_the_logger`) into one call.

`message.rs`, `context.rs`, `client.rs`, `agent.rs`,
`tasks/{mod,base,player}.rs`, `backends/mod.rs`,
`backends/{anthropic,openai,gemini,ollama,ollama_cloud,base}.rs`,
`prompt_builder.rs`, `registry.rs`, and `tool.rs` are unchanged from
`06_the_logger`; see `../06_the_logger/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/run_dsl.rs` | `RunDsl<'a, T: Task>` — wraps `&'a mut Registry<T>`; exposes only `tool(...)`, keeping the DSL surface intentionally small |

## Updated Files

| File | Change |
|---|---|
| `src/lib.rs` | Adds `pub fn run<T: Task>(...) -> Result<String, RunError>`; imports the five backend types + `PromptBackend`; exports `RunDsl`, `LoopError`, `RunError` |
| `src/config.rs` | Restores `mud_host`/`mud_port`/`mud_username`/`mud_password` (removed in `06_the_logger`, reinstated here — matches the Ruby/Python spec's own re-add at this step) |
| `src/errors.rs` | Restores `LoopError` (same as above; not yet raised anywhere); adds `RunError`, aggregating every fallible step inside `run()` |
| `src/logger.rs` | Adds `turn(n)` (not yet called by `agent.rs`) and `subscribe(callback)`; `write_log` now clones the event before merging `session_id`/`at` so subscribers see the pre-merge event, then notifies every subscriber after writing |

## run::\<T: Task\>

```rust
let result = run::<Player>(
    "Summarise lib/boukensha",
    None, None, None, None, None, None, None,
    Some(|dsl: &mut RunDsl<Player>| {
        dsl.tool("read_file", "Read a file from disk", params, |args| {
            std::fs::read_to_string(&args["path"]).map_err(|e| e.to_string())
        });
    }),
);
```

Rust has no named/keyword arguments, so the parameters are positional,
in the same order as Ruby's/Python's kwargs:

| Position | Default | Description |
|---|---|---|
| `task` | *(required)* | The user message handed to the agent |
| `system` | task's configured system prompt | System prompt override |
| `model` | task's configured model | Model name override |
| `backend` | task's configured provider | `"anthropic"`, `"openai"`, `"gemini"`, `"ollama"`, or `"ollama_cloud"` |
| `api_key` | matching `*_API_KEY` env var | API key for the chosen backend; not needed for `"ollama"` |
| `ollama_host` | `Ollama::new`'s own default (`http://localhost:11434`) | Ollama base URL override |
| `log` | `None` | Optional JSONL path override; by default logs go to `.boukensha/sessions/<session-id>.jsonl` |
| `max_output_tokens` | task's configured value | Per-reply output cap |
| `register` | `None` | Optional closure receiving a `RunDsl<T>` to register tools on |

`T: Task` is a compile-time type parameter (fixed since
`01_struct_skeleton`), so a call site picks the task via turbofish
(`run::<Player>(...)`) rather than passing a task object at runtime.
Config, system prompt, model, and backend all come from
`~/.boukensha` (or `BOUKENSHA_DIR`) via `T`'s settings.

## Before and after

**Step 6 — manual plumbing** (see `06_the_logger/examples/example.rs`):

```rust
let ctx: Context<Player> = Context::new(system_prompt);
let mut registry = Registry::new(ctx);
registry.tool("read_file", "Read a file", params, |args| { /* ... */ });
registry.context.add_message("user", "Read lib/boukensha.rs", None);

let backend: Box<dyn PromptBackend<Player>> = Box::new(Anthropic::new(api_key, &model)?);
let builder = PromptBuilder::new(backend);
let client = Client::new(&builder);
let mut logger = Logger::new(None, None, None, None);
let mut agent = Agent::new(&mut registry, &builder, &client, &mut logger, Some(&settings), None, None);
let result = agent.run()?;
```

**Step 7 — just describe what you want:**

```rust
let result = run::<Player>(
    "Read lib/boukensha.rs",
    None, None, None, None, None, None, None,
    Some(|dsl: &mut RunDsl<Player>| {
        dsl.tool("read_file", "Read a file", params, |args| { /* ... */ });
    }),
)?;
```

## No New Dependencies

`run_dsl.rs` and `run()` use only types already in this crate plus
`std::env`/`std::path`. No new crate — `Cargo.toml` is unchanged from
`06_the_logger`, matching Ruby's own `Gemfile`/`Gemfile.lock`, which
are unchanged in this step too.

## Porting notes

- **`RunDSL.new(registry).instance_eval(&block) if block` → a
  `register: Option<impl FnOnce(&mut RunDsl<T>)>` closure parameter.**
  Ruby's block is `instance_eval`'d so bare `tool "x", ...` calls
  resolve against the `RunDSL` receiver with no explicit `self`. Rust
  has no `instance_eval` equivalent and no decorator syntax either —
  every prior step already registers tools via a direct method call
  (`registry.tool(name, description, parameters, block)`), so the
  natural translation of "a block that registers tools against a
  narrow-surface object" is a closure taking `&mut RunDsl<T>`
  explicitly. This reuses the exact design already confirmed with the
  user for the sibling Python port
  (`docs/plans/python_port/07_the_run_dsl.md` Decision 2: callback
  receiving the DSL object, over a context-manager shape or dropping
  the wrapper to decorate `Registry` directly) — Rust's closure
  parameter is the direct mechanical equivalent of Python's callback
  function parameter, so this was not re-asked; see Decision 1 below.
- **`RunDsl<'a, T: Task>` wraps `&'a mut Registry<T>`, exposing only
  `tool(...)`, which forwards to `Registry::tool` unchanged.** Same
  file-for-file precedent as Ruby's `run_dsl.rb`/Python's
  `run_dsl.py`: the wrapper exists purely to narrow the callback's
  surface to one method (per Ruby's own comment), not to add behavior.
- **`run<T: Task>` is generic over the task type, matching every other
  core type in this port (`Context<T>`, `Registry<T>`, `Agent<'a, T>`,
  `PromptBuilder<T>`) — even though Ruby/Python's own `Boukensha.run`
  hardcodes `Tasks::Player`/`Player` internally.** This isn't a new
  expansion of behavior: `01_struct_skeleton`'s plan already decided
  Rust represents "the task class" as a compile-time type parameter
  everywhere else in the port (Decision 8 there), and `run()` is the
  first place that decision would otherwise be silently abandoned by
  hardcoding `Player`. The example call site still only ever
  instantiates `run::<Player>(...)`, so observable behavior is
  identical either way.
- **Backend dispatch matches on a plain `&str`, not a Ruby symbol.**
  `Task::provider` already returns a bare `String` (fixed since
  `01_struct_skeleton`), and every earlier step's `example.rs` already
  matches on `provider.as_str()` — `run()`'s backend dispatch continues
  that established simplification instead of introducing a
  symbol/string distinction neither Rust nor prior steps need.
- **A missing `api_key` becomes `api_key.unwrap_or_default()` (an
  empty `String`), not a Rust-side validation error.** Ruby/Python
  never validate API key presence at this layer either — a `nil`/
  `None` key just flows into the request and fails remotely (a 401).
  Every backend's `new(api_key: impl Into<String>, ...)` already
  requires an owned `String`, not `Option<String>`, so
  `unwrap_or_default()` is the direct translation of "pass whatever
  we have, even if it's nothing" rather than inventing a new failure
  mode Ruby/Python don't have.
- **`ollama_host` has no separate default constant in `run()`.**
  `Ollama::new(model, host: Option<String>)` already resolves
  `None` to `DEFAULT_HOST` internally (see `backends/ollama.rs`), the
  same pattern `client.rs`'s `DEFAULT_MAX_OUTPUT_TOKENS` const uses to
  avoid duplicating a Ruby/Python kwarg default — `run()` just forwards
  `ollama_host` straight through.
- **`ensure logger&.close` → sequential code, no `finally`/`Drop`
  trick needed.** Every fallible step in `run()` (`ConfigError`,
  `UnsupportedModelError`, an unrecognized backend string) happens
  *before* `Logger::new` runs; the only failure possible afterward is
  `agent.run()`'s `ApiError`, and Rust's control flow already reaches
  the `logger.close()` line unconditionally on both `Ok` and `Err`
  paths (`agent.run().map_err(RunError::Api)` is the second-to-last
  statement, `logger.close()` the last one before returning `result`)
  — no scope guard or `Drop` impl needed to reproduce Ruby's `ensure`/
  Python's `finally` here. (`Logger::close` is already a near-no-op in
  this codebase per `06_the_logger`'s Porting Notes — every write
  already flushes immediately — but it's still called explicitly for
  API-surface parity, same rationale as there.)
- **`Logger::write_log` now clones the event before merging in
  `session_id`/`at`, instead of mutating the caller's map in place.**
  `06_the_logger`'s `write_log` inserted `session_id`/`at` directly
  into the same `serde_json::Map` it was handed, since nothing else
  needed the pre-merge event. Ruby's/Python's `subscribe` feature
  requires subscribers to receive the *original*, unmerged event
  (`@subscribers&.each { |s| s.call(event) }` runs against the same
  `event` the phase method built, before `.merge(session_id:, at:)`
  was applied for the write) — so Rust's `write_log` now clones the
  object map for the line it writes, leaving the original `event`
  value intact to hand to each subscriber afterward. A required
  behavior change driven by this step's new feature, not a bug fix of
  the prior step's code (which had no subscriber concept to preserve
  the original for).
- **`Logger::subscribe` stores `Vec<Box<dyn FnMut(&serde_json::Value)>>`.**
  Ruby's `@subscribers ||= []` lazily allocates on first use and
  Python's port (this same step) initializes the list eagerly in
  `__init__` instead (see `docs/plans/python_port/07_the_run_dsl.md`
  Decision 4) — Rust follows the same simplification: the field is
  initialized to `Vec::new()` in `Logger::new`, no lazy-init branch.
  `FnMut` (not `Fn`) so a subscriber can accumulate state (e.g. a
  counter or buffer) across calls, the most permissive signature that
  still satisfies every current (non-)caller, since nothing in this
  step actually subscribes.
- **Restored `Config::mud_*`/`errors::LoopError` are ported even
  though nothing in this step's Ruby/Python/Rust code calls them
  yet.** Genuine re-additions in the spec (removed at `06_the_logger`,
  restored here), copied verbatim from `rust/00_config`'s still-live
  implementation rather than re-derived — same "match the spec, don't
  second-guess it" precedent `06_the_logger`'s own README documents
  for the removal direction, and the same source Python's port copied
  its restored version from
  (`docs/plans/python_port/07_the_run_dsl.md` Decision 3).

## Decisions

1. **`run()`'s block-replacement reuses the Python port's confirmed
   design (a `register` callback receiving the DSL object) rather than
   re-asking the user.** The underlying design question — callback vs.
   context-manager vs. dropping the wrapper — was already settled with
   the user in `docs/plans/python_port/07_the_run_dsl.md` for the
   identical Ruby source construct. Rust's closure parameter is the
   direct mechanical translation of Python's callback-function
   parameter (Rust has no context-manager-equivalent deferred-execution
   mechanism to even offer as a real alternative), so this is a reuse
   of prior precedent per this skill's own guidance, not a fresh
   ambiguity.
2. **`run<T: Task>` stays generic over the task type**, continuing
   `01_struct_skeleton`'s Decision 8 (task-as-compile-time-parameter)
   rather than hardcoding `Player` the way Ruby/Python's `run` does
   internally. See Porting Notes for why this isn't a behavior
   expansion.
3. **`RunError` is a plain enum aggregating `ConfigError` /
   `UnsupportedModelError` / an unrecognized-backend `String` /
   `ApiError`**, matching this codebase's established composite-error
   idiom (`DispatchError` in `06_the_logger`) instead of boxing a
   `dyn std::error::Error` — this project avoids `anyhow`-style dynamic
   error types throughout.
4. **`Logger::write_log` clones the event map instead of mutating it
   in place**, required so `subscribe`'s callbacks see the same
   pre-merge event Ruby/Python hand their subscribers. See Porting
   Notes for why this supersedes `06_the_logger`'s mutate-in-place
   version without contradicting it (no subscriber concept existed
   yet at that step).
5. **No new dependency.** `run()` and `run_dsl.rs` use only already-
   imported project types plus `std::env`/`std::path`. Ruby's
   `Gemfile`/`Gemfile.lock` are unchanged this step (`diff -rq`
   confirms no gem was added), so there's no Ruby-side dependency to
   mirror either.
6. **`bin/rust/07_the_run_dsl` launcher and the root `Cargo.toml`
   workspace-membership edit** — added per the repo's
   `bin/<language>/<step>` convention and the Rust port's own
   per-step requirement, matching `bin/rust/06_the_logger`'s shape.

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/07_the_run_dsl")
week1_baseline/rust/07_the_run_dsl/
  Cargo.toml                                    (edit: package name → boukensha_07_the_run_dsl; no new dependency)
  src/
    lib.rs                                      (edit: + run::<T: Task>; + backend imports; export RunDsl/LoopError/RunError)
    run_dsl.rs                                   (new)
    config.rs                                    (edit: + mud_host/mud_port/mud_username/mud_password)
    errors.rs                                    (edit: + LoopError, + RunError)
    logger.rs                                    (edit: + turn(n), + subscribe(callback); write_log clones before merging)
    agent.rs, client.rs, context.rs, message.rs,
    prompt_builder.rs, registry.rs, tool.rs,
    tasks/{mod,base,player}.rs, backends/*.rs     (unchanged)
  prompts/system.md                               (unchanged)
  README.md                                       (rewrite: this step's own docs)
  examples/example.rs                             (rewrite: call run::<Player>(...) with a register closure instead of manual wiring; banner → "Step 7: The Boukensha.run DSL"; drops the Provider/Model/Max-iterations/Max-output-tokens preamble lines run() now resolves internally)
week1_baseline/bin/rust/07_the_run_dsl            (new launcher)
```

## Behavior parity checklist

- [x] `Config::mud_host`/`mud_port`/`mud_username`/`mud_password`
      restored, reading `settings.yaml`'s `mud:` block via `dig`, with
      `"localhost"`/`4000` fallbacks for host/port and `None` for
      username/password
- [x] `errors::LoopError(pub String)` exists again (unused, matches
      Ruby/Python's `LoopError` being unraised anywhere)
- [x] `Logger::turn(n)` writes a `{"phase": "turn", "n": n}` event
      (unused by `agent.rs` this step, matching Ruby/Python)
- [x] `Logger::subscribe(callback)` registers a callback; `write_log`
      invokes every registered subscriber with the pre-merge event
      after the line is written and flushed
- [x] `RunDsl<'a, T: Task>::tool(...)` forwards unchanged to
      `Registry::tool`
- [x] `run::<T: Task>(task, system, model, backend, api_key,
      ollama_host, log, max_output_tokens, register) -> Result<String, RunError>`:
  - [x] calls `config()` first (loads `.env`, populates `std::env`)
  - [x] resolves `task_settings = cfg.tasks(Some(T::task_name()))`
  - [x] `system` defaults to `T::system_prompt(&task_settings,
        Some(&cfg.user_prompts_dir()), Some(Path::new(Config::PROMPTS_DIR)))`
        when not given
  - [x] `model` defaults to `T::model(&task_settings)`, `backend`
        defaults to `T::provider(&task_settings)`, both propagating
        `ConfigError` via `RunError::Config` when required and absent
  - [x] `api_key` defaults to the matching `ANTHROPIC_API_KEY` /
        `OPENAI_API_KEY` / `GEMINI_API_KEY` / `OLLAMA_API_KEY` env var
        for the resolved backend (no default for `"ollama"`), only
        when `api_key` was not passed explicitly
  - [x] builds `Context::<T>::new(system)` then `Registry::new(ctx)`
        **before** invoking `register`
  - [x] calls `register(&mut RunDsl::new(&mut registry))` if
        `register` is `Some`, before the backend is constructed
  - [x] dispatches to `Anthropic`/`OpenAI`/`Gemini`/`Ollama`/
        `OllamaCloud` by the resolved backend string, returning
        `RunError::UnknownBackend` on an unrecognized value
  - [x] builds `PromptBuilder::new(backend)`, `Client::new(&builder)`;
        resolves `effective_max_iterations = T::max_iterations(&task_settings)`
        and `effective_max_output_tokens = max_output_tokens.unwrap_or_else(|| T::max_output_tokens(&task_settings))`
  - [x] builds `Logger::new(None, None, log, Some(snapshot))` with
        `snapshot = {task, max_iterations, max_output_tokens, model, provider}`
  - [x] adds the `task` string as a `"user"` message on
        `registry.context` before constructing `Agent`
  - [x] builds `Agent::new(&mut registry, &builder, &client, &mut
        logger, Some(&task_settings), Some(effective_max_iterations),
        Some(effective_max_output_tokens))`, runs it, maps its
        `ApiError` to `RunError::Api`
  - [x] calls `logger.close()` unconditionally before returning the
        result, on both `Ok` and `Err` paths
- [x] `examples/example.rs` prints the same banner/Config line as
      Ruby/Python, registers `read_file`/`list_directory` via the
      `register` closure, and prints the same
      `=== FINAL RESPONSE ===` block — matching the real run's shape
      (live model text differs, expected)
- [x] Root `week1_baseline/Cargo.toml` lists `"rust/07_the_run_dsl"`
      in workspace `members`
- [x] `rust/07_the_run_dsl/Cargo.toml`'s package name is
      `boukensha_07_the_run_dsl`, not the stale copied-over
      `boukensha_06_the_logger`
- [x] `bin/rust/07_the_run_dsl` launcher exists, is executable, and
      runs `cargo run --quiet --example example` from the crate dir

## Run Example

```bash
./week1_baseline/bin/rust/07_the_run_dsl
```

This makes one or more real HTTP requests to whichever provider
`.boukensha/settings.yaml` configures (Anthropic, by default in this
repo's fixture) — it costs a small amount per model round-trip and
requires a valid API key in `.boukensha/.env`. It also writes a new
`.boukensha/sessions/<session-id>.jsonl` file.

Example output (the exact tool calls and final text are **not**
reproducible byte-for-byte — they're live model responses):

```
=== BOUKENSHA Step 7: The Boukensha.run DSL ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>


=== FINAL RESPONSE ===
## Summary

The **Boukensha MUD Player Assistant Framework** ... is a Rust-based AI agent system that can do the following:
...
```

Verified for real: `cargo build --workspace` compiles cleanly, and
`bin/rust/07_the_run_dsl` ran end-to-end against the live fixture,
producing a `.jsonl` session with `session_start` (correct
`task`/`max_iterations`/`max_output_tokens`/`model`/`provider`
snapshot fields), `iteration`, `prompt`, `response`, `tool_call`,
`tool_result`, and `turn_end` phases — same overall shape as the
verified Python transcript for this same step
(`docs/plans/python_port/07_the_run_dsl.md`), structurally compared
(field names/ordering/nesting), not byte-for-byte (live model text and
exact tool-call count vary run to run).

## Open questions

None outstanding — all decided above.
