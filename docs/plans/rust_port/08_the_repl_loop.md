# 08 · The REPL Loop (Rust port)

## Goal

Ports `python/08_the_repl_loop` (and its Ruby ground truth,
`ruby/08_the_repl_loop`) into `rust/08_the_repl_loop`. This step adds
a second top-level entry point, `boukensha::repl::<T: Task>`, that
reuses `run<T>`'s config/system/model/backend/api_key resolution and
`Context`/`Registry`/backend/`PromptBuilder`/`Client`/`Logger` wiring,
but instead of seeding one task message and calling `Agent::run()`
once, it builds a new `Repl<T>` and hands control to its interactive
loop. Conversation history now accumulates across turns (persisted via
`Agent`'s three return points calling `context.add_message("assistant",
...)`, new this step), and a handful of built-in `/`-commands
(`/help`, `/quiet`, `/loud`, `/clear`, `/exit`, `/quit`) are handled
locally instead of reaching the agent.

`rust/08_the_repl_loop` was created as an exact copy of
`rust/07_the_run_dsl` (confirmed via `diff -rq rust/07_the_run_dsl
rust/08_the_repl_loop` immediately after `cp -r` — no output), still
under the stale `boukensha_07_the_run_dsl` package name.

Diffing `python/07_the_run_dsl` → `python/08_the_repl_loop` (ignoring
`__pycache__`) shows the only real changes are:

- `lib/boukensha/version.py` (new — `VERSION = "0.8.0"`)
- `lib/boukensha/repl.py` (new — `Repl`)
- `lib/boukensha/__init__.py` (adds the top-level `repl()` function;
  imports/exports `Repl`, `VERSION`)
- `lib/boukensha/agent.py` (persists the final reply to `context` at
  all three return points)
- `lib/boukensha/client.py` (a specific `ApiError` message on HTTP 401)
- `lib/boukensha/config.py` (`_resolve_dir()` gains a `<cwd>/.boukensha`
  middle tier)
- `lib/boukensha/context.py` (adds `clear_messages()`)
- `examples/example.py`, `README.md`

Python is the direct reference; Ruby (`ruby/08_the_repl_loop`) is the
ultimate spec where the two disagree — read directly for this step
(`repl.rb`, `version.rb`, `boukensha.rb`'s `self.repl`, `agent.rb`,
`client.rb`, `config.rb`, `context.rb`, `examples/example.rb`) to
confirm exact banner/HELP text, command dispatch, and the
`rescue Interrupt`/`ensure` wrapping. Ruby's own `README.md` title
(`# Step 7 — The REPL Loop`) and "New primitives" command table
(omits `/quiet`/`/loud` even though `HELP` and the worked transcript
both use them) are stale drift, same category
`02_the_registry.md`/`05_agent_loop.md`/`06_the_logger.md`/
`07_the_run_dsl.md` already documented — the directory,
`VERSION = "0.8.0"`, and launcher all agree it's step 8; this plan and
a real verified transcript (captured earlier this session via
`docs/plans/python_port/08_the_repl_loop.md`, reused below rather than
re-run a third time against an unchanged fixture) are ground truth.

## Source files to port

