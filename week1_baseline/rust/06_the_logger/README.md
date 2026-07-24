# 06 · The Logger (Rust port)

Behavior port of `ruby/06_the_logger` / `python/06_the_logger` —
`boukensha_06_the_logger::Logger` records each agent run as structured
JSON Lines. It is a file logger, not user-facing display output: as of
this step, `Agent::run` prints **nothing** to stdout about iterations
or tool calls — every `println!` that did that in `05_agent_loop`
moves into the log file instead.

`message.rs`, `context.rs`, `client.rs`, `tasks/{mod,base,player}.rs`,
and `backends/mod.rs` are unchanged from `05_agent_loop`; see
`../05_agent_loop/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/logger.rs` | `Logger` — one method per phase, each appending a JSON line to a per-session file |

## Updated Files

| File | Change |
|---|---|
| `src/lib.rs` | Adds module-level `config()`, `quiet()`/`loud()`/`is_quiet()`, `debug()`/`is_debug()` (backed by `OnceLock<Config>`/`AtomicBool` statics), and exports `Logger`; no longer exports `LoopError` |
| `src/agent.rs` | `Agent::new` gains a required `logger: &mut Logger` param; `run`/`wrap_up`/`handle_tool_calls` log every phase; tool dispatch errors are caught and logged instead of `.expect()`-ing |
| `src/errors.rs` | Removes unused `LoopError`; adds `DispatchError`/`ToolError` |
| `src/tool.rs`, `src/registry.rs` | `Tool`'s closure and `Registry::dispatch` return `Result` instead of a bare `String`, so a failing tool call can be caught and logged (see Porting Notes) |
| `src/config.rs` | Removes unused `mud_host`/`mud_port`/`mud_username`/`mud_password` |
| `src/backends/base.rs` | `Backend` gains required `name()`/`model()` methods; `PromptBackend<T>: Backend` supertrait |
| `src/backends/{anthropic,openai,gemini,ollama,ollama_cloud}.rs` | Each implements the two new `Backend` methods |
| `src/prompt_builder.rs` | Adds `PromptBuilder::backend(&self) -> &dyn Backend`, a type-erased accessor for `Logger` |

## Session Logs

Each `Logger` instance creates a session id and writes one log file
for that session:

```text
.boukensha/sessions/<session-id>.jsonl
```

Every line is a complete JSON object with `session_id`, `at`, and
`phase` fields, plus phase-specific data — grep/tail friendly and
machine readable.

```json
{"phase":"session_start","session_id":"20260724T035819Z-d27527bf","at":"2026-07-23T22:58:19Z"}
{"phase":"iteration","n":1,"max":25,"session_id":"20260724T035819Z-d27527bf","at":"2026-07-23T22:58:19Z"}
```

`response` lines include the active task, provider, model, normalized
token counts, and estimated USD cost when the backend has token
pricing data:

```json
{"phase":"response","task":"player","provider":"anthropic","model":"claude-haiku-4-5","input_tokens":707,"output_tokens":56,"cost_usd":0.000987}
```

`at` is UTC (`...Z`) rather than Ruby's/Python's local-offset
timestamp — see Porting Notes.

## boukensha_06_the_logger::Logger

| Method | Phase | Logs |
|---|---|---|
| `iteration(n, max)` | `iteration` | loop counter and ceiling |
| `limit_reached(kind, n, max)` | `limit_reached` | iteration ceiling triggered |
| `turn_end(reason, iterations, tokens)` | `turn_end` | why/when the turn ended |
| `prompt(messages, tools)` | `prompt` | message count/roles, tool count/names |
| `tool_call(name, args)` | `tool_call` | tool name and arguments |
| `tool_result(name, result, ok, error)` | `tool_result` | stringified tool result, success flag |
| `response(text, usage, stop_reason, task, backend)` | `response` | response text, token usage, task/provider/model, estimated cost |
| `raw(data)` | `raw` | raw provider response, only when `boukensha_06_the_logger::is_debug()` is true |

## Task Configuration

Same task-based settings shape as earlier steps:

```yaml
tasks:
  player:
    provider: anthropic
    model: claude-haiku-4-5
    prompt_override:
      system: true
```

