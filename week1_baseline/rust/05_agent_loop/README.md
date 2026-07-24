# 05 · The Agent Loop (Rust port)

Behavior port of `ruby/05_agent_loop` / `python/05_agent_loop` — the
Agent Loop is the heart of BOUKENSHA. Everything before this —
structs, registry, prompt builder, client — was setup. The loop is
where the agent actually does work: send a request, dispatch any tool
calls the model asks for, and repeat until the model signals it's
done.

`message.rs`, `tool.rs`, `registry.rs`, `tasks/{mod,player}.rs`, and
every `backends/{anthropic,gemini,ollama,ollama_cloud,openai}.rs`'s
model tables are unchanged from `04_api_client`; see
`../04_api_client/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/agent.rs` | The agent loop — sends requests, dispatches tools, and knows when to stop |

## Updated Files

| File | Change |
|---|---|
| `src/prompt_builder.rs` | No longer stores `&Context`; `to_api_payload`/new `parse_response` take `context`/`response` per call (see Porting Notes) |
| `src/client.rs` | `call` takes `context: &Context<T>` and `tools: Option<&[serde_json::Value]>` per call |
| `src/context.rs` | Adds `add_assistant_message` |
| `src/message.rs` | Adds `content_blocks: Option<Vec<serde_json::Value>>` and a `Message::assistant` constructor |
| `src/errors.rs` | Adds `LoopError` (unused so far) |
| `src/tasks/base.rs` | `Task` trait gains `max_iterations`/`max_output_tokens` default methods |
| `src/backends/base.rs` | Adds `ParsedResponse`/`StopReason`; `PromptBackend` gains `parse_response`; `to_payload` gains a `tools` param; `chat_style_messages` gains an `assistant_message` closure param; adds `tool_call_assistant_message` |
| `src/backends/anthropic.rs` | `to_payload` accepts `tools`; adds `parse_response`; `to_messages` handles `content_blocks` inline |
| `src/backends/openai.rs` | `to_payload` accepts `tools`; adds `parse_response` and a private `assistant_message` fn |
| `src/backends/gemini.rs` | `to_payload` accepts `tools`; adds `parse_response` and `assistant_parts` |
| `src/backends/ollama.rs`, `ollama_cloud.rs` | `to_payload` accepts `tools`; adds `parse_response`, using the shared `tool_call_assistant_message` |

## How It Works

```
send messages to API
        ↓
stop_reason == ToolUse?
    yes → extract tool calls
        → dispatch each tool via Registry
        → inject results as tool_result messages
        → go back to top
    no  → return final text response
```

## boukensha_05_agent_loop::agent::Agent

| Method | Description |
|---|---|
| `Agent::new(registry, builder, client, task_settings, max_iterations, max_output_tokens)` | Wires up the loop's collaborators and resolves its limits |
| `run(&mut self) -> Result<String, ApiError>` | Runs the loop until the model signals `end_turn` (or the iteration ceiling triggers a wind-down), returning the final text |

## Every Backend Speaks the Same Normalized Shape

Five providers means five different response formats — Anthropic
nests tool calls inside `content`, Ollama puts them in
`message.tool_calls`, OpenAI nests them under
`choices[0].message.tool_calls`, and Gemini calls them `functionCall`
parts. Rather than teach the Agent loop about each of these, every
backend implements `parse_response`, converting its raw response into
one common shape:

```rust
pub struct ParsedResponse {
    pub stop_reason: StopReason,       // ToolUse | EndTurn
    pub content: Vec<serde_json::Value>, // [{"type": "text", ...} | {"type": "tool_use", ...}]
}
```

`Agent` only ever sees this shape via `builder.parse_response(&response)`,
which delegates to the backend, and never inspects a raw provider
response.

