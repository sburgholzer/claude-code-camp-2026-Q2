# 02 · The Registry (Rust port)

Behavior port of `ruby/02_the_registry` / `python/02_the_registry` — layers
a `Registry` on top of the `01_struct_skeleton` foundation, letting an
agent call tools *by name* instead of by direct reference. `config.rs`,
`message.rs`, and the `tasks/` modules are unchanged from
`01_struct_skeleton`; see `../01_struct_skeleton/README.md` for those.
`tool.rs` and `context.rs` carry forward too, but **are** edited this step
— see Porting notes below.

## New Files

| File | Description |
|---|---|
| `src/registry.rs` | The `Registry` struct — registers tools and dispatches calls |
| `src/errors.rs` | BOUKENSHA-specific error types |

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

## boukensha_02_the_registry::registry::Registry

| Method | Description |
|---|---|
| `tool(name, description, parameters, block)` | Constructs a `Tool` from the given metadata and closure, and registers it on the context |
| `dispatch(name, args) -> Result<String, UnknownToolError>` | Looks up a tool by name and calls it with the provided args |

## boukensha_02_the_registry::errors::UnknownToolError

Returned when `dispatch` is called with a name that has no registered tool.
A harness needs explicit error boundaries — an unrecognised tool name should never silently fail.

**Example:**
```
No tool registered as 'flee'
```

## Porting notes

- **`Registry::tool(...)` is a direct multi-arg method, not a
  decorator-factory.** Python's `@registry.tool(...)` decorator syntax is a
  Python-specific mechanism with no Rust equivalent; Rust has no
  decorators. Ruby's shape — `registry.tool(name, description:,
  parameters:) { |args| ... }`, a single call taking the name, metadata,
  and a trailing block — has a direct, literal Rust equivalent: a method
  taking `name`, `description`, `parameters`, and a closure argument.
  `Registry::tool` follows Ruby's shape (closest available primitive), not
  Python's (syntax Rust lacks).
- **`Tool.block` widens to `Box<dyn Fn(&HashMap<String, String>) ->
  String>`**, and — unlike Python and Ruby, where `tool.py`/`tool.rb` are
  byte-identical carry-forwards from `01_struct_skeleton` — `tool.rs` *is*
  edited this step. `01_struct_skeleton` narrowed the block signature to
  `Box<dyn Fn(&str) -> String>` because that step's only closure took one
  fixed argument and never got called. `Registry::dispatch` now has to call
  the same block for tools whose closures read differently-named single
  arguments (`move` reads `direction`, `shout` reads `message`) via a
  name→value args map — the direct analogue of Ruby's `tool.block.call(**args)`
  and Python's `tool.block(**args)`, neither of which fix an argument name
  or count at the type level. `&str` can't express "look up an arbitrary
  named key", so the signature widens to take the whole args map and let
  each closure pick its own key out of it.
- **`Context.tools` changes from `std::collections::HashMap` to
  `indexmap::IndexMap`.** The example registers `move` then `shout` and
  must print them in that order in the `Tools:` listing, matching
  Ruby/Python's insertion-ordered `Hash`/`dict` output verbatim.
  `01_struct_skeleton` never surfaced this gap because it only ever
  registered one tool. `indexmap` was already a resolved transitive
  dependency (pulled in by `serde_yaml_ng`, which backs `Tool.parameters`
  with an `indexmap`-based `Mapping` for the same insertion-order reason),
  so this promotes an already-vendored crate to a direct dependency rather
  than adding new supply chain.
- **`Registry<T: Task>` owns its `Context<T>` by value, via a public
  `context` field — not a borrowed reference.** Ruby/Python pass `context`
  into `Registry.new(context)` and keep using the original `ctx` variable
  afterward for direct reads (`ctx.tools.each_value`), because both
  languages hand around a shared mutable reference to the same object.
  Rust's borrow checker won't allow that shape: a `Registry` holding `&'a
  mut Context<T>` would keep that mutable borrow alive for as long as
  `registry` is used for `dispatch`, conflicting with the direct
  `ctx.tools`/`{ctx}` reads the example does in between registration and
  dispatch. Rather than reach for `Rc<RefCell<Context<T>>>` — interior
  mutability machinery this codebase doesn't otherwise use, purely to
  preserve a two-variable shape Rust doesn't need — `Registry` takes
  ownership of the `Context<T>` passed to `Registry::new` and exposes it as
  a public `context` field. Example code that used to read `ctx` directly
  now reads `registry.context` instead (`println!("Context: {}",
  registry.context)`, `registry.context.tools.values()`, etc.) — a real,
  visible divergence from the Ruby/Python call-site shape.
- Ruby's `dispatch` does `args.transform_keys(&:to_sym)` before calling the
  block, because Ruby needs symbol keys to satisfy a block declared with
  keyword parameters (`|direction:|`) — the API hands back string-keyed
  JSON but Ruby blocks expect symbols. **Rust closures reading
  `args.get("direction")` off a `HashMap<String, String>` have no
  symbol/string duality to begin with**, same reasoning Python's own
  README already gives for skipping this: porting a workaround for a
  problem that doesn't exist in the target language would be a fabricated
  behavior difference, not parity.
- `errors.rs` is a direct one-for-one port: a single `UnknownToolError`
  struct implementing `std::error::Error` and `Display`, with no subtypes —
  matching Ruby's `UnknownToolError < StandardError; end` and Python's
  `class UnknownToolError(Exception): pass`. Rust has no exceptions, so
  `Registry::dispatch` returns `Result<String, UnknownToolError>` instead
  of raising; `examples/example.rs` `.unwrap()`s the two successful
  dispatches and `match`es the `flee` dispatch to print the caught error,
  mirroring the Ruby/Python example's `begin/rescue` / `try/except` block.

## Run Example

```bash
./week1_baseline/bin/rust/02_the_registry
```

Expected output (values from your `.boukensha/`):

```
=== BOUKENSHA Step 2: Tool Registry ===

Config:  #<Boukensha::Config dir=/.../.boukensha tasks=player>
Context: #<Context task=player turns=0 tools=2>
Tools:
  #<Tool name=move description=Move the player in a direction (north, so params=["direction"]>
  #<Tool name=shout description=Shout a message so everyone in the zone c params=["message"]>

Dispatching 'shout' with message='dragon spotted'...
Result: DRAGON SPOTTED

Dispatching 'move' with direction='north'...
Result: You move north into a torch-lit corridor.

UnknownToolError caught: No tool registered as 'flee'
```

The `Tool` reprs render `params=["direction"]` / `params=["message"]`
(double-quoted, Rust's `Vec<&str>` `Debug` format) rather than Ruby's
`[:direction]` / `[:message]` or Python's `['direction']` — the same
cosmetic key-representation divergence already accepted in the
`01_struct_skeleton` port. Everything else matches verbatim, including
`Context`'s output.

## Considerations

We now register tools with the Registry, but our code still has direct
registration paths and tools live on the context. This likely should have
been reworked — the context should just hold a reference to the tools it's
using, with the full table of registered tools living on the Registry
instead. We'll correct this manually in a future step; for now things are
left in place, matching the Ruby/Python ports' own acknowledged rough edge.