When `prompt_override.system` is true, the player task reads
`.boukensha/prompts/player/system.md`. Otherwise it falls back to this
step's shipped `prompts/system.md`.

Default usage:

```rust
let mut logger = Logger::new(None, None, None, None);
let mut agent = Agent::new(&mut registry, &builder, &client, &mut logger, Some(&player_settings), None, None);
```

You can also provide a session id or override the destination
directory (`Logger::new(session_id, dir, log, snapshot)`):

```rust
Logger::new(Some("manual-session".to_string()), None, None, None)
Logger::new(None, Some(PathBuf::from("/tmp/boukensha-sessions")), None, None)
```

`log` still accepts an explicit file path for compatibility, but
normal iteration usage should write under `.boukensha/sessions`.

## Debug Events

Call `boukensha_06_the_logger::debug()` before running the agent to
include raw provider responses:

```rust
boukensha_06_the_logger::debug();
```

## New Dependency: `time`

Rust's `std` has no calendar/timezone support at all — `SystemTime`
gives a duration since the Unix epoch with no year/month/day/hour/
offset breakdown, and every log line's `at` field plus the session
id's timestamp component need one. Only the `formatting` feature is
enabled; `local-offset` (needed for a true local-time equivalent) is
deliberately not — see Porting Notes.

No dependency was added for JSON structure or randomness; see Porting
Notes.

## Considerations

**A tool's own failure no longer aborts the process.** Since
`05_agent_loop`, `read_file`/`list_directory` panicked on a bad path.
This step's `Registry::dispatch` returns a proper `Result`, and
`Agent::handle_tool_calls` catches it, logs `tool_result` with
`ok: false`, and feeds `"ERROR: ..."` back to the model as the tool
result — the turn continues instead of crashing.

**`Logger` never sees the raw request/response unless debug mode is
on.** `raw()` is the only phase gated behind `is_debug()` — every
other phase always logs, keeping the default `.jsonl` output compact.

## Porting notes

- **Ruby's module self-methods (`quiet!`/`loud!`/`quiet?`,
  `debug!`/`debug?`, memoized `config`) → free functions in `lib.rs`**,
  backed by `static` `AtomicBool`s and a `static OnceLock<Config>` —
  all `std`, no new dependency. `pub mod config;` (the existing
  submodule) and `pub fn config()` coexist without conflict: a module
  name and a function name live in separate Rust namespaces (type vs.
  value), the same non-collision Python's own port already noted for
  its `config` submodule/function pair.
- **New dependency: `time = { version = "0.3", features =
  ["formatting"] }`**, confirmed with the user (AskUserQuestion) over
  `chrono`, for RFC3339 timestamp formatting `std` has no equivalent
  for. `local-offset` (Ruby's `Time.now.iso8601`/Python's
  `datetime.now().astimezone()`) is deliberately not enabled — it
  carries a documented soundness caveat around concurrent
  `std::env::set_var` on Unix (RUSTSEC-2020-0071) not worth taking on
  for a timestamp field. Rust's `at` is UTC instead of local-offset —
  an accepted, explicitly-noted cosmetic gap.
- **Session id randomness uses `std::collections::hash_map::RandomState`
  instead of a `rand`/`getrandom` crate.** `RandomState::new()` already
  draws OS-seeded entropy for HashDoS protection on every call;
  `.build_hasher().finish()` with no `.write()` input still finalizes
  against those random keys, giving an effectively-random `u64` per
  call — enough for a unique session-id suffix (not a security
  boundary), with no new dependency.
- **Tool dispatch failures are caught and logged, not just "unknown
  tool name".** Confirmed with the user (AskUserQuestion) over a
  narrower alternative that only caught `UnknownToolError`. Ruby's
  `rescue StandardError`/Python's `except Exception` around
  `@registry.dispatch` catches *both* an unregistered tool name and
  any error the tool's own code raises — Ruby's own `example.rb`
  `read_file` block raises `Errno::ENOENT` on a bad path, a real
  failure mode. `Tool`'s closure signature changes from
  `Fn(&HashMap<String,String>) -> String` to `Fn(&HashMap<String,
  String>) -> Result<String, String>`; `Registry::dispatch` returns
  `Result<String, DispatchError>` (`UnknownTool` | `ToolFailed`); this
  **supersedes `05_agent_loop`'s Decision 12** (`.expect()`-ing an
  unknown-tool `Result`), which stops being correct once Ruby/Python's
  own dispatch-time rescue exists to port.
