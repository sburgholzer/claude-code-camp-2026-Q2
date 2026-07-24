# 07 · The Boukensha.run DSL (Rust port)

## Goal

Ports `python/07_the_run_dsl` (and its Ruby ground truth,
`ruby/07_the_run_dsl`) into `rust/07_the_run_dsl`. This step adds a
single top-level entry point, `boukensha::run::<T: Task>`, that wires
together `Context`, `Registry`, a backend, `PromptBuilder`, `Client`,
`Logger`, and `Agent` from just a `task` string and an optional
closure that registers tools — collapsing every previous step's manual
wiring (see `06_the_logger/examples/example.rs`) into one call.

`rust/07_the_run_dsl` currently equals `rust/06_the_logger` byte for
byte (confirmed via `diff -rq rust/06_the_logger rust/07_the_run_dsl`
— no output), still under the stale `boukensha_06_the_logger` package
name.

Diffing `python/06_the_logger` → `python/07_the_run_dsl` (ignoring
`__pycache__`) shows the only real changes are:

- `lib/boukensha/run_dsl.py` (new)
- `lib/boukensha/__init__.py` (adds the top-level `run()` function;
  imports/exports `RunDSL`, `LoopError`, all five backend classes)
- `lib/boukensha/config.py` (restores `mud_host`/`mud_port`/
  `mud_username`/`mud_password`, removed in `06_the_logger`)
- `lib/boukensha/errors.py` (restores `LoopError`, removed in
  `06_the_logger`)
- `lib/boukensha/logger.py` (adds `turn(n)`, unused by `agent.py`, and
  `subscribe(callback)`, with `_write_log` notifying subscribers)
- `examples/example.py`, `README.md`

Python is the direct reference; Ruby (`ruby/07_the_run_dsl`) is the
ultimate spec where the two disagree — read directly for this step
(`run_dsl.rb`, `boukensha.rb`'s `self.run`, `logger.rb`, `errors.rb`,
`config.rb`, `examples/example.rb`) to confirm exact argument order,
the `RunDSL#tool` proxy signature, and the `write_log`/`subscribers`
interaction. Ruby's own `README.md` title (`# Step 6 — The
Boukensha.run DSL`) is a stale copy-paste from the prior step — same
category of drift `02_the_registry.md`/`05_agent_loop.md`/
`06_the_logger.md` already documented; the directory, runtime banner,
and launcher all agree it's step 7, and this plan follows that.

## Source files to port

| File | Role |
|---|---|
| `python/07_the_run_dsl/lib/boukensha/run_dsl.py`, `ruby/07_the_run_dsl/lib/boukensha/run_dsl.rb` | `RunDSL`/`RunDsl`: tiny host object exposing only `tool(name, description:, parameters:, &block)`/`tool(name, description, parameters, block)`, proxying to `Registry#tool`/`Registry::tool` unchanged |
| `python/07_the_run_dsl/lib/boukensha/__init__.py`, `ruby/07_the_run_dsl/lib/boukensha.rb` | `boukensha.run()`/`Boukensha.run`: resolves task settings, system/model/backend/api_key defaults, builds `Context`→`Registry`, invokes the register callback/block, builds the matching backend/`PromptBuilder`/`Client`/`Logger` (with a `session_start` snapshot), builds `Agent`, seeds the user message, runs it, closes the logger unconditionally |
| `python/07_the_run_dsl/lib/boukensha/config.py`, `ruby/07_the_run_dsl/lib/boukensha/config.rb` | Restore `mud_host`/`mud_port`/`mud_username`/`mud_password` (removed in `06_the_logger`) |
| `python/07_the_run_dsl/lib/boukensha/errors.py`, `ruby/07_the_run_dsl/lib/boukensha/errors.rb` | Restore `LoopError` (removed in `06_the_logger`) |
| `python/07_the_run_dsl/lib/boukensha/logger.py`, `ruby/07_the_run_dsl/lib/boukensha/logger.rb` | Add `turn(n)` (unused by `agent.py`/`agent.rb` this step) and `subscribe(callback)`/`subscribe(&block)`; `_write_log`/`write_log` notifies every subscriber with the **pre-merge** event (no `session_id`/`at`) after writing |
| `python/07_the_run_dsl/examples/example.py`, `ruby/07_the_run_dsl/examples/example.rb` | Rewritten: registers `read_file`/`list_directory` via the new DSL, calls `boukensha.run(task=..., register=...)`/`Boukensha.run(task: ...) { ... }`, banner → "Step 7: The Boukensha.run DSL" |

