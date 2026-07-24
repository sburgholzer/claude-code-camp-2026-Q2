# 05 · The Agent Loop (Rust port)

## Goal

Ports `python/05_agent_loop` (and its Ruby ground truth,
`ruby/05_agent_loop`) into `rust/05_agent_loop`. This step adds the
tool-call loop that ties `Client`, `PromptBuilder`, and `Registry`
together: send a request, check `stop_reason`, dispatch any
`tool_use` blocks, loop until the model signals `end_turn`, with an
iteration ceiling and a tools-off wind-down call as the safety valve.

`rust/05_agent_loop` currently equals `rust/04_api_client` byte for
byte (confirmed via `diff -rq rust/04_api_client rust/05_agent_loop`
— no output), still under the stale `boukensha_04_api_client` package
name.

Diffing `python/04_api_client` → `python/05_agent_loop` (ignoring
`__pycache__`) shows the only real changes are:

- `lib/boukensha/agent.py` (new)
- `lib/boukensha/client.py` (`call` gains `tools=None`)
- `lib/boukensha/prompt_builder.py` (`to_api_payload` gains
  `tools=None`; adds `parse_response`)
- `lib/boukensha/errors.py` (adds `LoopError`, unused)
- `lib/boukensha/tasks/base.py` (adds `max_iterations`/
  `max_output_tokens` class methods)
- `lib/boukensha/backends/*.py` (all five: `tools=None` threading +
  `parse_response`; four of five also add a private
  `_assistant_message`/`_assistant_parts` inverse — Anthropic doesn't
  need one, its `content` array already is the wire format)
- `lib/boukensha/__init__.py` (export `Agent`, `LoopError`)
- `examples/example.py`, `README.md`

Python is the direct reference; Ruby (`ruby/05_agent_loop`) is the
ultimate spec where the two disagree — used here to confirm the exact
loop shape (trigger-threshold semantics, wrap-up call ordering,
multi-tool-call-per-turn handling) since this Rust step is written
directly against both, having ported Python step 05 in this same
session.

## Source files to port

| File | Role |
|---|---|
| `python/05_agent_loop/lib/boukensha/agent.py` | Reference for the loop, wind-down call, tool dispatch ordering |
| `ruby/05_agent_loop/lib/boukensha/agent.rb` | Ground truth: trigger-threshold iteration ceiling, `rescue ApiError` scoped to the wrap-up call only |
| `python/05_agent_loop/lib/boukensha/prompt_builder.py`, `client.py` | Reference for `tools=`/`parse_response` threading |
| `python/05_agent_loop/lib/boukensha/backends/*.py` | Reference for `parse_response` shapes and `_assistant_message`/`_assistant_parts` inverses per provider |
| `python/05_agent_loop/lib/boukensha/tasks/base.py` | Reference for `max_iterations`/`max_output_tokens` defaults (25 / 1024) |
| `python/05_agent_loop/examples/example.py` | New tools registered before `Agent` construction, `Agent` wiring, printed Config/Provider/Model/Max-iterations/Max-output-tokens lines |

## Runtime fixture to reuse

`.boukensha/` at the repo root, unchanged: `settings.yaml` configures
`tasks.player` for `anthropic`/`claude-haiku-4-5` with
`prompt_override.system: true` and no `max_iterations`/
`max_output_tokens` overrides (both fall back to the 25/1024
defaults); `.env` holds a real `ANTHROPIC_API_KEY`.

Verified transcript reused from this session's Python port (both the
real Ruby run and the real Python run were made moments earlier in
this same session, with the user's go-ahead for the live, billed
calls both times — not re-run here to avoid a third round of billed
calls against an unchanged fixture):

```
=== BOUKENSHA Step 5: Agent Loop ===

Config: #<Boukensha::Config dir=.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Max iterations: 25
Max output tokens: 1024

[iteration 1/25]
  tool call → read_file({'path': 'README.md'})
  tool result → # 05 · The Agent Loop (Python port)...
[iteration 2/25]

=== FINAL RESPONSE ===
## Summary
...
```

