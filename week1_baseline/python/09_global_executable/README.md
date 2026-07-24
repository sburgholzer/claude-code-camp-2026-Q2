# 09 · Global Executable (Python port)

Package BOUKENSHA as a real installable Python package so a `boukensha`
command works from anywhere `.venv` is on `PATH` — the Python analog of
Ruby's `gem build && gem install`. `context.py`, `agent.py`,
`message.py`, `tool.py`, `registry.py`, `run_dsl.py`,
`prompt_builder.py`, `logger.py`, `errors.py`, `tasks/*.py`, and
`backends/*.py` are unchanged from `08_the_repl_loop`; see
`../08_the_repl_loop/README.md` for those.

## New Files

| File | Description |
|---|---|
| `pyproject.toml` | Declares the `boukensha` package (`lib/boukensha/`, reused unchanged) and a `console_scripts` entry point: `boukensha = "boukensha_loader:main"` |
| `lib/boukensha_loader.py` | Resolves which step's `boukensha` package to load, then boots the REPL — Python's analog of Ruby's `bin/boukensha` + `boukensha_loader.rb` combined |
| `lib/boukensha_rc.py` | Parses `~/.boukensharc` (`KEY=VALUE` lines; legacy bare-path fallback) |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/version.py` | `VERSION = "0.9.0"` (was `"0.8.0"`) |
| `lib/boukensha/client.py` | Drops the HTTP 401 special-case `ApiError` message added in step 8 — a 401 now falls through to the generic failure message like any other non-retryable code |
| `lib/boukensha/config.py` | `_resolve_dir()` drops the `<cwd>/.boukensha` middle tier from step 8, replacing it with a `~/.boukensharc` `BOUKENSHA_DIR=...` tier (via `boukensha_rc`) |
| `lib/boukensha/repl.py` | `_banner()` simplified: separate `config:`/`provider:`/`model:` lines, no API-key-status text, no config-dir-existence check |
| `lib/boukensha/__init__.py` | `repl()`'s doc comment reworded (no behavior change) |

`examples/` is removed — the installed `boukensha` command (registering
no tools, same as Ruby's `bin/boukensha`) replaces it as the entry
point.

## Install

`.venv` lives at the **repo root** (sibling of `week1_baseline/`, per
`python/README.md`). Run the install from wherever you are, adjusting
the path to `python/09_global_executable` accordingly:

```bash
# from the repo root
.venv/bin/pip install -e week1_baseline/python/09_global_executable

# from week1_baseline/
../.venv/bin/pip install -e python/09_global_executable

# or, with the venv already activated (any cwd) — omit .venv/bin/
pip install -e python/09_global_executable   # from week1_baseline/
```

After that, `boukensha` is a real command in `.venv/bin/` — runnable
from anywhere once the venv is on `PATH` (`source .venv/bin/activate`,
or call `.venv/bin/boukensha` directly).

## Usage

```bash
source .venv/bin/activate   # once per shell session, puts boukensha on PATH (path relative to repo root — see Install above)
boukensha
```

Or, without activating the venv, call the script directly (same
repo-root-relative path as Install above):

```bash
.venv/bin/boukensha
```

Either form drops you into the interactive REPL (no tools registered,
same as Ruby's `bin/boukensha`), reading config from `~/.boukensha` by
default. Type `/help` for commands, `/exit` or `/quit` (or Ctrl-D) to
leave.

## Switching steps with BOUKENSHA_PATH

The loader resolves in this order:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | `BOUKENSHA_PATH` env var | `BOUKENSHA_PATH=~/Sites/boukensha/python/07_the_run_dsl boukensha` |
| 2 | `~/.boukensharc` file | `echo "BOUKENSHA_PATH=~/Sites/boukensha/python/07_the_run_dsl" > ~/.boukensharc` |
| 3 | Bundled default | just run `boukensha` |

`BOUKENSHA_PATH` must point to a step folder that contains
`lib/boukensha/__init__.py`. Older, un-packaged steps work too — no
`pip install` needed for them, the loader just prepends their `lib/` to
`sys.path`.

## Persistent config with ~/.boukensharc

`~/.boukensharc` can set `BOUKENSHA_PATH` and/or `BOUKENSHA_DIR` so you
don't have to export them in every shell session:

```
# ~/.boukensharc
BOUKENSHA_PATH=~/Sites/boukensha/python/07_the_run_dsl
BOUKENSHA_DIR=~/projects/mybot/.boukensha
```

Blank lines and `#` comments are ignored. An environment variable
always overrides the matching rc value.

Legacy format: a `~/.boukensharc` containing just a bare path (no `=`)
is still read as `BOUKENSHA_PATH`.

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
# => [boukensha] loading from: /path/to/step
```

## No New Runtime Dependencies

`requirements.txt` is unchanged from `08_the_repl_loop`. `pyproject.toml`
declares `setuptools>=61.0` as a *build-time* dependency only (fetched
automatically by `pip` into an isolated build environment) — it isn't
installed into `.venv`'s runtime site-packages.

## Porting notes

- **`module BoukenshaRc; def self.read; ...; end; end` → a module-level
  `read()` function in `boukensha_rc.py`**, not a class — same
  treatment already given to the top-level `Boukensha` Ruby module,
  ported as module-level functions in `__init__.py`.
- **`lib/boukensha_loader.rb`/`lib/boukensha_rc.rb` as siblings of
  `lib/boukensha/` → `lib/boukensha_loader.py`/`lib/boukensha_rc.py` as
  siblings of `lib/boukensha/`**, not submodules inside the package —
  a deliberate mirror of Ruby's gem layout, since packaging is this
  step's whole point.
- **`require main` (one exact file, by path) → `sys.path.insert(0,
  step_lib); import boukensha`.** Verified for real: pointing
  `BOUKENSHA_PATH` at `python/08_the_repl_loop` loads *that* step's
  `boukensha` package (its old v0.8.0 banner), not the bundled one.
- **`gem install` → `pip install -e`.** Editable install keeps
  `boukensha` pointing at `python/09_global_executable/lib` on disk,
  matching how every other step's launcher already runs straight from
  source rather than a built artifact.

See `docs/plans/python_port/09_global_executable.md` for the full
decision record, including the real `pip install -e` verification run.

## Run Example

```bash
.venv/bin/pip install -e week1_baseline/python/09_global_executable   # one-time, from the repo root
echo "BOUKENSHA_DIR=$(pwd)/.boukensha" > ~/.boukensharc                # one-time, points boukensha at this repo's fixture
.venv/bin/boukensha
```

`~/.boukensharc` is the point of this step's config resolution — set it
once and every future `boukensha` invocation (from any directory) picks
up `BOUKENSHA_DIR` without needing an inline env var each time.

This is an interactive REPL with **no tools registered** (unlike step
8's worked example) — each turn makes one real HTTP request to
whichever provider `.boukensha/settings.yaml` configures. It costs a
small amount per model round-trip and requires a valid API key in
`.boukensha/.env`.

Example output (the exact model reply is **not** reproducible
byte-for-byte — it's a live model response; the banner itself is
byte-for-byte identical to Ruby's):

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

A MUD (Multi-User Dungeon) is a text-based multiplayer game where players explore fantasy worlds, fight monsters, solve puzzles, and interact with other players through typed commands.
boukensha> /exit
Goodbye.
```

Verified against the real Ruby run for this same fixture, plus a real
`pip install -e` / `.venv/bin/boukensha` round trip (see
`docs/plans/python_port/09_global_executable.md`).