## Runtime fixture to reuse

`.boukensha/` at the repo root, unchanged: `settings.yaml` configures
`tasks.player` for `anthropic`/`claude-haiku-4-5` with
`prompt_override.system: true`, plus the `mud:` block the restored
`mud_*` accessors read; `.env` holds a real `ANTHROPIC_API_KEY`.

Verified transcript reused from this same session's live Python run
(`.boukensha/sessions/20260724T044847Z-02f315a8.jsonl`, produced by
`bin/python/07_the_run_dsl` earlier in this session — not re-run here
to avoid a second round of billed calls against an unchanged fixture).
Structurally decoded:

```
session_start  task=player max_iterations=25 max_output_tokens=1024 model=claude-haiku-4-5 provider=anthropic
iteration n=1 max=25
response  task=player provider=anthropic model=claude-haiku-4-5 in=707 out=56 cost=0.000987 stop_reason=tool_use text='(tool use — 1 call)'
tool_call name=read_file args={'path': 'README.md'}
tool_result name=read_file ok=True
iteration n=2 max=25
response  task=player provider=anthropic model=claude-haiku-4-5 stop_reason=end_turn text='## Summary...'
turn_end  reason=completed iterations=2
```

The Rust run is compared structurally (same phases present, same
`session_start` snapshot shape, same overall structure), not
byte-for-byte — live model text and exact tool-call count vary run to
run, same caveat as every prior step's plan doc.

## Decisions (confirmed)

