# Python Port Plan — 02 The Registry

## Goal

Port the behavior of `week1_baseline/ruby/02_the_registry/` to
`week1_baseline/python/02_the_registry/`. The directory already exists
but currently holds an unmodified copy of `01_struct_skeleton`'s
`lib/` (verified: its `config.py`, `message.py`, `tool.py`,
`context.py` are byte-identical to `01_struct_skeleton`'s, and
`examples/example.py` still registers tools directly on `Context`,
with no `Registry`, no `errors`, no `dispatch`, and no `UnknownToolError`
handling). This step layers the `Registry` — the piece that lets an
agent call tools *by name* instead of by direct reference — on top of
that unchanged foundation.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff` against `ruby/01_struct_skeleton`:
`config.rb`, `message.rb`, `tool.rb`, `context.rb`, `tasks/base.rb`,
`tasks/player.rb`, and the `Gemfile` are all byte-identical between
the two Ruby steps. Only `registry.rb` and `errors.rb` are new,
`boukensha.rb` adds two requires for them, and `example.rb` is
rewritten to build tools through the registry and exercise
`dispatch`. So this plan only makes new decisions about `Registry`
and error handling; everything already decided in
[`01_struct_skeleton.md`](01_struct_skeleton.md) carries forward
unchanged for the untouched files.

**Ruby README/code mismatch (do not port the README's stated
number):** `ruby/02_the_registry/README.md`'s "Expected Output"
section shows `Context: #<Context turns=0 tools=2 budget=8192>`, but
`context.rb`'s `to_s` has no `budget` field and does include `task=`.
Running `./week1_baseline/bin/ruby/02_the_registry` directly confirms
the real output is `#<Context task=player turns=0 tools=2>` — the
README text is stale (probably drifted from a later step's docs) and
the README's own "Considerations" section already flags that this
step's context/registry split is intentionally left rough ("We now
register tools with the Registry but our code still has direct
registration and tools in context... We'll correct this manually in a
future step"). The Python port follows the **actual running Ruby
code**, not the README's stale expected-output text — this plan's
checklist below uses the verified real output.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/02_the_registry/README.md` | Design spec: registry's two jobs, dispatch flow diagram, `UnknownToolError`, expected output (see mismatch note above) |
| `week1_baseline/ruby/02_the_registry/lib/boukensha/registry.rb` | `Boukensha::Registry` — `tool(name, description:, parameters:, &block)` registers on the context it wraps; `dispatch(name, args)` looks up and calls |
| `week1_baseline/ruby/02_the_registry/lib/boukensha/errors.rb` | `Boukensha::UnknownToolError < StandardError` |
| `week1_baseline/ruby/02_the_registry/lib/boukensha.rb` | Adds `errors` and `registry` requires on top of `01_struct_skeleton`'s |
| `week1_baseline/ruby/02_the_registry/examples/example.rb` | Rewritten smoke test: builds `Registry`, registers `move` + `shout` through it, prints tools from `ctx.tools`, dispatches `shout` and `move` successfully, then dispatches `flee` and catches `UnknownToolError` |
| `week1_baseline/ruby/02_the_registry/lib/boukensha/{config,message,tool,context}.rb`, `tasks/{base,player}.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `01_struct_skeleton` — carry forward as-is (already true of the current Python tree for these files; no changes needed) |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps —
`settings.yaml`, `.env`, `.boukensha/prompts/player/system.md`. No
fixture changes needed for this step.

## Target files to create/change (Python)

```
week1_baseline/python/02_the_registry/
  README.md                            (rewrite: registry docs, new-files table, considerations, corrected expected output)
  requirements.txt                     (unchanged — same two deps)
  examples/example.py                  (rewrite: build Registry, register move+shout, dispatch, catch error)
  lib/boukensha/__init__.py            (add Registry, UnknownToolError exports)
  lib/boukensha/config.py              (unchanged, already correct)
  lib/boukensha/message.py             (unchanged, already correct)
  lib/boukensha/tool.py                (unchanged, already correct)
  lib/boukensha/context.py             (unchanged, already correct)
  lib/boukensha/tasks/__init__.py      (unchanged)
  lib/boukensha/tasks/base.py          (unchanged)
  lib/boukensha/tasks/player.py        (unchanged)
  lib/boukensha/errors.py              (new: UnknownToolError)
  lib/boukensha/registry.py            (new: Registry)
```

Plus a launcher at `week1_baseline/bin/python/02_the_registry`,
matching `bin/python/01_struct_skeleton`'s shape and
`bin/ruby/02_the_registry`'s role (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/02_the_registry"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` (venv setup already
documented there, port-wide) — this step keeps installing into the
same shared root `.venv`.

## Behavior parity checklist (from the real Ruby output, not the stale README text)

- [ ] `UnknownToolError` — a plain exception type, no custom fields,
      matching `StandardError` subclass with no overrides
- [ ] `Registry.__init__(self, context)` stores the context it wraps
- [ ] `Registry.tool(name, *, description, parameters=None)` — a
      decorator factory: builds a `Tool(name, description, parameters
      or {}, block)`, registers it via `context.register_tool`, and
      returns the wrapped function unchanged (see porting notes for
      why a decorator, not a Ruby-style trailing block)
- [ ] `Registry.dispatch(name, args=None)` — looks up
      `context.tools[name]`; raises `UnknownToolError` with message
      `"No tool registered as '{name}'"` if missing; otherwise calls
      `tool.block(**(args or {}))`
