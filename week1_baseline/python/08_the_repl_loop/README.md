# 08 · The REPL Loop (Python port)

Behavior port of `ruby/08_the_repl_loop` — a second top-level entry
point, `boukensha.repl()`, that reuses the same `Context`/`Registry`/
backend/`PromptBuilder`/`Client`/`Logger` wiring as `boukensha.run()`
but hands control to an interactive `Repl` loop instead of running
once. Conversation history now accumulates across turns, and a
handful of built-in `/`-commands are handled locally instead of being
sent to the agent. `message.py`, `tool.py`, `registry.py`,
`run_dsl.py`, `prompt_builder.py`, `logger.py`, `errors.py`,
`tasks/*.py`, and `backends/*.py` are unchanged from `07_the_run_dsl`;
see `../07_the_run_dsl/README.md` for those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/version.py` | `VERSION = "0.8.0"` |
| `lib/boukensha/repl.py` | `Repl` — the interactive session loop |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/__init__.py` | Adds the top-level `repl()` entry point; imports and exports `Repl` and `VERSION` |
| `lib/boukensha/agent.py` | Persists the final assistant reply to `context` at all three return points, so a shared `Context` sees every reply, not just user turns and tool exchanges |
| `lib/boukensha/client.py` | Raises a specific `ApiError("authentication failed (401) — check your API key")` on an HTTP 401 response, instead of the generic failure message |
| `lib/boukensha/config.py` | `_resolve_dir()` gains a middle tier: checks `<cwd>/.boukensha` before falling back to `~/.boukensha` |
| `lib/boukensha/context.py` | Adds `clear_messages()`, wiping history while keeping tools registered |

## boukensha.repl()

```python
def register_tools(dsl):
    @dsl.tool(
        "read_file",
        description="Read a file from disk",
        parameters={"path": {"type": "string", "description": "File path"}},
    )
    def read_file(path):
        return Path(path).read_text()


boukensha.repl(register=register_tools)
```

Same options as `boukensha.run()`, minus `task` — the user supplies
tasks interactively at the `boukensha>` prompt instead.

## Built-in commands

| Command | Effect |
|---|---|
| `/clear` | Wipe conversation history (tools stay registered) |
| `/help` | Print the command list |
| `/quiet` | Suppress detailed logging |
| `/loud` | Re-enable logging |
| `/exit` / `/quit` | Leave the REPL |
| Ctrl-D | EOF — leave the REPL |
| Ctrl-C | Interrupt — leave the REPL gracefully |

## Before and after

| | Step 7 | Step 8 |
|---|---|---|
| Entry point | `boukensha.run(task="…")` | `boukensha.repl()` |
| Turns | one | many |
| History | discarded | accumulates across turns |
| User interaction | none | stdin prompt |

## No New Dependencies

`repl.py` uses only `sys` and `os` from the standard library.
`requirements.txt` is unchanged from `07_the_run_dsl` — matches
Ruby's own `Gemfile`/`Gemfile.lock`, which are unchanged in this step
too.

## Porting notes

- **`Repl#start`'s `$stdin.gets` / `break unless input` → `sys.stdin.readline()`,
  breaking on `""`.** Ruby's `IO#gets` returns `nil` at EOF; Python's
  `readline()` returns `""` at EOF — a direct structural match, no
  `try/except EOFError` needed.
- **Heredoc `banner`/`HELP` strings → line lists joined with `"\n"`.**
  Reproduces the exact blank-line placement Ruby's `<<~BANNER` squiggly
  heredoc produces, verified against a real transcript.
- **`Boukensha.repl(&block)` → `boukensha.repl(register=...)`,** same
  `register`-callback shape already established for `boukensha.run()`
  in `07_the_run_dsl` — no new decision needed.
- **`rescue Interrupt` around the whole method → `except KeyboardInterrupt`
  around the whole function**, with the same `finally: logger.close()`
  guard pattern as `boukensha.run()`.

See `docs/plans/python_port/08_the_repl_loop.md` for the full decision
record.

## Run Example

```bash
./week1_baseline/bin/python/08_the_repl_loop
```

This is an interactive REPL — each turn you type makes one or more
real HTTP requests to whichever provider `.boukensha/settings.yaml`
configures (Anthropic, by default in this repo's fixture). It costs a
small amount per model round-trip and requires a valid API key in
`.boukensha/.env`. It also writes a new
`.boukensha/sessions/<session-id>.jsonl` file.

Example output (the exact tool calls and final text are **not**
reproducible byte-for-byte — they're live model responses):

```
Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>

╔══════════════════════════════════════╗
║  BOUKENSHA MUD Assistant (v0.8.0)    ║
╚══════════════════════════════════════╝
  config:    /.../.boukensha
  provider:  anthropic (claude-haiku-4-5)  ✓ API key set

  /quiet or /loud   toggle logging
  /clear           reset conversation history
  /exit or /quit    leave the REPL

boukensha> list the files in the lib directory
...
boukensha> /clear
(conversation history cleared)
boukensha> /exit
Goodbye.
```

Verified against the real Ruby run for this same fixture (see
`docs/plans/python_port/08_the_repl_loop.md`).