1. **`run()`'s block-replacement reuses the Python port's already-
   confirmed design (a `register` callback receiving the DSL object)
   instead of re-asking the user.** The underlying design question —
   callback-receiving-DSL vs. a context-manager shape vs. dropping the
   wrapper to decorate `Registry` directly — was already put to the
   user via `AskUserQuestion` for this exact Ruby source construct in
   `docs/plans/python_port/07_the_run_dsl.md` (Decision 2 there), which
   settled on the callback shape for fidelity to Ruby's own stated
   intent (`RunDSL` narrows the block's surface to just `tool`). Rust's
   `register: Option<impl FnOnce(&mut RunDsl<T>)>` closure parameter is
   the direct mechanical translation of Python's callback-function
   parameter — Rust doesn't even have a context-manager-equivalent
   deferred-execution mechanism to offer as a real alternative — so
   this counts as reusing settled precedent per this skill's own
   guidance ("Reuse a prior step's precedent if one exists... rather
   than re-deciding something already settled"), not a fresh Rust-side
   ambiguity requiring its own confirmation.
2. **`run<T: Task>` stays generic over the task type**, matching every
   other core type already generic over `T` in this port (`Context<T>`,
   `Registry<T>`, `Agent<'a, T>`, `PromptBuilder<T>`) — even though
   Ruby/Python's own `Boukensha.run`/`boukensha.run()` hardcodes
   `Tasks::Player`/`Player` internally rather than taking a task
   parameter at all. This continues `01_struct_skeleton`'s Decision 8
   (representing "the task class" as a compile-time type parameter
   throughout the Rust port, where Ruby/Python use a runtime class
   reference) rather than a new capability the spec doesn't have —
   `run()` would otherwise be the one place that established pattern
   was silently abandoned. Every call site still only ever instantiates
   `run::<Player>(...)`, so observable behavior is identical to
   Ruby/Python's hardcoded version either way.
3. **`RunError` is a plain enum** (`Config(ConfigError)` |
   `UnsupportedModel(UnsupportedModelError)` |
   `UnknownBackend(String)` | `Api(ApiError)`), matching this
   codebase's established composite-error idiom — `DispatchError` in
   `06_the_logger` already aggregates multiple error types the same
   way for `Registry::dispatch` — instead of boxing a
   `dyn std::error::Error`, which this project avoids throughout
   (no `anyhow` or similar).
4. **A missing `api_key` becomes `api_key.unwrap_or_default()` (an
   empty `String`) fed straight to the backend constructor**, not a
   new Rust-side validation error. Neither Ruby nor Python validate API
   key presence at this layer — a `nil`/`None` key just flows into the
   HTTP request and fails remotely. Every backend's
   `new(api_key: impl Into<String>, ...)` requires an owned `String`
   (not `Option<String>`), so `unwrap_or_default()` is the direct
   translation of "pass whatever we have, even nothing," reproducing
   the same eventual-remote-failure behavior instead of inventing a
   local check Ruby/Python don't have.
5. **`ollama_host` gets no separate default constant inside `run()`.**
   `Ollama::new(model, host: Option<String>)` (fixed since
   `03_prompt_builder`/`04_api_client`) already resolves `None` to its
   own `DEFAULT_HOST` internally — the same "one canonical place for a
   Ruby/Python kwarg default" pattern `client.rs`'s
   `DEFAULT_MAX_OUTPUT_TOKENS` const already established for exactly
   this reason. `run()` just forwards `ollama_host` straight through
   rather than duplicating the literal `"http://localhost:11434"`
   string a second time.
6. **`Logger::write_log` is changed to clone the event map before
   merging in `session_id`/`at`, instead of mutating the caller's map
   in place (as `06_the_logger`'s version does).** Required so
   `subscribe`'s callbacks receive the same pre-merge event Ruby/
   Python hand their subscribers (`@subscribers&.each { |s| s.call(event) }`
   runs against the un-merged `event`, confirmed by reading
   `logger.rb`/`logger.py` directly). This supersedes, without
   contradicting, `06_the_logger`'s mutate-in-place version — no
   subscriber concept existed at that step to need the original
   preserved, so the prior implementation was correct for its own
   step's requirements.
7. **`Logger::subscribe` stores `Vec<Box<dyn FnMut(&serde_json::Value)>>`,
   initialized eagerly (`Vec::new()`) in `Logger::new`, not lazily on
   first `subscribe()` call.** Ruby's `@subscribers ||= []` lazily
   allocates; Python's port of this same step already chose eager
   `__init__`-time initialization instead (see
   `docs/plans/python_port/07_the_run_dsl.md` Decision 4) since an
   empty collection iterates identically to a `nil`-guarded skip. Rust
   follows the same reused simplification. `FnMut` (not `Fn`) is the
   most permissive signature that still covers every plausible
   subscriber shape (state-accumulating included), since nothing in
   this step actually subscribes to validate a narrower bound against.
8. **Restored `Config::mud_*`/`errors::LoopError` are ported even
   though nothing in this step's Ruby/Python/Rust code calls them
   yet.** Genuine re-additions in the spec (removed at `06_the_logger`,
   restored here) — copied verbatim from `rust/00_config`'s still-live
   implementation (byte-identical method bodies) rather than
   re-derived, same source Python's own restore copied from
   (`python/00_config`). Matches the "port the spec's re-add exactly,
   don't second-guess it" precedent `06_the_logger`'s README documents
   for the removal direction.
9. **No new dependency.** `run()` and `run_dsl.rs` use only
   already-imported project types plus `std::env`/`std::path`. Ruby's
   `Gemfile`/`Gemfile.lock` are unchanged this step (`diff -rq`
   confirms no gem was added), so there's no Ruby-side dependency to
   mirror, consistent with this project's stdlib-first default
   (`ITERATIONS.md`).
10. **`bin/rust/07_the_run_dsl` launcher and the root `Cargo.toml`
    workspace-membership edit** — added per the repo's
    `bin/<language>/<step>` convention and the Rust port's own
    per-step requirement, matching `bin/rust/06_the_logger`'s shape.

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/07_the_run_dsl")
week1_baseline/rust/07_the_run_dsl/
  Cargo.toml                                    (edit: package name → boukensha_07_the_run_dsl; no new dependency)
  src/
    lib.rs                                      (edit: + run::<T: Task>(...) -> Result<String, RunError>; + backend imports (Anthropic/Gemini/Ollama/OllamaCloud/OpenAI/PromptBackend); + pub mod run_dsl; export RunDsl/LoopError/RunError)
    run_dsl.rs                                   (new: RunDsl<'a, T: Task>)
    config.rs                                    (edit: + mud_host/mud_port/mud_username/mud_password, copied from rust/00_config)
    errors.rs                                    (edit: + LoopError(pub String), copied from rust/05_agent_loop; + RunError enum)
    logger.rs                                    (edit: + turn(n: u32); + subscribers: Vec<Box<dyn FnMut(&serde_json::Value)>> field + subscribe(callback); write_log clones the event map before merging session_id/at, then notifies subscribers with the original)
    agent.rs, client.rs, context.rs, message.rs,
    prompt_builder.rs, registry.rs, tool.rs,
    tasks/{mod,base,player}.rs, backends/*.rs     (unchanged — verified byte-identical to 06_the_logger)
  prompts/system.md                               (unchanged)
  README.md                                       (rewrite: this step's own docs)
  examples/example.rs                             (rewrite: call run::<Player>(...) with a register closure registering read_file/list_directory instead of manual Context/Registry/Backend/PromptBuilder/Client/Logger/Agent wiring; banner → "Step 7: The Boukensha.run DSL"; drops the Provider/Model/Max-iterations/Max-output-tokens preamble lines run() now resolves internally)
week1_baseline/bin/rust/07_the_run_dsl            (new launcher, matching bin/rust/06_the_logger's shape)
```

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`RunDSL.new(registry).instance_eval(&block) if block` →
  `register: Option<impl FnOnce(&mut RunDsl<T>)>`, called as
  `register(&mut RunDsl::new(&mut registry))`.** Ruby's block runs
  with `self` rebound to the `RunDSL` receiver via `instance_eval`, so
  bare `tool "x", ...` calls need no explicit receiver. Rust has
  neither `instance_eval` nor decorator syntax, and every earlier step
  already registers tools via a direct, explicit-receiver method call
  (`registry.tool(name, description, parameters, block)`) — so a
  closure taking `&mut RunDsl<T>` explicitly is the closest and only
  real idiomatic shape, matching the same design already chosen for
  Python (see Decision 1).
- **`RunDsl<'a, T: Task>` wraps `&'a mut Registry<T>` and exposes only
  `tool(...)`, forwarding to `Registry::tool` unchanged.** Same
  file-for-file structural precedent as Ruby's `run_dsl.rb`/Python's
  `run_dsl.py`: exists purely to narrow the register callback's
  surface to one method, per Ruby's own comment ("keeping the DSL
  surface intentionally small").
- **Backend dispatch (`case backend when :anthropic ...` /
  `if backend == "anthropic"`) → `match backend_name.as_str()`.**
  `Task::provider` already returns a plain `String` (fixed since
  `01_struct_skeleton`); every prior `example.rs` already matches on
  `provider.as_str()`. `run()`'s dispatch continues that established
  simplification rather than introducing symbol-like typing Rust
  doesn't have and prior steps never needed.
- **`ensure logger&.close` → plain sequential code, no `finally`/
  `Drop`/scope-guard needed.** Every fallible step in `run()`
  (`ConfigError` from `T::provider`/`T::model`, `UnsupportedModelError`
  from a backend constructor, an unrecognized backend string) happens
  strictly *before* `Logger::new` runs and uses `?` to return early;
  the only failure possible *after* the logger exists is `agent.run()`'s
  `ApiError`, and Rust's linear control flow already reaches
  `logger.close()` unconditionally on both the `Ok` and `Err` paths of
  that one call (`let result = agent.run().map_err(RunError::Api);
  logger.close(); result` — three plain statements, no early return
  in between). No language-level `ensure`/`finally` equivalent is
  needed to reproduce Ruby's/Python's cleanup-on-every-path guarantee
  here.
- **`Logger#subscribe`'s `@subscribers ||= []` lazy-init →
  `subscribers: Vec::new()` in `Logger::new`.** Same simplification
  Python's own port of this step already chose (eager `__init__`-time
  init over a lazy-allocate-on-first-use guard) — an empty `Vec`
  iterates as a no-op identically to Ruby's `nil&.each` short-circuit,
  so there's no lazy-init branch worth reproducing.
- **`write_log`'s event mutation → clone-then-merge.** `06_the_logger`'s
  `write_log` inserted `session_id`/`at` directly into the caller's
  `serde_json::Map`, since nothing needed the pre-merge value. This
  step's `subscribe` requires subscribers to see the original,
  unmerged event (confirmed against both `logger.rb`'s
  `@subscribers&.each { |s| s.call(event) }` and `logger.py`'s
  `subscriber(event)` — both reference the pre-merge local, not the
  written line) — so `write_log` now clones the object map for the
  line it writes to disk, leaving `event` itself untouched to pass to
  each subscriber afterward.

## Behavior parity checklist

- [x] `Config::mud_host`/`mud_port`/`mud_username`/`mud_password`
      restored, reading `settings.yaml`'s `mud:` block via `dig`, with
      `"localhost"`/`4000` fallbacks for host/port and `None` for
      username/password — verified identical to `rust/00_config`
- [x] `errors::LoopError(pub String)` restored — verified identical to
      `rust/05_agent_loop`
- [x] `Logger::turn(n: u32)` writes `{"phase": "turn", "n": n}`
      (unused by `agent.rs` this step, matching Ruby/Python)
- [x] `Logger::subscribe(callback: impl FnMut(&serde_json::Value) + 'static)`
      registers a callback; `write_log` invokes every registered
      subscriber with the **pre-merge** event (no `session_id`/`at`)
      after the line is written and flushed
- [x] `RunDsl::tool(name, description, parameters, block)` forwards
      unchanged to `Registry::tool`
- [x] `run::<T: Task>(task, system, model, backend, api_key,
      ollama_host, log, max_output_tokens, register) -> Result<String, RunError>`:
  - [x] calls `config()` first (loads `.env`, populates `std::env`)
  - [x] resolves `task_settings = cfg.tasks(Some(T::task_name())).unwrap_or_default()`
  - [x] `system` defaults to `T::system_prompt(&task_settings,
        Some(&cfg.user_prompts_dir()), Some(Path::new(Config::PROMPTS_DIR)))`
        when not given
  - [x] `model` defaults to `T::model(&task_settings)?`, `backend`
        defaults to `T::provider(&task_settings)?`, each mapping
        `ConfigError` to `RunError::Config`
  - [x] `api_key` defaults to the matching `ANTHROPIC_API_KEY` /
        `OPENAI_API_KEY` / `GEMINI_API_KEY` / `OLLAMA_API_KEY` env var
        for the resolved backend (no default for `"ollama"`), only
        when `api_key` was not passed explicitly
  - [x] builds `Context::<T>::new(system)` then `Registry::new(ctx)`
        **before** invoking `register`
  - [x] calls `register(&mut RunDsl::new(&mut registry))` when
        `register` is `Some`, before the backend is constructed
  - [x] dispatches to `Anthropic`/`OpenAI`/`Gemini`/`Ollama`/
        `OllamaCloud` by the resolved backend string, returning
        `RunError::UnknownBackend` on an unrecognized value
  - [x] builds `PromptBuilder::new(backend)`, `Client::new(&builder)`;
        resolves `effective_max_iterations = T::max_iterations(&task_settings)`
        and `effective_max_output_tokens =
        max_output_tokens.unwrap_or_else(|| T::max_output_tokens(&task_settings))`
  - [x] builds `Logger::new(None, None, log, Some(snapshot))` with
        `snapshot = {task, max_iterations, max_output_tokens, model, provider}`
  - [x] adds the `task` string as a `"user"` message on
        `registry.context` before constructing `Agent` (required
        ordering: `Agent::new` takes an exclusive borrow of `registry`)
  - [x] builds `Agent::new(&mut registry, &builder, &client, &mut
        logger, Some(&task_settings), Some(effective_max_iterations),
        Some(effective_max_output_tokens))`, runs it, maps `ApiError`
        to `RunError::Api`
  - [x] calls `logger.close()` unconditionally before returning the
        result, on both `Ok` and `Err` paths
- [x] `examples/example.rs` prints the same banner/Config line as
      Ruby/Python, registers `read_file`/`list_directory` via the
      `register` closure, and prints the same
      `=== FINAL RESPONSE ===` block
- [x] Root `week1_baseline/Cargo.toml`'s `members` includes
      `"rust/07_the_run_dsl"`
- [x] `rust/07_the_run_dsl/Cargo.toml`'s package name is
      `boukensha_07_the_run_dsl`
- [x] `bin/rust/07_the_run_dsl` exists, is executable, and runs
      `cargo run --quiet --example example` from the crate dir
- [x] `cargo build --workspace` compiles clean (only the pre-existing,
      unrelated cross-crate `example` binary-name collision warnings
      that already exist for every step)

Verified for real: `cargo build --workspace` succeeded, and
`bin/rust/07_the_run_dsl` ran end-to-end against the live
`.boukensha/` fixture (confirmed with the user first — real, billed
Anthropic API calls, same precedent as every prior real-run step). It
printed the banner, `Config: ...` line, two blank lines, then
`=== FINAL RESPONSE ===` with the model's summary — no
Provider/Model/Max-iterations/Max-output-tokens preamble (those moved
inside `run()`), matching the shape `06_the_logger`'s port established
for banner-vs-log-output separation. The produced
`.boukensha/sessions/20260724T050436Z-48ae6058.jsonl` had the correct
`session_start` snapshot (`task=player max_iterations=25
max_output_tokens=1024 model=claude-haiku-4-5 provider=anthropic`),
one `read_file` tool call at iteration 1, and `end_turn` at iteration
2 with a `turn_end` — same overall shape as the reused Python
transcript (live model behavior, not byte-for-byte, expected). All
behavior parity checklist items above are checked off against this
real run plus a direct read of the implemented `run()`/`RunDsl`/
`Config`/`errors.rs`/`logger.rs` source against the Ruby/Python source
line by line.

## Open questions

None outstanding — all decided above.
