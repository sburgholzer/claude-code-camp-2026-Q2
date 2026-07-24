# 09 · Global Executable (Rust port)

## Goal

Ports `python/09_global_executable` (and its Ruby ground truth,
`ruby/09_global_executable`) into `rust/09_global_executable`. **This
is the official global executable going forward** — per user
direction, Rust gets the real packaging/install treatment (`cargo
install`, producing an actual `~/.cargo/bin/boukensha` binary), not
just a delta-port of the library code behind an `examples/` runner.

`rust/09_global_executable` was created as an exact copy of
`rust/08_the_repl_loop` (confirmed via `diff -rq rust/08_the_repl_loop
rust/09_global_executable` immediately after `cp -r` — no output),
still under the stale `boukensha_08_the_repl_loop` package name.

Diffing `python/08_the_repl_loop` → `python/09_global_executable`
(ignoring `__pycache__`/`.egg-info`) shows the only real changes are:

- `lib/boukensha/version.py` (`"0.8.0"` → `"0.9.0"`)
- `lib/boukensha/client.py` (drops the HTTP 401 special-case `ApiError`
  message added in step 8 — a regression, not a fix; ported as-is
  since Ruby is the spec)
- `lib/boukensha/config.py` (`_resolve_dir()` drops the
  `<cwd>/.boukensha` middle tier, replacing it with a
  `~/.boukensharc` `BOUKENSHA_DIR=...` tier via the new `boukensha_rc`
  module)
- `lib/boukensha/repl.py` (`_banner()` simplified: separate
  `config:`/`provider:`/`model:` lines, no API-key-status text, no
  config-dir-existence check)
- `lib/boukensha/__init__.py` (comment-only: `repl()`'s doc comment
  reworded)
- new: `lib/boukensha_loader.py`, `lib/boukensha_rc.py`,
  `pyproject.toml`
- `examples/` removed entirely; `README.md` rewritten

Python is the direct reference; Ruby (`ruby/09_global_executable`) is
the ultimate spec where the two disagree — read directly for this step
(`lib/boukensha_loader.rb`, `lib/boukensha_rc.rb`, `bin/boukensha`,
`boukensha.gemspec`, `lib/boukensha/{client,config,repl,version}.rb`)
to confirm exact resolution order and banner text. Both Ruby's and
Python's own "global executable" mechanisms (`gem install` /
`pip install -e`) were already built and verified for real earlier
this session (see `docs/plans/python_port/09_global_executable.md`);
this plan does the same for Rust, adapted to Rust's static-compilation
model (see Decisions 1–2 below).

## Source files to port

| File | Role |
|---|---|
| `python/09_global_executable/lib/boukensha_loader.py`, `ruby/09_global_executable/lib/boukensha_loader.rb` + `bin/boukensha` | Resolves `BOUKENSHA_PATH` (env → `~/.boukensharc` → bundled default), then boots the REPL with no tools registered. Rust's binary can only ever use the bundled default (see Decision 1) — so this becomes `src/bin/boukensha.rs`: read `BOUKENSHA_PATH`, print a note if set, then always call `repl::<Player>` on the bundled crate |
| `python/09_global_executable/lib/boukensha_rc.py`, `ruby/09_global_executable/lib/boukensha_rc.rb` | Parses `~/.boukensharc`: `KEY=VALUE` lines (`#`/blank ignored); a file with no `=` on any line is a legacy bare `BOUKENSHA_PATH` value |
| `python/09_global_executable/lib/boukensha/client.py`, `ruby/09_global_executable/lib/boukensha/client.rb` | Drops the HTTP 401 special-case `ApiError` — falls through to the generic non-retryable-status message |
| `python/09_global_executable/lib/boukensha/config.py`, `ruby/09_global_executable/lib/boukensha/config.rb` | `_resolve_dir`/`resolve_dir` becomes: `BOUKENSHA_DIR` env var → `boukensha_rc`'s `BOUKENSHA_DIR` → `~/.boukensha` default |
| `python/09_global_executable/lib/boukensha/repl.py`, `ruby/09_global_executable/lib/boukensha/repl.rb` | `_banner`/`banner`: three separate `config:`/`provider:`/`model:` lines, no API-key-status text, no config-dir-existence check |
| `python/09_global_executable/lib/boukensha/version.py`, `ruby/09_global_executable/lib/boukensha/version.rb` | `VERSION = "0.9.0"` |
| `python/09_global_executable/lib/boukensha/__init__.py`, `ruby/09_global_executable/lib/boukensha.rb` | Comment-only: `repl()`'s doc comment reworded |
| `python/09_global_executable/pyproject.toml`, `ruby/09_global_executable/boukensha.gemspec` + `Gemfile` | Packaging manifests — Rust's analog is `Cargo.toml`'s new `[[bin]]` target (see Decision 3) |
| `python/09_global_executable/lib/boukensha/{context,agent,message,tool,registry,run_dsl,prompt_builder,logger,errors}.py`, `tasks/*.py`, `backends/*.py` | Unchanged from `08_the_repl_loop` — carry forward as-is |