- [ ] `examples/example.py` builds `Registry(ctx)`, registers `move`
      and `shout` through it (not directly on `ctx`), prints
      `ctx.tools.values()`, dispatches `shout` then `move`
      successfully, then dispatches `flee` and prints the caught
      `UnknownToolError`'s message

Expected output (verified by actually running
`./week1_baseline/bin/ruby/02_the_registry` — this supersedes the
stale text in the Ruby README):

```
=== BOUKENSHA Step 2: Tool Registry ===

Config:  #<Boukensha::Config dir=/.../.boukensha tasks=player>
Context: #<Context task=player turns=0 tools=2>
Tools:
  #<Tool name=move description=Move the player in a direction (north, so params=[:direction]>
  #<Tool name=shout description=Shout a message so everyone in the zone c params=[:message]>

Dispatching 'shout' with message='dragon spotted'...
Result: DRAGON SPOTTED

Dispatching 'move' with direction='north'...
Result: You move north into a torch-lit corridor.

UnknownToolError caught: No tool registered as 'flee'
```

The Python `Tool` reprs will render `params=['direction']` /
`params=['message']` rather than Ruby's `[:direction]` / `[:message]`
— same cosmetic symbols-vs-strings difference already accepted in the
`01_struct_skeleton` port. Everything else should match verbatim.

## Porting notes (Ruby idiom → Python)

- Ruby's trailing-block DSL —
  `registry.tool("move", description: ..., parameters: ...) do |direction:| ... end`
  — has no direct Python syntax equivalent. The natural Python
  analogue for "register this callable under this name with this
  metadata" is a **decorator factory**:
  ```python
  @registry.tool("move", description="...", parameters={"direction": {"type": "string"}})
  def move(direction):
      return f"You move {direction} into a torch-lit corridor."
  ```
  `Registry.tool(...)` returns an inner `decorator(block)` that
  constructs the `Tool`, registers it, and returns `block` unchanged
  (so the name stays usable as a normal function if needed). This
  keeps the call-site shape close to Ruby's (name + keyword metadata
  immediately followed by the callable body) without inventing a
  block-passing mechanism Python doesn't have.
- Ruby's `dispatch` does `args.transform_keys(&:to_sym)` before
  calling the block, because Ruby needs symbol keys to satisfy a
  block declared with keyword parameters (`|direction:|`) — the API
  hands back string-keyed JSON but Ruby blocks expect symbols. This
  is the "real gotcha" the Ruby README calls out. **Python has no
  symbol/string key duality** — a plain function `def move(direction)`
  already accepts `direction` as a keyword argument from a
  string-keyed dict via `tool.block(**args)`. So the Python
  `dispatch` does the double-splat call directly with no key
  transformation step; the gotcha the Ruby README highlights doesn't
  exist in Python and isn't ported, since porting a workaround for a
  problem the target language doesn't have would be a fabricated
  behavior difference, not parity.
- `parameters` defaults to `None` → `{}` in `Registry.tool` (Ruby
  defaults to `{}` directly in the keyword arg); using `None` as the
  sentinel follows the existing convention already in this codebase
  (e.g. `Config.tasks(name=None)`), rather than a mutable default
  argument.
- `errors.py` is a direct one-for-one port: a single exception class,
  `UnknownToolError(Exception)`, with no custom `__init__` — matching
  Ruby's `UnknownToolError < StandardError; end`.
- `config.py`, `message.py`, `tool.py`, `context.py`, `tasks/base.py`,
  `tasks/player.py` need **no edits** — they're already correct in
  the current `python/02_the_registry/` tree (verified identical to
  `01_struct_skeleton`'s versions, matching the Ruby side being
  byte-identical too). Only `__init__.py` needs new exports.
- The Python example's `BOUKENSHA_DIR` resolution
  (`Path(__file__).resolve().parents[4] / ".boukensha"`) already
  points at the repo root correctly and needs no fix — this is
  unrelated to the Ruby-only bugfix in this step's `example.rb` (its
  `../../../.boukensha` relative path was corrected to
  `../../../../.boukensha`), which was purely a symptom of Ruby's
  `File.expand_path(..., __dir__)` relative-path counting, not
  something the Python path math ever had.

## Decisions

1. **Registry API as a decorator factory** — `registry.tool(name, *,
   description, parameters=None)` returns a decorator that builds and
   registers a `Tool`, then returns the original function. This is
   the closest idiomatic Python match to Ruby's block-based
   registration DSL.

2. **No key-transformation step in `dispatch`** — Python's `**kwargs`
   calling convention already bridges string-keyed dicts to keyword
   parameters, so the Ruby `transform_keys(&:to_sym)` step is dropped
   entirely rather than emulated. This is a deliberate non-difference:
   the underlying gotcha Ruby's README calls out doesn't exist for
   Python callables.

3. **Follow the real Ruby runtime output, not the README's stale
   "Expected Output" text** — the shipped `context.rb`'s `to_s` has
   no `budget=` field; the Python `Context.__repr__` (already correct,
   unchanged from `01_struct_skeleton`) stays as `#<Context
   task=... turns=... tools=...>`, matching what
   `./week1_baseline/bin/ruby/02_the_registry` actually prints.

4. **`errors.py` and `registry.py` as new sibling modules** under
   `lib/boukensha/`, exported from `__init__.py` alongside the
   existing `Config`, `Context`, `Message`, `Player`, `Tool` —
   consistent with the flat per-step `lib/` layout established in
   `01_struct_skeleton.md`.

5. **`requirements.txt` unchanged** — no new dependency; `Registry`
   and `errors` are pure stdlib.

6. **`bin/python/02_the_registry` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/01_struct_skeleton`'s shape.

## Open questions

None outstanding — all decided above.
