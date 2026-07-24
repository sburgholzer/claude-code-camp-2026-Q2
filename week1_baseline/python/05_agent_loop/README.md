# 05 · The Agent Loop (Python port)

Behavior port of `ruby/05_agent_loop` — the Agent Loop is the heart of
BOUKENSHA. Everything before this — structs, registry, prompt builder,
client — was setup. The loop is where the agent actually does work.
`message.py`, `tool.py`, `context.py`, `registry.py`, `tasks/player.py`,
and `backends/base.py` are unchanged from `04_api_client`; see
`../04_api_client/README.md` for those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/agent.py` | The agent loop — sends requests, dispatches tools, and knows when to stop |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/client.py` | `call()` gains `tools=None`, threaded into `to_api_payload` |
| `lib/boukensha/prompt_builder.py` | Added `parse_response(response)`, delegating to the backend; `to_api_payload` gains `tools=None` |
| `lib/boukensha/errors.py` | Added `LoopError` for runaway agents (unused so far) |
| `lib/boukensha/tasks/base.py` | Added `max_iterations`/`max_output_tokens` class methods, backed by `DEFAULT_MAX_ITERATIONS`/`DEFAULT_MAX_OUTPUT_TOKENS` |
| `lib/boukensha/backends/anthropic.py` | `to_payload` accepts `tools=None`; added `parse_response` |
| `lib/boukensha/backends/openai.py` | `to_payload` accepts `tools=None`; added `parse_response` and `_assistant_message` |
| `lib/boukensha/backends/gemini.py` | `to_payload` accepts `tools=None`; added `parse_response` and `_assistant_parts` |
| `lib/boukensha/backends/ollama.py` | `to_payload` accepts `tools=None`; added `parse_response` and `_assistant_message` |
| `lib/boukensha/backends/ollama_cloud.py` | Same as `ollama.py` |

## How It Works

```
send messages to API
        ↓
stop_reason == "tool_use"?
    yes → extract tool calls
        → dispatch each tool via Registry
        → inject results as tool_result messages
        → go back to top
    no  → return final text response
```

## boukensha.agent.Agent

| Method | Description |
|---|---|
| `run()` | Starts the loop and returns the final text response when the agent is done |

## Every Backend Speaks the Same Normalized Shape

Five providers means five different response formats — Anthropic
nests tool calls inside `content`, Ollama puts them in
`message.tool_calls`, OpenAI nests them under
`choices[0].message.tool_calls`, and Gemini calls them `functionCall`
parts. Rather than teach the Agent loop about each of these, every
backend implements `parse_response`, converting its raw response into
one common shape:

```python
{
    "stop_reason": "tool_use" | "end_turn",
    "content": [
        {"type": "text", "text": "..."},
        {"type": "tool_use", "id": "...", "name": "...", "input": {...}},
    ],
}
```

`Agent` only ever sees this shape — it calls
`self.builder.parse_response(response)`, which delegates to the
backend, and never inspects a raw provider response.

The conversion also runs in reverse. When the conversation history is
replayed on the next request, Ollama, Ollama Cloud, OpenAI, and Gemini
each rebuild a provider-specific assistant message from the normalized
`content` blocks via a private `_assistant_message` (or
`_assistant_parts`) method — the inverse of `parse_response`.
Anthropic's `content` array doubles as both the normalized shape and
the wire format, so it needs no extra conversion.

**Tool call IDs aren't universal.** Anthropic and OpenAI assign every
tool call a unique `id`, echoed back in the `tool_result`. Ollama,
Ollama Cloud, and Gemini don't assign call ids at all — those backends
reuse the tool's `name` as its `id` and match the `tool_result` back
to the call by name.

## Task Configuration

This step uses the task-based configuration introduced in earlier
steps:

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

When `prompt_override.system` is true, Boukensha reads
`.boukensha/prompts/player/system.md`. Otherwise it falls back to this
step's shipped `prompts/system.md`. `max_iterations` controls model
round-trips per turn before wind-down, and `max_output_tokens` is
passed to each model reply.