## Runtime fixture to reuse

Same `.boukensha/` fixture at the repo root as prior steps —
`settings.yaml`, `.env`, `prompts/` unchanged.

Reused the already-verified Ruby transcript from this session's
`docs/plans/python_port/09_global_executable.md` (a real, billed
Anthropic API call, confirmed with the user at the time) rather than
re-running Ruby a second time against an unchanged fixture:

```
$ printf 'Briefly, in one sentence, what is a MUD?\n/exit\n' \
    | BOUKENSHA_DIR="$REPO_ROOT/.boukensha" bundle exec ruby bin/boukensha

╔══════════════════════════════════════╗
║  BOUKENSHA MUD Assistant (v0.9.0)    ║
╚══════════════════════════════════════╝
  config:        /Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha
  provider:      anthropic
  model:         claude-haiku-4-5

  /quiet or /loud   toggle logging
  /clear           reset conversation history
  /exit or /quit    leave the REPL

boukensha> 
A MUD (Multi-User Dungeon) is a text-based multiplayer game where players explore fantasy worlds, fight monsters, solve puzzles, and interact with other players through typed commands.
boukensha> Goodbye.
```

A **new** real run was captured for Rust specifically (see Behavior
parity checklist / Verification below), since this step's `boukensha`
binary is exercised through a genuinely new code path (`src/bin/
boukensha.rs`, never run before) rather than just cosmetic changes to
already-verified library code.

## Decisions (confirmed)

