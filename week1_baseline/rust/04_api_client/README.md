# 03 · The Prompt Builder (Rust port)

Behavior port of `ruby/03_prompt_builder` / `python/03_prompt_builder` — adds
a `PromptBuilder` and five per-provider backend types that serialize a
`Context` into the exact JSON shape each LLM API expects (Anthropic, OpenAI,
Gemini, Ollama, Ollama Cloud). No API calls are made here, only payload
assembly. `message.rs`, `tool.rs`, `context.rs`, `registry.rs`, and
`tasks/` are unchanged from `02_the_registry`; see
`../02_the_registry/README.md` for those.

Because LLM access, cost and quality are constantly changing, we want
to be able to switch between multiple LLMs that will drive the agent
loop. There are several SDKs that provide access to many LLMs but in
practice we only really need to focus on top-tier models:
- anthropic family
- openai family
- gemini family
- ollama cloud eg. kimi, minimax, llama

The Prompt Builder serializes `Context` for the exact format each API
expects. The `PromptBuilder` delegates to whichever backend you pass
in. `PromptBuilder` does not call the API, we are simply preparing the
format for API calls.

Configuration is task-based here, carried forward from the registry
step. The `player` task owns its provider, model, and prompt override
settings, and the context records the task that the prompt is being
built for.

## New Files

| File | Description |
|---|---|
| `src/prompt_builder.rs` | Delegates serialization to the active backend |
| `prompts/system.md` | Default system prompt used when a task does not override it |
| `src/backends/base.rs` | Shared backend contract for model validation and model metadata |
| `src/backends/anthropic.rs` | Serializes context into the Anthropic API format |
| `src/backends/ollama.rs` | Serializes context into the Ollama API format |
| `src/backends/ollama_cloud.rs` | Serializes context into the Ollama Cloud API format |
| `src/backends/openai.rs` | Serializes context into the OpenAI Chat Completions format |
| `src/backends/gemini.rs` | Serializes context into the Gemini `generateContent` format |

## How It Works

```
Context (Rust structs)
        ↓
PromptBuilder
        ↓
Backend (Anthropic, OpenAI, Gemini, Ollama, or OllamaCloud)
        ↓
API Payload (serde_json::Value)
        ↓
POST to API
```

## boukensha_03_prompt_builder::prompt_builder::PromptBuilder

| Method | Description |
|---|---|
| `to_api_payload(max_output_tokens: u32)` | Assembles the complete payload ready to POST |
| `headers()` | Returns the correct headers for the backend |
| `url()` | Returns the correct endpoint URL for the backend |

## Backends

Each API has its own conventions for how data is expected. Anthropic
and Gemini are the most alike (system prompt as a top-level field),
while OpenAI and Ollama share the same `function`-wrapped tool schema.

Backends also own their supported model table. A backend refuses to
initialize with an unknown model, so `settings.yaml` cannot silently
select an unsupported or misspelled model. Each model entry carries:

| Key | Meaning |
|---|---|
| `context_window` | The model's known token context window |
| `cost_per_million.input` | USD input token price per million tokens, when known |
| `cost_per_million.output` | USD output token price per million tokens, when known |
| `usage_unit` | `"tokens"`, `"local_compute"`, or `"ollama_cloud_usage"` |
| `usage_level` | Ollama Cloud usage tier, when applicable |

Backend instances expose `context_window()`,
`input_token_cost_per_million()`, `output_token_cost_per_million()`,
`usage_unit()`, `usage_level()`, and
`estimate_cost(input_tokens, output_tokens)`. For local Ollama
models, token API cost is `0.0`. For Ollama Cloud, public pricing is
plan/usage based rather than token based, so `estimate_cost` returns
`None`.

The prices in this step are static tutorial data, current as of June
16, 2026, and should be reviewed whenever the selected model set
changes.

### boukensha_03_prompt_builder::backends::Anthropic

Talks to `https://api.anthropic.com/v1/messages`. Requires an
`ANTHROPIC_API_KEY`. Supported models are listed in `Anthropic::models()`.

