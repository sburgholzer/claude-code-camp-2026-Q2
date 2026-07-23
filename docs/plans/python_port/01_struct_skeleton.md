# Python Port Plan — 01 Struct Skeleton

## Goal

Port the behavior of `week1_baseline/ruby/01_struct_skeleton/` to
`week1_baseline/python/01_struct_skeleton/` (directory already exists,
currently empty). End state: a runnable Python example that defines
`Boukensha.Tool`, `Boukensha.Message`, and `Boukensha.Context` — the
three data structures the rest of the port will pass around — and
wires them together exactly like the Ruby example does, reusing the
same `.boukensha/` fixture at the repo root that `00_config` already
validates against.

This is a behavior port, not a redesign — the Ruby version is the
spec. `Config`, `Tasks::Base`, and `Tasks::Player` are unchanged from
`00_config` in this step (confirmed by diffing the Ruby sources — see
below), so this plan only makes new decisions about the three new
structures; everything already decided in
[`00_config.md`](00_config.md) (YAML via PyYAML, `.env` via
python-dotenv, no packaging yet, shared root `.venv`, Python `>=3.11`)
carries forward unchanged.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/01_struct_skeleton/README.md` | Design spec: field tables + `to_s` examples for `Tool`/`Message`/`Context` |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha.rb` | Top-level require entrypoint — adds `tool`, `message`, `context` requires on top of `00_config`'s |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/tool.rb` | `Boukensha::Tool` — `Struct.new(:name, :description, :parameters, :block)` + custom `to_s` |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/message.rb` | `Boukensha::Message` — `Struct.new(:role, :content, :tool_use_id)` + custom `to_s` |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/context.rb` | `Boukensha::Context` — plain class; `task`/`system`/`messages`/`tools`, `register_tool`, `add_message`, `tool_count`/`turn_count`, `to_s` |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/config.rb` | Identical to `00_config`'s **except** it drops the `PROMPTS_DIR` constant (this step ships no `prompts/` dir of its own) — carry forward as-is minus that constant |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/tasks/base.rb` | Byte-identical to `00_config`'s — carry forward unchanged |
| `week1_baseline/ruby/01_struct_skeleton/lib/boukensha/tasks/player.rb` | Byte-identical to `00_config`'s — carry forward unchanged |
| `week1_baseline/ruby/01_struct_skeleton/examples/example.rb` | Runnable smoke test — acceptance test; Python port should have a line-for-line equivalent output |
| `week1_baseline/ruby/01_struct_skeleton/Gemfile` / `Gemfile.lock` | Same single dependency (`dotenv`) as `00_config`, needed because `Config` is still in play |

Confirmed via `diff` against `ruby/00_config`: `tasks/base.rb` and
`tasks/player.rb` are byte-identical; `config.rb`'s only diff is the
missing `PROMPTS_DIR` line; `boukensha.rb` only adds the three new
requires. No other Ruby files changed.

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture as `00_config` — `settings.yaml`, `.env`,
and `.boukensha/prompts/player/system.md`. This step's example still
resolves `player_settings` and computes a `system_prompt` via
`Tasks::Player.system_prompt(..., user_prompts_dir: config.user_prompts_dir)`
(no `default_prompts_dir` passed, matching Ruby, since this step ships
no `prompts/` dir of its own) — it's computed and handed to `Context`
but never printed by the example, so it's exercised but not asserted
on stdout.

## Target files to create (Python)

Mirrors the Ruby layout 1:1:

```
week1_baseline/python/01_struct_skeleton/
  README.md
  requirements.txt                     (same two deps as 00_config)
  examples/example.py
  lib/boukensha/__init__.py
  lib/boukensha/config.py              (copied from 00_config, minus PROMPTS_DIR)
  lib/boukensha/tasks/__init__.py
  lib/boukensha/tasks/base.py          (copied from 00_config, unchanged)
  lib/boukensha/tasks/player.py        (copied from 00_config, unchanged)
  lib/boukensha/tool.py
  lib/boukensha/message.py
  lib/boukensha/context.py
```

Each `python/NN_*` step carries its own full copy of `lib/`, matching
the Ruby layout (each `ruby/NN_*` has its own `lib/boukensha/config.rb`
etc.) and the pattern `00_config` already established — there is no
shared/importable package across steps yet.

Plus a launcher at `week1_baseline/bin/python/01_struct_skeleton`,
matching `bin/python/00_config`'s shape and `bin/ruby/01_struct_skeleton`'s
role (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/01_struct_skeleton"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes needed to `week1_baseline/python/README.md` (venv setup is
already documented there and is port-wide, not per-step) — this step's
own `requirements.txt` installs into that same shared `.venv`.

## Behavior parity checklist (from the Ruby README)

- [ ] `Tool` holds `name`, `description`, `parameters`, `block`, with a
      `to_s`/`__repr__` of the shape
      `#<Tool name=... description=...(41 chars max)... params=[...]>`
