# Python Port Plan — 08 The REPL Loop

## Goal

Port the behavior of `week1_baseline/ruby/08_the_repl_loop/` to
`week1_baseline/python/08_the_repl_loop/`. The directory already
existed but was byte-identical to `python/07_the_run_dsl/` (verified:
`diff -rq python/07_the_run_dsl python/08_the_repl_loop` reported no
differences). This step adds a second top-level entry point,
`boukensha.repl`, that keeps the same `Context`/`Registry`/backend/
`PromptBuilder`/`Client`/`Logger` wiring as `boukensha.run` but hands
control to a new interactive `Repl` loop instead of running once —
conversation history now accumulates across turns, and a handful of
built-in `/`-commands (`/help`, `/quiet`, `/loud`, `/clear`, `/exit`,
`/quit`) are handled locally instead of being sent to the agent.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff -rq ruby/07_the_run_dsl ruby/08_the_repl_loop`:
the new files are `lib/boukensha/repl.rb` and `lib/boukensha/version.rb`;
changed files are `lib/boukensha.rb` (adds `require_relative
"boukensha/version"`, trims `self.run`'s doc comment to a one-liner,
adds `self.repl`, requires `boukensha/repl` at the bottom),
`lib/boukensha/agent.rb` (persists the final assistant reply to
`@context` at all three return points instead of just returning it),
`lib/boukensha/client.rb` (raises a specific "authentication failed
(401)" `ApiError` instead of the generic message when the API
responds 401), `lib/boukensha/config.rb` (`resolve_dir` gains a
middle tier: checks `<cwd>/.boukensha` before falling back to
`~/.boukensha`), `lib/boukensha/context.rb` (adds `clear_messages!`,
plus a missing trailing newline). `README.md` and `examples/example.rb`
are rewritten. Everything else — `message.rb`, `tool.rb`,
`registry.rb`, `run_dsl.rb`, `prompt_builder.rb`, `logger.rb`,
`errors.rb`, `tasks/{base,player}.rb`, `backends/*.rb`,
`Gemfile`/`Gemfile.lock` — is byte-identical to `07_the_run_dsl`,
carried forward unchanged (confirmed via `diff -rq`, which reported
no other changed files).

**Ruby README title says "Step 7" (`# Step 7 — The REPL Loop`) and its
"New primitives" command table omits `/quiet`/`/loud`** even though
both the `HELP` text and the worked transcript later in the same
README use them — the same class of drift `02_the_registry.md`/
`05_agent_loop.md`/`06_the_logger.md`/`07_the_run_dsl.md` already
documented. The directory name, the launcher (`bin/ruby/08_the_repl_loop`),
and `VERSION = "0.8.0"` all agree it's step 8; this plan and the
verified real transcript below (not the README's transcript, which
also shows a stale `boukensha> /quiet` interaction not exercised here)
are ground truth. `Repl::HELP`'s command list (`/quiet`, `/loud`,
`/clear`, `/exit`, `/help`) is the real, complete command set — used
as-is.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/repl.rb` | **New.** `Boukensha::Repl` — the interactive session loop. `PROMPT = "boukensha> "`; `HELP` heredoc lists the five commands. `initialize` takes the same primitives `Agent` needs (`context:, registry:, builder:, client:, logger:`) plus REPL-only extras (`config_dir:, provider:, model:, version:, api_key:, task_settings:, max_iterations:, max_output_tokens:`) and a `@turn` counter starting at 0. `start` prints the banner, then loops: print `PROMPT` (no newline, flush stdout), read a line from stdin, break on `nil` (EOF/Ctrl-D), strip it, skip if empty, dispatch `/exit`\|`/quit`/`/help`/`/quiet`/`/loud`/`/clear` locally (`next` after each), otherwise call `run_turn(input)`. `banner` builds a boxed header showing config dir / provider+model+API-key-status / version, plus the command hints. `run_turn` increments `@turn`, calls `@logger.turn(n: @turn)`, adds the input as a `user` message, builds a **fresh `Agent` every turn** (reusing the shared `@context`/`@registry`/`@builder`/`@client`/`@logger`), runs it, prints the result, and rescues `LoopError`/`ApiError` into a `[error] ...` message printed to stdout (not raised) |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/version.rb` | **New.** `VERSION = "0.8.0"` |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha.rb` | Adds `require_relative "boukensha/version"` at the top and `require_relative "boukensha/repl"` at the bottom; adds `self.repl(system: nil, model: nil, backend: nil, api_key: nil, ollama_host: "http://localhost:11434", log: nil, max_output_tokens: nil, &block)` — same config/system/model/backend/api_key resolution, `Context`/`Registry`/backend/`PromptBuilder`/`Client`/`Logger` construction as `self.run`, but instead of seeding a task message and calling `Agent#run` once, it builds a `Repl` (passing `task_settings`, resolved `max_iterations`/`max_output_tokens`, `config_dir: cfg.dir`, `provider: backend`, `model:`, `version: VERSION`, `api_key:`) and calls `.start`; wraps the whole method body in `rescue Interrupt; puts "\nInterrupted."` (Ctrl-C) with the same `ensure logger&.close` as `self.run` |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/agent.rb` | `Agent#run`'s completed-turn return and both `_wrap_up` return paths (success and the `rescue ApiError` fallback) now call `@context.add_message(:assistant, text)` (or `msg`) before returning, so the REPL's shared `Context` accumulates every assistant reply, not just user turns and tool exchanges |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/client.rb` | Inside the `unless response.is_a?(Net::HTTPSuccess)` failure branch, checks `response.code.to_i == 401` first and raises `ApiError, "authentication failed (401) — check your API key"`; falls through to the existing generic message for every other non-2xx code |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/config.rb` | `resolve_dir` becomes a 3-tier lookup: (1) `ENV["BOUKENSHA_DIR"]` if set, (2) `<Dir.pwd>/.boukensha` if that directory exists, (3) `~/.boukensha` (`DEFAULT_DIR`) otherwise |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/context.rb` | Adds `clear_messages!` (`@messages = []`), used by the REPL's `/clear` command; also fixes the missing trailing newline `07_the_run_dsl.md` noted as whitespace-only |
| `week1_baseline/ruby/08_the_repl_loop/examples/example.rb` | Rewritten: drops the `ENV["BOUKENSHA_DIR"] ||= ...` line (now set by the launcher instead, matching `07_the_run_dsl`'s launcher-export precedent), prints `Config: ...` + a blank line, points `base_dir` at the **`07_the_run_dsl`** sibling folder (a stand-in playground with real source files to read/list), and calls `Boukensha.repl do ... end` registering `read_file`/`list_directory` tools (parameter descriptions reworded for the interactive context; `list_directory`'s result is now `.sort`ed) — no `task:`, no printed `FINAL RESPONSE` block, since the REPL prints each turn's reply itself |
| `week1_baseline/ruby/08_the_repl_loop/lib/boukensha/{message,tool,registry,run_dsl,prompt_builder,logger,errors}.rb`, `tasks/{base,player}.rb`, `backends/*.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `07_the_run_dsl` (verified via `diff -rq`) — carry forward as-is. In particular `Logger#turn(n:)` already existed, unused, since `07_the_run_dsl`; this step is the first to call it |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps —
`settings.yaml`, `.env`, `prompts/` unchanged. This step's runs add
their own new `.boukensha/sessions/<session-id>.jsonl` files alongside
existing ones, same as every prior step.

## Target files to create/change (Python)

```
week1_baseline/python/08_the_repl_loop/
  README.md                              (rewrite: boukensha.repl() docs, command table, before/after comparison, run example)
  requirements.txt                       (unchanged — no new dependency)
  prompts/system.md                      (unchanged, already correct)
  examples/example.py                    (rewrite: register_tools(dsl) pointed at ../../07_the_run_dsl, call boukensha.repl(register=register_tools), drop task/FINAL RESPONSE prints)
  lib/boukensha/__init__.py              (edit: import version + Repl; add top-level repl() function; extend __all__)
  lib/boukensha/version.py               (new: VERSION = "0.8.0")
  lib/boukensha/repl.py                  (new)
  lib/boukensha/agent.py                 (edit: persist final reply to context at all 3 return points)
  lib/boukensha/client.py                (edit: specific ApiError message for HTTP 401)
  lib/boukensha/config.py                (edit: _resolve_dir gains <cwd>/.boukensha middle tier)
  lib/boukensha/context.py               (edit: add clear_messages())
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/run_dsl.py               (unchanged, already correct)
  lib/boukensha/prompt_builder.py        (unchanged, already correct)
  lib/boukensha/logger.py                (unchanged, already correct — turn() already existed, unused, since 07)
  lib/boukensha/errors.py                (unchanged, already correct)
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

Plus a launcher at `week1_baseline/bin/python/08_the_repl_loop`
(executable bit set), matching `bin/python/07_the_run_dsl`'s shape
exactly (including the `BOUKENSHA_DIR` export, mirroring
`bin/ruby/08_the_repl_loop`'s own export):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

export BOUKENSHA_DIR="$REPO_ROOT/.boukensha"

cd "$SCRIPT_DIR/../../python/08_the_repl_loop"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency (stdin EOF handling and Ctrl-C both use
Python stdlib primitives with no third-party equivalent needed).

## Behavior parity checklist (from the real Ruby output)

- [x] `version.py` defines `VERSION = "0.8.0"`
- [x] `Context.clear_messages()` resets `self.messages` to `[]`,
      leaving `self.tools` untouched
- [x] `Config._resolve_dir()`: explicit `BOUKENSHA_DIR` env var wins;
      else `<cwd>/.boukensha` if that directory exists; else
      `~/.boukensha`
- [x] `Client.call()` raises `ApiError("authentication failed (401) —
      check your API key")` specifically on a 401 response, and the
      existing generic message for every other non-2xx, non-retryable
      code
- [x] `Agent.run()`'s completed-turn path, and both `_wrap_up` return
      paths (normal and the `ApiError` fallback), call
      `self.context.add_message("assistant", text)` (or the fallback
      message) before returning
- [x] `Repl(context=, registry=, builder=, client=, logger=,
      config_dir=None, provider=None, model=None, version=None,
      api_key=None, task_settings=None, max_iterations=None,
      max_output_tokens=None)`:
  - [x] `PROMPT = "boukensha> "`
  - [x] `start()` prints the banner once, then loops: print `PROMPT`
        with no trailing newline (flushed), read a line from stdin,
        stop on EOF, strip the line, skip empty lines
  - [x] recognizes `/exit`/`/quit` (prints `"Goodbye."`, stops the
        loop), `/help` (prints the command list), `/quiet` (calls
        `boukensha.quiet()`, prints the suppressed-logging notice),
        `/loud` (calls `boukensha.loud()`, prints the enabled notice),
        `/clear` (calls `context.clear_messages()`, resets the turn
        counter to 0, prints the cleared notice) — none of these reach
        the agent
  - [x] any other non-empty input becomes a turn: increments the turn
        counter, calls `logger.turn(n=...)`, adds it as a `"user"`
        message, builds a **new `Agent`** each turn from the shared
        `context`/`registry`/`builder`/`client`/`logger` (+
        `task_settings`/limits), runs it, prints a blank line then the
        result
  - [x] `LoopError`/`ApiError` raised during a turn are caught and
        printed as `"\n[error] ..."` (`LoopError`: bare message;
        `ApiError`: `"API call failed: {message}"`) — the REPL keeps
        running afterward
  - [x] banner shows: boxed `BOUKENSHA MUD Assistant (v{version})`
        header (version padding preserved), `config:` line (the
        resolved dir, or `"{dir or '(default)'}  ✗ directory not
        found"` if it doesn't exist), `provider:` line (`"{provider or
        'default'} ({model or 'default'})  {key status}"`, where key
        status is `"✓ API key set"`/`"✗ API key not set"` based on
        whether `api_key` is `None`/blank), and the three command
        hint lines
- [x] `boukensha.repl(*, system=None, model=None, backend=None,
      api_key=None, ollama_host="http://localhost:11434", log=None,
      max_output_tokens=None, register=None)`:
  - [x] same config/system/model/backend/api_key resolution as
        `boukensha.run()`
  - [x] builds `Context`/`Registry` before calling `register`
  - [x] calls `register(RunDSL(registry))` if given
  - [x] builds the backend, `PromptBuilder`, `Client`,
        `effective_max_iterations`/`effective_max_output_tokens`,
        `Logger` exactly like `boukensha.run()`
  - [x] builds a `Repl` with all the extra REPL-only fields
        (`config_dir=cfg.dir, provider=backend, model=model,
        version=VERSION, api_key=api_key, task_settings=...,
        max_iterations=..., max_output_tokens=...`) and calls
        `.start()` instead of seeding a task message and calling
        `agent.run()` once
  - [x] catches `KeyboardInterrupt` around the whole body, printing
        `"\nInterrupted."`
  - [x] closes the logger in a `finally`, same guard pattern as
        `boukensha.run()`
- [x] `examples/example.py` prints the same `Config: ...` line +
      blank line, registers `read_file`/`list_directory` (pointed at
      the `07_the_run_dsl` sibling directory, `list_directory` sorted)
      via `boukensha.repl(register=...)`, with no task/FINAL RESPONSE
      prints — the REPL itself prints each turn's reply

Expected output (verified by actually running
`week1_baseline/bin/ruby/08_the_repl_loop` from the repo root against
the real `.boukensha/` fixture, with a scripted multi-turn stdin
conversation piped in — real, billed Anthropic API calls, confirmed
with the user before running, same precedent as
`04_api_client.md`/`05_agent_loop.md`/`06_the_logger.md`/
`07_the_run_dsl.md`):

Command:
```
printf 'list the files in the lib directory\nnow read lib/boukensha/agent.rb and briefly explain the loop\n/clear\nwhat was the first file I asked you about?\n/exit\n' | bin/ruby/08_the_repl_loop
```

Output:
```
Config: #<Boukensha::Config dir=/Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha tasks=player>


╔══════════════════════════════════════╗
║  BOUKENSHA MUD Assistant (v0.8.0)    ║
╚══════════════════════════════════════╝
  config:    /Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha
  provider:  anthropic (claude-haiku-4-5)  ✓ API key set

  /quiet or /loud   toggle logging
  /clear           reset conversation history
  /exit or /quit    leave the REPL

boukensha> 
The `lib` directory contains:

1. **boukensha** (directory)
2. **boukensha.rb** (file)

Would you like me to explore further or examine any of these files?
boukensha> 
## Brief Explanation of the Main Loop

The `run` method contains an infinite loop that executes an agent's **action cycle** repeatedly:

1. **Check Limits**: If the iteration limit is reached, trigger a wind-down and exit
2. **Increment Counter**: Increment the iteration counter
3. **Make API Call**: Send the current context (messages and available tools) to the AI client
4. **Parse Response**: Parse the response to determine what the model wants to do
5. **Handle Response**: 
   - If the model wants to use a tool, execute the tool calls and add results back to context (loop continues)
   - If the model is done (no tool use), extract text response and return (loop exits)

The key idea is that it's a **feedback loop**: the agent makes API calls, executes tools based on the response, feeds results back into the context, and repeats until either it completes its task naturally or hits iteration/token limits. When limits are hit, a special "wrap-up" call produces a final summary before exiting.
boukensha> (conversation history cleared)
boukensha> 
I don't have any record of previous conversations with you. Each conversation starts fresh, and I don't have access to chat history from past sessions.

If you'd like me to help you with a file now, please let me know:
- The file path you're interested in
- What you'd like to do with it (read, list directory contents, etc.)

I'll be happy to assist!
boukensha> Goodbye.
```

This confirms: the banner renders correctly (config dir found, API
key set), `read_file`/`list_directory` tool calls against the
`07_the_run_dsl` sibling dir work, `/clear` visibly resets history
(the model has no memory of "the first file" after clearing, matching
`Context#clear_messages!` wiping `@messages`), and `/exit` prints
"Goodbye." and terminates cleanly. Live model text will differ between
runs (expected, same as every prior step).

## Verification

Ran `week1_baseline/bin/python/08_the_repl_loop` for real against the
same fixture (confirmed with the user first, same as the Ruby run —
real, billed Anthropic API calls), piping the same scripted multi-turn
stdin conversation used above (Python tool paths point at the
`python/07_the_run_dsl` sibling; the model text and exact directory
listing differ from Ruby's transcript because Python's `lib/` only
contains the `boukensha` package directory, not a top-level
`boukensha.py` file — expected, not a bug). Output matched
structurally and semantically: banner, `/clear` visibly wiping history
(the model has no memory of "the first file" afterward, confirming
`Context.clear_messages()` behaves like `Context#clear_messages!`),
and a clean `/exit` printing `"Goodbye."`.

Additional checks:
- `/help`, `/quiet`, `/loud` produce byte-identical text to Ruby's
  `HELP` constant and toggle messages.
- Piping input with no `/exit`/`/quit` (EOF via a closed stdin pipe)
  breaks the loop silently with no `"Goodbye."`, matching Ruby's
  `break unless input` (no message on EOF, only on explicit
  `/exit`/`/quit`).
- `diff` of the two languages' output for a `/exit`-only session
  (`printf '/exit\n' | bin/ruby/08_the_repl_loop` vs. the Python
  equivalent) is **byte-for-byte identical**, confirming the banner,
  config-dir resolution, provider/API-key-status line, and command
  hints all render identically character-for-character (including the
  version-padding math and box-drawing characters).
- Read the session JSONL written by the Python run
  (`.boukensha/sessions/20260724T053849Z-14eb1475.jsonl`): `turn`
  events show `n=1` for the first post-`/clear` turn, confirming the
  REPL's turn counter resets to 0 on `/clear` exactly like Ruby's
  `@turn = 0`.

All behavior parity checklist items above are checked off against
these real runs plus a direct read of the implemented `Repl`/`repl()`/
`Agent`/`Client`/`Config`/`Context` source against the Ruby source line
by line.

## Porting notes (Ruby idiom → Python)

- **`Boukensha.repl(&block)` → `boukensha.repl(*, register=None)`,
  same shape as `boukensha.run()`.** Already-settled precedent from
  `07_the_run_dsl.md` Decision 2 (`register` callback receiving a
  `RunDSL`) — no new decision needed, just reused verbatim for the
  second entry point.
- **`Repl#start`'s `loop do ... $stdin.gets ... break unless input`
  → a `while True` loop reading `sys.stdin.readline()`, breaking on
  `""`.** Ruby's `IO#gets` returns `nil` at EOF and a `"\n"`-terminated
  string otherwise; Python's `sys.stdin.readline()` returns `""` at
  EOF and a `"\n"`-terminated string otherwise — the empty-string
  check is the direct structural equivalent of Ruby's `nil` check,
  no `try/except EOFError` machinery needed (which is what a
  `input()`-based translation would require instead).
- **`case input when "/exit", "/quit" ... end` → chained `if`/`elif`
  string comparisons.** Python has no `case/when` multi-value match
  as concise as Ruby's here in a way that reads better than explicit
  `if input in ("/exit", "/quit")`; kept as a flat `if/elif` chain,
  matching the flat control-flow style already used for backend
  dispatch in `boukensha.run()`/`boukensha.repl()`.
- **Heredoc `banner`/`HELP` strings → f-string-built multi-line
  strings, reproducing the exact blank-line/newline count.** Ruby's
  `<<~BANNER` squiggly heredoc strips common leading whitespace and
  preserves each literal blank line; `puts` on a string already ending
  in `\n` does not add a second one. The Python `banner()` builds a
  list of lines (including empty strings for the blank lines) and
  joins with `"\n"`, then a single `print(...)` (which appends exactly
  one trailing `\n`) reproduces the identical blank-line placement
  verified in the real transcript above (two blank lines between
  `Config: ...` and the box; one blank line after the command hints,
  right before the first `boukensha> ` prompt).
- **`" " * (9 - ver.length)` version-padding → `" " * (9 -
  len(ver))`.** Direct translation; Python's `str.__mul__`/`len`
  behave identically to Ruby's `String#*`/`#length` for this ASCII
  version string.
- **`rescue LoopError => e` / `rescue ApiError => e` inside
  `run_turn` → `except LoopError as e: / except ApiError as e:`,
  same two-branch structure**, printing `f"\n[error] {e}"` and
  `f"\n[error] API call failed: {e}"` respectively — direct
  translation, no idiom gap.
- **`Boukensha.quiet!`/`Boukensha.loud!` module functions → already-
  existing `boukensha.quiet()`/`boukensha.loud()` top-level functions**
  (present since `python/00_config`, per the already-ported
  `_quiet` global in `__init__.py`) — the REPL's `/quiet`/`/loud`
  commands call these directly, no new plumbing.
- **`rescue Interrupt` around `Boukensha.repl`'s whole body → `except
  KeyboardInterrupt` around `boukensha.repl()`'s whole body.** Direct
  translation — Python raises `KeyboardInterrupt` on Ctrl-C the same
  way Ruby raises `Interrupt`, and both are ordinary catchable
  exceptions here (not signal handlers).
- **`Agent#run`/`_wrap_up`'s three added `@context.add_message(:assistant,
  ...)` calls → `self.context.add_message("assistant", ...)` at the
  matching three return points in `agent.py`.** Direct translation,
  no idiom gap — same pattern already used for `"user"`/`"tool_result"`
  messages elsewhere in `agent.py`.
- **`response.code.to_i == 401` → `e.code == 401`.** Ruby's
  `Net::HTTPResponse#code` is a string requiring `.to_i`; Python's
  `urllib.error.HTTPError.code` is already an `int`, so no coercion
  needed. The check is inserted in the same relative position as Ruby:
  after the retryable-status check has already decided not to retry
  (so a 401 — never in `RETRYABLE_STATUS_CODES` — always falls
  through to this check), before the generic fallback `ApiError`.
- **`resolve_dir`'s new `Dir.pwd`/`Pathname#directory?` middle tier →
  `os.getcwd()`/`os.path.isdir()`.** Direct translation; Ruby's
  `Pathname.new(Dir.pwd).join(".boukensha")` is exactly
  `os.path.join(os.getcwd(), ".boukensha")` in Python (both already
  absolute, no `expand_path`/`abspath` needed for this branch,
  matching Ruby's own `cwd_dir.to_s` not calling `expand_path` either).

## Decisions

1. **Ran the real Ruby step for a verified transcript, confirmed with
   the user first** — same precedent as `04_api_client.md`/
   `05_agent_loop.md`/`06_the_logger.md`/`07_the_run_dsl.md`: real,
   billed Anthropic API calls, explicit go-ahead given before piping a
   scripted multi-turn stdin conversation into
   `bin/ruby/08_the_repl_loop`.

2. **`boukensha.repl()` reuses the `register` callback shape from
   `boukensha.run()` unchanged** (no new AskUserQuestion needed) —
   `07_the_run_dsl.md` Decision 2 already settled the only genuinely
   ambiguous part of this idiom (block → callback), and `repl()`'s
   signature otherwise mirrors `run()`'s parameter-resolution logic
   line for line, so the same shape applies directly.

3. **REPL's stdin-read loop uses `sys.stdin.readline()` returning
   `""` on EOF, not `input()` raising `EOFError`.** `readline()`'s
   empty-string-at-EOF return value is a direct structural match for
   Ruby's `IO#gets` returning `nil` at EOF (`break unless input`
   becomes `if line == "": break`), avoiding a `try/except EOFError`
   wrapper that would diverge from the Ruby control flow's shape.

4. **Banner/help text built as joined line lists rather than a single
   triple-quoted string.** A plain Python triple-quoted string would
   require matching leading-whitespace stripping done manually anyway
   (no direct `<<~` squiggly-heredoc equivalent); building an explicit
   list of lines (including blank-line entries) makes the exact
   blank-line count verified in the real transcript self-evident at
   the call site, rather than relying on invisible whitespace in a
   triple-quoted block.

5. **`bin/python/08_the_repl_loop` launcher** — added per the repo's
   `bin/<language>/<step>` convention, copying `bin/python/07_the_run_dsl`
   exactly (including the `BOUKENSHA_DIR` export), since
   `examples/example.py` for this step drops its own
   `os.environ.setdefault(...)` BOUKENSHA_DIR fallback to match
   `example.rb`'s equivalent removal (now relies solely on the
   launcher's export, same as the Ruby side at this exact step).

6. **No new dependency.** `repl.py` uses only `sys` (for
   `sys.stdin`) from the standard library; every other change is
   pure-Python control flow and string building. Ruby's
   `Gemfile`/`Gemfile.lock` are unchanged this step (`diff -rq`
   confirms no gem was added), so there's no Ruby-side dependency to
   mirror either.

## Open questions

None outstanding — all decided above.
