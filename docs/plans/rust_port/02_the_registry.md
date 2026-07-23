# Rust Port Plan — 02 The Registry

## Goal

Port the behavior of `week1_baseline/python/02_the_registry/` (itself a
behavior port of `week1_baseline/ruby/02_the_registry/`) to
`week1_baseline/rust/02_the_registry/`. End state: a runnable Rust example
that layers a `Registry` on top of `01_struct_skeleton`'s `Context`, letting
tools be registered under a name and dispatched by name (with args) instead
of invoked directly — against the **same** `.boukensha/` fixture at the repo
root.

**Starting state note:** `rust/02_the_registry/` already exists but is a
stale, unmodified *copy* of `rust/01_struct_skeleton/` — every source file
is byte-for-byte identical to its `01_struct_skeleton` counterpart (verified
via `diff`), the README still says "01 · Struct Skeleton", there's no
`registry.rs` or `errors.rs`, and the workspace root `Cargo.toml` doesn't
even list `rust/02_the_registry` as a member yet. This plan treats those
carried-forward files as the correct starting point (same status as a fresh
copy from `01_struct_skeleton`) and specifies what actually needs to change
on top of them — it is not documenting a diff against working `02` code.

This is a behavior port, not a redesign. Python is the more direct
reference; Ruby's actual `registry.rb`/`errors.rb`/`tool.rb`/`context.rb`
code remains the ultimate spec where the two disagree, same precedent as
the `01_struct_skeleton` plan.

## Source files to port (read these to know what to build)

| File | Role |
|---|---|
| `week1_baseline/python/02_the_registry/README.md` | Design spec: `Registry` class, `UnknownToolError`, decorator-based tool registration, dispatch flow, expected example output |
| `week1_baseline/python/02_the_registry/lib/boukensha/registry.py` | `Registry(context)` — `tool(name, *, description, parameters=None)` decorator factory that constructs a `Tool` and calls `context.register_tool`; `dispatch(name, args=None)` looks up by name, raises `UnknownToolError`, else calls `tool.block(**args)` |
| `week1_baseline/python/02_the_registry/lib/boukensha/errors.py` | `UnknownToolError(Exception)` — single class, no custom `__init__` |
| `week1_baseline/python/02_the_registry/lib/boukensha/tool.py`, `message.py`, `context.py`, `config.py`, `tasks/base.py`, `tasks/player.py` | **Unchanged** from `01_struct_skeleton` (confirmed via `diff` — zero differences) |
| `week1_baseline/python/02_the_registry/lib/boukensha/__init__.py` | Adds `Registry`, `UnknownToolError` to the top-level exports |
| `week1_baseline/python/02_the_registry/examples/example.py` | Runnable smoke test — registers `move` and `shout` tools through the registry, prints config/context/tools, dispatches `shout` then `move`, dispatches an unknown `flee` and catches the error |
| `week1_baseline/ruby/02_the_registry/lib/boukensha/{registry,errors,tool,context}.rb` | Ground truth for actual behavior, incl. `dispatch`'s `args.transform_keys(&:to_sym)` (a Ruby-only symbol/string gotcha the Python port correctly declined to carry over — see Decision 6) |
| `week1_baseline/ruby/02_the_registry/README.md` | Original spec — **note:** its "Expected Output" still shows a stale `budget=8192` field on `Context` that the real `context.rb`/`context.py` code doesn't print (same stale-README pattern already flagged in the `01_struct_skeleton` plan). Follow the real code, not this line. |
| `week1_baseline/rust/01_struct_skeleton/src/{tool,context,config}.rs`, `src/tasks/{base,player}.rs`, `src/lib.rs`, `examples/example.rs` | The actual Rust code this step carries forward and builds on (already duplicated into `rust/02_the_registry/` — see decisions below on what changes) |

## Runtime fixture to reuse (do not duplicate)

