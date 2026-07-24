# 08 · The REPL Loop (Rust port)

Behavior port of `ruby/08_the_repl_loop` / `python/08_the_repl_loop` —
a second top-level entry point, `repl::<T>`, that reuses `run::<T>`'s
config/system/model/backend/api_key resolution and `Context`/
`Registry`/backend/`PromptBuilder`/`Client`/`Logger` wiring, but hands
control to an interactive `Repl<T>` loop instead of running once.
Conversation history now accumulates across turns, and a handful of
built-in `/`-commands are handled locally instead of reaching the
agent.

`message.rs`, `context.rs` (except the new `clear_messages()`),
`registry.rs`, `run_dsl.rs`, `prompt_builder.rs`, `tool.rs`,
`errors.rs`, `logger.rs`, `tasks/{mod,base,player}.rs`, and
`backends/*.rs` are unchanged from `07_the_run_dsl`; see
`../07_the_run_dsl/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/version.rs` | `pub const VERSION: &str = "0.8.0"` |
| `src/repl.rs` | `Repl<'a, T: Task>` — the interactive session loop |

## Updated Files

| File | Change |
|---|---|
| `src/lib.rs` | Adds `pub fn repl<T: Task>(...) -> Result<(), RunError>`; installs a `ctrlc` SIGINT handler; exports `Repl`, `VERSION` |
| `src/agent.rs` | Persists the final reply to `registry.context` at all three return points (the completed-turn path and both `wrap_up` paths), so a shared `Context` sees every reply, not just user turns and tool exchanges |
| `src/client.rs` | Returns a specific `ApiError("authentication failed (401) — check your API key")` on an HTTP 401 response, checked after the retryable-status branch |
| `src/config.rs` | `resolve_dir()` gains a middle tier: checks `<cwd>/.boukensha` before falling back to `~/.boukensha` |
| `src/context.rs` | Adds `clear_messages()`, wiping history while keeping tools registered |
| `Cargo.toml` | + `ctrlc = "3"` |

## repl::\<T: Task\>

```rust
repl::<Player>(
    None, None, None, None, None, None, None,
    Some(|dsl: &mut RunDsl<Player>| {
        dsl.tool("read_file", "Read a file from disk", params, |args| {
            std::fs::read_to_string(&args["path"]).map_err(|e| e.to_string())
        });
    }),
)?;
```

Same positional parameters as `run::<T>`, minus `task` — the user
supplies tasks interactively at the `boukensha>` prompt instead.
Returns `Result<(), RunError>`: the same setup-time failure surface as
`run::<T>` (config/model/backend resolution), since the REPL loop
itself never fails — turn-level `ApiError`s are caught and printed
inside the loop, not propagated.

## Built-in commands

| Command | Effect |
|---|---|
| `/clear` | Wipe conversation history (tools stay registered) |
| `/help` | Print the command list |
| `/quiet` | Suppress detailed logging |
| `/loud` | Re-enable logging |
| `/exit` / `/quit` | Leave the REPL |
| Ctrl-D | EOF — leave the REPL |
| Ctrl-C | Interrupt — prints `"Interrupted."` and exits |

## Before and after

| | Step 7 | Step 8 |
|---|---|---|
| Entry point | `run::<T>(task, ...)` | `repl::<T>(...)` |
| Turns | one | many |
| History | discarded | accumulates across turns |
| User interaction | none | stdin prompt |

## New Dependency: `ctrlc`

Ruby's `rescue Interrupt` / Python's `except KeyboardInterrupt` catch
Ctrl-C mid-loop for a graceful exit. Rust's `std` has no cross-platform
SIGINT handling at all — confirmed with the user (`AskUserQuestion`,
recorded in `docs/plans/rust_port/08_the_repl_loop.md` Decision 3)
before adding the small, widely-used `ctrlc` crate to close this gap,
rather than accepting an abrupt process kill. The handler prints
`"Interrupted."`, flushes stdout, and calls `std::process::exit(0)`
directly — it can't safely reach across into the REPL's borrowed
`Registry`/`Logger` from a signal-handling thread, and `Logger` already
flushes to disk after every write, so skipping `Logger::close()` from
the handler changes no observable state.

## Porting notes

- **`$stdin.gets` / `sys.stdin.readline()` → `stdin.lock().read_line(&mut
  line)`, breaking on `Ok(0)`.** Direct structural match for Ruby's
  `nil` / Python's `""` EOF sentinel.
- **Heredoc `banner` → a line array joined with `"\n"`**, reproducing
  the exact blank-line placement verified against the real Ruby/Python
  transcripts (`docs/plans/python_port/08_the_repl_loop.md` Decision 4
  used the same approach).
- **`repl()`'s block-replacement reuses `RunDsl<T>` unchanged** — same
  `register`-callback design already settled for `run<T>` in this
  codebase (`07_the_run_dsl.md` Decision 1).
- **`Repl<'a, T: Task>` holds the same four borrowed primitives
  `Agent::new` needs** (`&'a mut Registry<T>`, `&'a PromptBuilder<T>`,
  `&'a Client<'a, T>`, `&'a mut Logger`), building a **fresh `Agent`
  every turn** via reborrows (`&mut *self.registry`, `&mut
  *self.logger`) — matching Ruby's/Python's own explicit design.
- **`run_turn`'s error handling has one match arm (`ApiError`), not
  Ruby's/Python's two (`LoopError`/`ApiError`).** `Agent::run()`'s
  Rust signature is `Result<String, ApiError>` — the type system
  already rules out any other error variant. Not a capability
  reduction: `LoopError` is never constructed anywhere in the Rust
  port either, mirroring Ruby/Python never raising it.

See `docs/plans/rust_port/08_the_repl_loop.md` for the full decision
record.

## Run Example

```bash
./week1_baseline/bin/rust/08_the_repl_loop
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

Verified against the real Ruby/Python runs for this same fixture (see
`docs/plans/rust_port/08_the_repl_loop.md`).