One `read_file` tool call on iteration 1, `end_turn` on iteration 2,
no `list_directory` call — real, independent live-model behavior, not
reproducible byte-for-byte across runs or languages. The Rust run
below is compared structurally (same shape: which tool got called,
how many iterations, `=== FINAL RESPONSE ===` present), not
byte-for-byte.

## Decisions (confirmed)

1. **`PromptBuilder`/`Client` stop storing a long-lived `&'a
   Context<T>`; `Context` is passed as a per-call argument instead.**
   Confirmed with the user (AskUserQuestion) over a `RefCell`-based
   interior-mutability alternative. The direct port was blocked by a
   real borrow-checker conflict that Ruby/Python's dynamic aliasing
   never surfaces: `PromptBuilder<'a, T>` held `context: &'a
   Context<T>` for its whole lifetime (04_api_client's design), but
   the Agent loop needs `&Context` to build each request *and*
   `&mut Context` (`registry.context.add_message(...)`) between
   requests — those can't coexist under one long-lived shared borrow.
   Rejected the `RefCell<Vec<Message>>` alternative because it adds
   runtime-checked interior mutability to `Context`, a struct every
   future step also touches, purely to avoid a signature change that
   is otherwise small and mechanical. Concretely:
   - `PromptBuilder<T: Task>` drops its `context` field and the `'a`
     lifetime tied to it; it now only owns `backend: Box<dyn
     PromptBackend<T>>`.
   - `PromptBuilder::to_api_payload(&self, context: &Context<T>,
     max_output_tokens: u32, tools: Option<&[serde_json::Value]>) ->
     serde_json::Value` and the new `PromptBuilder::parse_response(&self,
     response: &serde_json::Value) -> ParsedResponse` take `context`/
     `response` per call.
   - `Client<'a, T>` keeps borrowing `&'a PromptBuilder<T>` (that
     borrow no longer touches `Context` at all, so it doesn't conflict
     with mutation); `Client::call(&self, context: &Context<T>,
     max_output_tokens: u32, tools: Option<&[serde_json::Value]>) ->
     Result<serde_json::Value, ApiError>` takes `context` per call.
   - `PromptBackend::to_payload` already took `context: &Context<T>`
     as a parameter (established in `04_api_client`), so backend trait
     implementations are unaffected by this decision beyond the new
     `tools` parameter (Decision 4) and `parse_response` addition.
   - No observable behavior change from Ruby/Python — this is purely
     a Rust ownership-shape fix forced by the language, the same
     category as `04_api_client`'s `http_status_as_error(false)` or
     `RequestBuilder`-per-attempt decisions.
2. **`chat_style_messages` gains an `assistant_message: impl Fn(&[serde_json::Value])
   -> serde_json::Value` parameter**, called only when a message's
   role is `"assistant"` and it carries `content_blocks` (Decision 3).
   Confirmed with the user (AskUserQuestion) over giving OpenAI,
   Ollama, and OllamaCloud fully separate `to_messages` methods.
   Keeps the existing single source of truth for the `tool_result`/
   plain-message branches (shared since `04_api_client`) while letting
   each backend supply its own assistant-message shape:
   - `openai.rs` passes a private `assistant_message` fn (includes a
     call `id`, JSON-stringifies `arguments`).
   - `ollama.rs`/`ollama_cloud.rs` both pass a new shared helper,
     `base::tool_call_assistant_message` (no `id`, raw `arguments`
     value) — identical shape in both providers, same as their
     existing shared `to_payload`/`to_messages` reuse.
   - Anthropic and Gemini already have their own bespoke
     `to_messages` (never used `chat_style_messages`), so their
     assistant-block handling is added inline to those existing
     methods rather than through this shared helper — no dedup
     decision needed there.
3. **`Message` gains an additive `content_blocks: Option<Vec<serde_json::Value>>`
   field**, rather than changing `content`'s type. Ruby/Python's
   `Message#content`/`.content` is duck-typed — a plain `String` for
   user/tool_result messages, or the raw array of normalized
   `{"type": ..., ...}` blocks for an assistant turn that included
   tool calls. Rust's `Message.content: String` (fixed since
   `01_struct_skeleton`) can't hold both shapes. Rather than turn
   `content` into an enum (which would touch every existing call site
   across all backends and `context.rs`, none of which changed in
   this step's Python diff), `content_blocks` is `None` for every
   existing message-construction path (`Message::new`, used by
   `add_message` for `user`/`tool_result`/plain-text messages — all
   unchanged) and `Some(blocks)` only for the new assistant-turn path:
   `Message::assistant(text, blocks)`, constructed via a new
   `Context::add_assistant_message`. Every backend's assistant-message
   handling branches on `content_blocks.is_some()` first, falling back
   to the plain-`content` behavior already in place — a message
   carrying no blocks (there aren't any in practice, since Ruby/Python
   only ever call `add_message(:assistant, ...)` from the tool-call
   branch, never the plain-text terminal branch) behaves exactly as
   `04_api_client`'s code already did.
4. **`to_payload` gains `tools: Option<&[serde_json::Value]>` across
   all five backends**, the direct analog of Ruby's `tools: nil` /
   Python's `tools=None` sentinel: `None` means "compute the default
   from `context.tools`" (`self.to_tools(context.tools)` in Ruby/
   Python becomes `self.to_tools(&context.tools)` when `tools.is_none()`
   in Rust); `Some(&[])`, used by the wind-down call, passes an
   explicit empty tool list straight through. `Option`/`None` is a
   more direct match for this sentinel than Ruby's `nil`/Python's
   `None` even was — no new ambiguity, direct one-line port per
   backend.