## No New Dependencies

`agent.py` and every backend edit use only the standard library
(`json`, already used elsewhere). `requirements.txt` is unchanged from
`04_api_client` — matches Ruby's own `Gemfile`/`Gemfile.lock`, which
are unchanged in this step too.

## Considerations

**The assistant message must be stored before the tool result.** The
Anthropic API requires the assistant's tool_use block to appear in the
message history before its corresponding tool_result. `_handle_tool_calls`
gets this order right — get it wrong and the API rejects the request.

**The model can call multiple tools in one turn.** The loop handles
this by iterating over all tool_use blocks in a single response before
making the next API call.

**`MAX_ITERATIONS` is a turn ceiling.** A poorly prompted agent can
loop forever if the model keeps calling tools. BOUKENSHA stops
starting new work after 25 iterations by default and makes one short
wrap-up call with tools disabled (`tools=[]`). This keeps the turn
bounded while still returning a useful final response.

**The agent has no way to stop itself.** The model signals it is done
via `stop_reason: "end_turn"`. BOUKENSHA watches for that signal and
exits the loop. The agent never decides unilaterally to stop.

## Porting notes

- **Ruby's `tools: nil` sentinel → Python's `tools=None` sentinel.**
  Every backend's `to_payload` uses
  `self.to_tools(context.tools) if tools is None else tools` — an
  explicitly supplied empty list (`tools=[]`, used by the wind-down
  call) passes straight through as "no tools," matching Ruby's
  `tools.nil? ? to_tools(context.tools) : tools`.
- **Ruby's `private` section → leading-underscore methods**, matching
  the convention already used in `backends/base.py`
  (`_configure_model`) and `tasks/base.py` (`_fetch`). `Agent`'s
  private helpers (`_resolve_max_iterations`, `_resolve_max_output_tokens`,
  `_iteration_limit_reached`, `_call_opts`, `_wrap_up`,
  `_fallback_message`, `_extract_text`, `_handle_tool_calls`) and each
  backend's `_assistant_message`/`_assistant_parts` follow the same
  pattern; only `run` stays public.
- **Ruby ivars → plain (non-underscored) Python instance attributes.**
  `self.max_iterations`, `self.max_output_tokens`, `self.iteration`
  have no explicit reader in Ruby either — the underscore convention
  in this codebase is reserved for methods, not attributes.
- **`result.to_s[0..60]` → `str(result)[:61]`**, the same `[0..N]` →
  `[:N+1]` translation already used for `message.py`'s `content[:61]`
  and `tool.py`'s `description[:41]`.
- **A cosmetic, already-accepted repr difference**: the
  `  tool call → name(args)` line prints Python's dict `repr()`
  (`{'path': 'README.md'}`) where Ruby prints a Ruby hash `#to_s`
  (`{"path" => "README.md"}`). Same category as earlier steps' symbol
  vs. string repr gaps — not fixed, just noted.
- **`Config`/`config.py` needed no edit.** Ruby's `config.rb` diff for
  this step only rewrites four `mud_*` accessors as endless methods
  (`def mud_host = ...`) — pure syntax, no behavior change. Python's
  `config.py` already expresses these as `@property` methods from an
  earlier step, which is the Pythonic equivalent of both Ruby forms.

## Run Example

```bash
./week1_baseline/bin/python/05_agent_loop
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
  tool call → read_file({'path': 'README.md'})
  tool result → # 05 · The Agent Loop (Python port)

Behavior port
[iteration 2/25]

=== FINAL RESPONSE ===
## Summary

This MUD player assistant framework (called **BOUKENSHA**) ...
```

Structurally identical to the verified Ruby run for the same fixture:
one `read_file` tool call on iteration 1, `end_turn` on iteration 2,
no `list_directory` call. Both are real, independent live-model
responses — matching structure (which tool, how many iterations), not
byte-identical text.