Same as `00_config`/`01_struct_skeleton` — `.boukensha/settings.yaml`,
`.boukensha/.env`, `.boukensha/prompts/player/system.md` at the repo root,
via `BOUKENSHA_DIR`, same pattern `rust/01_struct_skeleton/examples/example.rs`
already uses (`CARGO_MANIFEST_DIR` + `../../..` + `.canonicalize()`).

## Decisions (confirmed)

1. **Workspace membership — add the missing member.** The root
   `Cargo.toml` currently stops at `01_struct_skeleton`:
   ```toml
   [workspace]
   resolver = "2"
   members = ["rust/00_config", "rust/01_struct_skeleton", "rust/02_the_registry"]
   ```
   This is a real gap, not a style choice — without it `rust/02_the_registry`
   isn't part of the workspace build at all.

2. **Package rename — `boukensha_02_the_registry`.** The copied-forward
   `rust/02_the_registry/Cargo.toml` still has `name = "boukensha_01_struct_skeleton"`
   (a leftover from the copy). Fix to match the folder, same convention as
   `00_config`/`01_struct_skeleton`.

3. **`tool.rs`, `context.rs` carried forward but NOT byte-identical this
   time — unlike Python/Ruby, where every file `diff` confirms zero
   changes.** Python's `tool.py`/`context.py` didn't need to change between
   `01` and `02` because Python's `Callable` type hint is unenforced (any
   function shape satisfies it) and Python `dict`s are insertion-ordered
   natively. Rust's `01_struct_skeleton` intentionally narrowed
   `Tool.block` to `Box<dyn Fn(&str) -> String>` and `Context.tools` to
   `std::collections::HashMap` — both choices were fine when nothing ever
   *called* a block or depended on tool-iteration order (01's plan Decision
   5 explicitly flagged the block signature as provisional, "will likely
   need to generalize... once a later step actually dispatches tool
   calls"). This is that step. See Decisions 4 and 5.

4. **`Tool.block` generalizes to `Box<dyn Fn(&HashMap<String, String>) -> String>`.**
   `Registry::dispatch` must call the same block for tools whose closures
   take differently-named single arguments (`move` reads `direction`,
   `shout` reads `message`) via a name→value args map — the direct Rust
   analogue of Ruby's `tool.block.call(**args)` and Python's
   `tool.block(**args)`, neither of which fix an argument name or count at
   the type level. `&str` can't express "look up an arbitrary named key",
   so the block signature widens to take the whole args map and pick its
   own key out of it:
   ```rust
   registry.tool("move", "...", params, |args| {
       let direction = args.get("direction").cloned().unwrap_or_default();
       format!("You move {direction} into a torch-lit corridor.")
   });
   ```
   Args values stay plain `String` (not `serde_yaml_ng::Value`) because
   every value actually passed in this step's example is a string — no
   need to reuse `Tool.parameters`'s untyped-`Value` machinery for a field
   that's never anything but a string in practice. `Tool::new`'s block
   parameter type updates to match
   (`impl Fn(&HashMap<String, String>) -> String + 'static`).

5. **`Context.tools` changes from `std::collections::HashMap` to
   `indexmap::IndexMap`.** The example registers `move` then `shout` and
   must print them in that order (`Tools:` listing) to match Ruby/Python's
   insertion-ordered `Hash`/`dict` output verbatim — `01_struct_skeleton`
   never surfaced this because it only ever registered one tool. This is
   the same insertion-order reasoning `01_struct_skeleton`'s Decision 4
   already used to justify `serde_yaml_ng::Value::Mapping` for
   `Tool.parameters` (that type is *itself* backed by `indexmap`
   internally, per that decision's own note) — applying the same fix to
   `Context.tools` is consistent, not a new pattern. Concretely:
   `indexmap` is **already** a resolved transitive dependency in
   `Cargo.lock` (pulled in by `serde_yaml_ng`) at `2.14.0`, so adding
   `indexmap = "2"` directly to `rust/02_the_registry/Cargo.toml` promotes
   an already-vendored crate to a direct dependency rather than pulling in
   new supply chain. `register_tool`'s `self.tools.insert(...)` call is
   unchanged — `IndexMap` has the same `insert`/`get`/`values()` API shape
   HashMap does. `01_struct_skeleton/src/context.rs` itself is **not**
   touched (it never needed this), only `02_the_registry`'s copy.

6. **Ruby's `args.transform_keys(&:to_sym)` gotcha is not ported — same
   call the Python port already made.** Ruby needs it because a block
   declared `|direction:|` demands symbol keys, but the wire format is
   string-keyed JSON. Rust closures reading `args.get("direction")` off a
   `HashMap<String, String>` have no symbol/string duality to begin with —
   porting a workaround for a problem that doesn't exist in the target
   language would be a fabricated behavior difference, not parity. (Same
   reasoning Python's own README already documents for why it skipped this
   too.)

7. **`Registry<T: Task>` owns its `Context<T>` by value, via a `pub
   context` field — not a borrowed reference.** Ruby/Python pass `context`
   into `Registry.new(context)` and keep using the original `ctx` variable
   afterward for direct reads (`ctx.tools.each_value`), because both
   languages hand around a shared mutable reference to the same object.
   Rust's borrow checker won't allow that shape here: if `Registry` held
   `&'a mut Context<T>`, that mutable borrow would stay live for as long as
   `registry` is later used for `dispatch`, conflicting with the direct
   `ctx.tools`/`{ctx}` reads the example does in between registration and
   dispatch. Rather than introduce `Rc<RefCell<Context<T>>>` (interior
   mutability machinery this codebase doesn't otherwise use, purely to
   preserve a two-variable shape Rust doesn't need), `Registry` takes
   ownership of the `Context<T>` passed to `Registry::new`, exposes it as a
   public `context` field, and all example code that used to read `ctx`
   directly reads `registry.context` instead — `println!("Context: {}",
   registry.context)`, `registry.context.tools.values()`, etc. This is a
   real, visible divergence from the Ruby/Python call-site shape and gets
   called out in the README's porting notes, not silently absorbed.

8. **`Registry::tool(...)` is a direct multi-arg method, not a
   decorator-factory.** Python's `@registry.tool(...)` decorator syntax is
   a Python-specific mechanism with no Rust equivalent; Rust has no
   decorators. Ruby's shape — `registry.tool(name, description:,
   parameters:) { |args| ... }`, a single call taking the name, metadata,
   and a trailing block — has a direct, literal Rust equivalent: a method
   taking `name`, `description`, `parameters`, and a closure argument.
   `Registry::tool` follows Ruby's shape (closest available primitive),
   not Python's (syntax Rust lacks):
   ```rust
   pub fn tool(
       &mut self,
       name: impl Into<String>,
       description: impl Into<String>,
       parameters: serde_yaml_ng::Value,
       block: impl Fn(&HashMap<String, String>) -> String + 'static,
   )
   ```

9. **`UnknownToolError` — a plain struct implementing `std::error::Error`,
   not an enum like `ConfigError`.** `rust/01_struct_skeleton/src/config.rs`
   modeled `ConfigError` as an `enum` because Ruby/Python's `Config` layer
   can fail in more than one documented way (`MissingSetting` today, room
   for more). `UnknownToolError` mirrors Ruby's `UnknownToolError <
   StandardError; end` and Python's `class UnknownToolError(Exception):
   pass` — both are a single exception class with no subtypes and no
   custom fields beyond the message. A one-variant enum would invent
   structure the source doesn't have; a struct holding the tool `name` (so
   `Display` can format `"No tool registered as '{name}'"`) matches the
   real shape:
   ```rust
   #[derive(Debug)]
   pub struct UnknownToolError { pub name: String }

   impl fmt::Display for UnknownToolError {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "No tool registered as '{}'", self.name)
       }
   }

   impl std::error::Error for UnknownToolError {}
   ```

10. **`Registry::dispatch` returns `Result<String, UnknownToolError>`, not
    a panic.** Rust has no exceptions; `Result` + the caller's own
    `match`/`.unwrap()` at each call site is the standard substitute for
    Ruby's `raise`/`rescue` and Python's `raise`/`except`. `examples/example.rs`
    `.unwrap()`s the two successful dispatches (`shout`, `move`) and
    `match`es the `flee` dispatch to print the caught-error line, mirroring
    the Ruby/Python example's `begin/rescue` / `try/except` block.

## Target files (Rust)

```
week1_baseline/
  Cargo.toml                        # edit: members += "rust/02_the_registry"
  rust/
    02_the_registry/
      Cargo.toml                    # edit: package name fix + indexmap = "2" added
      README.md                     # rewrite: currently a stale copy of 01's README
      src/
        lib.rs                      # edit: + `pub mod {errors, registry};` + re-exports
        config.rs                   # unchanged (already correct copy from 00_config lineage)
        tasks/
          mod.rs, base.rs, player.rs # unchanged
        tool.rs                     # edit: block: Box<dyn Fn(&HashMap<String, String>) -> String>
        message.rs                  # unchanged
        context.rs                  # edit: tools: IndexMap<String, Tool> (was HashMap)
        errors.rs                   # new: UnknownToolError
        registry.rs                 # new: Registry<T: Task>
      examples/
        example.rs                  # rewrite: registers via registry.tool(...), dispatches 3x
