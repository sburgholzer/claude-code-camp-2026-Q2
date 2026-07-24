# Python Port Plan — 06 The Logger

## Goal

Port the behavior of `week1_baseline/ruby/06_the_logger/` to
`week1_baseline/python/06_the_logger/`. The directory already existed
but was byte-identical to `python/05_agent_loop/` (verified: `diff -rq
python/05_agent_loop python/06_the_logger` reported no differences).
This step adds `Boukensha::Logger` — a file logger that writes one
structured JSON Lines event per phase of an agent turn
(`session_start`, `iteration`, `prompt`, `tool_call`, `tool_result`,
`response`, `raw`, `turn_end`) to `.boukensha/sessions/<session-id>.jsonl`.
It replaces the `puts` progress lines in `agent.rb` — as of this step
the agent prints **nothing** about iterations/tool calls to stdout;
that output moved entirely into the log file (confirmed by the real
run in the Expected Output section below).

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff -rq ruby/05_agent_loop ruby/06_the_logger`:
the new file is `lib/boukensha/logger.rb`; changed files are
`lib/boukensha.rb` (module-level `quiet!`/`loud!`/`quiet?`/`debug!`/
`debug?`/`config` state, `require_relative "boukensha/logger"` and
`"boukensha/backends/base"`), `lib/boukensha/agent.rb` (`logger:`
kwarg threaded through every phase, tool-call error handling wrapped
in `begin/rescue`), `lib/boukensha/config.rb` (removes the four
`mud_*` endless methods — dead code, unrelated to logging),
`lib/boukensha/context.rb` (whitespace-only ivar alignment, zero
behavior change), `lib/boukensha/errors.rb` (removes unused
`LoopError`), `lib/boukensha/prompt_builder.rb` (adds
`attr_reader :backend`, already implicitly public in Python).
`README.md` and `examples/example.rb` are rewritten. Everything else —
`message.rb`, `tool.rb`, `registry.rb`, `tasks/{base,player}.rb`,
`backends/*.rb`, `client.rb`, `Gemfile`/`Gemfile.lock` — is
byte-identical to `05_agent_loop`, carried forward unchanged.

**Ruby README table (Logger API section) doesn't match the actual
`logger.rb` method signatures**, same pattern as `02_the_registry.md`
and `05_agent_loop.md`. The README documents `iteration(n:)`,
`prompt(messages:, tools:, budget:)`, `tool_result(name:, result:)`,
and `response(text:, usage:, task:, backend:)` — but the real
`logger.rb` has `iteration(n:, max:)` (no `budget` param exists
anywhere), `tool_result(name:, result:, ok: true, error: nil)`, and
`response(text:, usage: nil, stop_reason: nil, task: nil, backend:
nil)`. This plan follows the verified `logger.rb` source and the real
JSONL output (captured in Expected Output), not the README table.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/06_the_logger/lib/boukensha/logger.rb` | **New.** `Boukensha::Logger` — one method per phase, each calling a private `write_log` that merges `session_id`/`at` and appends a JSON line; private helpers derive task/provider/model/usage/cost metadata for `response` events |
| `week1_baseline/ruby/06_the_logger/lib/boukensha.rb` | Adds module-level `Boukensha.quiet!`/`.loud!`/`.quiet?`/`.debug!`/`.debug?`/`.config` (memoized `Config.new`) *before* the `require_relative` chain, plus `require_relative "boukensha/logger"` and `"boukensha/backends/base"` (the latter already loaded transitively before, now required explicitly — no Python effect, see Decisions) |
| `week1_baseline/ruby/06_the_logger/lib/boukensha/agent.rb` | `initialize` gains `logger: Logger.new` kwarg (Ruby evaluates this default *fresh per call* — see Porting Notes); `run` calls `@logger.limit_reached`/`.iteration`/`.prompt`/`.raw` at the matching points; `wrap_up` and the `stop_reason != "tool_use"` branch call a new `log_response`/`@logger.turn_end`; `handle_tool_calls` logs the reasoning text via `log_response`, calls `@logger.tool_call` before dispatch and `@logger.tool_result` after (success or `rescue StandardError`), and gains two private helpers: `log_response` (assembles `@logger.response(...)`) and `normalized_usage` (extracts a usage hash from Anthropic/Gemini/Ollama-shaped raw responses) |
| `week1_baseline/ruby/06_the_logger/lib/boukensha/errors.rb` | Removes unused `LoopError` |
| `week1_baseline/ruby/06_the_logger/lib/boukensha/config.rb` | Removes the four `mud_host`/`mud_port`/`mud_username`/`mud_password` endless methods (dead code cleanup, unrelated to this step's actual feature) |
| `week1_baseline/ruby/06_the_logger/examples/example.rb` | Rewritten: builds `logger = Boukensha::Logger.new`, threads it into `Agent.new(..., logger: logger)`, updates the banner to "Step 6: The Logger" |
| `week1_baseline/ruby/06_the_logger/lib/boukensha/{context,prompt_builder}.rb` | Whitespace/`attr_reader` only — no behavior change, no Python edit needed (see Decisions) |
| `week1_baseline/ruby/06_the_logger/lib/boukensha/{message,tool,registry}.rb`, `tasks/{base,player}.rb`, `backends/*.rb`, `client.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `05_agent_loop` (verified via `diff -rq`) — carry forward as-is |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps — no
changes to `settings.yaml`/`.env`/`prompts/`. This step additionally
writes to `.boukensha/sessions/<session-id>.jsonl` at runtime (already
a tracked path in the fixture — `.boukensha/sessions/` holds a
previously-committed session log from the user's own exploratory
Ruby run). No fixture changes needed; the Python run will add its own
new session file alongside it.

## Target files to create/change (Python)

```
week1_baseline/python/06_the_logger/
  README.md                              (rewrite: Logger docs, JSONL event shapes, task config, debug events, run example — following logger.py's real method signatures, not the Ruby README's drifted table)
  requirements.txt                       (unchanged — no new dependency; json/secrets/datetime are stdlib)
  prompts/system.md                      (unchanged, already correct)
  examples/example.py                    (rewrite: build logger = Logger(), thread into Agent(..., logger=logger), banner → "Step 6: The Logger")
  lib/boukensha/__init__.py              (edit: add module-level _quiet/_debug/_config state + quiet()/loud()/is_quiet()/debug()/is_debug()/config() functions before the class imports; add Logger export; drop LoopError export)
  lib/boukensha/config.py                (edit: remove mud_host/mud_port/mud_username/mud_password properties)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/client.py                (unchanged, already correct)
  lib/boukensha/prompt_builder.py        (unchanged — `self.backend` is already a plain public attribute, the Python equivalent of Ruby's new `attr_reader :backend`)
  lib/boukensha/errors.py                (edit: remove LoopError)
  lib/boukensha/logger.py                (new)
  lib/boukensha/agent.py                 (edit: __init__ gains logger=None → self.logger = logger if logger is not None else Logger(); run()/_wrap_up()/_handle_tool_calls() call the matching logger methods; add _log_response/_normalized_usage helpers)
  lib/boukensha/tasks/__init__.py        (unchanged)
  lib/boukensha/tasks/base.py            (unchanged, already correct)
  lib/boukensha/tasks/player.py          (unchanged, already correct)
  lib/boukensha/backends/base.py         (unchanged, already correct)
  lib/boukensha/backends/anthropic.py    (unchanged, already correct)
  lib/boukensha/backends/openai.py       (unchanged, already correct)
  lib/boukensha/backends/gemini.py       (unchanged, already correct)
  lib/boukensha/backends/ollama.py       (unchanged, already correct)
  lib/boukensha/backends/ollama_cloud.py (unchanged, already correct)
```

Plus a launcher at `week1_baseline/bin/python/06_the_logger`, matching
`bin/python/05_agent_loop`'s shape (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/06_the_logger"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency (see Decisions).

## Behavior parity checklist (from the real Ruby output)

- [x] `boukensha.config()` memoizes a single `Config()` instance
      across calls within the process (module-level, not per-import)
- [x] `boukensha.quiet()`/`.loud()` set a module-level flag;
      `boukensha.is_quiet()` reads it (unused by `Logger`/`Agent` in
      this step but ported for parity with `boukensha.rb`)
- [x] `boukensha.debug()` sets a module-level flag;
      `boukensha.is_debug()` reads it; `Logger.raw()` is a no-op
      unless `is_debug()` is true
- [x] `Logger(session_id=None, dir=None, log=None, snapshot=None)`:
      resolves `session_id` (generated if omitted), resolves `path` to
      `log` if given, else `{dir or default_dir}/{session_id}.jsonl`;
      creates parent dirs; opens the file in append mode; immediately
      writes a `session_start` event merged with `snapshot`
- [x] `default_dir` is `{boukensha.config().dir}/sessions`
- [x] Every phase method (`iteration`, `limit_reached`, `turn_end`,
      `prompt`, `tool_call`, `tool_result`, `response`, `raw`) writes
      exactly one JSON line via a shared `_write_log`, which merges
      `session_id` and an ISO-8601 `at` timestamp into the event dict
      and flushes after every write
- [x] `prompt(messages, tools)` logs `message_count`, a
      `{"role", "content"}` projection of each message, `tool_count`,
      and the sorted/insertion-order tool names
- [x] `tool_result(name, result, ok=True, error=None)` stringifies
      `result` before logging
- [x] `raw(data)` is a no-op unless `boukensha.is_debug()` is true
- [x] `response(text, usage=None, stop_reason=None, task=None,
      backend=None)` logs `text.strip()`, `usage`, `stop_reason`, and
      merges in execution metadata (`task`, `provider`, `model`,
      `usage_unit`, `usage_level`, `input_tokens`, `output_tokens`,
      `cost_usd`) — only the metadata keys with non-`None` values are
      included (Ruby's `.compact`)
- [x] `task` metadata resolves via `task.task_name()` when the task
      class defines it, else `str(task)`, else `None`
- [x] `provider` metadata is the backend's class name converted from
      CamelCase to snake_case (e.g. `Anthropic` → `anthropic`,
      `OllamaCloud` → `ollama_cloud`)
- [x] `usage_tokens` reads `input_tokens`/`output_tokens` from the
      first matching key across `input_tokens`/`prompt_tokens`/
      `promptTokenCount`/`prompt_eval_count` (and the output-side
      equivalents), coercing to `int`, returning `None` on missing or
      non-numeric values
- [x] `cost_usd` calls `backend.estimate_cost(...)` only when both
      token counts are present and the backend defines
      `estimate_cost`
- [x] `Agent.__init__` accepts `logger=None`; when omitted, a **new**
      `Logger()` is created per `Agent` instance (not a shared
      default) — this opens a new session file per agent
- [x] `Agent.run()`: on iteration-limit, logs
      `limit_reached(kind="max_iterations", n=self.iteration,
      max=self.max_iterations)` before calling `_wrap_up`; otherwise
      increments, logs `iteration(n=self.iteration,
      max=self.max_iterations)`, logs `prompt(messages=context.messages,
      tools=context.tools)`, calls the client, logs `raw(data=response)`,
      then either dispatches tool calls or logs the final `response` +
      `turn_end(reason="completed", iterations=self.iteration)` and
      returns the text
- [x] `_wrap_up`: on success, logs `response` then
      `turn_end(reason=reason, iterations=self.iteration)` before
      returning; on `ApiError`, still logs
      `turn_end(reason=reason, iterations=self.iteration)` before
      returning the fallback message
- [x] `_handle_tool_calls(content, response)`: logs the reasoning text
      (or a `"(tool use — N call(s))"` placeholder when the model
      emitted no text) via `_log_response` **before** dispatching any
      tool; stores the assistant message; for each tool call, logs
      `tool_call(name, args)`, dispatches, and logs `tool_result` with
      `ok=True` on success or `ok=False` + `error=str(e)` when
      `registry.dispatch` raises — the stringified error result is
      still stored as the `tool_result` message content either way
- [x] `_log_response`/`_normalized_usage` are private helpers on
      `Agent` (not `Logger`) that assemble the `response()` call's
      `usage`/`stop_reason`/`task`/`backend` args from the raw API
      response, checking `response["usage"]` (Anthropic/OpenAI),
      `response["usageMetadata"]` (Gemini), then falling back to
      `prompt_eval_count`/`eval_count` (Ollama)
- [x] `examples/example.py` builds `logger = Logger()`, passes it to
      `Agent(..., logger=logger)`, prints the same
      Config/Provider/Model/Max-iterations/Max-output-tokens banner
      under `=== BOUKENSHA Step 6: The Logger ===`, and — matching the
      real Ruby run — prints **no** per-iteration/tool-call lines,
      only the banner and `=== FINAL RESPONSE ===` block

Expected output (verified by actually running
`./week1_baseline/bin/ruby/06_the_logger` from the repo root against
the real `.boukensha/` fixture — real, billed Anthropic API calls,
confirmed with the user before running):

```
=== BOUKENSHA Step 6: The Logger ===

Config: #<Boukensha::Config dir=/Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Max iterations: 25
Max output tokens: 1024


=== FINAL RESPONSE ===
## Summary

Based on the README.md file, here's what the **Boukensha MUD Player Assistant Framework** can do:

### Core Functionality:

1. **Agent-Based MUD Automation** - The framework provides an AI-powered agent (`Boukensha::Agent`) that can play MUD games on behalf of players by issuing commands and responding to game states.
...
The framework appears to be a Ruby-based system designed to automate MUD gameplay through AI agents while maintaining detailed, structured logs of all operations.
```

Note: unlike step 5's console transcript, there are **no**
`[iteration N/M]` / `tool call →` / `tool result →` lines — that
output moved entirely into the session log file. The model's exact
tool-call sequence (which files it reads, how many iterations) is
live, non-reproducible model behavior, same caveat as `05_agent_loop.md`.

The real run also produced
`.boukensha/sessions/20260724T034716Z-e3eb931d.jsonl` (20 lines: 1
`session_start`, 4 `iteration`, 4 `prompt`, 4 `response`, 3
`tool_call`, 3 `tool_result`, 1 `turn_end`). A representative excerpt:

```json
{"phase":"session_start","session_id":"20260724T034716Z-e3eb931d","at":"2026-07-23T23:47:16-04:00"}
{"phase":"iteration","n":1,"max":25,"session_id":"20260724T034716Z-e3eb931d","at":"..."}
{"phase":"prompt","message_count":1,"messages":[{"role":"user","content":"Read the README.md file and summarise what this MUD player assistant framework can do."}],"tool_count":2,"tools":["read_file","list_directory"],"session_id":"20260724T034716Z-e3eb931d","at":"..."}
{"phase":"response","text":"I'll read the README.md file for you.","usage":{"input_tokens":707,"output_tokens":67,"...":"..."},"stop_reason":"tool_use","task":"player","provider":"anthropic","model":"claude-haiku-4-5","usage_unit":"tokens","input_tokens":707,"output_tokens":67,"cost_usd":0.001042,"session_id":"20260724T034716Z-e3eb931d","at":"..."}
{"phase":"tool_call","name":"read_file","args":{"path":"README.md"},"session_id":"20260724T034716Z-e3eb931d","at":"..."}
{"phase":"tool_result","name":"read_file","result":"# Step 6 - The Logger\n...","ok":true,"error":null,"session_id":"20260724T034716Z-e3eb931d","at":"..."}
...
{"phase":"turn_end","reason":"completed","iterations":4,"tokens":null,"session_id":"20260724T034716Z-e3eb931d","at":"..."}
```

## Verification

Ran `week1_baseline/bin/python/06_the_logger` for real against the
same fixture (confirmed with the user first, same as the Ruby run —
real, billed Anthropic API calls). Console output matched the expected
shape exactly: the Config/Provider/Model/Max-iterations/Max-output-tokens
banner, then two blank lines, then `=== FINAL RESPONSE ===` with the
model's summary — no `[iteration]`/`tool call →`/`tool result →` lines,
confirming the progress-output-moved-to-the-log-file behavior change
carried over correctly. It produced
`.boukensha/sessions/20260724T035554Z-2269195e.jsonl` (10 lines: 1
`session_start`, 2 `iteration`, 2 `prompt`, 2 `response`, 1
`tool_call`, 1 `tool_result`, 1 `turn_end` — the model took a shorter
path than the Ruby run's, calling `read_file` once and finishing at
iteration 2, which is expected live-model variance, not a bug). Event
shapes, field names, and `response` metadata (`task`, `provider`,
`model`, `usage_unit`, `input_tokens`, `output_tokens`, `cost_usd`)
matched the Ruby JSONL structure field-for-field. No new cosmetic
diffs beyond the already-accepted categories from prior steps (dict
`repr()` vs. Ruby hash `#to_s`, not actually exercised in this step's
log output since `args`/`result` are JSON-serialized, not printed via
`repr`).

All behavior parity checklist items above are checked off against this
real run plus the isolated `Logger`/module-state smoke test performed
first (module-level `quiet()`/`loud()`/`is_quiet()`/`debug()`/
`is_debug()` toggled correctly; a standalone `Logger` correctly wrote
`session_start`/`iteration`/`limit_reached`/`turn_end`/`prompt`/
`tool_call`/`tool_result` (both `ok=True` and `ok=False` branches)/
`response`/`raw` events with correct JSON shapes).

## Porting notes (Ruby idiom → Python)

- **`module Boukensha` self-methods → module-level functions in
  `lib/boukensha/__init__.py`.** No prior precedent in this port for
  Ruby module-level (`self.`) singleton state — every earlier step's
  state lived on instances. Placed at the *top* of `__init__.py`,
  before any `from .xxx import Yyy` line that could depend on them
  (mirroring `boukensha.rb`'s ordering: the `module Boukensha ... end`
  block comes before `require_relative "boukensha/logger"`), so that
  `logger.py`'s `from . import config, is_debug` sees fully-defined
  functions on the partially-initialized `boukensha` package object —
  the same circular-import-safe ordering trick Ruby gets for free via
  top-to-bottom `require_relative` execution.
- **`quiet!`/`loud!`/`quiet?` and `debug!`/`debug?` → `quiet()`/
  `loud()`/`is_quiet()` and `debug()`/`is_debug()`.** Confirmed with
  the user: predicates get an `is_` prefix (the standard Python
  resolution when a bare-verb setter already claims the noun name);
  setters keep the bare Ruby verb. This preserves Ruby's two one-way
  verbs (`quiet!`/`loud!`) rather than collapsing them into a single
  boolean-argument setter, which would be an unrequested semantic
  change. `debug()` has no inverse in this step (matching Ruby — there
  is no `undebug!`).
- **`Boukensha.config` (memoized) → `config()`.** Confirmed with the
  user: matches Ruby's name exactly. This reassigns the
  `boukensha.config` package attribute from the `config` submodule
  (set automatically as an import-system side effect of `from .config
  import Config`) to this function — a deliberate, harmless shadow,
  since nothing in the codebase looks up `boukensha.config` as a
  module reference (everything imports `Config` directly via `from
  .config import Config`).
- **`logger: Logger.new` default kwarg → `logger=None` sentinel +
  lazy construction.** Ruby evaluates keyword-argument default value
  *expressions* fresh on every call, so `Agent.new` without an
  explicit `logger:` creates a brand new `Logger` (and thus a new
  session file) each time. Python evaluates a default argument value
  once, at function-*definition* time — a literal `def __init__(self,
  ..., logger=Logger()): ...` would construct exactly one `Logger`
  (opening one file) at import time and silently share it across every
  `Agent` instance that doesn't pass its own. Ported as `logger=None`
  in the signature, then `self.logger = logger if logger is not None
  else Logger()` in the body, which reconstructs Ruby's "fresh
  instance per call" semantics exactly.
- **`rescue StandardError => e` around `@registry.dispatch` → `except
  Exception as e`.** Same broad-catch translation already used
  elsewhere in this codebase's `client.py` (`except
  self.TRANSIENT_ERRORS`) — dispatch can raise `UnknownToolError` or
  anything the tool's own block raises, and the Ruby code intentionally
  catches broadly so a failing tool doesn't kill the whole turn.
- **`e.class}: #{e.message}` interpolation → `f"{type(e).__name__}:
  {e}"`.** Direct equivalent; same pattern already used in
  `client.py`'s transient-error message.
- **`backend.class.name.split("::").last.gsub(/([a-z\d])([A-Z])/,
  '\1_\2').downcase` → `re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_",
  type(backend).__name__).lower()`.** Python class names have no
  `::` namespacing to split on (`type(backend).__name__` already gives
  the bare class name, e.g. `"OllamaCloud"`); the regex boundary
  insertion is a direct port of the same camel-to-snake transform.
- **`Time.now.iso8601` → `datetime.now().astimezone().isoformat()`.**
  Both produce a local-offset ISO-8601 timestamp
  (`2026-07-23T23:47:16-04:00` shape); `astimezone()` with no
  arguments attaches the system-local UTC offset the same way Ruby's
  `Time.now` (already local) does.
- **`SecureRandom.hex(4)` → `secrets.token_hex(4)`.** Direct stdlib
  equivalent (`secrets` is Python's cryptographically-secure random
  module, same role as Ruby's `SecureRandom`); both produce an 8-hex-char
  string from 4 random bytes. No new dependency — stdlib both sides
  (see Decisions).
- **`FileUtils.mkdir_p(File.dirname(@path))` → `Path(self.path).parent.mkdir(parents=True,
  exist_ok=True)`.** Direct equivalent.
- **`Logger`'s `private` section → leading-underscore methods**,
  same convention as `Agent`/`tasks/base.py`/`backends/base.py`:
  `_default_dir`, `_write_log`, `_generate_session_id`,
  `_serialize_message`, `_execution_metadata`, `_task_name`,
  `_provider_name`, `_usage_tokens`, `_first_integer`,
  `_estimate_cost`. `Agent`'s new `log_response`/`normalized_usage`
  helpers (under Ruby's `private`) become `_log_response`/
  `_normalized_usage`, same convention.
- **`hash[key] || hash[key.to_sym]` string/symbol duality in
  `first_integer` → plain `dict.get(key)`.** Python dicts have no
  symbol/string key duality (this project's established precedent,
  e.g. `tasks/base.py`'s `_fetch`) — every usage dict here already
  comes from `json.loads`, so keys are always plain strings; the
  Ruby `key.to_sym` fallback has no Python counterpart to port.
- **`Integer(value)` in `first_integer` → `int(value)`, catching
  `(TypeError, ValueError)`** instead of Ruby's `(ArgumentError,
  TypeError)` — same pair of "this value can't become an int"
  exception classes, different names per language stdlib.
- **`event.merge(session_id:, at:)` → `{**event, "session_id": ...,
  "at": ...}`** (or `event | {...}` — either works; used dict unpacking
  for consistency with dict-literal style already used in this
  codebase, e.g. `backends/base.py`'s model dicts).
- `context.py`, `prompt_builder.py`, `message.py`, `tool.py`,
  `registry.py`, `client.py`, `tasks/base.py`, `tasks/player.py`,
  `backends/*.py` need **no edits** — already correct in the current
  `python/06_the_logger/` tree (verified identical to `05_agent_loop`'s
  versions, matching the Ruby side's non-behavioral or already-covered
  changes).

## Decisions

1. **Ran the real Ruby step for a verified transcript, confirmed with
   the user first** — same precedent as `04_api_client.md`/
   `05_agent_loop.md`: real, billed Anthropic API calls, explicit
   go-ahead given before executing `bin/ruby/06_the_logger`.

2. **Module-level toggle API: `quiet()`/`loud()`/`is_quiet()` and
   `debug()`/`is_debug()`.** Asked the user directly (genuinely
   ambiguous — no precedent for Ruby module self-methods anywhere
   earlier in this port). Chose the `is_`-prefix-for-predicates shape
   over collapsing `quiet!`/`loud!` into a single `set_quiet(bool)`,
   to keep Ruby's two explicit one-way verbs intact rather than
   introducing an unrequested boolean-argument API.

3. **Memoized config accessor: `config()`, accepting the harmless
   `boukensha.config` submodule-attribute shadow.** Asked the user
   directly; chose exact name parity with Ruby's `Boukensha.config`
   over the collision-avoiding alternative (`get_config()`), since
   nothing in the codebase relies on `boukensha.config` resolving to
   the submodule.

4. **`Agent.__init__`'s `logger=None` sentinel, not a literal
   `Logger()` default.** Required to reproduce Ruby's per-call default
   evaluation (Ruby: default *expressions* run per call; Python:
   default *values* are computed once, at def time). A literal default
   would silently share one `Logger`/one open file handle across every
   `Agent` built without an explicit `logger=`. See Porting Notes.

5. **`config.py`'s `mud_*` properties are deleted, not left in
   place.** Unlike `05_agent_loop.md`'s Decision 2 (where the Ruby
   change was a no-op syntax rewrite with an already-idiomatic Python
   equivalent already in place), this step's Ruby diff *removes* the
   four `mud_*` methods outright — genuine dead-code removal, not a
   syntax change. Porting "nothing" here would leave Python carrying
   four methods Ruby no longer has, diverging from the spec. Deleted
   to match.

6. **`LoopError` is removed from `errors.py` and `__init__.py`'s
   imports/`__all__`.** Mirrors Ruby's `errors.rb` diff exactly (the
   unused error class introduced as a placeholder in step 5 is deleted
   in step 6, never having been raised anywhere). No other file
   references `LoopError`, so removal is a clean deletion with no
   ripple.

7. **`prompt_builder.py` gets no edit.** Ruby's only change to
   `prompt_builder.rb` this step is `attr_reader :backend` — Python's
   `self.backend` set in `__init__` is already a public attribute with
   no explicit accessor needed, so it's already the idiomatic
   equivalent (same class of no-op as `05_agent_loop.md`'s Decision 2
   for `config.rb`'s endless methods).

8. **No new dependency.** `logger.py` uses only stdlib (`json`,
   `secrets`, `datetime`, `pathlib`) — the same stdlib-first default
   this project has followed throughout (`ITERATIONS.md`), and Ruby's
   `Gemfile`/`Gemfile.lock` are unchanged this step (`diff -rq`
   confirms no gem was added), so there's no Ruby-side dependency to
   mirror either.

9. **`bin/python/06_the_logger` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/05_agent_loop`'s shape.

## Open questions

None outstanding — all decided above.