- **`Backend` gains `name(&self) -> &'static str`/`model(&self) ->
  &str`, and `PromptBackend<T>` becomes `PromptBackend<T>: Backend`.**
  `Logger::response` needs the active backend's provider name, model
  string, usage unit/level, and estimated cost, but
  `PromptBuilder<T>.backend: Box<dyn PromptBackend<T>>` only exposed
  `to_payload`/`parse_response`/`headers`/`url` — none of `Backend`'s
  metadata was reachable through that trait object, and `backend_name()`
  itself isn't dyn-callable (`where Self: Sized`). Each backend adds
  `name(&self) -> &'static str { Self::backend_name() }` (valid despite
  `backend_name`'s `Sized` bound — this call is monomorphized against
  the concrete backend type, not generic) and `model(&self) -> &str {
  &self.model }`. The new supertrait bound makes `Box<dyn
  PromptBackend<T>>` also satisfy `Backend`, so
  `PromptBuilder::backend(&self) -> &dyn Backend` returns a fully
  type-erased reference — `Logger` stays non-generic over `T: Task`,
  matching Ruby/Python's `Logger` not knowing about tasks at all.
- **`Logger::prompt`'s message serialization uses `Message.content_blocks`
  when present, falling back to `Message.content`** — reproducing
  Ruby/Python's duck-typed `msg.content` (a plain string, or the raw
  block array for an assistant turn with tool calls) using the
  additive field `05_agent_loop` already introduced for this exact
  gap, instead of always logging the already-flattened text and
  silently losing tool_use block detail from multi-tool-call turns.
- **`Logger` stays fully non-generic — `response()` takes `task:
  Option<&str>` (already resolved), not a task object.** Ruby/Python
  hand `Logger` a task object and let it call `task_name`/
  `task.task_name()`. Rust's `Task` (fixed since `01_struct_skeleton`)
  is a compile-time-only marker with no runtime instance to hand
  over — `Agent::log_response` calls `Some(T::task_name())` itself. A
  direct simplification, not a capability reduction: the same string
  ends up in the log line either way.
- **`Agent::new` takes `logger: &'a mut Logger` as a required
  parameter**, not `Option<&mut Logger>` defaulting to a lazily
  constructed one. Python's own port needs a `None` sentinel + lazy
  construction specifically because Python evaluates default
  *argument values* once, at function-definition time — a literal
  `logger=Logger()` default would share one Logger/session file across
  every `Agent`. Rust has no default-argument syntax at all, so there
  is no equivalent pitfall to guard against.
- **`LoopError` removed; `Config::mud_*` accessors removed.** Direct
  ports of Ruby's/Python's own removals this step — both were already
  unused in the Rust port too, so this is pure deletion.

## Run Example

```bash
./week1_baseline/bin/rust/06_the_logger
```

This makes one or more real HTTP requests to whichever provider
`.boukensha/settings.yaml` configures (Anthropic, by default in this
repo's fixture) — it costs a small amount per model round-trip and
requires a valid API key in `.boukensha/.env`. It also writes a new
`.boukensha/sessions/<session-id>.jsonl` file.

Example output (the exact tool calls and final text are **not**
reproducible byte-for-byte — they're live model responses):

```
=== BOUKENSHA Step 6: The Logger ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Max iterations: 25
Max output tokens: 1024


=== FINAL RESPONSE ===
## Summary

Based on the README.md file, here's what the **Boukensha MUD Player Assistant Framework** can do:
...
```

Unlike step 5's console transcript, there are no `[iteration N/M]` /
`tool call →` / `tool result →` lines — that progress output now lives
only in the session's `.jsonl` file, structurally verified against the
same session's real Python run for this fixture (see
`docs/plans/rust_port/06_the_logger.md`).