### boukensha_03_prompt_builder::backends::Ollama

Talks to `http://localhost:11434/api/chat`. Requires `ollama serve`
running locally. No API key needed. Supported models are listed in
`Ollama::models()`.

### boukensha_03_prompt_builder::backends::OllamaCloud

Talks to `https://ollama.com/api/chat`. Requires an `OLLAMA_API_KEY`.
Supported models are listed in `OllamaCloud::models()`.

### boukensha_03_prompt_builder::backends::OpenAI

Talks to `https://api.openai.com/v1/chat/completions`. Requires an
`OPENAI_API_KEY`. Supported models are listed in `OpenAI::models()`.

### boukensha_03_prompt_builder::backends::Gemini

Talks to `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`.
Requires a `GEMINI_API_KEY`. Supported models are listed in
`Gemini::models()`.

### System Prompt

Anthropic and Gemini send the system prompt as a top-level field,
separate from the messages array. Ollama and OpenAI put it inside the
messages array as a `role: system` message.

```json
// Anthropic
{ "system": "You are a MUD player assistant.", "messages": [ ... ] }

// Gemini
{ "systemInstruction": { "parts": [{ "text": "You are a MUD player assistant." }] }, "contents": [ ... ] }

// Ollama / OpenAI
{ "messages": [ { "role": "system", "content": "You are a MUD player assistant." }, ... ] }
```

### Tool Results

Anthropic wraps tool results in a user message. Ollama and OpenAI use
their own `role: tool` message type (with slightly different
identifier fields). Gemini wraps results in a `functionResponse` part
on a `user` message.

```json
// Anthropic
{ "role": "user", "content": [{ "type": "tool_result", "tool_use_id": "toolu_01X", "content": "A damp stone corridor stretches north. Torches flicker on the walls." }] }

// Ollama
{ "role": "tool", "tool_name": "look", "content": "A damp stone corridor stretches north. Torches flicker on the walls." }

// OpenAI
{ "role": "tool", "tool_call_id": "toolu_01X", "content": "A damp stone corridor stretches north. Torches flicker on the walls." }

// Gemini
{ "role": "user", "parts": [{ "functionResponse": { "name": "toolu_01X", "response": { "content": "A damp stone corridor stretches north. Torches flicker on the walls." } } }] }
```

### Tool Definitions

Anthropic uses `input_schema`. Ollama and OpenAI wrap everything in a
`function` envelope with `parameters`. Gemini wraps tools in a
`functionDeclarations` array.

```json
// Anthropic
{ "name": "move", "description": "Move the player in a direction (north, south, east, west, up, down)", "input_schema": { "type": "object", "properties": { "direction": { "type": "string", "description": "The direction to move" } }, "required": ["direction"] } }

// Ollama / OpenAI
{ "type": "function", "function": { "name": "move", "description": "Move the player in a direction (north, south, east, west, up, down)", "parameters": { "type": "object", "properties": { "direction": { "type": "string", "description": "The direction to move" } }, "required": ["direction"] } } }

// Gemini
{ "functionDeclarations": [ { "name": "move", "description": "Move the player in a direction (north, south, east, west, up, down)", "parameters": { "type": "object", "properties": { "direction": { "type": "string", "description": "The direction to move" } }, "required": ["direction"] } } ] }
```

### Message Roles

Anthropic, Ollama, and OpenAI all use `assistant` for the model's
turn. Gemini calls it `model`.

```json
// Anthropic / Ollama / OpenAI
{ "role": "assistant", "content": "Let me take a look around first." }

// Gemini
{ "role": "model", "parts": [{ "text": "Let me take a look around first." }] }
```

## Considerations

**The conversation is stateless.** The model has no memory between
turns. Every API call includes the entire history from the beginning.
BOUKENSHA is responsible for carrying that state.

**Tool results are user messages on Anthropic.** This feels
counterintuitive — the result came from BOUKENSHA, not the human — but
it reflects how the Anthropic API models the conversation. Ollama,
OpenAI, and Gemini all handle this with dedicated message/part types
instead.