```

Launcher `week1_baseline/bin/rust/02_the_registry` (currently missing —
`bin/python/02_the_registry` and `bin/ruby/02_the_registry` already exist),
matching `bin/rust/01_struct_skeleton`'s shape:
```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../rust/02_the_registry"
cargo run --quiet --example example
```

`02_the_registry/README.md` (ported from `python/02_the_registry/README.md`,
replacing the current stale 01-struct-skeleton content) keeps the "New
Files" table, the "How It Works" ASCII dialogue, the `Registry`/
`UnknownToolError` reference tables, and adds Porting-notes bullets for:
- The `Tool.block` signature widening to `&HashMap<String, String>` this
  step (Decision 4) — and that, unlike Python/Ruby, `tool.rs` isn't a
  byte-identical carry-forward from `01_struct_skeleton` here.
- `IndexMap` swapped in for `Context.tools` to preserve registration order
  in the `Tools:` listing (Decision 5), and that `indexmap` was already a
  resolved transitive dependency.
- `Registry` owning `Context` by value (`registry.context`) instead of
  Ruby/Python's shared-reference `ctx` variable (Decision 7), and why.
- Ruby's `transform_keys(&:to_sym)` gotcha not applying to Rust (Decision 6).
- Same "Considerations" closing note Python's README carries: tools still
  live on `Context` and are reachable both directly and through `Registry`
  — an acknowledged rough edge left in place on purpose, matching the
  Ruby/Python ports.

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`errors.rs`** — see Decision 9 above for the full type.

- **`registry.rs`**:
  ```rust
  use std::collections::HashMap;

  use serde_yaml_ng::Value;

  use crate::context::Context;
  use crate::errors::UnknownToolError;
  use crate::tasks::Task;
  use crate::tool::Tool;

  pub struct Registry<T> {
      pub context: Context<T>,
  }

  impl<T: Task> Registry<T> {
      pub fn new(context: Context<T>) -> Self {
          Self { context }
      }

      pub fn tool(
          &mut self,
          name: impl Into<String>,
          description: impl Into<String>,
          parameters: Value,
          block: impl Fn(&HashMap<String, String>) -> String + 'static,
      ) {
          self.context.register_tool(Tool::new(name, description, parameters, block));
      }

      pub fn dispatch(&self, name: &str, args: &HashMap<String, String>) -> Result<String, UnknownToolError> {
          let tool = self.context.tools.get(name).ok_or_else(|| UnknownToolError { name: name.to_string() })?;
          Ok((tool.block)(args))
      }
  }
  ```

- **`tool.rs` edit** — only the `block` field/constructor parameter type
  changes (`&str` → `&HashMap<String, String>`); `Display` is untouched.

- **`context.rs` edit** — only the `tools` field type changes
  (`std::collections::HashMap` → `indexmap::IndexMap`, plus the import);
  `register_tool`, `tool_count`, `Display` bodies are untouched since the
  API surface used is identical between the two map types.

- **`examples/example.rs` shape**:
  ```rust
  let ctx: Context<Player> = Context::new(system_prompt);
  let mut registry = Registry::new(ctx);

  let move_params: Value = serde_yaml_ng::from_str("direction:\n  type: string\n")?;
  registry.tool(
      "move",
      "Move the player in a direction (north, south, east, west, up, down)",
      move_params,
      |args| {
          let direction = args.get("direction").cloned().unwrap_or_default();
          format!("You move {direction} into a torch-lit corridor.")
      },
  );

  let shout_params: Value = serde_yaml_ng::from_str("message:\n  type: string\n")?;
  registry.tool(
      "shout",
      "Shout a message so everyone in the zone can hear it",
      shout_params,
      |args| args.get("message").cloned().unwrap_or_default().to_uppercase(),
  );

  println!("=== BOUKENSHA Step 2: Tool Registry ===");
  println!();
  println!("Config:  {config}");
  println!("Context: {}", registry.context);
  println!("Tools:");
  for t in registry.context.tools.values() {
      println!("  {t}");
  }
  println!();

  println!("Dispatching 'shout' with message='dragon spotted'...");
  let result = registry
      .dispatch("shout", &HashMap::from([("message".to_string(), "dragon spotted".to_string())]))
      .unwrap();
  println!("Result: {result}");
  println!();

  println!("Dispatching 'move' with direction='north'...");
  let result = registry
      .dispatch("move", &HashMap::from([("direction".to_string(), "north".to_string())]))
      .unwrap();
  println!("Result: {result}");
  println!();

  match registry.dispatch("flee", &HashMap::new()) {
      Ok(_) => unreachable!("flee should not be registered"),
      Err(e) => println!("UnknownToolError caught: {e}"),
  }
  ```
  (No `unreachable!` message leaks into normal output — only reached if the
  test setup itself is broken.)

## Behavior parity checklist (from the Ruby/Python spec)

- [ ] `Registry<T: Task> { context: Context<T> }`; `Registry::new(context)`,
      `tool(name, description, parameters, block)`, `dispatch(name, args) ->
      Result<String, UnknownToolError>`
- [ ] `UnknownToolError { name }` — `Display` renders `No tool registered
      as '<name>'`, implements `std::error::Error`
- [ ] `Tool.block` widened to `Box<dyn Fn(&HashMap<String, String>) ->
      String>`; `Tool` `Display` output unchanged from `01_struct_skeleton`
- [ ] `Context.tools` is an `IndexMap<String, Tool>`; registering `move`
      then `shout` prints them in that order
- [ ] `examples/example.rs` registers `move` and `shout` via
      `registry.tool(...)`, prints `Config`, `Context` (via
      `registry.context`), then both tools in registration order, then
      dispatches `shout` (args: `message=dragon spotted`), `move` (args:
      `direction=north`), then `flee` (unregistered) and prints the caught
      `UnknownToolError`
- [ ] Root `Cargo.toml` workspace `members` includes `rust/02_the_registry`
- [ ] `rust/02_the_registry/Cargo.toml` package name is
      `boukensha_02_the_registry`, deps add `indexmap = "2"`
- [ ] `bin/rust/02_the_registry` launcher exists, mirrors
      `bin/rust/01_struct_skeleton`'s shape
- [ ] `rust/02_the_registry/README.md` rewritten from its current stale
      01-struct-skeleton content to match `python/02_the_registry/README.md`'s
      structure, with Rust-specific porting notes (Decisions 4–7)

## Open questions

None outstanding — all decided above.
