# 03 · The Prompt Builder (Python port)

Behavior port of `ruby/03_prompt_builder` — adds a `PromptBuilder` and
five per-provider `Backends` classes that serialize a `Context` into
the exact JSON shape each LLM API expects (Anthropic, OpenAI, Gemini,
Ollama, Ollama Cloud). No API calls are made here, only payload
assembly. `message.py`, `tool.py`, `context.py`, `registry.py`, and
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
| `lib/boukensha/prompt_builder.py` | Delegates serialization to the active backend |
| `prompts/system.md` | Default system prompt used when a task does not override it |
| `lib/boukensha/backends/base.py` | Shared backend contract for model validation and model metadata |
| `lib/boukensha/backends/anthropic.py` | Serializes context into the Anthropic API format |
| `lib/boukensha/backends/ollama.py` | Serializes context into the Ollama API format |
| `lib/boukensha/backends/ollama_cloud.py` | Serializes context into the Ollama Cloud API format |
| `lib/boukensha/backends/openai.py` | Serializes context into the OpenAI Chat Completions format |
| `lib/boukensha/backends/gemini.py` | Serializes context into the Gemini `generateContent` format |

## How It Works

```
Context (Python objects)
        ↓
PromptBuilder
        ↓
Backend (Anthropic, OpenAI, Gemini, Ollama, or OllamaCloud)
        ↓
API Payload (plain dicts and lists)
        ↓
POST to API
```

## boukensha.prompt_builder.PromptBuilder

| Method | Description |
|---|---|
| `to_messages()` | Delegates message serialization to the backend |
| `to_tools()` | Delegates tool serialization to the backend |
| `to_api_payload(max_output_tokens=1024)` | Assembles the complete payload ready to POST |
| `headers` (property) | Returns the correct headers for the backend |
| `url` (property) | Returns the correct endpoint URL for the backend |

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

Backend instances expose `context_window`,
`input_token_cost_per_million`, `output_token_cost_per_million`,
`usage_unit`, `usage_level`, and
`estimate_cost(input_tokens=, output_tokens=)`. For local Ollama
models, token API cost is `0.0`. For Ollama Cloud, public pricing is
plan/usage based rather than token based, so `estimate_cost` returns
`None`.

The prices in this step are static tutorial data, current as of June
16, 2026, and should be reviewed whenever the selected model set
changes.

### boukensha.backends.Anthropic

Talks to `https://api.anthropic.com/v1/messages`. Requires an
`ANTHROPIC_API_KEY`. Supported models are listed in
`Anthropic.MODELS`.

### boukensha.backends.Ollama

Talks to `http://localhost:11434/api/chat`. Requires `ollama serve`
running locally. No API key needed. Supported models are listed in
`Ollama.MODELS`.

### boukensha.backends.OllamaCloud

Talks to `https://ollama.com/api/chat`. Requires an `OLLAMA_API_KEY`.
Supported models are listed in `OllamaCloud.MODELS`.

### boukensha.backends.OpenAI

Talks to `https://api.openai.com/v1/chat/completions`. Requires an
`OPENAI_API_KEY`. Supported models are listed in `OpenAI.MODELS`.

### boukensha.backends.Gemini

Talks to `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`.
Requires a `GEMINI_API_KEY`. Supported models are listed in
`Gemini.MODELS`.

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

- **`headers`/`url` become `@property`, not zero-arg methods.** Ruby's
  parens-optional calls make `backend.headers`/`backend.url` read like
  attribute access even though they're regular methods; Python has no
  such optional-parens syntax, so the port picks `@property`
  explicitly — matching this codebase's existing convention for
  always-zero-argument, noun-named accessors (`Config.mud_host`,
  `Context.tool_count`) and Python's own
  `requests.Response.headers`/`.url` for the same kind of data.
- **Name collision: `Base.model_info` is both a class method and an
  instance method in Ruby.** Ruby's class (singleton) methods and
  instance methods live in separate namespaces, so `self.model_info`
  and `model_info` never collide there. Python has no such separation,
  so the port renames the classmethod lookup to `_model_info(cls,
  model)` and keeps the public instance accessor as `model_info`
  (a `@property` backed by `self._info`). Purely an internal
  disambiguation — not an observable behavior difference.
- **`PromptBuilder.to_messages()`/`to_tools()` stay a plain
  zero-argument passthrough**, matching Ruby's
  `@backend.to_messages(@context.messages)` exactly. `OpenAI`,
  `Ollama`, and `OllamaCloud` each define `to_messages(system,
  messages)` (two arguments) while `Anthropic`/`Gemini` define the
  one-argument form, so `builder.to_messages()` isn't uniformly
  callable across every backend yet — but neither `example.rb` nor
  `example.py` ever calls it directly (only `to_api_payload()`, which
  each backend's own `to_payload` satisfies correctly with the right
  arity internally). Error handling for cases like this is coming in a
  future step, so this port just carries the current Ruby shape
  forward as-is.
- **`tool.parameters.keys.map(&:to_s)` → `list(tool.parameters.keys())`.**
  Ruby tool parameter hashes are built with symbol keys, so each
  backend's `to_tools` must stringify keys before putting them in a
  JSON `required` array. Python tool parameter dicts are already
  string-keyed, so no conversion step is needed — same reasoning
  already applied to `Registry.dispatch` in the `02_the_registry`
  port.
- **`Config.PROMPTS_DIR`** — computed once at class-definition time:
  `str(Path(__file__).resolve().parent.parent.parent / "prompts")`
  (`config.py` lives at `lib/boukensha/config.py`, so three `.parent`
  hops land on `03_prompt_builder/`), matching Ruby's
  `File.expand_path("../../prompts", __dir__)`.
- **`ENV.fetch("ANTHROPIC_API_KEY")` → `os.environ["ANTHROPIC_API_KEY"]`.**
  Both raise if the key is entirely unset; both succeed (with an empty
  string) when the fixture's `.env` declares the key with no value, as
  it does here. Not `os.environ.get(...)` — that would silently pass
  `None` instead of matching Ruby's fail-loud-if-truly-missing
  behavior.
- **Provider dispatch `case`/`when` → `if`/`elif` chain**, raising
  `ValueError` for an unrecognized provider string, matching Ruby's
  `raise ArgumentError, "Unsupported provider for player task: ..."`.
- `message.py`, `tool.py`, `context.py`, `registry.py`,
  `tasks/base.py`, `tasks/player.py` needed **no edits** — already
  correct from `02_the_registry`. `tasks/base.py`'s
  `default_prompts_dir` parameter already existed and was unused only
  because no caller passed it until now; wiring `Config.PROMPTS_DIR`
  through `example.py` is what activates it.

## Run Example

```bash
./week1_baseline/bin/python/03_prompt_builder
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
`prompts/system.md`. The JSON structure matches the Ruby output
byte-for-byte — `JSON.pretty_generate` on a hash with symbol keys and
`json.dumps(..., indent=2)` on the equivalent string-keyed dict render
identically, since both languages' JSON serializers stringify
hash/dict keys the same way.
