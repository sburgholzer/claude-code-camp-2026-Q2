# Python Port Plan — 00 Config

## Goal

Port the behavior of `week1_baseline/ruby/00_config/` to
`week1_baseline/python/00_config/` (directory already exists, currently
empty). End state: a runnable Python example that resolves the same
`.boukensha/` config directory, loads the same `settings.yaml` /
`.env`, and exposes task-settings lookups + system-prompt resolution
with the same behavior as the Ruby version, against the **same**
`.boukensha/` fixture at the repo root (`/.boukensha/`) so both
languages can be verified against one shared config.

This is a behavior port, not a redesign — the Ruby version is the spec.
Where Python has no stdlib equivalent for something Ruby did with a gem,
that's called out below as a question rather than decided unilaterally.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/00_config/README.md` | Design spec: dir resolution order, config schema, task/prompt resolution rules, expected example output |
| `week1_baseline/ruby/00_config/lib/boukensha.rb` | Top-level require entrypoint |
| `week1_baseline/ruby/00_config/lib/boukensha/config.rb` | `Boukensha::Config` — dir resolution, `.env` loading, `settings.yaml` loading, `tasks`, `mud_*`, `dig`, `to_s`/`inspect` |
| `week1_baseline/ruby/00_config/lib/boukensha/tasks/base.rb` | `Boukensha::Tasks::Base` — abstract, stateless class-method API: `.task_name`, `.provider`, `.model`, `.prompt_override?`, `.prompt`/`.system_prompt`, private `fetch`/`read_user_prompt`/`read_default_prompt`/`read_file` |
| `week1_baseline/ruby/00_config/lib/boukensha/tasks/player.rb` | `Boukensha::Tasks::Player < Base` — concrete task, just sets `task_name = "player"` |
| `week1_baseline/ruby/00_config/prompts/system.md` | Default system prompt shipped with the library (fallback when no task override) |
| `week1_baseline/ruby/00_config/examples/example.rb` | Runnable smoke test — this is effectively the acceptance test; the Python port should have a line-for-line equivalent output |
| `week1_baseline/ruby/00_config/Gemfile` / `Gemfile.lock` | Declares the one allowed external dependency (`dotenv`) — see design consideration in the README about minimizing third-party deps |

## Runtime fixture to reuse (do not duplicate)

| Path | Role |
|---|---|
| `.boukensha/settings.yaml` | Real settings fixture at repo root — `tasks.player.{provider,model,prompt_override.system}`, `mud.{host,port,username,password}` |
| `.boukensha/.env` | Real secrets fixture (e.g. `ANTHROPIC_API_KEY`) |
| `.boukensha/prompts/player/system.md` | Per-task prompt override fixture, exercises the `prompt_override.system: true` path |

The Ruby example points `BOUKENSHA_DIR` at this same directory (via a
relative path from `examples/example.rb`). The Python example should do
the same, so both ports are validated against one source of truth
instead of drifting fixtures.

## Target files to create (Python)

Mirrors the Ruby layout 1:1 unless a question below changes it:

```
week1_baseline/python/00_config/
  README.md
  examples/example.py
  lib/boukensha/__init__.py
  lib/boukensha/config.py
  lib/boukensha/tasks/__init__.py
  lib/boukensha/tasks/base.py
  lib/boukensha/tasks/player.py
  prompts/system.md          (copy of ruby's, verbatim)
```

Plus a launcher at `week1_baseline/bin/python/00_config`, following the
repo's per-language `bin/<language>/<step>` convention:
`bin/ruby/00_config` already exists there with the Ruby launcher;
`bin/rust/00_config` is a placeholder to ignore for now. Unlike
`bundle exec` (which resolves gems implicitly via the `Gemfile` sitting
next to the code), Python has no ambient equivalent — the launcher has
to point explicitly at the shared venv's interpreter (see decision 3):

```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../python/00_config"
"$(dirname "$0")/../../../.venv/bin/python" examples/example.py
```

(executable bit set, matching `bin/ruby/00_config`'s permissions;
path assumes the repo-root `.venv` from decision 3 — confirm that's
really "root of the project" and not `week1_baseline/.venv`)

Plus `week1_baseline/python/README.md` — a new, port-wide (not
per-step) README living above `00_config/`, since the venv it
documents is shared across every future `python/NN_*` folder rather
than scoped to this one step. Holds the one-time setup instructions:
create the venv, activate it, install this step's `requirements.txt`.
`python/00_config/README.md` (the step's own README, ported from the
Ruby one) links to it instead of repeating setup steps.

## Behavior parity checklist (from the Ruby README)

- [ ] Config dir resolution order: `BOUKENSHA_DIR` env var, else
      `~/.boukensha` (`config.py` / `Config.DEFAULT_DIR` equivalent)
- [ ] Loads `.env` from the resolved dir if present, before reading settings
- [ ] Loads `settings.yaml` from the resolved dir if present, else `{}`
- [ ] `Config.tasks()` with no arg returns the full `tasks:` map; with a
      name returns that task's settings dict (or `None`)
- [ ] `Config.user_prompts_dir` = `<dir>/prompts`
- [ ] `Config.mud_host` / `mud_port` / `mud_username` / `mud_password`
      with the same defaults (`"localhost"`, `4000`)