The conversion also runs in reverse. When the conversation history is
replayed on the next request, OpenAI, Ollama, and OllamaCloud each
rebuild a provider-specific assistant message from the normalized
`content_blocks` via `assistant_message`/`tool_call_assistant_message`
— the inverse of `parse_response`. Anthropic and Gemini handle this
inline in their own `to_messages`. Anthropic's `content` array doubles
as both the normalized shape and the wire format, so its inverse is
just `blocks.clone()`.

**Tool call IDs aren't universal.** Anthropic and OpenAI assign every
tool call a unique `id`, echoed back in the `tool_result`. Ollama,
Ollama Cloud, and Gemini don't assign call ids at all — those backends
reuse the tool's `name` as its `id` and match the `tool_result` back
to the call by name.

## Task Configuration

Same task-based configuration as prior steps:

```yaml
tasks:
  player:
    provider: anthropic
    model: claude-haiku-4-5
    prompt_override:
      system: true
    max_iterations: 25
    max_output_tokens: 1024
```

`max_iterations` controls model round-trips per turn before wind-down;
`max_output_tokens` is passed to each model reply. Both default to 25
and 1024 respectively when absent — see `Task::DEFAULT_MAX_ITERATIONS`/
`DEFAULT_MAX_OUTPUT_TOKENS`.

## No New Dependencies

`agent.rs` and every backend edit use only `serde_json`, already a
dependency since `03_prompt_builder`. `Cargo.toml` gains no new crate.

## Considerations

**The assistant message must be stored before the tool result.** The
Anthropic API requires the assistant's tool_use block to appear in the
message history before its corresponding tool_result.
`Agent::handle_tool_calls` gets this order right — get it wrong and
the API rejects the request.

**The model can call multiple tools in one turn.** The loop handles
this by iterating over every `tool_use` block in a single response
before making the next API call.

**The iteration ceiling is a trigger threshold, not a hard cap.** A
poorly prompted agent can loop forever if the model keeps calling
tools. BOUKENSHA stops starting new work after 25 iterations by
default and makes one short wind-down call with tools disabled
(`tools: Some(&[])`). This keeps the turn bounded while still
returning a useful final response, rather than aborting mid-turn.

**The agent has no way to stop itself.** The model signals it is done
via `StopReason::EndTurn`. BOUKENSHA watches for that signal and exits
the loop. The agent never decides unilaterally to stop.

## Porting notes