**The agent only sees schemas.** The `description` field on each tool
is the only thing the agent uses to decide which tool to call. The
actual function body never leaves BOUKENSHA.

## Porting notes

- **`Backend` (model catalog) and `PromptBackend<T: Task>` (payload
  serialization) are two separate traits, not one class.** Ruby/Python's
  `Base`/`Anthropic`/etc. mix two genuinely different kinds of behavior:
  (a) model-table lookup and cost/usage accessors, identical in shape
  across all five backends and independent of any `Context`/`Task`, and
  (b) `to_payload`/`headers`/`url`, which depend on a generic `Context<T>`.
  Rust expresses (a) as a plain, non-generic trait with default methods
  (mirroring the `Base` mixin), while (b) is generic over `T: Task`
  because `Context<T>` is. Splitting them lets `PromptBuilder<T>` hold
  `Box<dyn PromptBackend<T>>` (the direct analogue of "whichever backend
  you pass in") while the model-catalog side stays plain associated-
  function lookup that never needs dynamic dispatch.
- **`PromptBuilder.to_messages()`/`to_tools()` are dropped entirely,
  not carried forward as an unreachable rough edge.** Python's own
  porting notes flag that these two zero-arg passthroughs aren't
  uniformly callable across backends already (`Anthropic`/`Gemini` take
  one arg, `OpenAI`/`Ollama`/`OllamaCloud` take two) and only ship
  because neither `example.rb` nor `example.py` ever calls them —
  duck typing lets an unexercised method with a backend-dependent arity
  mismatch sit unnoticed. Rust's trait system can't paper over that:
  putting `to_messages`/`to_tools` on `PromptBackend<T>` would force one
  arity for every implementor, which is false for this codebase, and
  Rust has no duck-typed "call whatever shape happens to be there"
  fallback. This port omits both methods from the public builder API;
  each backend still exposes its own inherent `to_messages`/`to_tools`
  (used internally by its own `to_payload`), so no behavior is lost —
  only the unused, arity-inconsistent passthrough wrapper. This is a
  real, intentional reduction of the public surface versus Ruby/Python,
  even though nothing calls the dropped methods.
- **`Config::PROMPTS_DIR` restored verbatim from `00_config`, not
  reinvented under a new name/shape.** `00_config` already solved "Rust
  has no `__file__`" once; `01_struct_skeleton` deliberately dropped
  `PROMPTS_DIR` (matching Python's/Ruby's own drop) because nothing used
  it until this step reintroduced the concept. The restored constant is
  `concat!(env!("CARGO_MANIFEST_DIR"), "/prompts")`, baking the build
  machine's absolute path into the binary — the same tradeoff
  `00_config`'s README already accepted "for now."
- **`serde_json` is a genuinely new dependency, not a promoted
  transitive one** (unlike `indexmap` in `02_the_registry`). It ships
  with the `preserve_order` feature — not cosmetic: without it,
  `serde_json::Map` iterates in sorted-key order, silently reordering a
  tool's `properties`/`required` keys relative to the YAML mapping's
  insertion order, the same ordering bug `02_the_registry` already fixed
  once for `Context.tools` via `IndexMap`.
- **`Tool.parameters` stays `serde_yaml_ng::Value`; the YAML→JSON
  conversion happens inside each backend's `to_tools`/`schema_parts`,
  not upstream in `Tool` itself.** Tools are authored via YAML literals
  (established in `02_the_registry`), and changing that field's type now
  would be an unrelated, unforced edit to a file this step's own Python
  precedent says needs none. `schema_parts` in `backends/base.rs` does a
  single generic, lossless re-serialization (`serde_json::to_value`) —
  YAML's `Value` and JSON's `Value` model the same scalar/sequence/
  mapping shapes — keeping the YAML→JSON crossing localized to exactly
  where it's needed.
- **The identical `to_tools` body shared by `Ollama`/`OpenAI`/
  `OllamaCloud`, and the near-identical `to_messages` body shared by
  `Ollama`/`OllamaCloud` (differing from `OpenAI` only in one field
  name, `tool_name` vs. `tool_call_id`), are factored into two shared
  helpers in `backends/base.rs`** (`function_wrapped_tools`,
  `chat_style_messages`) rather than tripled verbatim. This is a
  deliberate, optional dedup — not a source-structure deviation — since
  the output stays identical to what three separate copy-pasted methods
  would produce. `Anthropic`/`Gemini`'s `to_messages` are genuinely
  different shapes (tool results as content blocks / `functionResponse`
  parts, no system-message prefix) and stay as separate inherent
  methods, matching the source.
- **No default arguments.** Ruby's `max_output_tokens: 1024` keyword
  default and Python's `max_output_tokens=1024` have no Rust syntax
  equivalent; every call site passes `1024` explicitly
  (`builder.to_api_payload(1024)`).
- **Name collision Python had to dodge (`Base.model_info` classmethod
  vs. instance method) does not apply here.** Rust's associated
  functions (`Self::models()`, `Self::validate_model()`) and instance
  methods (`self.info()`, the accessor methods) already live in
  non-overlapping call syntaxes — `Type::function()` vs. `value.method()`
  — so no rename analogous to Python's `_model_info` is needed.
- **Provider dispatch: `if`/`elif`/`raise ValueError` → `match` +
  `panic!`.** Matches this step's own unhandled-exception semantics —
  neither `example.rb` nor `example.py` catches an unsupported-provider
  error, so an uncaught Rust `panic!` with the same message
  (`"Unsupported provider for player task: {other}"`) is the direct
  equivalent.
- `message.rs`, `tool.rs`, `context.rs`, `registry.rs`,
  `tasks/{base,player}.rs` needed **no edits** — already correct from
  `02_the_registry`. `errors.rs` gains `UnsupportedModelError(String)`
  alongside the existing `UnknownToolError`, wrapping a pre-formatted
  message string rather than structured fields, since the message
  itself (backend name + attempted model + sorted supported list) is
  assembled once at the raise site.

## Run Example

```bash
./week1_baseline/bin/rust/03_prompt_builder
```

Expected output (values from your `.boukensha/`):

```
=== BOUKENSHA Step 3: Prompt Builder ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
{
  "model": "claude-haiku-4-5",
  "system": "You are a MUD Journey player agent.\n\nYou are playing the MUD on behalf of the player, and the player will issue you goals to complete. \n\nUse the tools available to you to help the player explore, fight, and interact with the world.",
  "max_tokens": 1024,
  "tools": [
    {
      "name": "look",
      "description": "Look around the current room for details",
      "input_schema": {
        "type": "object",
        "properties": {},
        "required": []
      }
    },
    {
      "name": "move",
      "description": "Move the player in a direction (north, south, east, west, up, down)",
      "input_schema": {
        "type": "object",
        "properties": {
          "direction": {
            "type": "string",
            "description": "The direction to move"
          }
        },
        "required": [
          "direction"
        ]
      }
    }
  ],
  "messages": [
    {
      "role": "user",
      "content": "I just arrived in the dungeon. What's around me, and can you move north?"
    },
    {
      "role": "assistant",
      "content": "Let me take a look around first."
    },
    {
      "role": "user",
      "content": [
        {
          "type": "tool_result",
          "tool_use_id": "toolu_01X",
          "content": "A damp stone corridor stretches north. Torches flicker on the walls."
        }
      ]
    }
  ]
}
```

The system text above comes from `.boukensha/prompts/player/system.md`
(the fixture's override, since `settings.yaml` sets
`prompt_override.system: true`), not from the library-default
`prompts/system.md`. The JSON structure matches the Ruby/Python output
byte-for-byte — `serde_json::to_string_pretty` on an `IndexMap`-backed
`serde_json::Value` (via the `preserve_order` feature) renders the same
key order and 2-space indentation as `JSON.pretty_generate` and
`json.dumps(..., indent=2)`.
