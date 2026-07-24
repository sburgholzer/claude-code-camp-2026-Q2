# Python Port Plan — 05 Agent Loop

## Goal

Port the behavior of `week1_baseline/ruby/05_agent_loop/` to
`week1_baseline/python/05_agent_loop/`. The directory already existed
but was byte-identical to `python/04_api_client/` (verified: `diff -rq
python/04_api_client python/05_agent_loop` reported no differences).
This step adds `Boukensha::Agent` — the tool-call loop that ties
`Client`, `PromptBuilder`, and `Registry` together: send a request,
check `stop_reason`, dispatch any `tool_use` blocks, loop until the
model signals `end_turn`, with an iteration ceiling and a tools-off
wind-down call as the safety valve.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff -rq ruby/04_api_client ruby/05_agent_loop`:
the new file is `lib/boukensha/agent.rb`; changed files are
`lib/boukensha.rb` (require), `lib/boukensha/client.rb` (`tools:`
kwarg threaded through), `lib/boukensha/config.rb` (Ruby-only endless
method syntax, no behavior change), `lib/boukensha/errors.rb` (adds
`LoopError`, unused), `lib/boukensha/prompt_builder.rb` (adds
`parse_response`), `lib/boukensha/tasks/base.rb` (adds
`max_iterations`/`max_output_tokens`), and all five
`lib/boukensha/backends/*.rb` (`tools:` kwarg + `parse_response` +,
for every backend except Anthropic, a private `assistant_message`/
`assistant_parts` inverse). `README.md` and `examples/example.rb` are
rewritten. Everything else — `message.rb`, `tool.rb`, `context.rb`,
`registry.rb`, `tasks/player.rb`, `Gemfile`/`Gemfile.lock` — is
byte-identical to `04_api_client`, carried forward unchanged.

**Ruby README table overstates the diff, same pattern as `04_api_client.md`.**
`ruby/05_agent_loop/README.md`'s "New Files" table lists
`backends/base.rb`, `tasks/base.rb`, `tasks/player.rb`,
`backends/openai.rb`, `backends/gemini.rb`, `backends/ollama_cloud.rb`,
and `prompts/system.md` as new — but `diff -rq ruby/04_api_client
ruby/05_agent_loop` shows all of those already existed byte-identical
in `04_api_client` (they were introduced in an earlier baseline step,
before this port's step-by-step history begins). Only `agent.rb` is
genuinely new in this diff. This plan follows the verified diff, not
the README table.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/05_agent_loop/README.md` | Design spec: the loop shape, the normalized `parse_response` contract, task config (`max_iterations`/`max_output_tokens`), considerations (assistant-before-tool_result ordering, multi-tool-call turns, iteration ceiling, no self-stop) |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/agent.rb` | **New.** `Boukensha::Agent` — the loop itself: `run`, iteration ceiling + wind-down (`wrap_up`), tool dispatch (`handle_tool_calls`), text extraction |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/client.rb` | `call` gains a `tools:` kwarg, threaded into `to_api_payload` |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/prompt_builder.rb` | `to_api_payload` gains `tools:`; adds `parse_response(response)` delegating to the backend |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/errors.rb` | Adds `LoopError < StandardError` (never actually raised anywhere in this step — the loop uses `wrap_up`, not an exception, as its ceiling behavior) |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/tasks/base.rb` | Adds `DEFAULT_MAX_ITERATIONS`/`DEFAULT_MAX_OUTPUT_TOKENS` constants and `max_iterations`/`max_output_tokens` class methods backed by a private `integer_setting` helper |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/config.rb` | Rewrites `mud_host`/`mud_port`/`mud_username`/`mud_password` as Ruby endless methods (`def mud_host = ...`) — purely a Ruby syntax change, zero behavior difference; Python's `config.py` already uses `@property` for these, so **no Python edit needed** (see Decisions) |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/backends/anthropic.rb` | `to_payload` gains `tools:` (uses caller-supplied tools when not `nil`, else `to_tools(context.tools)`); adds `parse_response`. No `assistant_message` needed — Anthropic's `content` array already doubles as wire format |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/backends/openai.rb` | Same `tools:` threading; adds `parse_response` (reads `choices[0].message`, converts `tool_calls` + JSON-parses `arguments`) and a private `assistant_message` inverse (used from `to_messages` for `:assistant` role) |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/backends/gemini.rb` | Same `tools:` threading; adds `parse_response` (reads `candidates[0].content.parts`, `functionCall`/`text` parts) and a private `assistant_parts` inverse (used from `to_messages` for `:assistant` role) |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/backends/ollama.rb` | Same `tools:` threading; adds `parse_response` (reads `message.tool_calls`, reuses function `name` as call `id`) and a private `assistant_message` inverse |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/backends/ollama_cloud.rb` | Same as `ollama.rb` |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha.rb` | Adds `require_relative "boukensha/agent"` |
| `week1_baseline/ruby/05_agent_loop/examples/example.rb` | Rewritten: builds `Agent` with `task_settings: player_settings`, registers `read_file`/`list_directory`, seeds one user message asking to read the README and summarize, prints config/provider/model/max_iterations/max_output_tokens, then `agent.run`'s final text |
| `week1_baseline/ruby/05_agent_loop/lib/boukensha/{message,tool,context,registry}.rb`, `tasks/player.rb`, `backends/base.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `04_api_client` (verified via `diff -rq`) — carry forward as-is |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps.
`settings.yaml`'s `tasks.player` sets `provider: anthropic`, `model:
claude-haiku-4-5`, `prompt_override.system: true`, and no
`max_iterations`/`max_output_tokens` overrides, so both fall back to
the task-class defaults (25, 1024) — `.boukensha/.env` has a real
`ANTHROPIC_API_KEY`. No fixture changes needed.

## Target files to create/change (Python)

```
week1_baseline/python/05_agent_loop/
  README.md                              (rewrite: Agent loop docs, normalized-shape contract, task config, considerations, porting notes, run example)
  requirements.txt                       (unchanged — no new dependency; json/urllib are stdlib)
  prompts/system.md                      (unchanged, already correct)
  examples/example.py                    (rewrite: register read_file+list_directory, seed one user message about README.md, build Agent with task_settings, print config/provider/model/max_iterations/max_output_tokens, then agent.run()'s result)
  lib/boukensha/__init__.py              (add Agent, LoopError exports)
  lib/boukensha/config.py                (unchanged — mud_* already @property; see Decisions)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/client.py                (edit: call() gains tools=None, threaded into to_api_payload)
  lib/boukensha/prompt_builder.py        (edit: to_api_payload gains tools=None; add parse_response(response))
  lib/boukensha/errors.py                (add LoopError)
  lib/boukensha/agent.py                 (new)
  lib/boukensha/tasks/__init__.py        (unchanged)
  lib/boukensha/tasks/base.py            (edit: add DEFAULT_MAX_ITERATIONS/DEFAULT_MAX_OUTPUT_TOKENS, max_iterations/max_output_tokens classmethods, _integer_setting helper)
  lib/boukensha/tasks/player.py          (unchanged, already correct)
  lib/boukensha/backends/base.py         (unchanged, already correct)
  lib/boukensha/backends/anthropic.py    (edit: to_payload gains tools=None; add parse_response)
  lib/boukensha/backends/openai.py       (edit: tools=None threading; add parse_response, _assistant_message; to_messages handles "assistant" role; add `import json`)
  lib/boukensha/backends/gemini.py       (edit: tools=None threading; add parse_response, _assistant_parts; to_messages handles "assistant" role)
  lib/boukensha/backends/ollama.py       (edit: tools=None threading; add parse_response, _assistant_message; to_messages handles "assistant" role)
  lib/boukensha/backends/ollama_cloud.py (edit: same as ollama.py)
```

Plus a launcher at `week1_baseline/bin/python/05_agent_loop`, matching
`bin/python/04_api_client`'s shape (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/05_agent_loop"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency (see Decisions).

## Behavior parity checklist (from the real Ruby output)

- [x] `Client.call(max_output_tokens=1024, tools=None)` — `tools=None`
      means "let the backend build tools from `context.tools`";
      `tools=[]` (or any concrete list) is passed through verbatim,
      which is how the wind-down call disables tools
- [x] `PromptBuilder.to_api_payload(max_output_tokens=1024, tools=None)`
      forwards both to the backend; `PromptBuilder.parse_response(response)`
      delegates to `backend.parse_response(response)`
- [x] Every backend's `to_payload` uses `tools` verbatim when it is
      not `None`, else falls back to `self.to_tools(context.tools)`
- [x] Every backend's `parse_response` returns
      `{"stop_reason": "tool_use" | "end_turn", "content": [...]}`
      using the normalized block shape
      (`{"type": "text", "text": ...}` /
      `{"type": "tool_use", "id": ..., "name": ..., "input": ...}`)
- [x] Non-Anthropic backends rebuild a provider-specific assistant
      message from normalized content via `_assistant_message`
      (OpenAI/Ollama/OllamaCloud) or `_assistant_parts` (Gemini) —
      Anthropic needs no inverse, its `content` array already is the
      wire format
- [x] Ollama/OllamaCloud/Gemini reuse the tool's `name` as its `id` in
      both directions (no call-id concept upstream); Anthropic/OpenAI
      use the provider's real call id
- [x] `Tasks::Base.max_iterations(settings)` / `.max_output_tokens(settings)`
      read `max_iterations`/`max_output_tokens` from settings via
      `_integer_setting`, defaulting to 25 / 1024 respectively when
      absent or `None`
- [x] `Agent(context=, registry=, builder=, client=, task_settings=None, max_iterations=None, max_output_tokens=None)`
      — explicit `max_iterations`/`max_output_tokens` win; otherwise
      `task_settings` + `context.task.max_iterations/max_output_tokens`
      is used when available; otherwise `Agent.MAX_ITERATIONS` (25) /
      `None`
- [x] `Agent.run()` loops: return `_wrap_up("max_iterations")` once
      the iteration ceiling is reached (checked *before* incrementing,
      as a trigger threshold not a hard cap); otherwise increment,
      print `[iteration {n}/{max}]`, call the client, parse the
      response; on `stop_reason == "tool_use"` dispatch tool calls and
      loop; on `end_turn` return the extracted text
- [x] `_handle_tool_calls` stores the assistant message (full
      normalized content, including tool_use blocks) **before**
      dispatching any tool and appending its `tool_result` message —
      ordering matters for Anthropic's API
- [x] A single response can carry multiple `tool_use` blocks; all are
      dispatched in one iteration before the next API call
- [x] `_wrap_up`: appends the wind-down directive as a `user` message,
      calls the client with `tools=[]` (tools hard-disabled) and
      `max_output_tokens=WRAP_UP_OUTPUT_TOKENS` (400), runs *outside*
      the counted loop (never re-checks the ceiling, never increments
      `iteration`); returns the extracted text, or a deterministic
      fallback message if the text is blank or the call raises
      `ApiError`
- [x] `examples/example.py` registers `read_file`/`list_directory`,
      seeds one user message ("Read the README.md file and summarise
      what this MUD player assistant framework can do."), resolves
      provider/model/backend from `Player.provider/model`, builds
      `Agent(..., task_settings=player_settings)`, prints
      `Config`/`Provider`/`Model`/`Max iterations`/`Max output
      tokens`, then `agent.run()`'s final text under `=== FINAL
      RESPONSE ===`

Expected output (verified by actually running
`./week1_baseline/bin/ruby/05_agent_loop` from the repo root against
the real `.boukensha/` fixture — this step makes real, billed
Anthropic API calls, confirmed with the user before running):

```
=== BOUKENSHA Step 5: Agent Loop ===

Config: #<Boukensha::Config dir=/Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Max iterations: 25
Max output tokens: 1024

[iteration 1/25]
  tool call → read_file({"path" => "README.md"})
  tool result → # The Agent Loop

The Agent Loop is the heart of BOUKENSHA. E
[iteration 2/25]

=== FINAL RESPONSE ===
## Summary

**BOUKENSHA** is a MUD player assistant framework that uses an **agent loop** to automate gameplay. Here's what it can do:

### Core Functionality:
- **Agent Loop**: Orchestrates interactions between an AI model and the MUD by:
  - Sending messages to an LLM API
  - Receiving tool calls (like `list_directory`, `read_file`)
  - Dispatching those tool calls and injecting results back into the conversation
  - Repeating until the model signals it's done

### Multi-Provider Support:
The framework supports **5 different LLM backends**:
- **Anthropic** (Claude models)
- **OpenAI** (GPT models)
- **Google Gemini**
- **Ollama** (local or cloud-hosted)

Each backend has different response formats, but BOUKENSHA normalizes them into a single unified shape, keeping the agent loop simple and provider-agnostic.

### Key Features:
- **Task Configuration**: Flexible YAML-based configuration specifying which provider/model to use, system prompts, iteration limits, and output token limits
- **Tool Dispatch**: Automatically extracts tool calls from model responses and executes them (like reading files, listing directories)
- **Safety Limits**: Built-in `MAX_ITERATIONS` to prevent runaway agents
- **Message History**: Properly maintains conversation history including tool results for subsequent API calls

### Result:
The agent can autonomously explore the MUD world, read files, execute commands, and respond to player goals without requiring manual interaction between each step.
```

Note: the model only called `read_file` once and then finished at
iteration 2 with `end_turn` — no `list_directory` call, no
`  tool call →`/`  tool result →` lines under `[iteration 2/25]`. This
is real, non-reproducible model behavior (a live model choosing not to
call a second tool), not a bug; Python's own run is expected to differ
in exactly this same live-model-dependent way (see Verification
section once run).

## Porting notes (Ruby idiom → Python)

- **`tools: nil` sentinel → `tools=None` sentinel.** Every backend's
  `to_payload` uses `tools.nil? ? to_tools(context.tools) : tools` —
  Ruby's `nil` and Python's `None` both mean "not supplied, compute
  the default"; a supplied empty array/list (`[]`) is falsy in neither
  language's `nil?`/`is None` check, so `tools=[]` from the wind-down
  call correctly passes through as "explicitly no tools" rather than
  triggering the default. Direct one-line port, no ambiguity.
- **`Agent`'s `private` section → leading-underscore methods**, same
  convention already established for `backends/base.rb`'s `private`
  (`_configure_model`, `_model_info` in `base.py`) and
  `tasks/base.rb`'s `private` (`_fetch`, `_read_user_prompt`, etc. in
  `tasks/base.py`). Applied to `_resolve_max_iterations`,
  `_resolve_max_output_tokens`, `_iteration_limit_reached`,
  `_call_opts`, `_wrap_up`, `_fallback_message`, `_extract_text`,
  `_handle_tool_calls`. Only `run` stays public, matching Ruby's only
  public instance method.
- **Backends' `private` `assistant_message`/`assistant_parts` → same
  leading-underscore convention**: `_assistant_message` (OpenAI,
  Ollama, OllamaCloud) and `_assistant_parts` (Gemini).
- **`iteration_limit_reached?` → plain method, not `@property`.**
  Earlier steps established `@property` for noun-named zero-arg
  accessors (e.g. `Context.tool_count`/`turn_count`,
  `backends/base.py`'s `context_window`/`usage_unit`). This is a
  predicate over mutable internal state computed fresh each call, not
  a noun-ish accessor, so it stays a regular method
  (`self._iteration_limit_reached()`), consistent with the other
  `_resolve_*`/`_call_opts`/`_wrap_up` helper methods around it that
  are all plain methods too.
- **Ruby ivars (`@max_iterations`, `@iteration`, etc.) → plain
  (non-underscored) Python instance attributes**, matching the
  existing codebase convention (`Client.builder`, `PromptBuilder.context`/
  `.backend`, etc. are all unprefixed) — Ruby ivars have no explicit
  reader and are implicitly private to the class already; the
  leading-underscore convention in this codebase is reserved for
  methods that were explicitly under Ruby's `private` keyword, not for
  ivars.
- **`Integer(value)` → `int(value)`** in `_integer_setting`. Direct
  equivalent; both raise on a non-numeric string, which is the desired
  behavior (a malformed `max_iterations:` setting should fail loudly,
  not silently coerce to 0).
- **`respond_to?(:max_iterations)` → `hasattr(self.context.task,
  "max_iterations")`.** Direct port. In practice this is always `True`
  post-port, since `Tasks::Base`/`tasks/base.py` now unconditionally
  defines both methods for every task subclass — same as in Ruby,
  where the check is a leftover duck-typing guard rather than a real
  branch point. Kept for literal fidelity rather than simplified away,
  since simplifying it would be an unrequested behavior/structure
  change to a straightforward one-line port.
- **Heredoc `WRAP_UP_DIRECTIVE` → a plain triple-quoted (or
  concatenated) Python string, written out verbatim rather than
  computed with `textwrap.dedent`.** Ruby's `<<~MSG.strip` squiggly
  heredoc dedents each line and then `.strip()` removes the trailing
  newline; since the three source lines have no meaningful leading
  whitespace to strip once typed as a literal, the Python string is
  just the three lines joined by `\n` with no trailing newline —
  textually identical output, no need for a dedent call to reproduce a
  transformation that has no effect on this particular literal.
- **`result.to_s[0..60]` → `str(result)[:61]`.** Same `[0..N]` →
  `[:N+1]` index-range translation already used in this codebase
  (`message.py`'s `content[:61]`, `tool.py`'s `description[:41]`).
- **OpenAI's `require "json"` → Python's `import json` at the top of
  `backends/openai.py`.** `JSON.parse`/`.to_json` in Ruby needs an
  explicit `require`; Python's `json` module needs an explicit
  `import` in whichever file calls `json.loads`/`json.dumps`, same
  requirement, different keyword. Not a new dependency — stdlib both
  sides.
- **Ruby hash `#to_s` repr vs Python dict `repr()` in the `tool call →`
  print line.** Ruby prints tool-call args as `{"path" => "README.md"}`;
  Python's `str(dict)` prints `{'path': 'README.md'}` (single quotes,
  colon instead of `=>`). Same category of already-accepted, purely
  cosmetic difference as prior steps' symbol-repr gaps (`:direction` vs
  `'direction'`) — not fixed, not hidden, called out here and in the
  Verification/README notes.
- **`config.rb`'s endless-method rewrite needs no Python change.**
  Ruby went from `def mud_host; ...; end` to `def mud_host = ...` —
  pure syntax, zero behavior change, and Python's `config.py` already
  uses `@property def mud_host(self): return ...` from an earlier
  step, which is the Pythonic equivalent of *both* forms. Nothing to
  port here.
- `message.py`, `tool.py`, `context.py`, `registry.py`,
  `tasks/player.py`, `backends/base.py` need **no edits** — already
  correct in the current `python/05_agent_loop/` tree (verified
  identical to `04_api_client`'s versions, matching the Ruby side
  being byte-identical too).

## Decisions

1. **Ran the real Ruby step for a verified transcript, confirmed with
   the user first** — same precedent as `04_api_client.md`: this is
   an agentic loop that can make multiple real, billed Anthropic API
   calls per run, so an explicit go-ahead was required (and given)
   before executing `bin/ruby/05_agent_loop`.

2. **`Config`/`config.py` gets no edit.** Ruby's `config.rb` diff for
   this step is purely `def x; ...; end` → `def x = ...` endless-method
   syntax for the four `mud_*` accessors — no behavior change. Python's
   `config.py` already expresses these as `@property` methods from an
   earlier port step, which already is the idiomatic Python shape for
   both Ruby forms. Porting "nothing changed behaviorally" produces no
   diff.

3. **`Agent`/`LoopError` join the flat `boukensha` namespace** in
   `__init__.py`, following `04_api_client.md`'s Decision 3 precedent
   (`Client`/`ApiError` went flat because neither name is generic
   enough to crowd the namespace, unlike `backends/*`). `LoopError` is
   added even though nothing in this step raises it — Ruby's own
   `errors.rb` adds it unused too (a forward-declared error for a
   future runaway-agent-detection step), so the port mirrors that
   exactly rather than second-guessing an unused-but-intentional
   addition.

4. **`iteration_limit_reached?` ports to a plain method, not
   `@property`.** See Porting Notes — it's a predicate over mutable
   state recomputed each call, not a noun accessor, so it doesn't fit
   the `@property` pattern established for things like
   `Context.tool_count`.

5. **No new dependency.** `agent.py` and every backend edit use only
   already-imported stdlib (`json`, already used elsewhere in
   `backends/openai.py` after this step's edit). Matches Ruby's
   `Gemfile`/`Gemfile.lock`, which are unchanged in this step (`diff
   -rq` confirms no gem was added), and this project's stdlib-first
   default (`ITERATIONS.md`).

6. **`bin/python/05_agent_loop` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/04_api_client`'s shape.

## Open questions

None outstanding — all decided above.
