# Python Port Plan — 09 Global Executable

## Goal

Port the behavior of `week1_baseline/ruby/09_global_executable/` to
`week1_baseline/python/09_global_executable/`. The directory already
existed but was byte-identical to `python/08_the_repl_loop/` (verified:
`diff -rq python/08_the_repl_loop python/09_global_executable` reported
no differences).

Confirmed via `diff -rq ruby/08_the_repl_loop ruby/09_global_executable`:
new files are `bin/boukensha`, `boukensha.gemspec`,
`lib/boukensha_loader.rb`, `lib/boukensha_rc.rb`; changed files are
`Gemfile`/`Gemfile.lock` (adds the `gemspec` self-dependency),
`README.md` (rewritten for install/BOUKENSHA_PATH/`.boukensharc`/debug
docs), `lib/boukensha/client.rb` (drops the HTTP 401 special-case
`ApiError` message added in step 8, falling through to the generic
message for every non-2xx code — a real regression relative to step 8,
not a fix, but Ruby is the spec so it's ported as-is), `lib/boukensha/
config.rb` (`resolve_dir` drops the `<cwd>/.boukensha` middle tier from
step 8 and replaces it with a `~/.boukensharc` `BOUKENSHA_DIR=...` tier,
via the new `BoukenshaRc` module), `lib/boukensha/repl.rb` (`banner`
drops the API-key-status and config-dir-existence checks, replacing the
combined `provider:` line with separate `provider:`/`model:` lines),
`lib/boukensha/version.rb` (`0.8.0` → `0.9.0`), `lib/boukensha.rb`
(comment-only: trims `self.repl`'s doc comment). `examples/` is removed
entirely — `bin/boukensha` (calling `Boukensha.repl` with no block, so
no tools are registered) replaces it as the entry point. Everything
else — `context.rb`, `agent.rb`, `message.rb`, `tool.rb`, `registry.rb`,
`run_dsl.rb`, `prompt_builder.rb`, `logger.rb`, `errors.rb`,
`tasks/{base,player}.rb`, `backends/*.rb` — is byte-identical to
`08_the_repl_loop`, carried forward unchanged (confirmed via `diff -rq`,
which reported no other changed files).

This step's whole point is turning BOUKENSHA into a real global
command. Python has no prior packaging step to mirror (every earlier
`python/NN_*` step is a directory imported via a `sys.path.insert` hack
in `examples/example.py`, run through a numbered `bin/python/NN_*`
launcher — there's no `bin/ruby/09_global_executable` either, since
Ruby's gem replaces that pattern entirely). Per user direction, this
port builds a **real Python equivalent**: `pyproject.toml` +
`console_scripts` (Python's direct analog of a gemspec + `bin/`
executable), installed into the shared root `.venv` via
`pip install -e python/09_global_executable`, producing a genuine
`.venv/bin/boukensha` command — mirroring Ruby's `gem install` result
as closely as Python's packaging model allows. This was built and
verified for real (see Verification) before being written up here, not
guessed at.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/09_global_executable/lib/boukensha_loader.rb` | **New.** `BoukenshaLoader` resolves which step's `lib/boukensha.rb` to load: (1) `BOUKENSHA_PATH` env var, (2) `~/.boukensharc`'s `BOUKENSHA_PATH=` line (via `BoukenshaRc`), (3) the gem's own bundled `lib/`. `.resolve` returns the file path to `require`, aborting with a specific message if an explicit `BOUKENSHA_PATH`/rc value doesn't resolve to a real `lib/boukensha.rb`. `.load_and_start_repl` requires the resolved file, prints a `[boukensha] loading from: ...` debug line if `BOUKENSHA_DEBUG` is set, aborts if the loaded `Boukensha` doesn't `respond_to?(:repl)` (i.e. step < 7), and otherwise calls `Boukensha.repl` with **no block** (no tools registered) |
| `week1_baseline/ruby/09_global_executable/lib/boukensha_rc.rb` | **New.** `BoukenshaRc.read` parses `~/.boukensharc` into a `Hash`: `KEY=VALUE` lines (`#` comments/blank lines ignored); if no line contains `=`, the whole trimmed file is treated as a legacy bare `BOUKENSHA_PATH` value (backward-compat with the pre-step-9 single-path `.boukensharc` format) |
| `week1_baseline/ruby/09_global_executable/bin/boukensha` | **New.** The gem's shebang executable: unshifts `lib/` onto `$LOAD_PATH`, requires `boukensha_loader`, calls `BoukenshaLoader.load_and_start_repl` |
| `week1_baseline/ruby/09_global_executable/boukensha.gemspec` | **New.** Declares the `boukensha` gem: name, `VERSION`, `spec.files = Dir["lib/**/*.rb"] + ["bin/boukensha"]`, `spec.bindir`/`spec.executables = ["boukensha"]`. No runtime dependency declared beyond what `Gemfile` already had (`dotenv`) |
| `week1_baseline/ruby/09_global_executable/Gemfile` / `Gemfile.lock` | Adds `gemspec` to `Gemfile` so Bundler resolves `boukensha` itself as a path dependency (needed for `bundle exec` to work against the gemspec's declared executable/files) |
| `week1_baseline/ruby/09_global_executable/lib/boukensha/client.rb` | The `unless response.is_a?(Net::HTTPSuccess)` branch **drops** the `response.code.to_i == 401` special case added in step 8 — every non-2xx, non-retryable response (including 401) now raises the generic `"API request failed after N attempt(s) (CODE): BODY"` message |
| `week1_baseline/ruby/09_global_executable/lib/boukensha/config.rb` | `resolve_dir` becomes: (1) `ENV["BOUKENSHA_DIR"]`, (2) `BoukenshaRc.read["BOUKENSHA_DIR"]`, (3) `DEFAULT_DIR` (`~/.boukensha`) — replacing step 8's `<Dir.pwd>/.boukensha` middle tier |
| `week1_baseline/ruby/09_global_executable/lib/boukensha/repl.rb` | `banner` drops `key_status`/`provider_line`/`config_exists`/`config_line` computation entirely. New body: `config:        #{@config_dir \|\| "(default)"}`, `provider:      #{@provider \|\| "(default)"}`, `model:         #{@model \|\| "(default)"}` (three separate lines, wider label column, no API-key-status text, no "directory not found" check). `@api_key` is still accepted/stored by `initialize` but is now unused dead state (Ruby doesn't clean it up; neither does this port) |
| `week1_baseline/ruby/09_global_executable/lib/boukensha/version.rb` | `VERSION = "0.9.0"` (was `"0.8.0"`) |
| `week1_baseline/ruby/09_global_executable/lib/boukensha.rb` | Comment-only: `self.repl`'s doc comment is trimmed/reworded. No behavior change |
| `week1_baseline/ruby/09_global_executable/lib/boukensha/{context,agent,message,tool,registry,run_dsl,prompt_builder,logger,errors}.rb`, `tasks/{base,player}.rb`, `backends/*.rb` | Byte-identical to `08_the_repl_loop` (verified via `diff -rq`) — carry forward as-is |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps —
`settings.yaml`, `.env`, `prompts/` unchanged. Runs add their own new
`.boukensha/sessions/<session-id>.jsonl` files alongside existing ones.

## Target files to create/change (Python)

```
week1_baseline/python/09_global_executable/
  pyproject.toml                         (new — see "Making Python a global executable" below)
  README.md                              (rewrite: install via pip install -e, BOUKENSHA_PATH table, ~/.boukensharc docs, debug mode)
  requirements.txt                       (unchanged — no new runtime dependency)
  prompts/system.md                      (unchanged, already correct)
  examples/                              (removed — mirrors Ruby dropping examples/; the installed `boukensha` command is the entry point)
  lib/boukensha_loader.py                (new — console-script entry point, mirrors boukensha_loader.rb)
  lib/boukensha_rc.py                    (new — mirrors boukensha_rc.rb)
  lib/boukensha/__init__.py              (edit: repl() doc comment reworded, mirroring boukensha.rb's comment-only change)
  lib/boukensha/version.py               (edit: "0.9.0")
  lib/boukensha/client.py                (edit: drop the HTTP 401 special-case ApiError branch)
  lib/boukensha/config.py                (edit: _resolve_dir drops the <cwd>/.boukensha tier, adds boukensha_rc's BOUKENSHA_DIR tier)
  lib/boukensha/repl.py                  (edit: _banner() simplified — separate config/provider/model lines, no key-status/dir-exists checks; drops now-unused `import os`)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/agent.py                 (unchanged, already correct)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/run_dsl.py               (unchanged, already correct)
  lib/boukensha/prompt_builder.py        (unchanged, already correct)
  lib/boukensha/logger.py                (unchanged, already correct)
  lib/boukensha/errors.py                (unchanged, already correct)
  lib/boukensha/tasks/__init__.py        (unchanged)
  lib/boukensha/tasks/base.py            (unchanged, already correct)
  lib/boukensha/tasks/player.py          (unchanged, already correct)
  lib/boukensha/backends/*.py            (unchanged, already correct)
```

No `bin/python/09_global_executable` launcher — mirrors Ruby exactly
(no `bin/ruby/09_global_executable` either). The installed `boukensha`
command replaces the launcher pattern for this step, same as Ruby's
`gem install` replaces `bundle exec ruby examples/example.rb`.

### Making Python a global executable

Python's direct analog of a gemspec + `bin/` executable is a
`pyproject.toml` `[project.scripts]` entry point installed via
`pip install -e`. This reuses the existing `lib/boukensha/` package
layout unchanged (`package-dir = {"" = "lib"}` maps the package root to
`lib/`, exactly like every prior step's `sys.path.insert(0, LIB_DIR)`
hack pointed at the same directory — no restructuring needed) and adds
two sibling top-level modules next to the `boukensha` package, mirroring
Ruby's `lib/boukensha_loader.rb`/`lib/boukensha_rc.rb` sitting beside
`lib/boukensha/`:

```toml
[project.scripts]
boukensha = "boukensha_loader:main"

[tool.setuptools]
package-dir = {"" = "lib"}
py-modules = ["boukensha_loader", "boukensha_rc"]

[tool.setuptools.packages.find]
where = ["lib"]
```

Install once into the shared root `.venv` (documented in the rewritten
`README.md`):

```bash
.venv/bin/pip install -e week1_baseline/python/09_global_executable
```

This produces a real `.venv/bin/boukensha` script — verified for real
(see Verification) to behave exactly like Ruby's `gem install`-produced
`boukensha`: same banner, same `BOUKENSHA_PATH`/`~/.boukensharc`
resolution (able to load *any* older un-packaged `python/NN_*` step by
prepending its `lib/` to `sys.path` before importing `boukensha` — no
`pip install` needed for those, exactly like Ruby's loader pointing
`require` at an arbitrary step's `lib/boukensha.rb`), same
`BOUKENSHA_DEBUG` output, same "doesn't support the interactive REPL"
abort for steps before the REPL was added.

## Behavior parity checklist (from the real Ruby output)

- [x] `version.py` defines `VERSION = "0.9.0"`
- [x] `Client.call()` no longer raises a 401-specific message — a 401
      response falls through to the same generic
      `f"API request failed after {attempts} attempt{suffix} ({code}): {body}"`
      as any other non-retryable, non-2xx code
- [x] `Config._resolve_dir()`: `BOUKENSHA_DIR` env var wins; else
      `boukensha_rc.read().get("BOUKENSHA_DIR")`; else `DEFAULT_DIR`
      (`~/.boukensha`) — the step-8 `<cwd>/.boukensha` tier is gone
- [x] `boukensha_rc.read()`: parses `~/.boukensharc` into a dict of
      `KEY=VALUE` pairs (`#` comments/blank lines ignored); a file with
      no `=` on any line is treated as a legacy bare `BOUKENSHA_PATH`
      value; missing/empty file returns `{}`
- [x] `Repl._banner()`: three separate lines —
      `f"  config:        {self.config_dir or '(default)'}"`,
      `f"  provider:      {self.provider or '(default)'}"`,
      `f"  model:         {self.model or '(default)'}"` — no API-key
      status text, no config-dir-existence check
- [x] `boukensha_loader.resolve()`: `BOUKENSHA_PATH` env var → then
      `boukensha_rc.read().get("BOUKENSHA_PATH")` → else `None`
      (bundled default); an explicit value that doesn't resolve to a
      real `lib/boukensha` package aborts with a specific message
      naming the source (env var vs `~/.boukensharc`)
- [x] `boukensha_loader.main()`: prints `f"[boukensha] loading from: {step_dir}"`
      when `BOUKENSHA_DEBUG` is set, prepends the resolved step's `lib/`
      to `sys.path` (skipped for the bundled default), imports
      `boukensha`, aborts if it has no `repl` attribute, otherwise calls
      `boukensha.repl()` with **no tools registered**
- [x] `pip install -e python/09_global_executable` into the shared
      `.venv` produces a working `.venv/bin/boukensha` console script
- [x] `.venv/bin/boukensha` run with only `BOUKENSHA_DIR` set (no
      `BOUKENSHA_PATH`) uses the bundled `lib/boukensha/` (this step's
      own package) and prints the v0.9.0 banner
- [x] `.venv/bin/boukensha` run with `BOUKENSHA_PATH` pointing at an
      older un-packaged step (`python/08_the_repl_loop`) loads *that*
      step's `boukensha` package instead (verified: prints the v0.8.0
      banner with the old API-key-status/single provider line, proving
      a real swap, not the bundled default)
- [x] `.venv/bin/boukensha` run with `BOUKENSHA_PATH` pointing at a
      step with no `repl` (`python/06_the_logger`) aborts with the
      "does not support the interactive REPL" message, referencing
      `python {step_dir}/examples/*.py`
- [x] `~/.boukensharc` containing `BOUKENSHA_DIR=...` (no env var set)
      resolves the config dir correctly through the real installed
      command

## Expected output

Real run of Ruby step 9, via `bundle exec ruby bin/boukensha` from
`ruby/09_global_executable/` (no `bin/ruby/09_global_executable`
launcher exists for this step — confirmed with the user before running;
real, billed Anthropic API call):

Command:
```
printf 'Briefly, in one sentence, what is a MUD?\n/exit\n' \
  | BOUKENSHA_DIR="$REPO_ROOT/.boukensha" bundle exec ruby bin/boukensha
```

Output:
```
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

This confirms: the simplified banner (separate `provider:`/`model:`
lines, no key-status text), and that `bin/boukensha` calls
`Boukensha.repl` with no tools registered (the reply is purely
conversational — no `read_file`/`list_directory` tools exist to call,
unlike step 8's worked example).

`~/.boukensharc` end-to-end (real run, `BOUKENSHA_DIR` set only via the
rc file, no env var):
```
$ echo "BOUKENSHA_DIR=$REPO_ROOT/.boukensha" > ~/.boukensharc
$ printf '/exit\n' | bundle exec ruby bin/boukensha
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

## Verification

Built and installed the real Python package for real
(`.venv/bin/pip install -e python/09_global_executable`) and ran the
resulting `.venv/bin/boukensha` against the same fixture:

```
$ printf '/exit\n' | BOUKENSHA_DIR="$REPO_ROOT/.boukensha" .venv/bin/boukensha

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

**Byte-for-byte identical** to Ruby's `/exit`-only banner (box-drawing
characters, label column width, version padding all match).

Additional real checks:
- `~/.boukensharc` containing only `BOUKENSHA_DIR=...` (no env var)
  resolved the config dir correctly through `.venv/bin/boukensha` —
  matches the Ruby rc-file run above.
- `BOUKENSHA_PATH=.../python/06_the_logger .venv/bin/boukensha` aborted
  with `"boukensha: the step at .../06_the_logger\n       does not
  support the interactive REPL (added in step 7).\n       Run its
  examples directly, e.g.:\n         python
  .../06_the_logger/examples/*.py\n       Or point BOUKENSHA_PATH at
  step 7 or later."` — matches Ruby's abort message shape exactly
  (`python .../examples/*.py` instead of `ruby .../examples/*.rb`).
- `BOUKENSHA_DEBUG=1 BOUKENSHA_PATH=.../python/08_the_repl_loop
  .venv/bin/boukensha` printed `"[boukensha] loading from:
  .../python/08_the_repl_loop"` and then rendered **step 8's own**
  banner (`v0.8.0`, the old combined `provider:  anthropic
  (claude-haiku-4-5)  ✓ API key set` line) — proof the loader really
  swaps which `boukensha` package gets imported via `sys.path`
  ordering, not just cosmetically re-reading the same bundled code.
- Inspected the generated `.venv/lib/python3.12/site-packages/
  __editable__.boukensha-0.9.0.pth`: contains exactly
  `.../python/09_global_executable/lib`, confirming the editable
  install adds `lib/` to `sys.path` the same way every prior step's
  `examples/example.py` did manually — and the generated
  `.venv/bin/boukensha` script is a normal `#!.../python3.12` shebang
  script that does `from boukensha_loader import main; sys.exit(main())`,
  confirming `[project.scripts]` wired up correctly.

All behavior parity checklist items above are checked off against
these real runs plus a direct read of the implemented `boukensha_loader`/
`boukensha_rc`/`Config`/`Client`/`Repl` source against the Ruby source
line by line.

## Porting notes (Ruby idiom → Python)

- **`module BoukenshaRc; def self.read; ...; end; end` → a plain
  module-level function `read()` in `boukensha_rc.py`, not a class.**
  Direct precedent: the top-level `Boukensha` Ruby module (`def
  self.run`, `def self.config`, etc.) was already ported to
  module-level functions in `__init__.py` rather than a class with
  static methods — `BoukenshaRc` is the same shape (a Ruby module used
  purely as a singleton-method namespace), so it gets the same
  treatment.
- **`lib/boukensha_loader.rb`/`lib/boukensha_rc.rb` sitting as siblings
  of `lib/boukensha/` → `lib/boukensha_loader.py`/`lib/boukensha_rc.py`
  as siblings of `lib/boukensha/`, not submodules inside the package.**
  This is a deliberate structural mirror, not the "package-internal
  submodule" shape a plain library port would default to — the whole
  point of this step is packaging, so the file layout mirrors Ruby's
  gem layout exactly (`package-dir`/`py-modules` in `pyproject.toml`
  install both the `boukensha` package *and* the two sibling modules as
  top-level importables, exactly matching how `require_relative
  "../boukensha_rc"` reaches a sibling file from inside the package in
  Ruby).
- **`File.exist?(main)` checking for a specific `lib/boukensha.rb` file
  → `os.path.isfile(os.path.join(step_dir, "lib", "boukensha",
  "__init__.py"))`.** Ruby's single-file `require` target becomes
  Python's package marker file — the natural equivalent existence
  check for "does this directory have an importable `boukensha`
  package," used identically for the bundled default, `BOUKENSHA_PATH`,
  and rc-file cases.
- **`require main` (loads one exact file by path) → `sys.path.insert(0,
  step_lib); import boukensha`.** Ruby's `require` takes an absolute
  file path directly; Python's `import` resolves by module name against
  `sys.path`, so the equivalent is prepending the target step's `lib/`
  directory so its `boukensha` package shadows the one already
  installed from the bundled default — verified for real (see
  Verification) to correctly load an older step's package instead of
  the bundled one.
- **`response.code.to_i == 401` removed, falling through to the
  generic branch → the equivalent `if e.code == 401: raise
  ApiError(...)` branch (added in the `08_the_repl_loop` port) is
  simply deleted from `client.py`.** No idiom gap — this is a pure
  behavior regression in the Ruby source (dropping a step-8 addition),
  ported as-is per the "Ruby is always the spec" guardrail, not
  silently kept for niceness.
- **`Pathname.new(raw).expand_path.to_s` → `os.path.abspath(os.path.expanduser(raw))`.**
  Same translation already used in every prior `Config._resolve_dir`
  port; unchanged here.
- **`gem install` (installs a built `.gem` into the Ruby/Bundler
  environment, adding `boukensha` to `$PATH` via RubyGems' bin dir) →
  `pip install -e` (installs the package in "editable" mode into the
  active Python environment, adding `boukensha` to the venv's `bin/`
  via a generated console-script shim).** `-e`/editable was chosen over
  a regular non-editable install because every other step in this repo
  is meant to be runnable straight from its source directory without a
  build/reinstall step in between — editable install keeps `boukensha`
  pointing at `python/09_global_executable/lib` on disk, matching how
  Ruby's `bundle exec` in every other step's launcher already runs
  straight from source rather than a built gem.

## Decisions

1. **Ran the real Ruby step for a verified transcript, confirmed with
   the user first** — same precedent as `04_api_client.md` through
   `08_the_repl_loop.md`: a real, billed Anthropic API call, explicit
   go-ahead given before piping a scripted single-turn stdin
   conversation into `bin/boukensha` (via `bundle exec ruby
   bin/boukensha`, since no `bin/ruby/09_global_executable` launcher
   exists for this step).

2. **Python gets a real `pyproject.toml` + `console_scripts` package,
   confirmed with the user via AskUserQuestion** (this step's whole
   point is "global executable," and Ruby's own step drops its numbered
   launcher entirely in favor of a real installed command — carrying
   forward the `bin/python/NN_*` + `examples/example.py` pattern
   unchanged would have quietly missed the point of the step). The
   `package-dir`/`py-modules`/`console_scripts` design was built and
   verified end-to-end against the shared `.venv` (see Verification)
   before being written up here, not left as an unverified sketch.

3. **`examples/` is removed, matching Ruby exactly.** Ruby's step 9
   deletes `examples/` entirely because `bin/boukensha` (calling
   `Boukensha.repl` with no block) is now the real entry point; the
   Python port mirrors this rather than keeping a now-redundant
   `examples/example.py` around.

4. **No new pip dependency.** `pyproject.toml`'s `[build-system]`
   declares `setuptools>=61.0`, but that's a *build-time* dependency
   pip fetches automatically into an isolated build environment during
   `pip install -e` — it is not installed into `.venv`'s runtime
   site-packages and does not need adding to `requirements.txt` or
   `python/README.md`'s shared-venv setup instructions. No stdlib gap
   exists that would justify a new runtime dependency either — `pip`
   itself is the only new tool required, and it's already a standard
   part of any Python installation, same bar `04_api_client.md`
   applied when it chose to hardcode a cert path over adding an HTTP
   library.

## Open questions

None outstanding — all decided above and verified with real runs
(Ruby transcript + a real `pip install -e` / `.venv/bin/boukensha`
round trip, including a live `BOUKENSHA_PATH` swap to an older step and
an abort-path step).