- [ ] `Tasks::Base` stays abstract/stateless: `task_name` raises if not
      overridden; `provider`/`model` raise `ArgumentError`-equivalent if
      missing from settings
- [ ] `prompt_override?(settings, prompt="system")` reads
      `settings["prompt_override"][prompt]`, defaults `False`
- [ ] System prompt resolution order: per-task override file
      (`<user_prompts_dir>/<task_name>/system.md`) if
      `prompt_override?` is true and the file exists, else the
      default `prompts/system.md` shipped with the library
- [ ] `Tasks::Player.task_name == "player"`, no other behavior
- [ ] `examples/example.py` produces the same fields as
      `examples/example.rb` (config dir, task list, provider, model,
      prompt-override flag, truncated system prompt, mud host/port/user,
      whether the API key env var is set, and a `repr`/`str` of the
      config object)

## Porting notes (Ruby idiom → Python — proposed, pending answers below)

- Ruby's `dig(*keys)` checks both string and symbol keys because Ruby
  YAML/hash access is ambiguous that way. Python's `yaml.safe_load`
  only ever produces `str` keys, so the Python config lookup can just
  be plain `dict.get(...)` chains / a simpler `dig(*keys)` without the
  symbol-fallback — flagging this simplification rather than assuming
  it's fine.
- Ruby's `Tasks::Base` is a class with only classmethods and no
  instances (by design, since future steps pass a `settings` dict
  straight to stateless lookups). Proposed Python equivalent: a class
  with `@classmethod` methods and `NotImplementedError` for
  `task_name`, so `Tasks.Player.provider(settings)` reads the same as
  the Ruby call site.
- `Config#to_s` / `#inspect` → Python `__repr__` on `Config`.
- `File.exist? && File.read(...).strip` → `pathlib.Path.read_text().strip()`
  guarded by `.exists()`.

## Decisions

1. **YAML parsing** — **revised**: use `PyYAML` (`yaml.safe_load`)
   instead of a hand-rolled parser. Originally this called for a
   hand-rolled generic block-YAML parser to avoid a new dependency,
   since Python stdlib has no YAML parser. That parser was built and
   verified against `.boukensha/settings.yaml`, but the constraint was
   lifted mid-port (PyYAML is fine to depend on), so `config.py` now
   calls `yaml.safe_load(...)` directly and the hand-rolled parser
   (`lib/boukensha/_yaml.py`) was deleted. `PyYAML` is declared in
   `requirements.txt` alongside `python-dotenv`.
   `Config.dig(...)` / `Config.tasks(...)` work over the resulting
   plain nested `dict`/`list` structure the same way Ruby's `dig` works
   over the Ruby hash — just without the string/symbol dual-key
   fallback (Python dict keys here are always plain `str`).

2. **`.env` loading** — use `python-dotenv`, the direct Python
   equivalent of Ruby's `dotenv` gem, rather than hand-rolling a
   parser. This mirrors the Ruby README's own accepted exception
   ("we will need to add `dotenv` gem") — same one-dependency carve-out,
   same library family, just the Python package. `Config._load_env`
   calls `dotenv.load_dotenv(env_file)` when
   `<dir>/.env` exists, matching `Dotenv.load(env_file) if
   File.exist?(env_file)` in `config.rb`. This becomes the Python
   port's one declared third-party dependency.

3. **Packaging/imports/isolation** — no `pyproject.toml` at this step,
   matching Ruby's no-gem-packaging-yet approach. `python-dotenv` is
   declared in a plain `requirements.txt` per step
   (`week1_baseline/python/00_config/requirements.txt`).
   Dependency **isolation** (the actual role `bundle exec` +
   `Gemfile`/`Gemfile.lock` plays on the Ruby side) is a single shared
   virtualenv for the whole Python port line, not a per-step venv —
   reasoning being every future `python/NN_*` iteration reuses it, so
   one place to create/activate is simpler than N. Per your answer:
   the venv lives at **the root of the project** and is created
   manually by the user (not auto-created by any script), following
   instructions documented up front in `python/README.md`:
   ```sh
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r python/00_config/requirements.txt
   ```
   Every step's tooling (the `bin/python/<step>` launchers) then
   assumes that venv exists and calls its interpreter directly rather
   than a bare `python3` — see the launcher shape above. Since there's
   no installable package, `examples/example.py` still imports
   `lib/boukensha` via a `sys.path` adjustment (e.g. inserting the
   step's `lib/` dir before importing) — the closest parity to Ruby's
   `require_relative`.
   > Flagging: "root of the project" is being read literally as the
   > git repo root (sibling to `week0_explore/`, `week1_baseline/`,
   > `docs/`) rather than `week1_baseline/`, since that's the more
   > literal reading — correct me if you meant the latter.

4. **Minimum Python version** — target `>=3.11`. The dev machine runs
   3.12.3, so 3.11 is a floor below the actual runtime rather than a
   pin to it, and it's the version `tomllib` landed in stdlib, leaving
   headroom if a later iteration wants it. No version-gated syntax
   needed for this step's code either way.

5. **`bin/python/00_config` launcher** — added per the repo's
   `bin/<language>/<step>` convention (see Target files section above).

## Open questions

None outstanding — all decided above.
