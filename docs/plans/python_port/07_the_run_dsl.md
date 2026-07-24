# Python Port Plan — 07 The Boukensha.run DSL

## Goal

Port the behavior of `week1_baseline/ruby/07_the_run_dsl/` to
`week1_baseline/python/07_the_run_dsl/`. The directory already existed
but was byte-identical to `python/06_the_logger/` (verified: `diff -rq
python/06_the_logger python/07_the_run_dsl` reported no differences).
This step adds a single top-level entry point, `Boukensha.run`, that
wires together `Context`, `Registry`, a backend, `PromptBuilder`,
`Client`, `Logger`, and `Agent` from just a `task:` string and an
optional block that registers tools — collapsing every previous
step's manual plumbing into one call.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff -rq ruby/06_the_logger ruby/07_the_run_dsl`:
the new file is `lib/boukensha/run_dsl.rb`; changed files are
`lib/boukensha.rb` (adds `self.run`, plus `require_relative
"boukensha/run_dsl"`), `lib/boukensha/config.rb` (restores the four
`mud_*` endless methods — removed in `06_the_logger`, brought back
here; genuinely re-added by the Ruby spec at this step, not dead code
to skip), `lib/boukensha/errors.rb` (restores unused `LoopError`, same
removed-then-restored pattern), `lib/boukensha/logger.rb` (adds
`turn(n:)`, not yet called by `agent.rb`, and `subscribe(&block)` with
`write_log` notifying subscribers after every write),
`lib/boukensha/context.rb` (whitespace-only ivar alignment + missing
trailing newline, zero behavior change). `README.md` and
`examples/example.rb` are rewritten. Everything else —
`message.rb`, `tool.rb`, `registry.rb`, `agent.rb`,
`tasks/{base,player}.rb`, `backends/*.rb`, `client.rb`,
`prompt_builder.rb`, `Gemfile`/`Gemfile.lock` — is byte-identical to
`06_the_logger`, carried forward unchanged.