1. **`BOUKENSHA_PATH` step-switching has no Rust equivalent — confirmed
   with the user via `AskUserQuestion` after discussion.** Ruby/Python
   can `require`/`import` a *different* step's code at runtime because
   both are dynamically loaded; Rust binaries are compiled
   ahead-of-time, so `boukensha` (this step's binary) can never load
   another step's compiled code without unsafe dynamic library loading
   (`dlopen`/FFI) — a fundamentally heavier architecture this project's
   stdlib-first philosophy wouldn't justify for a teaching convenience
   that already has a working alternative: every step already has its
   own `bin/rust/<step>` launcher (`bin/rust/04_api_client`, etc.),
   which is the real, already-working way to "run an older version of
   the Rust app." This is a genuine capability reduction, not a style
   choice — documented prominently in `README.md`'s "A genuine
   capability gap" section, not silently dropped.
2. **`BOUKENSHA_PATH` is still read (env var, then `~/.boukensharc`)
   solely to print a note, then the binary proceeds with its own
   bundled step regardless — confirmed with the user.** Rejected
   alternatives: hard-erroring (would make the binary *less* usable
   than just ignoring the variable) and silently ignoring it entirely
   (would leave a Ruby/Python user's habitual `BOUKENSHA_PATH=...
   boukensha` invocation silently do the wrong thing with no
   explanation). The note points at the real alternative
   (`bin/rust/<step>`). `BOUKENSHA_DIR` has no such gap — it only
   selects a config *directory* to read at runtime, fully portable —
   and gets the direct `env var → rc file → default` port with no
   caveats.
3. **The installed binary is named `boukensha` via an explicit
   `[[bin]] name = "boukensha"` target in `Cargo.toml`, while the
   package itself keeps the `boukensha_09_global_executable` name.**
   This is the Rust analog of Ruby's `spec.executables = ["boukensha"]`
   (gemspec) and Python's `[project.scripts] boukensha = ...`
   (pyproject.toml) — the product name is "boukensha" even though the
   crate/package that builds it follows this repo's own numbered
   convention. Kept the package name on-convention (`rust/README.md`'s
   documented `boukensha_NN_name` rule) rather than special-casing
   step 9 to be named bare `boukensha`, since `[[bin]] name` already
   fully decouples the installed command's name from the package name
   — no reason to break the one naming rule this workspace has for a
   cosmetic win.
4. **`src/rc.rs` is a plain library module (`pub mod rc` in `lib.rs`),
   not a sibling top-level file the way Python's `boukensha_rc.py`
   had to be.** Python needed `boukensha_rc.py`/`boukensha_loader.py`
   as siblings of the `boukensha/` package specifically because of how
   `pyproject.toml`'s `package-dir`/`py-modules` install both as
   separate top-level importables (mirroring Ruby's `lib/boukensha_rc.rb`
   sitting beside `lib/boukensha/`). Rust has no such packaging-driven
   need — the whole crate (library *and* binary) is one compilation
   unit, so `rc::read()` is just an ordinary function `config.rs`
   calls via `crate::rc::read()`, no structural mirroring required.
5. **`src/bin/boukensha.rs` calls `repl::<Player>(...)` with
   `None::<fn(&mut RunDsl<Player>)>` for the register parameter** — no
   tools registered, matching Ruby's `Boukensha.repl` and Python's
   `boukensha.repl()` being called with no block/`register` callback
   from their own loaders. `repl<T>`'s `register` parameter is
   `Option<impl FnOnce(&mut RunDsl<T>)>` (an implicit generic, not a
   trait object), so passing plain `None` doesn't type-check without
   an explicit type — `None::<fn(&mut RunDsl<Player>)>` supplies one
   using a zero-sized function-pointer type that's never actually
   constructed, the standard Rust idiom for "no closure, but the
   compiler still needs a concrete type for the generic."
6. **`Repl`'s `api_key` field is kept but marked `#[allow(dead_code)]`
   rather than removed**, exactly mirroring Ruby's `@api_key`/Python's
   `self.api_key` being equally unused dead state since `banner()`
   dropped its API-key-status line this step (see `docs/plans/
   python_port/09_global_executable.md`'s source-file table noting the
   same thing for Ruby's `repl.rb`) — consistent with this project's
   "port the behavior as observed, don't clean up while in the
   neighborhood" rule; Rust just needs an explicit annotation where
   Ruby/Python's dynamism lets a dead ivar pass silently.
7. **No new crate dependency.** `src/rc.rs` uses only
   `std::fs`/`std::collections::HashMap` plus the already-present
   `dirs` crate (`home_dir()`, already used by `config.rs` since
   `00_config`) — no `std` gap needs filling here.
8. **`examples/` and `bin/rust/09_global_executable` are both dropped
   — confirmed with the user via `AskUserQuestion`.** Mirrors Ruby
   exactly (no `bin/ruby/09_global_executable` either — the gem
   replaces that pattern) and Python's already-confirmed choice for
   this same step. The real entry point is `cargo install --path
   rust/09_global_executable`, documented in `README.md`.

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/09_global_executable")
week1_baseline/rust/09_global_executable/
  Cargo.toml                                    (edit: package name → boukensha_09_global_executable; + [[bin]] name = "boukensha", path = "src/bin/boukensha.rs")
  src/
    lib.rs                                      (edit: + pub mod rc; repl()'s doc comment reworded)
    rc.rs                                        (new: pub fn read() -> HashMap<String, String>)
    bin/
      boukensha.rs                                (new: the boukensha binary's main())
    version.rs                                    (edit: "0.8.0" → "0.9.0")
    client.rs                                     (edit: drop the 401-specific ApiError branch)
    config.rs                                     (edit: resolve_dir drops the <cwd>/.boukensha tier, adds the rc-based BOUKENSHA_DIR tier)
    repl.rs                                       (edit: banner() simplified; api_key field #[allow(dead_code)])
    agent.rs, context.rs, message.rs, tool.rs,
    registry.rs, run_dsl.rs, prompt_builder.rs,
    logger.rs, errors.rs,
    backends/*.rs, tasks/{mod,base,player}.rs      (unchanged — verified byte-identical to 08_the_repl_loop)
  prompts/system.md                               (unchanged)
  README.md                                       (rewrite: this step's own docs, incl. the BOUKENSHA_PATH capability-gap section)
  examples/                                       (removed)
week1_baseline/rust/README.md                     (edit: + "09_global_executable: a real boukensha binary" section)
```

No `bin/rust/09_global_executable` launcher (per Decision 8).

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`module BoukenshaRc; def self.read; ...; end; end` /
  `boukensha_rc.py`'s module-level `read()` → `pub fn read() ->
  HashMap<String, String>` in `rc.rs`.** Direct translation — Rust has
  no bare-module-with-singleton-methods concept distinct from a free
  function in a module, so this is the most literal shape available,
  consistent with `lib.rs`'s own `quiet()`/`loud()`/`config()` free
  functions already mirroring `Boukensha`'s Ruby module methods.
- **`File.exist?(main)` / `os.path.isfile(...)` checking for a step's
  loadable code → N/A.** Rust never checks "does this directory have
  loadable code," because it never attempts to load another
  directory's code at all (Decision 1) — there is no equivalent
  existence check to write.
- **`require main` / `sys.path.insert(0, step_lib); import boukensha`
  → nothing; the binary always statically links its own bundled
  `boukensha_09_global_executable` lib crate.** The closest analog of
  "switching what gets loaded" in Rust would be recompiling against a
  different crate entirely (i.e., using that step's own `bin/rust/
  <step>` launcher) — a build-time choice, not a runtime one.
- **`response.code.to_i == 401` / `e.code == 401` → simply deleted.**
  No idiom gap; this mirrors Ruby's real regression (dropping the
  step-8 addition), ported as-is per "Ruby is always the spec."
- **`gem install` / `pip install -e` → `cargo install --path`.**
  Builds a release binary and copies it into `~/.cargo/bin` — verified
  for real (see Behavior parity checklist) to behave like Ruby's/
  Python's global commands: same banner, same `BOUKENSHA_DIR`/
  `~/.boukensharc` resolution, runnable from any directory.
- **`Option<impl FnOnce(&mut RunDsl<T>)>` called with no closure →
  `None::<fn(&mut RunDsl<Player>)>`.** Rust's `impl Trait` in argument
  position is sugar for an unnamed generic type parameter; unlike a
  dynamically-typed `nil`/`None` in Ruby/Python, the compiler needs a
  concrete type to monomorphize `repl::<Player>` against even when no
  value is ever constructed — an explicit `fn` pointer type is the
  standard zero-cost choice here (no closure captures anything, so
  `fn(...)` rather than a boxed trait object is both correct and
  cheapest).

## Behavior parity checklist

- [x] `version.rs` defines `pub const VERSION: &str = "0.9.0"`
- [x] `Client::call`'s HTTP-failure branch no longer special-cases 401
      — falls through to the same generic `"API request failed after
      {attempts} attempt{suffix} ({status}): {body}"` as any other
      non-retryable, non-2xx code
- [x] `rc::read()`: parses `~/.boukensharc` into a `HashMap<String,
      String>` of `KEY=VALUE` pairs (`#`/blank lines ignored); a file
      with no `=` on any line is read as a legacy bare `BOUKENSHA_PATH`
      value; missing/empty file returns `{}`
- [x] `Config::resolve_dir()`: `BOUKENSHA_DIR` env var wins; else
      `rc::read().get("BOUKENSHA_DIR")`; else `DEFAULT_DIR`
      (`~/.boukensha`) — the step-8 `<cwd>/.boukensha` tier is gone
- [x] `Repl::banner()`: three separate lines — `"  config:        {dir
      or (default)}"`, `"  provider:      {provider or (default)}"`,
      `"  model:         {model or (default)}"` — no API-key status
      text, no config-dir-existence check
- [x] `src/bin/boukensha.rs`: if `BOUKENSHA_PATH` is set (env var or
      rc file), prints the two-line note to stderr, then proceeds
      regardless; if `BOUKENSHA_DEBUG` is set, prints `"[boukensha]
      loading from: bundled (v0.9.0)"`; calls `repl::<Player>` with no
      tools registered; on `Err`, prints `"boukensha: {e}"` to stderr
      and exits 1
- [x] `cargo build` (workspace) compiles the new member with **zero**
      warnings (verified: the `api_key` dead-field warning is silenced
      via `#[allow(dead_code)]`, not left unaddressed)
- [x] `cargo install --path rust/09_global_executable` produces a
      working `~/.cargo/bin/boukensha` binary, confirmed runnable from
      an unrelated directory (`/tmp`) with `which boukensha` resolving
      to `~/.cargo/bin/boukensha`
- [x] `boukensha` run with only `BOUKENSHA_DIR` set (no
      `BOUKENSHA_PATH`) prints the v0.9.0 banner **byte-for-byte
      identical** to the reused Ruby transcript above
- [x] `boukensha` run with `BOUKENSHA_PATH` pointing at
      `rust/04_api_client` prints the two-line capability-gap note to
      stderr, then still renders the normal v0.9.0 banner and runs
      normally (bundled step, not step 4)
- [x] `~/.boukensharc` containing only `BOUKENSHA_DIR=...` (no env
      var) resolves the config dir correctly through the real
      installed `boukensha` binary
- [x] A full live turn (`"Briefly, in one sentence, what is a MUD?"`)
      through the real installed `~/.cargo/bin/boukensha` produces a
      normal conversational reply — confirms the whole `Agent`/
      `Client`/`PromptBuilder` pipeline works end-to-end through the
      new `src/bin/boukensha.rs` entry point, not just the banner

### Verification (real runs)

All of the following were run for real against the shared `.boukensha/`
fixture (billed Anthropic API call for the conversational-turn check):

```
$ printf '/exit\n' | BOUKENSHA_DIR="$REPO_ROOT/.boukensha" target/debug/boukensha
╔══════════════════════════════════════╗
║  BOUKENSHA MUD Assistant (v0.9.0)    ║
╚══════════════════════════════════════╝
  config:        /Users/scottburgholzer/Documents/examproco/claude-code-camp-2026-Q2/.boukensha
  provider:      anthropic
  model:         claude-haiku-4-5

  /quiet or /loud   toggle logging
  /clear           reset conversation history
  /exit or /quit    leave the REPL

boukensha> Goodbye.
```

Byte-for-byte identical to Ruby's/Python's `/exit`-only banner.

```
$ echo "BOUKENSHA_DIR=$REPO_ROOT/.boukensha" > ~/.boukensharc
$ printf '/exit\n' | target/debug/boukensha
[... same banner, config dir resolved from the rc file alone ...]
```

```
$ BOUKENSHA_PATH="$REPO_ROOT/week1_baseline/rust/04_api_client" \
    BOUKENSHA_DIR="$REPO_ROOT/.boukensha" target/debug/boukensha
[boukensha] BOUKENSHA_PATH=.../rust/04_api_client is set, but this Rust build can't switch which step's code runs at runtime (Rust binaries are compiled ahead-of-time, unlike Ruby/Python's dynamic require/import).
[boukensha] Running the bundled step (v0.9.0) instead. To run an older step directly, use its own launcher, e.g. bin/rust/<step>, or `cargo run --example example` inside rust/<step>.

[... normal v0.9.0 banner follows ...]
```

```
$ cargo install --path week1_baseline/rust/09_global_executable
    Finished `release` profile [optimized] target(s) in 17.83s
  Installing /Users/scottburgholzer/.cargo/bin/boukensha
   Installed package `boukensha_09_global_executable v0.1.0 (...)` (executable `boukensha`)

$ cd /tmp && which boukensha
/Users/scottburgholzer/.cargo/bin/boukensha

$ printf 'Briefly, in one sentence, what is a MUD?\n/exit\n' \
    | BOUKENSHA_DIR="$REPO_ROOT/.boukensha" boukensha
[... banner ...]
boukensha> 
A MUD (Multi-User Dungeon) is a text-based multiplayer role-playing game where players explore worlds, complete quests, and interact with other players and NPCs through typed commands.
boukensha> Goodbye.
```

All behavior parity checklist items above are checked off against
these real runs plus a direct read of the implemented `rc`/`Config`/
`Client`/`Repl`/`boukensha.rs` source against the Ruby/Python source
line by line.

## Open questions

None outstanding — the two genuinely ambiguous points (the
`BOUKENSHA_PATH` capability gap, and whether to keep a launcher/
`examples/`) were both confirmed with the user via `AskUserQuestion`
before implementation, recorded as Decisions 1–2 and 8 above.