| File | Role |
|---|---|
| `python/08_the_repl_loop/lib/boukensha/repl.py`, `ruby/08_the_repl_loop/lib/boukensha/repl.rb` | `Repl`: interactive session loop. `PROMPT`/`HELP` constants; `start()` prints the banner then loops reading stdin, dispatching built-in commands or calling `run_turn`; `run_turn` builds a **fresh `Agent` every turn** from shared state, runs it, prints the result, and turns an API failure into a printed `[error] ...` line instead of propagating |
| `python/08_the_repl_loop/lib/boukensha/version.py`, `ruby/08_the_repl_loop/lib/boukensha/version.rb` | `VERSION = "0.8.0"` |
| `python/08_the_repl_loop/lib/boukensha/__init__.py`, `ruby/08_the_repl_loop/lib/boukensha.rb` | `boukensha.repl()`/`Boukensha.repl`: same resolution/wiring as `run()`, then builds `Repl` (passing `task_settings`, resolved limits, `config_dir`, `provider`, `model`, `version`, `api_key`) and calls `.start()` instead of seeding a task message and calling `Agent::run()` once; wraps the whole call in Ctrl-C handling with the same unconditional logger-close guarantee as `run()` |
| `python/08_the_repl_loop/lib/boukensha/agent.py`, `ruby/08_the_repl_loop/lib/boukensha/agent.rb` | The completed-turn return and both `wrap_up`/`_wrap_up` return paths now persist the reply to `context` before returning it |
| `python/08_the_repl_loop/lib/boukensha/client.py`, `ruby/08_the_repl_loop/lib/boukensha/client.rb` | A specific `ApiError`/`ApiError` message (`"authentication failed (401) — check your API key"`) on an HTTP 401 response, checked after the retryable-status branch, before the generic failure message |
| `python/08_the_repl_loop/lib/boukensha/config.py`, `ruby/08_the_repl_loop/lib/boukensha/config.rb` | `resolve_dir`/`_resolve_dir` becomes a 3-tier lookup: explicit `BOUKENSHA_DIR` env var, then `<cwd>/.boukensha` if that directory exists, then `~/.boukensha` |
| `python/08_the_repl_loop/lib/boukensha/context.py`, `ruby/08_the_repl_loop/lib/boukensha/context.rb` | Adds `clear_messages()`/`clear_messages!`, wiping history while keeping tools registered |
| `python/08_the_repl_loop/examples/example.py`, `ruby/08_the_repl_loop/examples/example.rb` | Rewritten: points `base_dir` at the `07_the_run_dsl` sibling directory (a playground with real source to read/list), registers `read_file`/`list_directory` via the `register` callback, calls `boukensha.repl(register=...)`/`Boukensha.repl { ... }` — no task string, no printed FINAL RESPONSE block, since the REPL itself prints each turn's reply |

## Runtime fixture to reuse

`.boukensha/` at the repo root, unchanged: `settings.yaml` configures
`tasks.player` for `anthropic`/`claude-haiku-4-5`.

Verified transcript reused from this same session's live Python and
Ruby runs (`docs/plans/python_port/08_the_repl_loop.md`), each piping
the same scripted multi-turn stdin conversation:

```
printf 'list the files in the lib directory\nnow read lib/boukensha/agent.<ext> and briefly explain the loop\n/clear\nwhat was the first file I asked you about?\n/exit\n' | bin/<lang>/08_the_repl_loop
```

Structurally: banner renders (config dir found, API key set), a
`read_file`/`list_directory` tool-using turn, a second tool-using turn
explaining the agent loop, `/clear` visibly wiping history (the model
has no memory of "the first file" afterward), and a clean `/exit`
printing `"Goodbye."`. The Rust run is compared structurally against
this same shape (same banner text verified byte-for-byte between Ruby
and Python already; same command behaviors), not by re-running a third
round of billed calls purely to re-derive an already-verified
transcript — the new, Rust-specific things to verify are the banner
byte-shape, `/clear`'s turn-counter reset, and the new Ctrl-C handler,
plus a fresh real run to catch any Rust-specific runtime surprise.

## Decisions (confirmed)

1. **`repl()`'s block-replacement reuses the Python port's already-
   confirmed `register`-callback design**, same reasoning as
   `07_the_run_dsl.md` Decision 1 — this is the same construct
   (`RunDSL`/`RunDsl` receiving tool registrations), already settled
   for `run()`/`run<T>` in this exact codebase; `repl<T>` reuses
   `RunDsl<T>` unchanged rather than inventing a second DSL type.
2. **`repl<T: Task>` stays generic over the task type**, matching
   `run<T>` and every other core type already generic over `T`
   (`Context<T>`, `Registry<T>`, `Agent<'a, T>`, `PromptBuilder<T>`,
   `Repl<'a, T>` — see Decision 7) — same reasoning as
   `07_the_run_dsl.md` Decision 2, continuing `01_struct_skeleton`'s
   Decision 8.