5. **`ParsedResponse { stop_reason: StopReason, content: Vec<serde_json::Value>
   }` with `enum StopReason { ToolUse, EndTurn }`**, replacing
   Ruby/Python's `{stop_reason: "tool_use" | "end_turn", content:
   [...]}` hash/dict. An enum for `stop_reason` is strictly safer than
   carrying the string around (the `Agent`'s `match` is exhaustive and
   typo-proof) and follows this codebase's existing precedent of
   preferring small typed enums/structs over loosely-typed values at
   API boundaries (`ConfigError`, `ModelInfo`) while keeping `content`
   as `Vec<serde_json::Value>` — untyped JSON blocks, matching how
   `to_payload`/`to_tools`/`function_wrapped_tools` already return
   `serde_json::Value` throughout this codebase rather than
   backend-specific structs.
6. **`PromptBackend<T>` gains a required `parse_response(&self,
   response: &serde_json::Value) -> ParsedResponse` method.** Direct
   analog of Ruby/Python's per-backend `parse_response`, added to the
   same trait `to_payload`/`headers`/`url` already live on
   (`backends/base.rs`).
7. **Task trait's `respond_to?`/`hasattr` duck-typing check has no
   Rust equivalent needed — and isn't ported as a runtime check.**
   Ruby/Python's `Agent#resolve_max_iterations` checks `task_settings
   && @context.task.respond_to?(:max_iterations)` before delegating,
   because a *duck-typed* task object might not implement the method.
   In Rust, `T: Task` is a compile-time trait bound; every `T` that
   satisfies it has `max_iterations`/`max_output_tokens` (Decision 8),
   full stop — the "does it respond to this" question is answered by
   the type system before the code even compiles, the same class of
   capability note as `04_api_client`'s Decision 7 (`settings.is_a?(Hash)`
   guard needing no Rust equivalent). `Agent`'s resolver keeps Ruby's
   *observable* three-tier fallback (explicit override → task-settings-
   derived value → `Agent`'s own constant) by branching only on
   whether `task_settings` itself is `Some`/`None` — not on whether the
   trait method exists, since it always does:
   ```rust
   fn resolve_max_iterations(task_settings: Option<&Settings>, explicit: Option<u32>) -> u32 {
       explicit.unwrap_or_else(|| task_settings.map(T::max_iterations).unwrap_or(Self::MAX_ITERATIONS))
   }
   ```
   Numerically this always agrees with Ruby/Python: `Task::max_iterations`'s
   own internal default (25) and `Agent::MAX_ITERATIONS` (25) are the
   same literal value by design in every language's port, so the two
   fallback tiers are behaviorally indistinguishable except in the one
   case Rust *does* still branch on (`task_settings` missing
   entirely).
8. **`Task` trait gains `max_iterations`/`max_output_tokens` default
   methods plus `DEFAULT_MAX_ITERATIONS`/`DEFAULT_MAX_OUTPUT_TOKENS`
   associated consts**, mirroring Ruby's `Tasks::Base`/Python's
   `tasks/base.py` additions this step, using the same
   `fetch`-then-`unwrap_or`-default shape already established for
   `Config::mud_port` (`00_config`) and `Task::provider`/`model`
   (`04_api_client`) — no new pattern.
9. **`LoopError` ports to `errors.rs` as `LoopError(pub String)`**,
   following the exact `ApiError`/`UnsupportedModelError` tuple-struct
   pattern already established, exported from `lib.rs`. Unused in this
   step's actual code path, same as Ruby's own addition — see Ruby's
   `errors.rb` comment; the Rust port carries the same "added but not
   yet raised" status rather than second-guessing it.
10. **Tool-call `input` (a JSON object) is converted to
    `HashMap<String, String>` at the `Agent`'s dispatch call site**,
    not by changing `Tool`/`Registry`'s block signature. Neither
    `tool.py`/`tool.rs` nor `registry.py`/`registry.rs` changed in
    this step's Python diff, so per the port's core rule they stay
    untouched here too — `Registry::dispatch`'s existing signature
    (`&HashMap<String, String>`, fixed since `02_the_registry`) is the
    port target, not something this step gets to redesign. A small
    free function in `agent.rs`, `json_object_to_string_map`, converts
    each JSON object value to a string (using the string's own value
    directly when it's already a JSON string, else that value's
    compact JSON text) before dispatch — sufficient for this step's
    fixture (`read_file`/`list_directory`, both single string-typed
    `path` params) and the general case for any JSON-object `input`.
11. **`Agent::run(&mut self) -> Result<String, ApiError>`.** Ruby's
    `run` lets an `ApiError` from a normal loop iteration's
    `@client.call(...)` propagate uncaught (only the wind-down call in
    `wrap_up` has a `rescue ApiError`) — the direct Rust translation of
    "uncaught exception propagates to the caller" is `Result` + `?`,
    with the fallback handling scoped to exactly the one call site
    (`wrap_up`) that Ruby scopes it to, via an explicit `match` instead
    of `?`. `main()` (`examples/example.rs`) `.expect()`s the
    top-level `Result`, consistent with every other fallible call in
    this example (`Config`, backend construction, etc.).
12. **`Registry::dispatch`'s `Err(UnknownToolError)` is `.expect()`-ed,
    not propagated as part of `Agent::run`'s `Result`.** Ruby/Python
    never catch this either — an unregistered tool name reaching
    `dispatch` is a programming bug in the example wiring, not a
    recoverable runtime condition, and Ruby's own uncaught
    `NoMethodError`-adjacent crash is already what happens. Mixing a
    second error type into `Agent::run`'s `Result<String, ApiError>`
    for a case that's supposed to be unreachable would add ceremony
    without a corresponding behavior to preserve.
13. **No new crate dependency.** `agent.rs` and every backend edit use
    only already-present dependencies (`serde_json`, already used
    throughout `backends/*.rs` and `client.rs`). Matches Ruby's/
    Python's own unchanged `Gemfile`/`requirements.txt` for this step.

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/05_agent_loop")
week1_baseline/rust/05_agent_loop/
  Cargo.toml                                    (edit: package name → boukensha_05_agent_loop; no new deps)
  src/
    lib.rs                                      (edit: pub mod agent; export Agent, LoopError)
    agent.rs                                    (new)
    errors.rs                                   (edit: + LoopError)
    prompt_builder.rs                           (edit: drop stored Context/'a; to_api_payload takes context+tools params; + parse_response)
    client.rs                                   (edit: call() takes context+tools params)
    config.rs                                   (unchanged — no Ruby diff this step touches config.rb behaviorally, see Porting Notes)
    context.rs                                  (edit: + add_assistant_message)
    message.rs                                  (edit: + content_blocks field, Message::assistant ctor)
    tasks/base.rs                                (edit: + DEFAULT_MAX_ITERATIONS/DEFAULT_MAX_OUTPUT_TOKENS consts, max_iterations/max_output_tokens default methods)
    tasks/mod.rs, tasks/player.rs, tool.rs,
    registry.rs                                  (unchanged)
    backends/mod.rs                              (unchanged — same exports)
    backends/base.rs                             (edit: PromptBackend gains parse_response; chat_style_messages gains assistant_message closure param; + ParsedResponse/StopReason; + tool_call_assistant_message helper for Ollama/OllamaCloud)
    backends/anthropic.rs                        (edit: to_payload gains tools param; to_messages handles content_blocks inline; + parse_response)
    backends/openai.rs                           (edit: to_payload gains tools param, threads new assistant_message closure into chat_style_messages; + parse_response, private assistant_message fn)
    backends/gemini.rs                           (edit: to_payload gains tools param; to_messages/assistant_parts handle content_blocks; + parse_response)
    backends/ollama.rs                           (edit: to_payload gains tools param, threads tool_call_assistant_message into chat_style_messages; + parse_response)
    backends/ollama_cloud.rs                     (edit: same as ollama.rs)
  prompts/system.md                              (unchanged, already correct)
  README.md                                      (rewrite: this step's own docs — see Porting Notes; rust/04_api_client's README was never actually updated for its own step, an out-of-scope pre-existing gap this plan doesn't fix)
  examples/example.rs                            (edit: register read_file/list_directory after Agent is wired, seed README-reading user message, build Agent, print Config/Provider/Model/Max-iterations/Max-output-tokens, run and print result)
week1_baseline/bin/rust/05_agent_loop            (new launcher)
```

**Note on README.md**: none of `rust/00_config.md` through `04_api_client.md`'s
target-file lists actually included `README.md` as an edit target, and
`rust/04_api_client/README.md` was consequently still titled "03 · The
Prompt Builder" — confirmed live during this step's verification run,
when the agent's `read_file` tool call surfaced that exact stale title
from the copied-forward file. That predated this session and was
initially left out of scope (fixing it wasn't part of this step's own
target-files boundary). The user then explicitly asked for
`rust/04_api_client/README.md` to be fixed as a separate, authorized
follow-up — it's now rewritten to correctly describe `Client`/
`ApiError`/the `ureq` HTTP+TLS decisions, matching this step's own
`README.md`'s structure. `rust/05_agent_loop/README.md`'s
cross-reference note about the staleness was removed once the
referenced file was fixed.

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`Agent<'a, T: Task>`** holds `registry: &'a mut Registry<T>`,
  `builder: &'a PromptBuilder<T>`, `client: &'a Client<'a, T>`,
  `max_iterations: u32`, `max_output_tokens: Option<u32>`,
  `iteration: u32` — mirroring Ruby's `@context`/`@registry`/
  `@builder`/`@client`/`@max_iterations`/`@max_output_tokens`/
  `@iteration` ivars, minus a separate `context` field since
  `Registry<T>` already owns `Context<T>` (`02_the_registry`'s
  Decision 7) — `self.registry.context` stands in for Ruby's
  `@context` throughout.
- **`run(&mut self) -> Result<String, ApiError>`** loops with
  `loop { ... }`, checking `iteration_limit_reached()` before
  incrementing (trigger threshold, not a hard cap — same as Ruby),
  then `self.client.call(&self.registry.context, ...)?`, then
  `self.builder.parse_response(&response)`, then a `match
  parsed.stop_reason { StopReason::ToolUse => ..., StopReason::EndTurn
  => return Ok(...) }` — structurally identical branch shape to
  Ruby's `if parsed[:stop_reason] == "tool_use"`.
- **`wrap_up(&mut self, reason: &str) -> String`** is infallible
  (never returns `Result`) — it always produces *some* text, matching
  Ruby's `rescue ApiError` inside `wrap_up` swallowing the error into
  `fallback_message`. Calls `self.client.call(&self.registry.context,
  WRAP_UP_OUTPUT_TOKENS, Some(&[]))` and `match`es `Ok`/`Err` instead
  of Ruby's `begin/rescue`.
- **Private helpers keep Ruby's `private` section as regular
  (non-`pub`) methods** — Rust's module-privacy is the direct
  equivalent of Ruby's `private` keyword (no leading-underscore
  convention needed the way Python's port used one, since Rust's
  visibility is enforced by the compiler, not a naming convention).
- **`WRAP_UP_DIRECTIVE: &str`** is a single string literal with
  embedded `\n`s, the same flattening Python's port already did for
  Ruby's squiggly heredoc (`<<~MSG.strip`) — no meaningful leading
  whitespace to strip once typed as a literal.
- **`json_object_to_string_map`** — free function converting a
  `&serde_json::Value` object into `HashMap<String, String>` for
  `Registry::dispatch`, extracting `Value::String`s directly and
  falling back to `to_string()` (compact JSON) for any other JSON
  type. See Decision 10.

## Behavior parity checklist

- [x] `errors.rs`: `LoopError(pub String)` added, exported from `lib.rs`
- [x] `prompt_builder.rs`: `PromptBuilder<T: Task>` no longer stores
      `Context`; `to_api_payload`/`parse_response` take `context`/
      `response` per call (Decision 1)
- [x] `client.rs`: `call` takes `context: &Context<T>` and
      `tools: Option<&[serde_json::Value]>` params, otherwise same
      retry/backoff/status logic as `04_api_client`
- [x] `context.rs`: `add_assistant_message(&mut self, content, blocks)`
      added
- [x] `message.rs`: `content_blocks: Option<Vec<serde_json::Value>>`
      field, `Message::assistant(content, blocks)` constructor added;
      `Message::new` unchanged (still sets `content_blocks: None`)
- [x] `tasks/base.rs`: `Task::max_iterations`/`max_output_tokens`
      default methods, backed by `DEFAULT_MAX_ITERATIONS = 25`/
      `DEFAULT_MAX_OUTPUT_TOKENS = 1024` consts
- [x] `backends/base.rs`: `ParsedResponse`/`StopReason` added;
      `PromptBackend::parse_response` added to the trait;
      `chat_style_messages` gains the `assistant_message` closure
      param; `tool_call_assistant_message` shared helper added for
      Ollama/OllamaCloud
- [x] Every backend's `to_payload` accepts `tools: Option<&[serde_json::Value]>`,
      using it verbatim when `Some`, else falling back to
      `self.to_tools(&context.tools)`
- [x] Every backend implements `parse_response`, returning the
      normalized `{stop_reason, content}` shape as `ParsedResponse`
- [x] OpenAI/Ollama/OllamaCloud rebuild a provider-specific assistant
      message from `content_blocks` via `assistant_message`/
      `tool_call_assistant_message`; Anthropic/Gemini handle it inline
      in their existing bespoke `to_messages`
- [x] Ollama/OllamaCloud/Gemini reuse the tool's `name` as its `id` in
      both directions; Anthropic/OpenAI use the provider's real call id
- [x] `agent.rs`: `Agent::run` loops, respects the iteration ceiling as
      a trigger threshold (checked before incrementing), dispatches
      all `tool_use` blocks in a response before the next call, stores
      the assistant message before any `tool_result` message
      (`handle_tool_calls`)
- [x] `agent.rs`: `wrap_up` sends the directive as a `user` message,
      calls with `tools: Some(&[])` and `WRAP_UP_OUTPUT_TOKENS` (400),
      runs outside the counted loop, falls back to a deterministic
      message on blank text or `ApiError`
- [x] `Cargo.toml`: package renamed `boukensha_05_agent_loop`; no new
      dependency
- [x] root `Cargo.toml`: `rust/05_agent_loop` added to workspace
      `members`
- [x] `bin/rust/05_agent_loop` launcher added, executable
- [x] `examples/example.rs`: registers `read_file`/`list_directory`,
      seeds the README-reading user message, builds `Agent` with
      `task_settings`, prints Config/Provider/Model/Max
      iterations/Max output tokens, then the loop's result
- [x] `cargo build --workspace` succeeds
- [x] `bin/rust/05_agent_loop` runs against the live fixture and
      produces a structurally equivalent transcript to the verified
      Python run (same overall shape: iteration counter, tool call/
      result lines, `=== FINAL RESPONSE ===`) — see Verification notes
      for the actual live-model outcome, which is expected to vary
      run to run same as Ruby vs. Python already did.

## Open questions

None outstanding — all decided above.
