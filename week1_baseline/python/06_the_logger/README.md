# 06 · The Logger (Python port)

Behavior port of `ruby/06_the_logger` — `boukensha.logger.Logger`
records each agent run as structured JSON Lines. It is a file logger,
not user-facing display output: as of this step, `Agent.run()` prints
**nothing** to stdout about iterations or tool calls — that output
moved entirely into the log file. `message.py`, `tool.py`,
`context.py`, `registry.py`, `client.py`, `prompt_builder.py`,
`tasks/*.py`, and `backends/*.py` are unchanged from `05_agent_loop`;
see `../05_agent_loop/README.md` for those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/logger.py` | `Logger` — one method per phase, each appending a JSON line to a per-session file |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/__init__.py` | Adds module-level `config()`, `quiet()`/`loud()`/`is_quiet()`, `debug()`/`is_debug()`, and exports `Logger` |
| `lib/boukensha/agent.py` | `Agent.__init__` gains `logger=None`; `run`/`_wrap_up`/`_handle_tool_calls` log every phase; tool dispatch errors are caught and logged instead of propagating |
| `lib/boukensha/errors.py` | Removes unused `LoopError` |
| `lib/boukensha/config.py` | Removes unused `mud_host`/`mud_port`/`mud_username`/`mud_password` properties |

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
{"phase":"session_start","session_id":"20260724T034716Z-e3eb931d","at":"2026-07-23T23:47:16-04:00"}
{"phase":"iteration","n":1,"max":25,"session_id":"20260724T034716Z-e3eb931d","at":"2026-07-23T23:47:16-04:00"}
```

`response` lines include the active task, provider, model, normalized
token counts, and estimated USD cost when the backend has token
pricing data:

```json
{"phase":"response","task":"player","provider":"anthropic","model":"claude-haiku-4-5","input_tokens":707,"output_tokens":67,"cost_usd":0.001042}
```

## boukensha.logger.Logger

| Method | Phase | Logs |
|---|---|---|
| `iteration(n, max)` | `iteration` | loop counter and ceiling |
| `limit_reached(kind, n, max)` | `limit_reached` | iteration ceiling triggered |
| `turn_end(reason, iterations, tokens=None)` | `turn_end` | why/when the turn ended |
| `prompt(messages, tools)` | `prompt` | message count/roles, tool count/names |
| `tool_call(name, args)` | `tool_call` | tool name and arguments |
| `tool_result(name, result, ok=True, error=None)` | `tool_result` | stringified tool result, success flag |
| `response(text, usage=None, stop_reason=None, task=None, backend=None)` | `response` | response text, token usage, task/provider/model, estimated cost |
| `raw(data)` | `raw` | raw provider response, only when `boukensha.is_debug()` is true |

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

```python
logger = Logger()
agent = Agent(context=ctx, registry=registry, builder=builder,
              client=client, logger=logger)
```

You can also provide a session id or override the destination
directory:

```python
Logger(session_id="manual-session")
Logger(dir="/tmp/boukensha-sessions")
```

For compatibility, `log=` still accepts an explicit file path, but
normal iteration usage should write under `.boukensha/sessions`.

## Debug Events

Call `boukensha.debug()` before running the agent to include raw
provider responses:

```python
import boukensha
boukensha.debug()
```

## No New Dependencies

`logger.py` uses only the standard library (`json`, `secrets`,
`datetime`, `pathlib`). `requirements.txt` is unchanged from
`05_agent_loop` — matches Ruby's own `Gemfile`/`Gemfile.lock`, which
are unchanged in this step too.

## Porting notes

- **Ruby's `module Boukensha` self-methods (`quiet!`/`loud!`/`quiet?`,
  `debug!`/`debug?`, memoized `config`) → module-level functions in
  `lib/boukensha/__init__.py`.** No prior precedent for Ruby
  module-level singleton state in this port. Predicates get an `is_`
  prefix (`is_quiet()`, `is_debug()`) since the bare noun is already
  claimed by the setter verb (`quiet()`, `debug()`); the memoized
  accessor keeps Ruby's exact name (`config()`), which harmlessly
  shadows the `config` submodule's auto-bound package attribute —
  nothing in the codebase looks that submodule up via
  `boukensha.config`.
- **`logger: Logger.new` Ruby default kwarg → `logger=None` sentinel +
  lazy construction.** Ruby evaluates default *expressions* fresh on
  every call, so `Agent.new` without an explicit `logger:` builds a
  brand new `Logger`/session file each time. Python evaluates default
  *values* once, at function-definition time — a literal
  `logger=Logger()` would construct exactly one `Logger` at import
  time and share it across every `Agent`. Ported as
  `self.logger = logger if logger is not None else Logger()` inside
  `__init__` to reproduce the per-call semantics.
- **`Logger.raw`/`Logger._default_dir`'s references to
  `Boukensha.debug?`/`Boukensha.config` → function-local `from . import
  is_debug` / `from . import config`.** `logger.py` is itself imported
  by `boukensha/__init__.py`, so a module-level `from . import
  is_debug` in `logger.py` would run during `__init__.py`'s own
  execution. Deferring the import to inside the method body (resolved
  only when the method is actually called, long after the package has
  fully loaded) sidesteps any import-ordering fragility — and mirrors
  Ruby's own late-bound method dispatch, where `Boukensha.debug?` is
  resolved at call time regardless of `require` order.
- **`rescue StandardError => e` around `@registry.dispatch` → `except
  Exception as e`.** Same broad-catch translation already used in
  `client.py`'s transient-error handling — a failing tool shouldn't
  kill the whole turn.
- **Ruby truthiness (`{}` is truthy) vs. Python truthiness (`{}` is
  falsy).** `Agent._normalized_usage` checks
  `response.get("usage") is not None` rather than a bare truthy check,
  because an empty-but-present `{}` should still short-circuit the
  fallback chain in both languages — a bare `if response.get("usage"):`
  would incorrectly fall through to the next branch in Python where
  Ruby's `if response["usage"]` would not.
- **`backend.class.name.split("::").last.gsub(/([a-z\d])([A-Z])/,
  '\1_\2').downcase` → `re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_",
  type(backend).__name__).lower()`.** Python class names have no `::`
  namespacing to strip (`type(backend).__name__` is already the bare
  name); the regex boundary-insertion camel-to-snake transform is a
  direct port (`OllamaCloud` → `ollama_cloud`).
- **`SecureRandom.hex(4)` → `secrets.token_hex(4)`**, and
  **`Time.now.iso8601` → `datetime.now().astimezone().isoformat()`** —
  direct stdlib equivalents, no new dependency.
- **Ruby's `private` section → leading-underscore methods**, same
  convention as `agent.py`/`tasks/base.py`/`backends/base.py`. Only
  the eight phase methods and `close()` stay public on `Logger`.
- **`config.rb`'s removed `mud_*` methods → deleted, not left in
  place.** Unlike earlier steps' Ruby-syntax-only changes (which
  needed no Python edit), this diff genuinely removes dead code;
  Python's `config.py` mirrors the removal to match the spec exactly.

## Run Example

```bash
./week1_baseline/bin/python/06_the_logger
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
only in the session's `.jsonl` file, verified against the real Ruby
run for this same fixture (see
`docs/plans/python_port/06_the_logger.md`).