3. **Ctrl-C is handled with the `ctrlc` crate, confirmed with the
   user via `AskUserQuestion`** (genuinely ambiguous — no earlier Rust
   step has an interactive loop, so no precedent settled this).
   `std` has no cross-platform SIGINT interception at all; the two
   real alternatives were (a) accept the OS's default abrupt-kill
   behavior as a documented capability reduction, or (b) add `ctrlc`
   (a small, widely-used crate) for behavioral parity with Ruby's
   `rescue Interrupt` / Python's `except KeyboardInterrupt`. The user
   chose (b). Implementation: `boukensha::repl<T>` calls
   `ctrlc::set_handler(...)` once, installing a handler that prints
   `"\nInterrupted."`, flushes stdout, and calls
   `std::process::exit(0)` directly from the handler — rather than
   setting a flag for the main loop to observe. This is necessary, not
   just simpler: the main loop is blocked in a synchronous
   `stdin.read_line()` call, `ctrlc`'s handler runs on a signal-handling
   thread with no safe way to hand a `&mut Logger`/`&mut Repl` across
   without `Arc<Mutex<_>>` machinery this step doesn't otherwise need,
   and — critically — `Logger::write_log` already calls
   `self.log_io.flush()` after **every** event (fixed since
   `06_the_logger`), so `Logger::close()` is already redundant with
   per-write flushing; skipping it in the interrupt path changes no
   observable on-disk state, only the (Ruby/Python-shaped)
   `ensure`/`finally` call itself, which was already a no-op flush.
4. **`ctrlc = "3"` is the only new dependency this step adds.** It
   fills a genuine `std` gap (no first-party SIGINT handling), same
   bar the project has applied to `ureq`/`dirs`/`dotenvy`/`serde_yaml_ng`
   in earlier steps (`ITERATIONS.md`'s stdlib-first default is not a
   blanket ban — it requires each new crate to close a real capability
   gap, not just be more convenient than a `std`-only workaround; there
   is no `std`-only workaround here at all).
