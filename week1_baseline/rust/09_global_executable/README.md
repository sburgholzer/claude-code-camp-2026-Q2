# 09 · Global Executable (Rust port)

Package BOUKENSHA as a real installable Rust binary — `cargo install
--path`, the Rust analog of Ruby's `gem build && gem install` and
Python's `pip install -e`. **This is the official global executable
going forward** — a real, statically-linked `boukensha` binary in
`~/.cargo/bin/`, not a delta-port of library code behind an
`examples/` runner. `agent.rs`, `context.rs`, `message.rs`, `tool.rs`,
`registry.rs`, `run_dsl.rs`, `prompt_builder.rs`, `logger.rs`,
`errors.rs`, `tasks/*.rs`, and `backends/*.rs` are unchanged from
`08_the_repl_loop`; see `../08_the_repl_loop/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/bin/boukensha.rs` | The `boukensha` binary's `main()` — resolves `BOUKENSHA_PATH`/`BOUKENSHA_DIR`, then calls `repl::<Player>` with no tools registered. Rust's analog of Ruby's `bin/boukensha` + `boukensha_loader.rb` and Python's `boukensha_loader.py`, combined |
| `src/rc.rs` | Parses `~/.boukensharc` (`KEY=VALUE` lines; legacy bare-path fallback) — direct port of `boukensha_rc.rb`/`boukensha_rc.py` |

## Updated Files

| File | Change |
|---|---|
| `src/version.rs` | `VERSION = "0.9.0"` (was `"0.8.0"`) |
| `src/client.rs` | Drops the HTTP 401 special-case `ApiError` message added in step 8 — a 401 now falls through to the generic failure message like any other non-retryable code |
| `src/config.rs` | `resolve_dir()` drops the `<cwd>/.boukensha` middle tier from step 8, replacing it with a `~/.boukensharc` `BOUKENSHA_DIR=...` tier (via `rc::read()`) |
| `src/repl.rs` | `banner()` simplified: separate `config:`/`provider:`/`model:` lines, no API-key-status text, no config-dir-existence check. `api_key` field kept (unused, `#[allow(dead_code)]`) for structural parity with Ruby/Python's equally-dead `@api_key`/`self.api_key` |
| `src/lib.rs` | `+ pub mod rc;`; `repl()`'s doc comment reworded (no behavior change) |
| `Cargo.toml` | Package stays `boukensha_09_global_executable` (repo convention), but adds a `[[bin]] name = "boukensha"` target — the *installed* binary is named `boukensha` regardless of the package name |

`examples/` is removed, and there's no `bin/rust/09_global_executable`
launcher — mirrors Ruby/Python exactly. The real entry point is the
installed `boukensha` binary itself.

## A genuine capability gap: BOUKENSHA_PATH

Ruby/Python's `boukensha` command can dynamically load a *different*
step's code at runtime (`require`/`import` pointed at an arbitrary
directory). **Rust can't do this** — `boukensha` is a single
statically-compiled binary; there's no safe way for it to load another
step's compiled code without something like `dlopen`/unsafe FFI, which
this project doesn't take on for a teaching convenience that already
has a working alternative.

So in the Rust build, `BOUKENSHA_PATH` (env var or `~/.boukensharc`) is
still *read* — but only to print a note, then the binary proceeds with
its own bundled step regardless:

```
$ BOUKENSHA_PATH=~/Sites/boukensha/04_api_client boukensha
[boukensha] BOUKENSHA_PATH=~/Sites/boukensha/04_api_client is set, but this Rust build can't switch which step's code runs at runtime (Rust binaries are compiled ahead-of-time, unlike Ruby/Python's dynamic require/import).
[boukensha] Running the bundled step (v0.9.0) instead. To run an older step directly, use its own launcher, e.g. bin/rust/<step>, or `cargo run --example example` inside rust/<step>.

╔══════════════════════════════════════╗
...
```

To actually run an older Rust step, use its own existing launcher —
`bin/rust/04_api_client`, or `cd rust/04_api_client && cargo run
--example example` — which already works today and needs no changes
from this step.

`BOUKENSHA_DIR` has no such gap — it only selects a config
*directory* to read at runtime, which is fully portable and works
exactly like Ruby/Python.

## Install

```bash
# from the repo root
cargo install --path week1_baseline/rust/09_global_executable
```

This builds a release binary and installs it as `~/.cargo/bin/boukensha`
— a real global command, runnable from any directory once
`~/.cargo/bin` is on `PATH` (the default after a standard `rustup`
install).