**Ruby README title says "Step 6" (`# Step 6 — The Boukensha.run
DSL`)** — a copy-paste leftover from the previous step's README, same
class of drift `02_the_registry.md`/`05_agent_loop.md`/
`06_the_logger.md` already documented. The directory, the runtime
banner (`"=== BOUKENSHA Step 7: The Boukensha.run DSL ==="`), and the
launcher (`bin/ruby/07_the_run_dsl`) all agree it's step 7; this plan
and the Python README follow the real numbering.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/run_dsl.rb` | **New.** `Boukensha::RunDSL` — tiny host object; `Boukensha.run`'s block is `instance_eval`'d against it so bare `tool` calls resolve here; exposes only `tool(name, description:, parameters: {}, &block)`, which proxies to `@registry.tool` |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha.rb` | Adds `self.run(task:, system: nil, model: nil, backend: nil, api_key: nil, ollama_host: "http://localhost:11434", log: nil, max_output_tokens: nil, &block)` — resolves config/system/model/backend/api_key from the `player` task's settings, builds `Context`→`Registry`, evals the block against a `RunDSL`, builds the matching backend, `PromptBuilder`, `Client`, `Logger` (snapshot includes task/limits/model/provider), `Agent`, seeds the user message, runs the agent, and closes the logger in an `ensure` |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/config.rb` | Restores `mud_host`/`mud_port`/`mud_username`/`mud_password` (removed in `06_the_logger`) |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/errors.rb` | Restores `LoopError` (removed in `06_the_logger`) |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/logger.rb` | Adds `turn(n:)` (unused by `agent.rb` in this step) and `subscribe(&block)`; `write_log` now also calls every subscriber with the raw event hash after writing |
| `week1_baseline/ruby/07_the_run_dsl/examples/example.rb` | Rewritten: sets `ENV["BOUKENSHA_DIR"]` before requiring the lib (as before), then calls `Boukensha.run(task: ...) do ... end` registering `read_file`/`list_directory` tools inline, banner → "Step 7: The Boukensha.run DSL" |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/context.rb` | Whitespace/ivar-alignment only, plus a missing trailing newline — no behavior change, no Python edit needed |
| `week1_baseline/ruby/07_the_run_dsl/lib/boukensha/{message,tool,registry,agent,client,prompt_builder}.rb`, `tasks/{base,player}.rb`, `backends/*.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `06_the_logger` (verified via `diff -rq`) — carry forward as-is |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps —
`settings.yaml` (including the `mud:` block the restored `mud_*`
accessors read), `.env`, `prompts/` unchanged. This step's runs add
their own new `.boukensha/sessions/<session-id>.jsonl` files alongside
existing ones, same as every prior step.

## Target files to create/change (Python)

```
week1_baseline/python/07_the_run_dsl/
  README.md                              (rewrite: boukensha.run() docs, options table, before/after comparison, run example)
  requirements.txt                       (unchanged — no new dependency)
  prompts/system.md                      (unchanged, already correct)
  examples/example.py                    (rewrite: define register_tools(dsl) registering read_file/list_directory via @dsl.tool(...), call boukensha.run(task=..., register=register_tools), banner → "Step 7: The Boukensha.run DSL")
  lib/boukensha/__init__.py              (edit: import backends + RunDSL + LoopError; add top-level run() function; extend __all__)
  lib/boukensha/config.py                (edit: restore mud_host/mud_port/mud_username/mud_password properties)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/client.py                (unchanged, already correct)
  lib/boukensha/prompt_builder.py        (unchanged, already correct)
  lib/boukensha/errors.py                (edit: restore LoopError)
  lib/boukensha/logger.py                (edit: add turn(n) and subscribe(callback); _write_log notifies subscribers)
  lib/boukensha/run_dsl.py               (new)
  lib/boukensha/agent.py                 (unchanged, already correct)
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

Plus a launcher at `week1_baseline/bin/python/07_the_run_dsl`
(executable bit set). Unlike every prior Python launcher, this one
exports `BOUKENSHA_DIR` before invoking the interpreter, mirroring
`bin/ruby/07_the_run_dsl`'s own new export (the Ruby launcher added it
at this exact step — see Decisions):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

export BOUKENSHA_DIR="$REPO_ROOT/.boukensha"

cd "$SCRIPT_DIR/../../python/07_the_run_dsl"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency.

## Behavior parity checklist (from the real Ruby output)

- [x] `Config.mud_host`/`mud_port`/`mud_username`/`mud_password`
      restored, reading `settings.yaml`'s `mud:` block via `dig`, with
      `"localhost"`/`4000` fallbacks for host/port and `None` for
      username/password
- [x] `errors.py` has a `LoopError` exception class again (unused,
      matches Ruby's `LoopError` being unraised anywhere)
- [x] `Logger.turn(n)` writes a `{"phase": "turn", "n": n}` event
      (unused by `agent.py` this step, matching Ruby)
- [x] `Logger.subscribe(callback)` registers a callback; every
      `_write_log` call invokes all registered subscribers with the
      raw event dict, after the line is written and flushed
- [x] `RunDSL(registry)` exposes only `tool(name, *, description,
      parameters=None)`, which returns `registry.tool(...)`'s
      decorator unchanged — so `@dsl.tool(...)` behaves identically to
      `@registry.tool(...)`
- [x] `boukensha.run(*, task, system=None, model=None, backend=None,
      api_key=None, ollama_host="http://localhost:11434", log=None,
      max_output_tokens=None, register=None)`:
  - [x] calls `config()` first (loads `.env`, populates `os.environ`)
  - [x] resolves `task_class = Player`, `task_settings =
        cfg.tasks("player")`
  - [x] `system` defaults to `Player.system_prompt(task_settings,
        user_prompts_dir=cfg.user_prompts_dir,
        default_prompts_dir=Config.PROMPTS_DIR)` when not given
  - [x] `model` defaults to `Player.model(task_settings)`, `backend`
        defaults to `Player.provider(task_settings)`, when not given
  - [x] `api_key` defaults to the matching `ANTHROPIC_API_KEY` /
        `OPENAI_API_KEY` / `GEMINI_API_KEY` / `OLLAMA_API_KEY` env var
        for the resolved backend (no default for `"ollama"`), only
        when `api_key` was not passed explicitly
  - [x] builds `Context(task=Player, system=system)` then
        `Registry(ctx)` **before** calling `register`
  - [x] calls `register(RunDSL(registry))` if `register` is given,
        before the backend is constructed
  - [x] dispatches to `Anthropic`/`OpenAI`/`Gemini`/`Ollama`/
        `OllamaCloud` by the resolved `backend` string, raising
        `ValueError` on an unrecognized value
  - [x] builds `PromptBuilder(ctx, be)`, `Client(builder)`; resolves
        `effective_max_iterations = Player.max_iterations(task_settings)`
        and `effective_max_output_tokens = max_output_tokens or
        Player.max_output_tokens(task_settings)`
  - [x] builds `Logger(log=log, snapshot={task, max_iterations,
        max_output_tokens, model, provider})`
  - [x] builds `Agent(context=ctx, registry=registry, builder=builder,
        client=client, logger=logger, task_settings=task_settings,
        max_iterations=effective_max_iterations,
        max_output_tokens=effective_max_output_tokens)`
  - [x] adds the `task` string as a `"user"` message, runs the agent,
        returns its result
  - [x] closes the logger in a `finally`, even if an error occurred
        before or during the run (guarded against `logger` never
        having been constructed)
- [x] `examples/example.py` prints the same banner/Config line as
      Ruby, registers `read_file`/`list_directory` via the
      `register` callback, and prints the same
      `=== FINAL RESPONSE ===` block — matching the real Ruby run's
      shape (live model text differs, expected)

Expected output (verified by actually running
`./week1_baseline/bin/ruby/07_the_run_dsl` from the repo root against
the real `.boukensha/` fixture — real, billed Anthropic API calls,
confirmed with the user before running, same precedent as
`04_api_client.md`/`05_agent_loop.md`/`06_the_logger.md`):

```
=== BOUKENSHA Step 7: The Boukensha.run DSL ===

Config: #<Boukensha::Config dir=/Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha tasks=player>


=== FINAL RESPONSE ===
## Summary

This is **Boukensha**, a MUD player assistant framework written in Ruby. Here's what it can do:

### Core Capabilities:
1. **AI-Powered MUD Agent** — Uses LLMs (Claude via Anthropic or local Ollama) to control gameplay actions on your behalf
2. **Tool Integration** — Allows you to define custom tools (like reading files, listing directories, etc.) that the AI agent can use to interact with the game world
3. **Agentic Loop** — Implements a reasoning loop where the AI can plan multi-step actions, call tools, and react to results
...
Essentially, it's an **autonomous MUD player** framework that lets you describe what you want to accomplish in plain English, while the AI handles the step-by-step gameplay execution.
```

Unlike Ruby's own step-6 banner text, this step's Config line has no
`Provider:`/`Model:`/`Max iterations:`/`Max output tokens:` lines —
`Boukensha.run` resolves those internally now and `example.rb` no
longer prints them, matching the real transcript above exactly (not
the README's stale before/after snippet, which was written before the
banner's final shape).

## Verification

Ran `week1_baseline/bin/python/07_the_run_dsl` for real against the
same fixture (confirmed with the user first, same as the Ruby run —
real, billed Anthropic API calls). Console output matched the expected
shape: the banner, `Config: ...` line, two blank lines, then
`=== FINAL RESPONSE ===` with the model's summary. It produced
`.boukensha/sessions/20260724T044847Z-02f315a8.jsonl` (10 lines: 1
`session_start`, 2 `iteration`, 2 `prompt`, 2 `response`, 1
`tool_call`, 1 `tool_result`, 1 `turn_end`) — the model took a shorter
tool-call path than the Ruby run above (calling `read_file` once and
finishing at iteration 2), expected live-model variance, not a bug.
The `session_start` event's snapshot correctly included
`task`/`max_iterations`/`max_output_tokens`/`model`/`provider`, coming
from `boukensha.run()`'s new `Logger(..., snapshot={...})` call. This
run happened before the Python `README.md` rewrite landed, so the
`read_file` tool's actual content (used only to build the model's
answer, not checked by this plan) reflects the prior `06_the_logger`
README text — irrelevant to verifying `boukensha.run()`'s plumbing,
which is what this step tests.

All behavior parity checklist items above are checked off against this
real run plus a direct read of the implemented `run()`/`RunDSL`/
`Config`/`Logger`/`errors.py` source against the Ruby source line by
line.

## Porting notes (Ruby idiom → Python)

- **`RunDSL.new(registry).instance_eval(&block) if block` → a
  `register` callback parameter receiving a `RunDSL` instance
  directly.** Ruby's block is `instance_eval`'d so bare `tool "x", ...`
  calls resolve against the `RunDSL` receiver with no explicit `self`.
  Python has no `instance_eval` equivalent, and every earlier step's
  block-translation precedent (`Registry.tool` in `02_the_registry`)
  already requires an explicit receiver (`@registry.tool(...)`), so
  the closest idiomatic match is a plain function parameter that
  receives the `RunDSL` explicitly and decorates tools onto it
  (`@dsl.tool(...)`) rather than inventing an implicit-`self`
  mechanism Python doesn't have. This is a new pattern (a block
  containing *multiple* decorator calls, not a single trailing block
  like `Registry.tool`'s), so it was confirmed with the user before
  implementing — three real alternatives existed (this callback shape;
  a context manager that defers agent execution to `__exit__`;
  dropping `RunDSL` and decorating `registry` directly) and none had
  precedent in an earlier plan doc.
- **`RunDSL#tool` → thin proxy to `Registry.tool`, unchanged
  signature.** `RunDSL` exists only to narrow the DSL surface to one
  method (per Ruby's own comment: "keeping the DSL surface
  intentionally small"); the Python class does the same —
  `RunDSL.tool` forwards `name`/`description`/`parameters` to
  `self._registry.tool(...)` and returns its decorator unchanged.
- **Ruby symbol `backend` (`task_class.provider(task_settings).to_sym`,
  `case backend when :anthropic ...`) → plain Python string
  throughout.** `example.py` has compared provider strings directly
  since `03_prompt_builder` (`if provider == "anthropic"`); `Player.provider`
  already returns a bare string. `boukensha.run()`'s backend dispatch
  keeps that established simplification (`if backend == "anthropic":
  ...`) instead of introducing a symbol/string distinction Python
  doesn't need — same precedent as `02_the_registry.md`'s "no
  symbol/string transform needed in dispatch."
- **`ensure logger&.close` → `logger = None` before the whole body,
  `finally: if logger is not None: logger.close()`.** Ruby's local
  `logger` variable is implicitly `nil` if an exception (e.g. the
  unknown-backend `ArgumentError`) fires before the `Logger.new` line
  runs, and `&.close` safely no-ops on `nil`. Python raises
  `UnboundLocalError` referencing a name that was never assigned on
  that code path, so `logger` is explicitly pre-bound to `None`,
  making the "only close it if it was actually built" guard explicit
  rather than implicit.
- **`@subscribers ||= []` lazy-init in `Logger#subscribe` →
  `self._subscribers = []` in `Logger.__init__`.** Ruby lazily
  allocates the array on first `subscribe` call and guards iteration
  with `@subscribers&.each`. Python has no attribute analogous to
  Ruby's implicit-nil-until-assigned ivars; eagerly initializing the
  list in `__init__` (iterating an empty list is already a no-op, same
  observable behavior as `nil&.each`) avoids a lazy-init branch that
  `logger.py`'s existing style doesn't otherwise use.
- **Restored `Config.mud_*`/`errors.LoopError` are ported even though
  nothing in this step's Ruby code calls them yet.** They're genuine
  re-additions in the Ruby diff (removed at `06_the_logger`, restored
  here) rather than dead code the port should skip — matching the spec
  exactly, per the same "match the spec, don't second-guess it"
  principle `06_the_logger.md`'s Decision 5/6 documented for the
  removal direction. `python/00_config/lib/boukensha/config.py`
  already has the exact `mud_*` implementation this step restores
  (verbatim `dig("mud", "host") or "localhost"` style), so the restore
  is a direct copy-back, not new design work.
- `context.py`, `message.py`, `tool.py`, `registry.py`, `client.py`,
  `prompt_builder.py`, `agent.py`, `tasks/base.py`, `tasks/player.py`,
  `backends/*.py` need **no edits** — already correct in the current
  `python/07_the_run_dsl/` tree (verified identical to
  `06_the_logger`'s versions, matching the Ruby side's non-behavioral
  or already-covered changes).

## Decisions

1. **Ran the real Ruby step for a verified transcript, confirmed with
   the user first** — same precedent as `04_api_client.md`/
   `05_agent_loop.md`/`06_the_logger.md`: real, billed Anthropic API
   calls, explicit go-ahead given before executing
   `bin/ruby/07_the_run_dsl`.

2. **`boukensha.run()`'s block-replacement is a `register` callback
   receiving a `RunDSL` instance, not a context manager or a raw
   `registry` callback.** Asked the user directly (genuinely
   ambiguous — first multi-decorator block in this port, no earlier
   precedent settles it). Chosen over a context-manager shape (defers
   agent execution to `__exit__`, adds machinery and an implicit
   deferred-execution model the Ruby code doesn't need to express) and
   over dropping `RunDSL` to decorate `registry` directly (simpler,
   but loses the Ruby version's explicit intent of restricting the
   block's surface to just `tool`). The callback keeps `RunDSL` as a
   real, minimal class matching Ruby's `run_dsl.rb` file-for-file, and
   reuses the already-established decorator-factory idiom without
   inventing new control flow.

3. **`Config`'s restored `mud_*` properties and `errors.py`'s restored
   `LoopError` are brought back exactly as they existed before
   `06_the_logger` removed them**, copied from `python/00_config`'s
   still-live implementation rather than re-derived, since the Ruby
   diff shows a straight re-add with no signature or behavior change
   from the pre-removal version.

4. **`Logger.subscribe`'s subscriber list is initialized eagerly in
   `__init__`, not lazily on first `subscribe()` call.** Ruby's
   `@subscribers ||= []` guard is idiomatic Ruby for "allocate on
   first use," but has no ivar-nil-by-default equivalent worth
   reproducing in Python; an empty list already behaves identically
   under iteration, so eager init in `__init__` is the simpler,
   equally-correct choice.

5. **`bin/python/07_the_run_dsl` exports `BOUKENSHA_DIR` before
   invoking the interpreter**, matching `bin/ruby/07_the_run_dsl`'s
   own new export at this exact step (every earlier `bin/python/*`
   launcher relied solely on `example.py`'s own
   `os.environ.setdefault(...)` fallback, same as Ruby's
   `ENV["BOUKENSHA_DIR"] ||= ...` line in `example.rb`, and both still
   work without the launcher-level export in this repo's layout — the
   export is additional robustness, not a bug fix, ported for
   consistency with the Ruby launcher rather than left out because the
   fallback alone still works here).

6. **No new dependency.** `run_dsl.py` uses no imports; the `run()`
   function in `__init__.py` only imports already-ported project
   modules and `os` (stdlib). Ruby's `Gemfile`/`Gemfile.lock` are
   unchanged this step (`diff -rq` confirms no gem was added), so
   there's no Ruby-side dependency to mirror either.

7. **`bin/python/07_the_run_dsl` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/06_the_logger`'s shape plus the `BOUKENSHA_DIR` export
   from Decision 5.

## Open questions

None outstanding — all decided above.