- [ ] `Message` holds `role`, `content`, `tool_use_id` (optional,
      default `None`), with a `to_s`/`__repr__` of the shape
      `#<Message role=...[ [id]] content=...(61 chars max)...>` — the
      literal `...` suffix is always appended, not conditional on
      truncation (matches Ruby's unconditional `...`)
- [ ] `Context.__init__` takes `task` (required) and `system`
      (optional, defaults to `None`) as keyword args, matching Ruby's
      `initialize(task:, system: nil)`
- [ ] `Context.register_tool(tool)` stores by `tool.name` in a dict
- [ ] `Context.add_message(role, content, tool_use_id=None)` appends a
      `Message` to the message list
- [ ] `Context.tool_count` / `Context.turn_count` reflect dict/list
      sizes
- [ ] `Context.__repr__` of the shape `#<Context task=... turns=... tools=...>`
- [ ] `Config`, `Tasks.Base`, `Tasks.Player` behave exactly as in
      `00_config` (no re-verification needed beyond that step's
      checklist — just confirm the copied files still pass)
- [ ] `examples/example.py` produces the same fields as
      `examples/example.rb`: config repr, context repr, the `move`
      tool's repr, and both queued messages' reprs

Expected output (values from your `.boukensha/`), from running the
Ruby version:

```
=== Boukensha Step 1: Struct Skeleton ===

Config:   #<Boukensha::Config dir=/.../.boukensha tasks=player>
Context:  #<Context task=player turns=2 tools=1>
Tool:     #<Tool name=move description=Move the player in a direction (north, so params=[:direction]>
Messages:
  #<Message role=user content=Explore north and tell me what you find....>
  #<Message role=assistant content=Sure, let me head north and take a look....>
```

The Python `params=` field will render as `['direction']` rather than
Ruby's `[:direction]` (see porting notes) — everything else should
match verbatim.

## Porting notes (Ruby idiom → Python — proposed, pending answers below)

- Ruby's `Struct.new(:a, :b, ...) do ... end` (lightweight, positional,
  mutable value object + custom method) → Python stdlib
  `@dataclass` for `Tool` and `Message`, with `__repr__` overridden to
  match the Ruby `to_s` format. This is the same "lightweight value
  object, avoid a full class" intent the Ruby README states, using the
  closest stdlib equivalent (no new dependency).
- Ruby symbol keys (`parameters: { direction: {...} }`, `role: :user`)
  have no Python equivalent — the Python port uses plain strings
  throughout (`{"direction": {...}}`, `role="user"`), consistent with
  how `00_config`'s `dig`/`tasks` already dropped Ruby's
  symbol/string dual-key fallback. This means the printed
  `parameters.keys` list renders as `['direction']` instead of Ruby's
  `[:direction]` — a display difference, not a behavior difference,
  called out in the parity checklist above rather than assumed away.
- Ruby's inclusive-range truncation `str[0..40]` / `str[0..60]` (41 /
  61 chars respectively) → Python slices `[:41]` / `[:61]` (Python
  slices are exclusive of the end index, so the upper bound is one
  higher than Ruby's inclusive range endpoint).
- Ruby's safe navigation `task&.task_name` (prints `""` if `task` is
  `nil`) → Python has no direct equivalent; since `Context.task` is
  always provided in practice (Ruby's own constructor requires it as a
  non-optional keyword arg), the Python `__repr__` can just call
  `self.task.task_name()` directly without a None-guard. Flagging
  this as an intentional non-parity edge case (Ruby *could* print a
  blank task name for a nil task; Python would raise) since the
  constructor contract makes it unreachable either way.
- `task` is a **class** reference (`Boukensha::Tasks::Player`, not an
  instance), and `task_name` is itself a class-level accessor — the
  Python equivalent is a classmethod call, `self.task.task_name()`,
  not an instance attribute.
- Ruby's forced-keyword constructor `initialize(task:, system: nil)` →
  Python `def __init__(self, *, task, system=None):`, using
  keyword-only parameters to match the Ruby call-site shape
  (`Context.new(task: ..., system: ...)`).
- `config.py`, `tasks/base.py`, `tasks/player.py` are otherwise a
  straight copy-forward from `00_config`'s Python files (not a
  re-port) — the only edit is dropping `Config.PROMPTS_DIR` from
  `config.py`, mirroring the one line Ruby's own `config.rb` drops in
  this step.

## Decisions

1. **Struct equivalent** — use `@dataclass` (stdlib `dataclasses`) for
   `Tool` and `Message`, each with a hand-written `__repr__` matching
   Ruby's `to_s` format. No new dependency; this is the closest
   stdlib match to Ruby's `Struct.new`.

2. **Symbols → strings** — no attempt to emulate Ruby symbols. All
   dict keys and role values are plain Python strings, consistent with
   the precedent already set in `00_config`. The resulting cosmetic
   diff in the `Tool` repr's `params=` field is accepted, not
   worked around.

3. **`Context` keyword-only args** — `task` and `system` are
   keyword-only (`*, task, system=None`) to mirror Ruby's forced
   keyword constructor syntax at the call site.

4. **Per-step `lib/` copy, no shared package** — `config.py`,
   `tasks/base.py`, `tasks/player.py` are duplicated into
   `python/01_struct_skeleton/lib/boukensha/` rather than imported
   from `python/00_config/`, matching the Ruby port's per-step `lib/`
   duplication (each `ruby/NN_*` is a self-contained tree) rather than
   introducing cross-step Python imports that don't exist on the Ruby
   side.

5. **`requirements.txt`** — duplicated from `00_config`
   (`python-dotenv`, `PyYAML`), same two deps, since `Config` still
   needs both. Installed into the same shared root `.venv` per
   `python/README.md`.

6. **`bin/python/01_struct_skeleton` launcher** — added per the
   repo's `bin/<language>/<step>` convention, matching
   `bin/python/00_config`'s shape.

## Open questions

None outstanding — all decided above.