## Usage

```bash
boukensha
```

Drops you into the interactive REPL (no tools registered, same as
Ruby's `bin/boukensha`), reading config from `~/.boukensha` by
default. Type `/help` for commands, `/exit` or `/quit` (or Ctrl-D) to
leave.

## Persistent config with ~/.boukensharc

`~/.boukensharc` can set `BOUKENSHA_DIR` (and `BOUKENSHA_PATH`, for the
note above) so you don't have to export it in every shell session:

```
# ~/.boukensharc
BOUKENSHA_DIR=~/projects/mybot/.boukensha
```

Blank lines and `#` comments are ignored. An environment variable
always overrides the matching rc value. A legacy `~/.boukensharc`
containing just a bare path (no `=`) is still read as `BOUKENSHA_PATH`.

`BOUKENSHA_DIR` picks the config directory (`settings.yaml`, `.env`,
prompt overrides) and resolves in this order:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | `BOUKENSHA_DIR` env var | `BOUKENSHA_DIR=~/projects/mybot/.boukensha boukensha` |
| 2 | `~/.boukensharc` file | `BOUKENSHA_DIR=~/projects/mybot/.boukensha` line |
| 3 | `~/.boukensha` default | just run `boukensha` |

## Debug mode

```bash
BOUKENSHA_DEBUG=1 boukensha
# => [boukensha] loading from: bundled (v0.9.0)
```

## No New Dependencies

`Cargo.toml`'s dependency list is unchanged from `08_the_repl_loop`.
`src/rc.rs` uses only `std::fs`/`std::collections::HashMap` plus the
already-present `dirs` crate (for `home_dir()`, already used by
`config.rs`).

## Porting notes

- **`module BoukenshaRc; def self.read; ...; end; end` → a plain
  module function `rc::read()`**, not a struct/trait — same shape
  already used for `Boukensha`'s own module-level functions
  (`quiet()`/`loud()`/`config()` in `lib.rs`).
- **`lib/boukensha_loader.rb`/`lib/boukensha_rc.rb` as siblings of
  `lib/boukensha/` → `src/bin/boukensha.rs` (the binary) + `src/rc.rs`
  (a library module)**, not two sibling top-level files the way Python
  needed — Rust's single-crate model means `rc.rs` is just an ordinary
  `pub mod` inside the same crate `config.rs` already lives in, no
  packaging-driven sibling-module trick required.
- **`gem install` / `pip install -e` → `cargo install --path`.**
  Rust's closest equivalent — builds a release binary and copies it
  into `~/.cargo/bin`, verified for real (see
  `docs/plans/rust_port/09_global_executable.md`) to work exactly like
  the Ruby/Python global commands from any directory.
- **BOUKENSHA_PATH step-switching has no Rust equivalent — documented
  as a genuine capability reduction, not silently dropped.** See "A
  genuine capability gap" above.

See `docs/plans/rust_port/09_global_executable.md` for the full
decision record, including the real `cargo install` verification run.

## Run Example

```bash
cargo install --path week1_baseline/rust/09_global_executable   # one-time, from the repo root
echo "BOUKENSHA_DIR=$(pwd)/.boukensha" > ~/.boukensharc            # one-time, points boukensha at this repo's fixture
boukensha
```

This is an interactive REPL with **no tools registered** (unlike step
8's worked example) — each turn makes one real HTTP request to
whichever provider `.boukensha/settings.yaml` configures. It costs a
small amount per model round-trip and requires a valid API key in
`.boukensha/.env`.

Example output (the exact model reply is **not** reproducible
byte-for-byte — it's a live model response; the banner itself is
byte-for-byte identical to Ruby's and Python's):

```
╔══════════════════════════════════════╗
║  BOUKENSHA MUD Assistant (v0.9.0)    ║
╚══════════════════════════════════════╝
  config:        /.../.boukensha
  provider:      anthropic
  model:         claude-haiku-4-5

  /quiet or /loud   toggle logging
  /clear           reset conversation history
  /exit or /quit    leave the REPL

boukensha> Briefly, in one sentence, what is a MUD?

A MUD (Multi-User Dungeon) is a text-based multiplayer role-playing game where players explore worlds, complete quests, and interact with other players and NPCs through typed commands.
boukensha> /exit
Goodbye.
```

Verified against the real Ruby and Python runs for this same fixture,
plus a real `cargo install` / `~/.cargo/bin/boukensha` round trip (see
`docs/plans/rust_port/09_global_executable.md`).