- **`PromptBuilder`/`Client` stop storing a long-lived `&Context`;
  `Context` is passed per call instead.** The direct port hit a real
  borrow-checker conflict Ruby/Python's dynamic aliasing never
  surfaces: `PromptBuilder<'a, T>` held `context: &'a Context<T>` for
  its whole lifetime in `04_api_client`, but the Agent loop needs
  `&Context` to build each request *and* `&mut Context`
  (`registry.context.add_message(...)`) between requests — those can't
  coexist under one long-lived shared borrow. Confirmed with the user
  (AskUserQuestion) over a `RefCell`-based interior-mutability
  alternative, which was rejected for adding runtime-checked mutability
  to a struct every future step also touches. `PromptBuilder<T>` now
  only owns `backend: Box<dyn PromptBackend<T>>`; `Client<'a, T>` still
  borrows `&'a PromptBuilder<T>` (that borrow no longer touches
  `Context`, so it doesn't conflict with mutation elsewhere).
- **`chat_style_messages` gains an `assistant_message` closure
  parameter**, confirmed with the user (AskUserQuestion) over giving
  OpenAI/Ollama/OllamaCloud fully separate `to_messages` methods. Keeps
  the existing single source of truth for the `tool_result`/plain-
  message branches while letting each backend supply its own
  assistant-message shape (OpenAI includes a call `id` and
  JSON-stringifies `arguments`; Ollama/OllamaCloud share one helper,
  `tool_call_assistant_message`, with neither).
- **`Message` gains an additive `content_blocks: Option<Vec<serde_json::Value>>`
  field, rather than changing `content`'s type.** Ruby/Python's message
  content is duck-typed — a plain string, or the raw array of
  normalized blocks for an assistant turn that included tool calls.
  Rust's `Message.content: String` (fixed since `01_struct_skeleton`)
  can't hold both shapes; turning it into an enum would touch every
  existing call site across all backends and `context.rs`, none of
  which changed in this step's Python diff. `content_blocks` stays
  `None` for every pre-existing message path (`Message::new`) and is
  only ever `Some(blocks)` on the new `Message::assistant` path.
- **`Task`'s `respond_to?`/`hasattr` duck-typing check has no Rust
  equivalent — and isn't ported as a runtime check.** `T: Task` is a
  compile-time trait bound; every `T` satisfying it has
  `max_iterations`/`max_output_tokens`, full stop. `Agent`'s resolver
  keeps Ruby's observable three-tier fallback (explicit override →
  task-settings-derived value → `Agent`'s own constant) by branching
  only on whether `task_settings` itself is `Some`/`None`, since the
  "does the method exist" question that motivated Ruby's `respond_to?`
  is answered by the type system before the code compiles.
- **`Agent::run` returns `Result<String, ApiError>`**, the direct
  translation of Ruby's "an `ApiError` from a normal loop iteration
  propagates uncaught; only `wrap_up`'s wind-down call `rescue`s it."
  `wrap_up` itself stays infallible (`-> String`), `match`ing
  `Ok`/`Err` instead of Ruby's `begin/rescue`, exactly where Ruby
  scopes the rescue.
- **Tool-call `input` (a JSON object) is converted to `HashMap<String,
  String>` at `Agent`'s dispatch call site**, not by changing
  `Tool`/`Registry`'s block signature — neither changed in this step's
  Python diff, so per the port's core rule they stay untouched.
  `json_object_to_string_map` extracts each JSON string value directly,
  falling back to that value's compact JSON text otherwise.
- **A cosmetic, already-accepted repr difference**: the
  `  tool call → name(args)` line prints the tool_use block's raw
  `serde_json::Value` `input` (compact JSON, e.g.
  `{"path":"README.md"}`), differing from Ruby's `=>`-style hash repr
  and Python's `'single-quoted'` dict repr. Same category as earlier
  steps' cross-language repr gaps — not fixed, just noted.
- **`ParsedResponse`/`StopReason`** replace Ruby/Python's untyped
  `{stop_reason: "tool_use" | "end_turn", content: [...]}` hash/dict.
  An enum for `stop_reason` is strictly safer than carrying the string
  around and follows this codebase's existing preference for small
  typed structs at API boundaries (`ConfigError`, `ModelInfo`), while
  `content` stays `Vec<serde_json::Value>` — untyped JSON blocks,
  matching how `to_payload`/`to_tools` already return `serde_json::Value`
  throughout this codebase.

## Run Example

```bash
./week1_baseline/bin/rust/05_agent_loop
```

This makes one or more real HTTP requests to whichever provider
`.boukensha/settings.yaml` configures (Anthropic, by default in this
repo's fixture) — it costs a small amount per model round-trip and
requires a valid API key in `.boukensha/.env`.

Example output (the exact tool calls and final text are **not**
reproducible byte-for-byte — they're live model responses, and the
model may choose different tools, or none, from one run to the next):

```
=== BOUKENSHA Step 5: Agent Loop ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Max iterations: 25
Max output tokens: 1024

[iteration 1/25]
  tool call → read_file({"path":"README.md"})
  tool result → # 05 · The Agent Loop (Rust port)...
[iteration 2/25]

=== FINAL RESPONSE ===
## Summary of the MUD Player Assistant Framework
...
```

Structurally identical to the verified Ruby and Python runs for the
same fixture: one `read_file` tool call, `end_turn` on iteration 2, no
`list_directory` call. All three are real, independent live-model
responses — matching structure (which tool, how many iterations), not
byte-identical text.
