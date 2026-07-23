# Python Port Plan — 03 Prompt Builder

## Goal

Port the behavior of `week1_baseline/ruby/03_prompt_builder/` to
`week1_baseline/python/03_prompt_builder/`. The directory already
exists but is currently byte-identical to `python/02_the_registry/`
(verified: `diff -rq python/02_the_registry python/03_prompt_builder`
reports no differences at all). This step adds the `PromptBuilder` and
the five per-provider `Backends` classes that serialize a `Context`
into the exact JSON shape each LLM API expects (Anthropic, OpenAI,
Gemini, Ollama, Ollama Cloud) — no API calls are made, only payload
assembly.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff` against `ruby/02_the_registry`:
`context.rb`, `tasks/base.rb`, `tasks/player.rb`, and `Gemfile`/
`Gemfile.lock` are unchanged (the only `context.rb` diff is a trailing
newline). `config.rb` gains one new constant (`PROMPTS_DIR`),
`errors.rb` gains one new exception (`UnsupportedModelError`),
`boukensha.rb` adds requires for the new files, and `example.rb` is
rewritten to register tools directly on `Registry.tool` blocks, seed a
short conversation, build a backend from `settings.yaml`'s configured
provider/model, and print the assembled API payload as JSON. So this
plan only makes new decisions about `PromptBuilder` and `Backends`;
everything already decided in [`02_the_registry.md`](02_the_registry.md)
(and, transitively, `01_struct_skeleton.md`) carries forward unchanged
for the untouched files.

**Verified real output** (below) comes from actually running
`./week1_baseline/bin/ruby/03_prompt_builder` from the repo root,
against the repo's real `.boukensha/` fixture — not from reading the
Ruby README, which does not print a full example transcript for this
step (its own "Run Example" section only shows the invocation command).

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/03_prompt_builder/README.md` | Design spec: PromptBuilder/backend responsibilities, model-table shape, per-provider payload/message/tool/role differences, considerations |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/prompt_builder.rb` | `Boukensha::PromptBuilder` — thin delegator: `to_messages`, `to_tools`, `to_api_payload(max_output_tokens:)`, `headers`, `url`, all forwarding to the backend it wraps |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/base.rb` | `Boukensha::Backends::Base` — shared `MODELS` lookup/validation (`UnsupportedModelError`) and cost/context-window accessors |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/anthropic.rb` | Anthropic `/v1/messages` payload: top-level `system`, `input_schema` tools, `tool_result` wrapped as a `user` message |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/openai.rb` | OpenAI chat-completions payload: `system` folded into `messages`, `function`-wrapped tools, `role: tool` + `tool_call_id` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/gemini.rb` | Gemini `generateContent` payload: `systemInstruction`, `contents` with `model` role and `parts`, `functionDeclarations`, `functionResponse` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/ollama.rb` | Local Ollama `/api/chat` payload: `system` folded into `messages`, `function`-wrapped tools, `role: tool` + `tool_name` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/backends/ollama_cloud.rb` | Same shape as `ollama.rb` but hosted (`https://ollama.com/api/chat`, bearer auth, `nil` token costs) |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/errors.rb` | Adds `Boukensha::UnsupportedModelError < StandardError` alongside existing `UnknownToolError` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/config.rb` | Adds `PROMPTS_DIR` — the library's own bundled `prompts/` dir, used as `default_prompts_dir` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha.rb` | Adds requires for `prompt_builder` and all five `backends/*` files |
| `week1_baseline/ruby/03_prompt_builder/prompts/system.md` | Default system prompt shipped with the library (used when a task has no override, or the override file is missing) |
| `week1_baseline/ruby/03_prompt_builder/examples/example.rb` | Rewritten smoke test: registers `look`+`move` via `Registry.tool` blocks, seeds a 3-message conversation (user → assistant → tool_result), resolves provider/model/backend from `settings.yaml`, builds a `PromptBuilder`, and prints `JSON.pretty_generate(builder.to_api_payload)` |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/{message,tool,context}.rb`, `tasks/{base,player}.rb`, `Gemfile`/`Gemfile.lock` | Unchanged from `02_the_registry` — carry forward as-is (already true of the current Python tree for these files; no changes needed) |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps. Relevant
to this step: `.boukensha/settings.yaml` sets `tasks.player.provider:
anthropic`, `tasks.player.model: claude-haiku-4-5`, and
`tasks.player.prompt_override.system: true`; `.boukensha/.env` sets
(empty) `ANTHROPIC_API_KEY`; `.boukensha/prompts/player/system.md`
supplies the override text actually used in the verified output below
(it does not match the library-default `prompts/system.md` text — that
default is only reached when no override applies, which is not this
step's configured case, but the default-lookup path is still built and
exercised by `default_prompts_dir` resolving correctly). No fixture
changes needed for this step.

## Target files to create/change (Python)

```
week1_baseline/python/03_prompt_builder/
  README.md                              (rewrite: PromptBuilder/backend docs, new-files table, considerations, verified expected output)
  requirements.txt                       (unchanged — same two deps; no HTTP calls happen in this step)
  prompts/system.md                      (new: default system prompt, byte-identical text to Ruby's)
  examples/example.py                    (rewrite: register look+move via Registry.tool decorators, seed conversation, resolve backend, print JSON payload)
  lib/boukensha/__init__.py              (add PromptBuilder, UnsupportedModelError, backends package exports)
  lib/boukensha/config.py                (add PROMPTS_DIR)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/errors.py                (add UnsupportedModelError)
  lib/boukensha/prompt_builder.py        (new)
  lib/boukensha/tasks/__init__.py        (unchanged)
  lib/boukensha/tasks/base.py            (unchanged, already correct — already supports default_prompts_dir)
  lib/boukensha/tasks/player.py          (unchanged, already correct)
  lib/boukensha/backends/__init__.py     (new: exports Base, Anthropic, Gemini, Ollama, OllamaCloud, OpenAI)
  lib/boukensha/backends/base.py         (new)
  lib/boukensha/backends/anthropic.py    (new)
  lib/boukensha/backends/gemini.py       (new)
  lib/boukensha/backends/ollama.py       (new)
  lib/boukensha/backends/ollama_cloud.py (new)
  lib/boukensha/backends/openai.py       (new)
```

Plus a launcher at `week1_baseline/bin/python/03_prompt_builder`,
matching `bin/python/02_the_registry`'s shape (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/03_prompt_builder"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency to install.

## Behavior parity checklist (from the real Ruby output)

- [ ] `UnsupportedModelError` — plain `Exception` subclass, no custom
      fields, added alongside `UnknownToolError` in `errors.py`
- [ ] `Config.PROMPTS_DIR` — class attribute, the library's own
      `prompts/` dir (sibling of `lib/`), computed once from
      `Path(__file__)`, not per-instance
- [ ] `Backends.Base.models()` — classmethod, returns the subclass's
      `MODELS` dict; raises `NotImplementedError` if the subclass
      doesn't define one
- [ ] `Backends.Base.validate_model(model)` — classmethod; returns
      `str(model)` if it's a key in `models()`; otherwise raises
      `UnsupportedModelError` with message `"{ClassName} does not
      support model '{model}'. Supported models: {sorted, comma-joined
      list}"`
- [ ] `Backends.Base` instance surface — `context_window`,
      `input_token_cost_per_million`, `output_token_cost_per_million`,
      `usage_unit`, `usage_level` (may be `None`), and
      `estimate_cost(input_tokens=, output_tokens=)` returning `None`
      when either cost is `None` (Ollama Cloud), otherwise the USD
      estimate; local Ollama models return `0.0`, never `None`
- [ ] `Backends.Anthropic` — `MODELS` table for the 4 listed models;
      `to_messages` wraps `tool_result` role as `{"role": "user",
      "content": [{"type": "tool_result", "tool_use_id":
      ..., "content": ...}]}`; `to_tools` uses `input_schema`;
      `to_payload` includes top-level `system`; `headers` (`@property`)
      includes `x-api-key` and `anthropic-version: 2023-06-01`; `url`
      (`@property`) is `https://api.anthropic.com/v1/messages`
- [ ] `Backends.OpenAI` — `MODELS` table for the 3 listed models;
      `to_messages(system, messages)` prepends a `role: system`
      message, `tool_result` becomes `{"role": "tool",
      "tool_call_id": ..., "content": ...}`; `to_tools` wraps each
      tool in `{"type": "function", "function": {...,
      "parameters": {...}}}`; `to_payload` uses
      `max_completion_tokens`; `headers` (`@property`) includes
      `Authorization: Bearer ...`; `url` (`@property`) is
      `https://api.openai.com/v1/chat/completions`
- [ ] `Backends.Gemini` — `MODELS` table for the 5 listed models;
      `to_messages` maps `assistant` → `role: "model"`, wraps text in
      `parts`, `tool_result` becomes a `functionResponse` part on a
      `user` role; `to_tools` returns `[]` for no tools, otherwise one
      `{"functionDeclarations": [...]}` entry; `to_payload` uses
      `systemInstruction`/`contents`/`generationConfig.maxOutputTokens`;
      `headers` (`@property`) uses `x-goog-api-key`; `url`
      (`@property`) is `{BASE_URL}/{model}:generateContent`
- [ ] `Backends.Ollama` — `MODELS` table for the 9 listed local
      models, all `cost_per_million: {input: 0.0, output: 0.0}`,
      `usage_unit: "local_compute"`; `to_messages(system, messages)`
      prepends `role: system`, `tool_result` becomes `{"role": "tool",
      "tool_name": ..., "content": ...}`; `to_payload` includes
      `stream: False`; no `Authorization` header; `headers`/`url` are
      `@property`; `url` is `{host}/api/chat`, `host` defaults to
      `http://localhost:11434`
- [ ] `Backends.OllamaCloud` — `MODELS` table for the 3 listed cloud
      models, `cost_per_million: {input: None, output: None}`,
      `usage_unit: "ollama_cloud_usage"`, per-model `usage_level`;
      same message/tool shape as `Ollama` but with `Authorization:
      Bearer ...` header and `url` (both `@property`)
      `https://ollama.com/api/chat`
- [ ] `PromptBuilder(context, backend)` — `to_messages()` forwards to
      `backend.to_messages(context.messages)`; `to_tools()` forwards
      to `backend.to_tools(context.tools)`;
      `to_api_payload(max_output_tokens=1024)` forwards to
      `backend.to_payload(context, max_output_tokens=...)`;
      `headers`/`url` are `@property` (accessed as `builder.headers`/
      `builder.url`, no parens), each forwarding to the backend's own
      `headers`/`url` property — see porting notes for a straight
      one-argument passthrough on `to_messages`/`to_tools` that isn't
      universally callable across backends yet (not a concern for this
      step)
- [ ] `examples/example.py` registers `look` (no params) and `move`
      (`direction` param, now with a `description` sub-key) via
      `@registry.tool(...)` decorators, seeds `ctx` with a user →
      assistant → tool_result exchange, resolves
      `provider`/`model` from `Player.provider/model`, builds the
      matching `Backends.*` instance (raising `ValueError` for an
      unsupported provider string), builds a `PromptBuilder`, and
      prints `Config`, `provider`, `model`, then
      `json.dumps(builder.to_api_payload(), indent=2)`

Expected output (verified by actually running
`./week1_baseline/bin/ruby/03_prompt_builder` from the repo root,
against the real `.boukensha/` fixture — the Ruby README doesn't print
a transcript for this step, so this is the ground truth, not a
cross-check against README text):

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
`prompts/system.md` — the Python port must resolve the same override
first (`Player.system_prompt(..., user_prompts_dir=...,
default_prompts_dir=...)` already does this correctly, unchanged from
this step's `tasks/base.py`). The JSON structure itself should match
byte-for-byte: Ruby's `JSON.pretty_generate` on a hash with symbol keys
renders identically to Python's `json.dumps(..., indent=2)` on the
equivalent string-keyed dict — there is no cosmetic symbols-vs-strings
divergence here (unlike the `Tool`/`Context` `repr` strings in prior
steps), because both languages' JSON serializers stringify hash/dict
keys the same way.

## Porting notes (Ruby idiom → Python)

- **`headers`/`url` become `@property`, not zero-arg methods.** Ruby's
  parens-optional calls make `backend.headers`/`backend.url` read like
  attribute access even though they're regular methods; Python has no
  such optional-parens syntax, so the port has to pick one shape
  explicitly. `@property` was chosen over a plain `def headers(self):`
  because `headers`/`url` are noun-named, always-zero-argument state
  accessors — the same category this codebase already ports to
  `@property` elsewhere (`Config.mud_host`, `Context.tool_count`) —
  whereas `to_messages`/`to_tools`/`to_payload` are verb-named
  conversion methods that require positional args on every backend
  (`messages`, `tools`, or `system, messages`), so they can only ever
  be regular methods regardless of what's picked for `headers`/`url`;
  there's no actual "stay consistent with the other methods on this
  class" tension since those other methods aren't zero-arg to begin
  with. `requests.Response.headers`/`.url` in Python's own `requests`
  library follow the identical noun-property shape for the same kind
  of HTTP-request-description data these backends model.
- **Name collision: `Base.model_info` is both a class method (Ruby
  `self.model_info(model)`) and an instance method (Ruby
  `model_info`) with the same name.** Ruby allows this because class
  (singleton) methods and instance methods live in separate namespaces
  — `self.model_info` and `model_info` never collide. Python has no
  such separation: a `@classmethod` and an instance `@property`
  defined under the same name in one class body would just have the
  second definition silently overwrite the first in the class's
  `__dict__`. The port resolves this by renaming the *classmethod*
  lookup to `_model_info(cls, model)` (private-by-convention, takes an
  arg) and keeping the *instance* accessor as the public
  `model_info` property (no arg, returns the instance's resolved
  info dict via a `self._info` backing attribute). `validate_model`
  and `_configure_model` call `cls._model_info(...)` /
  `self.__class__._model_info(...)`; nothing external depends on the
  renamed classmethod, so this is purely an internal Python-only
  disambiguation, not a behavior difference.
- **`PromptBuilder#to_messages`/`#to_tools` stay a plain one-argument
  passthrough**, matching Ruby's `@backend.to_messages(@context.messages)`
  exactly. `OpenAI`, `Ollama`, and `OllamaCloud` each define
  `to_messages(system, messages)` (two arguments) while `Anthropic`/
  `Gemini` define the one-argument form, so calling
  `builder.to_messages` directly isn't uniformly callable across every
  backend yet — but neither `example.rb` nor the new `example.py` ever
  calls it directly (only `to_api_payload`, which each backend's own
  `to_payload` satisfies correctly with the right arity internally).
  Per the user: error handling for cases like this is coming in a
  future step, so this port just carries the current Ruby shape
  forward as-is rather than reworking it now.
- **`tool.parameters.keys.map(&:to_s)` → `list(tool.parameters.keys())`.**
  Ruby tool parameter hashes are built with symbol keys
  (`{direction: {...}}`), so each backend's `to_tools` must
  `.map(&:to_s)` before putting them in a JSON `required` array.
  Python tool parameter dicts are already string-keyed
  (`{"direction": {...}}`), so no conversion step is needed — same
  "Python has no symbol/string duality" reasoning already applied to
  `Registry#dispatch` in the `02_the_registry` port.
- **`Config::PROMPTS_DIR`** — Ruby: `File.expand_path("../../prompts",
  __dir__)` from `lib/boukensha/config.rb`, resolving to
  `03_prompt_builder/prompts`. Python equivalent, computed once at
  class-definition time in `config.py`:
  `PROMPTS_DIR = str(Path(__file__).resolve().parent.parent.parent / "prompts")`
  (`config.py` lives at `lib/boukensha/config.py`; `.parent` → `lib/boukensha`,
  `.parent.parent` → `lib`, `.parent.parent.parent` → `03_prompt_builder`,
  then `/ "prompts"`) — same three-level walk-up Ruby's two `..`
  segments plus `__dir__` perform.
- **`ENV.fetch("ANTHROPIC_API_KEY")` → `os.environ["ANTHROPIC_API_KEY"]`.**
  Both raise if the key is entirely unset; both succeed (with an empty
  string) when the fixture's `.env` declares the key with no value, as
  it does here. Not `os.environ.get(...)` — that would silently pass
  `None` instead of matching Ruby's fail-loud-if-truly-missing
  behavior.
- **Provider dispatch `case`/`when` → `if`/`elif` chain (or dict
  dispatch)** in `example.py`, raising `ValueError` (Python's
  `ArgumentError` analogue) for an unrecognized provider string,
  matching Ruby's `raise ArgumentError, "Unsupported provider for
  player task: #{provider}"`.
- **`registry.tool("look", ...) do ... end` (no block params) →
  `@registry.tool("look", description=..., parameters={})` decorating
  a zero-arg `def look(): ...`** — same decorator-factory pattern
  established in `02_the_registry.md`, just with an empty parameter
  set this time (`parameters={}` rather than omitted, since the Ruby
  source passes `parameters: {}` explicitly here too).
- `message.py`, `tool.py`, `context.py`, `registry.py`,
  `tasks/base.py`, `tasks/player.py` need **no edits** — already
  correct in the current `python/03_prompt_builder/` tree (it's
  presently an exact copy of the already-correct `02_the_registry`
  tree). `tasks/base.py`'s `default_prompts_dir` parameter already
  exists and is unused only because no caller passed it until now —
  wiring `Config.PROMPTS_DIR` through `example.py` is what activates
  it, not a change to `tasks/base.py` itself.

## Decisions

1. **`Backends` as a Python sub-package** (`lib/boukensha/backends/`)
   mirroring Ruby's `Boukensha::Backends` module — one file per
   provider plus `base.py`, exported from a `backends/__init__.py`
   (`Base`, `Anthropic`, `Gemini`, `Ollama`, `OllamaCloud`, `OpenAI`).
   Not flattened into the top-level `boukensha/__init__.py`, to avoid
   dumping generic names like `Base` and `OpenAI` into the package's
   flat namespace — callers reach them via `boukensha.backends.X` or a
   direct submodule import, matching how `example.py` already imports
   `Config`/`Context`/etc. from their submodules rather than the
   top-level package.

2. **`headers`/`url` are `@property` on both `Backends.*` and
   `PromptBuilder`**, not zero-arg methods — they're the only
   always-zero-argument, noun-named accessors on these classes
   (`to_messages`/`to_tools`/`to_payload` all require positional args
   at the backend layer, so there's no "consistency with sibling
   methods" argument for making them methods too). Matches this
   codebase's existing `@property`-for-noun-accessors convention
   (`Config.mud_host`, `Context.tool_count`) and Python's own
   `requests.Response.headers`/`.url` precedent for the same kind of
   data. Confirmed with the user over the alternative (plain zero-arg
   methods) before implementing.

3. **`Base._model_info(cls, model)` classmethod, `model_info` instance
   property, `_info` backing attribute** — resolves the Ruby
   class-method/instance-method name reuse that has no direct Python
   equivalent (see porting notes). Internal-only rename; no observable
   behavior change.

4. **Preserve the `to_messages`/`to_tools` one-argument passthrough in
   `PromptBuilder`** as shipped Ruby behavior, even though it isn't
   uniformly callable across the two-argument `OpenAI`/`Ollama`/
   `OllamaCloud` backends yet — no example code calls it directly, and
   the user confirmed proper error handling for this area is coming in
   a future step, so there's nothing to solve here now.

5. **`errors.py` gains `UnsupportedModelError(Exception): pass`**
   alongside the existing `UnknownToolError`, direct one-for-one port
   of `UnsupportedModelError < StandardError; end`.

6. **`prompts/system.md` copied byte-for-byte** into
   `python/03_prompt_builder/prompts/system.md`, matching Ruby's
   `ruby/03_prompt_builder/prompts/system.md`. `Config.PROMPTS_DIR`
   points at this directory the same way `Config::PROMPTS_DIR` does in
   Ruby.

7. **`requirements.txt` unchanged** — this step never issues an HTTP
   request (`PromptBuilder` only assembles payload data structures),
   so no HTTP client dependency is added, matching Ruby's `Gemfile`
   (still just `dotenv`, no `net/http`-adjacent gem).

8. **`bin/python/03_prompt_builder` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/02_the_registry`'s shape.

## Open questions

None outstanding — all decided above.