5. **`Agent::run()`'s three added `context.add_message("assistant",
   ...)` calls are direct translations** of Ruby's/Python's matching
   additions — `text.clone()`/`msg.clone()` passed to `add_message`
   before the owned `String` moves into the `Ok`/return value, since
   `Context::add_message` takes ownership of its `content` parameter
   (`impl Into<String>`) while the caller still needs the original to
   return.
6. **`Repl::run_turn`'s error handling has exactly one match arm
   (`ApiError`), not Ruby's/Python's two (`LoopError`/`ApiError`).**
   `Agent::run()`'s Rust signature is `Result<String, ApiError>` — the
   type system already statically guarantees no other error variant
   can occur, so there is no `LoopError`-shaped branch to write. This
   is not a capability reduction: `errors::LoopError` is defined
   (carried forward unchanged from `05_agent_loop`) but never
   constructed or returned anywhere in the Rust port, exactly mirroring
   Ruby's/Python's own `LoopError` being defined but never raised —
   the Ruby/Python second `rescue`/`except` clause is provably dead
   code in every language's actual implementation, not just Rust's.
7. **`Repl<'a, T: Task>` holds `&'a mut Registry<T>`, `&'a mut
   Logger`, `&'a PromptBuilder<T>`, and `&'a Client<'a, T>`** — the
   same four primitives `run<T>` borrows into `Agent::new`, just held
   across the whole session instead of a single call. `run_turn`
   builds a **fresh `Agent`** each turn via reborrows
   (`&mut *self.registry`, `&mut *self.logger`), matching Ruby's/
   Python's own explicit "fresh Agent every turn, shared Context"
   design (see `repl.rb`'s doc comment) — this is a straightforward
   mechanical translation, not a new design choice, since `Agent<'a, T>`
   already required exactly these four borrowed fields since
   `05_agent_loop`.
8. **`repl<T>` returns `Result<(), RunError>`**, reusing `RunError`
   unchanged (no new error variant) for the same setup-time failures
   `run<T>` can hit (`T::model`/`T::provider` returning `ConfigError`,
   an unsupported model, an unrecognized backend string) — even though
   the REPL loop itself produces no return *value* the way `run()`
   returns the agent's final text. `Repl::start()` cannot itself fail
   (turn-level `ApiError`s are caught and printed inside `run_turn`,
   never propagated), so once a `Repl` is successfully constructed,
   `logger.close()` runs unconditionally as the next statement after
   `.start()` returns — same "no `ensure`/`finally`/`Drop` needed,
   Rust's linear control flow already reaches cleanup on every path"
   reasoning as `07_the_run_dsl.md`'s matching decision for `run<T>`.
9. **`Config::resolve_dir`'s new `<cwd>/.boukensha` tier uses
   `std::env::current_dir()` + `PathBuf::is_dir()`** — direct
   translation of Ruby's `Dir.pwd`/`Pathname#directory?` and Python's
   `os.getcwd()`/`os.path.isdir()`; no `expand_path`/`absolute()` call
   needed for this branch since `current_dir()` already returns an
   absolute path, matching Ruby's own `cwd_dir.to_s` not calling
   `expand_path` either.
10. **`version.rs` is a new module exporting `pub const VERSION: &str
    = "0.8.0"`**, mirroring `version.py`/`version.rb` file-for-file
    rather than inlining the literal into `lib.rs` or `repl.rs`,
    keeping the same one-fact-one-file shape the Ruby/Python sources
    use.
11. **`bin/rust/08_the_repl_loop` launcher and the root `Cargo.toml`
    workspace-membership edit** — added per the repo's
    `bin/<language>/<step>` convention and the Rust port's own
    per-step requirement, matching `bin/rust/07_the_run_dsl`'s shape.

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/08_the_repl_loop")
week1_baseline/rust/08_the_repl_loop/
  Cargo.toml                                    (edit: package name → boukensha_08_the_repl_loop; + ctrlc = "3")
  src/
    lib.rs                                      (edit: + mod version; + mod repl; export VERSION/Repl; + repl::<T: Task>(...) -> Result<(), RunError>, installing the ctrlc handler)
    version.rs                                   (new: pub const VERSION: &str = "0.8.0")
    repl.rs                                       (new: Repl<'a, T: Task>)
    agent.rs                                      (edit: + context.add_message("assistant", ...) at the 3 return points)
    client.rs                                     (edit: + 401-specific ApiError before the generic failure message)
    config.rs                                     (edit: resolve_dir gains the <cwd>/.boukensha middle tier)
    context.rs                                    (edit: + clear_messages())
    backends/*.rs, tasks/{mod,base,player}.rs,
    message.rs, prompt_builder.rs, registry.rs,
    run_dsl.rs, tool.rs, errors.rs, logger.rs      (unchanged — verified byte-identical to 07_the_run_dsl)
  prompts/system.md                               (unchanged)
  README.md                                       (rewrite: this step's own docs)
  examples/example.rs                             (rewrite: call repl::<Player>(...) with a register closure pointed at the ../07_the_run_dsl sibling crate dir; no task string, no FINAL RESPONSE block)
week1_baseline/bin/rust/08_the_repl_loop          (new launcher, matching bin/rust/07_the_run_dsl's shape)
```

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`$stdin.gets` / `sys.stdin.readline()` → `io::stdin().lock().read_line(&mut line)`,
  breaking the loop on `Ok(0)`.** Rust's `read_line` returns the byte
  count read; `0` means EOF, the same structural signal Ruby's `nil`
  and Python's `""` already encode — no new control-flow shape needed,
  just a different sentinel value.
- **Heredoc `banner`/`HELP` strings → `&[&str]`/line arrays joined with
  `"\n"`**, same approach as `docs/plans/python_port/08_the_repl_loop.md`
  Decision 4 (list-of-lines rather than a raw multi-line literal),
  reproducing the exact blank-line placement verified against the real
  Ruby/Python transcripts.
- **`case input when "/exit", "/quit" ... end` → `match trimmed { "/exit"
  | "/quit" => ..., "/help" => ..., ... }`.** Direct translation — Rust's
  `match` on `&str` with `|`-alternation patterns is at least as
  concise as Ruby's `case/when`, unlike the `run()`/`repl()` backend
  dispatch (`match backend_name.as_str()`) which already established
  string-match idioms in this codebase.
- **`Boukensha.quiet!`/`Boukensha.loud!` → already-existing
  `crate::quiet()`/`crate::loud()`** (present since `rust/00_config`,
  backing the same `QUIET: AtomicBool` global `lib.rs` already
  exposes) — the REPL's `/quiet`/`/loud` commands call these directly,
  no new plumbing.
- **`rescue Interrupt` / `except KeyboardInterrupt` → `ctrlc::set_handler`
  installed once inside `repl<T>`**, per Decision 3 above.
- **`response.code.to_i == 401` → `status == 401`.** `ureq`'s
  `response.status().as_u16()` (already extracted as `status: u16` in
  `client.rs`) needs no coercion, unlike Ruby's string `#code` field.

## Behavior parity checklist

- [x] `version::VERSION` is `"0.8.0"`
- [x] `Context::clear_messages()` clears `self.messages`, leaving
      `self.tools` untouched
- [x] `Config::resolve_dir()`: explicit `BOUKENSHA_DIR` wins; else
      `<cwd>/.boukensha` if that directory exists; else `~/.boukensha`
- [x] `Client::call()` returns `ApiError("authentication failed (401)
      — check your API key")` specifically on a 401 response, and the
      existing generic message for every other non-2xx, non-retryable
      status
- [x] `Agent::run()`'s completed-turn path, and both `wrap_up` return
      paths (success and the `Err` fallback), call
      `self.registry.context.add_message("assistant", ...)` before
      returning
- [x] `Repl<'a, T: Task>` holds the same four borrowed primitives
      `Agent` needs, plus `config_dir`/`provider`/`model`/`version`/
      `api_key`/`task_settings`/`max_iterations`/`max_output_tokens`/
      `turn`
  - [x] `PROMPT = "boukensha> "`
  - [x] `start()` prints the banner once, then loops: prints `PROMPT`
        with no trailing newline (flushed), reads a line from stdin,
        stops on EOF (`Ok(0)`), trims the line, skips empty lines
  - [x] recognizes `/exit`/`/quit` (prints `"Goodbye."`, breaks),
        `/help` (prints the command list), `/quiet`/`/loud` (calls
        `crate::quiet()`/`crate::loud()`, prints the matching notice),
        `/clear` (calls `context.clear_messages()`, resets `turn` to
        0, prints the cleared notice) — none reach the agent
  - [x] any other non-empty input becomes a turn: increments `turn`,
        calls `logger.turn(n)`, adds it as a `"user"` message, builds
        a **new `Agent`** each turn via reborrows of the shared
        `registry`/`builder`/`client`/`logger`, runs it, prints a
        blank line then the result
  - [x] an `Err(ApiError)` from `agent.run()` is caught and printed as
        `"\n[error] API call failed: {e}"` — the REPL keeps running
        afterward
  - [x] banner shows the boxed `BOUKENSHA MUD Assistant (v{version})`
        header (version padding preserved), `config:`/`provider:`
        lines with the same fallback text as Ruby/Python, and the
        three command hint lines
- [x] `repl::<T: Task>(system, model, backend, api_key, ollama_host,
      log, max_output_tokens, register) -> Result<(), RunError>`:
  - [x] same config/system/model/backend/api_key resolution as `run<T>`
  - [x] builds `Context`/`Registry` before invoking `register`
  - [x] calls `register(&mut RunDsl::new(&mut registry))` if given
  - [x] builds the backend, `PromptBuilder`, `Client`,
        `effective_max_iterations`/`effective_max_output_tokens`,
        `Logger` exactly like `run<T>`
  - [x] installs the `ctrlc` handler (Decision 3) before building `Repl`
  - [x] builds a `Repl` with the extra REPL-only fields
        (`config_dir = Some(cfg.dir.clone())`, `provider =
        Some(backend_name.clone())`, `model = Some(model.clone())`,
        `version = Some(VERSION)`, `api_key`, `task_settings`,
        `max_iterations`, `max_output_tokens`) and calls `.start()`
        instead of seeding a task message and calling `agent.run()`
        once
  - [x] calls `logger.close()` unconditionally after `.start()`
        returns
- [x] `examples/example.rs` prints the same `Config: ...` line + blank
      line, registers `read_file`/`list_directory` (pointed at the
      `../07_the_run_dsl` sibling crate dir, `list_directory` sorted)
      via `repl::<Player>(..., register)`, with no task/FINAL RESPONSE
      prints
- [x] Root `week1_baseline/Cargo.toml`'s `members` includes
      `"rust/08_the_repl_loop"`
- [x] `rust/08_the_repl_loop/Cargo.toml`'s package name is
      `boukensha_08_the_repl_loop`, with `ctrlc = "3"` added
- [x] `bin/rust/08_the_repl_loop` exists, is executable, and runs
      `cargo run --quiet --example example` from the crate dir
- [x] `cargo build --workspace` compiles clean

Verified for real: `cargo build --workspace` compiled clean (no
errors, no warnings). `bin/rust/08_the_repl_loop` ran end-to-end
against the live `.boukensha/` fixture with the same scripted
multi-turn stdin conversation used for the Ruby/Python transcripts
(confirmed with the user first — real, billed Anthropic API calls,
same precedent as every prior real-run step):

```
printf 'list the files in the lib directory\nnow read src/agent.rs and briefly explain the loop\n/clear\nwhat was the first file I asked you about?\n/exit\n' | bin/rust/08_the_repl_loop
```

The banner rendered correctly (config dir found, API key set); the
model correctly reported no `lib` directory exists (the Rust example's
`base_dir` points at `../07_the_run_dsl`, a Cargo crate with `src/`,
not `lib/` — expected structural difference from the Ruby/Python
examples' own language-specific directory layout, not a bug);
`read_file` successfully read `src/agent.rs` and explained the loop;
`/clear` visibly wiped history (the model had no memory of "the first
file" afterward); `/exit` printed `"Goodbye."` and exited cleanly.

Additional checks:
- `printf '/exit\n' | bin/ruby/08_the_repl_loop` vs. the same for
  Rust: **byte-for-byte identical** output — banner, config-dir
  resolution, provider/API-key-status line, and command hints all
  render identically character-for-character, confirming the
  `format!`-based banner and the version-padding math exactly match
  Ruby's heredoc.
- `/help`, `/quiet`, `/loud` produce text identical to the verified
  Ruby/Python transcripts.
- Piping input with no `/exit`/`/quit` (stdin EOF) breaks the loop
  silently with no `"Goodbye."` and exit code 0, matching Ruby's
  `break unless input`.
- Read the session JSONL from the tool-using run: `turn` events show
  `n=1` for the first post-`/clear` turn, confirming `Repl::turn`
  resets to 0 on `/clear` exactly like Ruby's `@turn = 0` / Python's
  `self.turn = 0`.
- Ctrl-C handling verified directly: launched the compiled example
  binary with a live stdin pipe (via a Python subprocess harness, since
  a backgrounded shell job's stdin closes immediately and would exit
  the loop via EOF before a signal could be sent), sent `SIGINT` after
  2 seconds. Output ended with `"\nInterrupted.\n"` and the process
  exited with code `0` — confirming the `ctrlc` handler (Decision 3)
  works as designed.

All behavior parity checklist items above are checked off against
these real runs plus a direct read of the implemented `repl<T>`/
`Repl`/`Agent`/`Client`/`Config`/`Context` source against the Ruby/
Python source line by line.

One cosmetic implementation note not called out in the Rust idiom
choices above: `Repl::HELP` is a single `const &'static str` literal
with embedded `\n` escapes, not a runtime line-array-join like
`banner()` — `HELP` has no interpolated values, so a plain `const`
literal is the more idiomatic, zero-allocation Rust shape; `banner()`
uses the array-join approach because it *is* interpolating
(`config_line`/`provider_line`/`ver`) and needs to build the string at
call time.

## Open questions

None outstanding — all decided above.
