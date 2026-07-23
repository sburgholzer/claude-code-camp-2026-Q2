# 02 · The Registry (Python port)

Behavior port of `ruby/02_the_registry` — layers a `Registry` on top of
the `01_struct_skeleton` foundation, letting an agent call tools *by
name* instead of by direct reference. `config.py`, `message.py`,
`tool.py`, `context.py`, and the `tasks/` classes are unchanged from
`01_struct_skeleton`; see `../01_struct_skeleton/README.md` for those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/registry.py` | The `Registry` class — registers tools and dispatches calls |
| `lib/boukensha/errors.py` | BOUKENSHA-specific error classes |

## How It Works

The agent NEVER calls a tool directly.
It emits a structured request (name and args) and the Registry looks up the tool and runs it.

```
Agent:    "Hey registry call move with direction='north'"
Registry: "looking up "move" in the tool table"
Registry: "Found it now calling the block with the provided args"
Registry: "Here's the result"
Agent:    "Thanks buddy"
Registry: "Thats why you pay me the big tokes"
```

## boukensha.registry.Registry

| Method | Description |
|---|---|
| `tool(name, *, description, parameters=None)` | Decorator factory: registers the decorated function as a new tool on the context |
| `dispatch(name, args=None)` | Looks up a tool by name and calls it with the provided args |

## boukensha.errors.UnknownToolError

Raised when `dispatch` is called with a name that has no registered tool.
A harness needs explicit error boundaries — an unrecognised tool name should never silently fail.

**Example:**
```
UnknownToolError: No tool registered as 'flee'
```

## Porting notes

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
  constructs the `Tool`, registers it, and returns `block` unchanged —
  keeping the call-site shape close to Ruby's (name + keyword metadata
  immediately followed by the callable body) without inventing a
  block-passing mechanism Python doesn't have.
- Ruby's `dispatch` does `args.transform_keys(&:to_sym)` before calling
  the block, because Ruby needs symbol keys to satisfy a block declared
  with keyword parameters (`|direction:|`) — the API hands back
  string-keyed JSON but Ruby blocks expect symbols. **Python has no
  symbol/string key duality** — a plain function `def move(direction)`
  already accepts `direction` as a keyword argument from a
  string-keyed dict via `tool.block(**args)`. So the Python `dispatch`
  does the double-splat call directly with no key transformation step;
  the gotcha the Ruby README calls out doesn't exist in Python and
  isn't ported, since emulating a workaround for a problem the target
  language doesn't have would be a fabricated behavior difference, not
  parity.
- `parameters` defaults to `None` → `{}` in `Registry.tool` (Ruby
  defaults to `{}` directly in the keyword arg); `None` is used as the
  sentinel to follow the existing convention already in this codebase
  (e.g. `Config.tasks(name=None)`), rather than a mutable default
  argument.
- `errors.py` is a direct one-for-one port: a single exception class,
  `UnknownToolError(Exception)`, with no custom `__init__` — matching
  Ruby's `UnknownToolError < StandardError; end`.

## Run Example

```bash
./week1_baseline/bin/python/02_the_registry
```

Expected output (values from your `.boukensha/`):

```
=== BOUKENSHA Step 2: Tool Registry ===

Config:  #<Boukensha::Config dir=/.../.boukensha tasks=player>
Context: #<Context task=player turns=0 tools=2>
Tools:
  #<Tool name=move description=Move the player in a direction (north, so params=['direction']>
  #<Tool name=shout description=Shout a message so everyone in the zone c params=['message']>

Dispatching 'shout' with message='dragon spotted'...
Result: DRAGON SPOTTED

Dispatching 'move' with direction='north'...
Result: You move north into a torch-lit corridor.

UnknownToolError caught: No tool registered as 'flee'
```

The `Tool` reprs render `params=['direction']` / `params=['message']`
rather than Ruby's `[:direction]` / `[:message]` — the same
cosmetic symbols-vs-strings difference already accepted in the
`01_struct_skeleton` port. Everything else matches verbatim, including
`Context`'s output — the shipped Ruby `context.rb`'s `to_s` has no
`budget=` field (unlike the stale text in the Ruby README's own
"Expected Output" section), so this port follows the real
`task=... turns=... tools=...>` shape both languages actually print.

## Considerations

We now register tools with the Registry, but our code still has
direct registration paths and tools live on the context. This likely
should have been reworked — the context should just hold a reference
to the tools it's using, with the full table of registered tools
living on the Registry instead. We'll correct this manually in a
future step; for now things are left in place, matching the Ruby
port's own acknowledged rough edge.
