# Rust Port Plan — 03 Prompt Builder

## Goal

Port the behavior of `week1_baseline/python/03_prompt_builder/` (itself a
behavior port of `week1_baseline/ruby/03_prompt_builder/`) to
`week1_baseline/rust/03_prompt_builder/`. End state: a runnable Rust example
that adds a `PromptBuilder` and five per-provider backend types (Anthropic,
OpenAI, Gemini, Ollama, Ollama Cloud) on top of `02_the_registry`'s
`Registry`/`Context`, serializing a `Context` into the exact JSON payload
shape each provider's API expects — no API calls are made, only payload
assembly — against the **same** `.boukensha/` fixture at the repo root.

**Starting state note:** `rust/03_prompt_builder/` already exists but is a
stale, unmodified *copy* of `rust/02_the_registry/` — every source file is
byte-for-byte identical to its `02_the_registry` counterpart (verified via
`diff` across `context.rs`, `tool.rs`, `registry.rs`, `errors.rs`, `lib.rs`,
`message.rs`, `config.rs`, `tasks/{base,player}.rs`, `examples/example.rs`,
`Cargo.toml`, and `README.md` — zero differences on every one), there's no
`prompt_builder.rs` or `backends/` module, and the workspace root
`Cargo.toml` doesn't list `rust/03_prompt_builder` as a member yet. Same
situation as `02_the_registry`'s starting state: this plan treats those
carried-forward files as the correct starting point, not a diff against
working `03` code.

This is a behavior port, not a redesign. Python is the more direct
reference; Ruby's actual `prompt_builder.rb`/`backends/*.rb`/`errors.rb`
code remains the ultimate spec where the two disagree, same precedent as
the `01_struct_skeleton` and `02_the_registry` plans.

## Source files to port (read these to know what to build)

| File | Role |
|---|---|
| `week1_baseline/python/03_prompt_builder/README.md` | Design spec: `PromptBuilder`, five backends, model tables, system-prompt/tool-result/tool-definition/role JSON shape differences per provider, expected example output |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/prompt_builder.py` | `PromptBuilder(context, backend)` — `to_messages()`, `to_tools()` (unused passthroughs, see Decision 8), `to_api_payload(max_output_tokens=1024)`, `headers`/`url` properties, all delegating to `backend` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/base.py` | `Base` — `MODELS`/`validate_model`/`_model_info` classmethods, `model_info`/`context_window`/`input_token_cost_per_million`/`output_token_cost_per_million`/`usage_unit`/`usage_level` properties, `estimate_cost(input_tokens=, output_tokens=)`, `_configure_model(model)` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/anthropic.py` | `Anthropic(Base)` — `to_messages`/`to_tools` take one arg (messages/tools only); `to_payload` puts `system` as a top-level field |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/openai.py` | `OpenAI(Base)` — `to_messages(system, messages)` (two args), prepends a `role: system` message; `to_tools` wraps each tool in `{"type": "function", "function": {...}}`; tool results use `tool_call_id` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/ollama.py` | `Ollama(Base)` — same shape as `OpenAI.to_messages`/`to_tools` but tool results use `tool_name`; no API key, `host` defaults to `http://localhost:11434` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/ollama_cloud.py` | `OllamaCloud(Base)` — identical `to_messages`/`to_tools` bodies to `Ollama.py` (byte-for-byte in the Python source), but takes an API key and a fixed `https://ollama.com` host |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/backends/gemini.py` | `Gemini(Base)` — one-arg `to_messages`; renames `assistant`→`model`, wraps everything in `parts`, tool results become `functionResponse` parts, tools wrapped in a single `functionDeclarations` array, `system` becomes `systemInstruction` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/errors.py` | Adds `UnsupportedModelError(Exception)` alongside the existing `UnknownToolError` |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/config.py` | Adds `PROMPTS_DIR` — the library's own default-prompts directory, computed once from `__file__` |
| `week1_baseline/python/03_prompt_builder/prompts/system.md` | Library-default system prompt (used when a task doesn't override it) |
| `week1_baseline/python/03_prompt_builder/lib/boukensha/{message,tool,context,registry}.py`, `tasks/{base,player}.py` | **Unchanged** from `02_the_registry` (confirmed via `diff` — zero differences) |
| `week1_baseline/python/03_prompt_builder/examples/example.py` | Runnable smoke test — registers `look` (no params) and `move` tools, adds three messages directly to `ctx` (including a `tool_result`), resolves `provider`/`model` from task settings, picks a backend via `if`/`elif`, builds a `PromptBuilder`, and prints `Config`/`Provider`/`Model`/the full JSON payload |
| `week1_baseline/ruby/03_prompt_builder/lib/boukensha/{prompt_builder,errors,tool,config}.rb`, `lib/boukensha/backends/{base,anthropic,openai,ollama,ollama_cloud,gemini}.rb` | Ground truth for actual behavior — confirms `Base.model_info` is both a class and instance method (Ruby-only, see Decision 9), `Tool` stays a plain `Struct` with symbol-keyed `parameters` |
| `week1_baseline/rust/02_the_registry/src/{tool,context,message,registry,errors,config}.rs`, `src/tasks/{base,player}.rs`, `src/lib.rs`, `examples/example.rs` | The actual Rust code this step carries forward and builds on (already duplicated into `rust/03_prompt_builder/` — see decisions below on what changes vs. what's a no-op carry-forward) |

## Runtime fixture to reuse (do not duplicate)

Same as `00_config`/`01_struct_skeleton`/`02_the_registry` —
`.boukensha/settings.yaml`, `.boukensha/.env`,
`.boukensha/prompts/player/system.md` at the repo root, via `BOUKENSHA_DIR`,
same pattern `rust/02_the_registry/examples/example.rs` already uses
(`CARGO_MANIFEST_DIR` + `../../..` + `.canonicalize()`). The fixture's
`tasks.player` settings already have `provider: anthropic`, `model:
claude-haiku-4-5`, and `prompt_override.system: true`; `.boukensha/.env`
already declares a real `ANTHROPIC_API_KEY`. No fixture changes needed.

## Decisions (confirmed)

1. **Workspace membership — add the missing member.** Root `Cargo.toml`
   currently stops at `02_the_registry`:
   ```toml
   [workspace]
   resolver = "2"
   members = ["rust/00_config", "rust/01_struct_skeleton", "rust/02_the_registry"]
   ```
   Add `"rust/03_prompt_builder"` — without it the crate isn't part of the
   workspace build at all, same real gap as `02_the_registry`'s starting
   state.

2. **Package rename — `boukensha_03_prompt_builder`.** The copied-forward
   `rust/03_prompt_builder/Cargo.toml` still says `name =
   "boukensha_02_the_registry"`. Fix to match the folder, same convention
   as every prior step.

3. **New dependency: `serde_json` with `preserve_order`.** Unlike
   `indexmap` in the `02_the_registry` step (already a resolved transitive
   dependency via `serde_yaml_ng`, promoted to direct), `serde_json` is
   **not** currently in `Cargo.lock` at all — this is a genuinely new
   dependency, needed because the whole point of this step is producing a
   JSON payload (`json.dumps(..., indent=2)` in Python, `JSON.pretty_generate`
   in Ruby). Add:
   ```toml
   serde_json = { version = "1", features = ["preserve_order"] }
   ```
   `preserve_order` (backed by the `indexmap` already in this workspace)
   is not optional decoration — without it, `serde_json::Map` iterates in
   sorted-key order, silently reordering a tool's `properties`/`required`
   keys relative to the YAML mapping's insertion order, which is exactly
   the ordering bug `02_the_registry`'s Decision 5 already fixed once for
   `Context.tools` by switching to `IndexMap`. `serde_json::to_string_pretty`
   uses 2-space indentation matching Python's `json.dumps(indent=2)` and
   Ruby's `JSON.pretty_generate` closely enough to reproduce the same
   structure and content (exact byte-for-byte whitespace parity across all
   three languages' JSON writers is not independently verified by this
   plan — only structural/content equivalence is a hard requirement).

4. **`message.rs`, `tool.rs`, `context.rs`, `registry.rs`, `errors.rs`
   (existing `UnknownToolError`), `tasks/{base,player}.rs` are carried
   forward unedited from `02_the_registry`** — mirroring the Python port's
   own finding ("needed no edits... already correct from
   `02_the_registry`"). `tool.rs`'s `parameters: Value`
   (`serde_yaml_ng::Value`) in particular is **not** changed to
   `serde_json::Value` — see Decision 7 for why the YAML→JSON conversion
   happens at the backend boundary instead of upstream in `Tool`.

5. **`UnsupportedModelError` added to `errors.rs`, alongside
   `UnknownToolError`.** Ruby/Python both define it as a bare exception
   class with no custom fields — the *message itself* (backend name +
   attempted model + sorted supported list) is assembled at the raise
   site, not from structured fields. The direct Rust analogue is a
   newtype wrapping the fully-formatted string, not a struct with a
   `name` field like `UnknownToolError` (which does have real structured
   data — `UnknownToolError`'s `name` field is reused by its `Display`
   impl; `UnsupportedModelError`'s message has three pieces of context
   baked in already, so a second `Display`-computed field would just
   duplicate string-building logic that already happened once at the
   call site):
   ```rust
   #[derive(Debug)]
   pub struct UnsupportedModelError(pub String);

   impl fmt::Display for UnsupportedModelError {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "{}", self.0)
       }
   }

   impl std::error::Error for UnsupportedModelError {}
   ```

6. **`Config::PROMPTS_DIR` restored verbatim from `rust/00_config` — not
   reinvented as a new function.** This is not new ground: `rust/00_config`
   already solved "Rust has no `__file__`" once, and documented the
   tradeoff explicitly in its README's "Shipped-prompts-dir resolution"
   section. `01_struct_skeleton`'s own plan (Decision 6) then *deliberately
   dropped* `PROMPTS_DIR` and the `prompts/` directory — matching Python's
   and Ruby's own drop, confirmed via `diff` — because nothing used it
   until a later step reintroduced the concept. This step is that later
   step. Reusing the exact original name and shape (rather than a
   differently-named/differently-typed function invented fresh for this
   step) is what keeps this a restoration of dropped-then-reintroduced
   code, matching how Python/Ruby's own `03_prompt_builder/config.py`/
   `config.rb` re-add the identical `PROMPTS_DIR` attribute they'd
   dropped in `01_struct_skeleton`, not a new mechanism:
   ```rust
   impl Config {
       // Default prompts shipped alongside the library code, baked in at
       // compile time (see rust/00_config/README.md's Design Considerations
       // for the tradeoffs of this vs. `include_str!`).
       pub const PROMPTS_DIR: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/prompts");

       // ...existing methods unchanged
   }
   ```
   Same tradeoff `00_config` already accepted "for now" applies unchanged
   here: the constant bakes the build machine's absolute path into the
   binary, which is fine because this project only ever builds and runs
   from the same repo checkout — not a new risk introduced by this step,
   and not something this step needs to revisit (a future packaging step
   would, per `00_config`'s own README note).

7. **`Tool.parameters` stays `serde_yaml_ng::Value`; conversion to JSON
   happens inside each backend's `to_tools`, not upstream in `Tool`
   itself.** Ruby/Python tool parameter dicts are already the same
   in-memory representation used for both authoring (`{"direction": {...}}`
   literal) and JSON output (`dict`/`Hash` serializes straight to JSON) —
   there's no format boundary to cross. Rust's `Tool.parameters` uses
   `serde_yaml_ng::Value` specifically because tools are authored via YAML
   literals (`serde_yaml_ng::from_str("direction:\n  type: string\n")`,
   established in `02_the_registry`), and changing that field's type now
   would be an unrelated, unforced edit to a file this step's own Python
   precedent says needs none (Decision 4). Converting at the backend
   boundary instead — `serde_json::to_value(&tool.parameters)` — is a
   single generic, lossless re-serialization (YAML's `Value` and JSON's
   `Value` model the same scalar/sequence/mapping shapes) and keeps the
   YAML→JSON crossing localized to exactly where it's needed: producing
   an API payload. A shared helper in `backends/base.rs` do the
   YAML→JSON conversion plus `required` key extraction once, reused by
   every backend's own `to_tools` (see Decision 11):
   ```rust
   /// Converts a tool's YAML-authored parameter schema into
   /// (`properties`, `required`) for a JSON Schema-shaped `parameters`/
   /// `input_schema` object.
   pub fn schema_parts(parameters: &Value) -> (serde_json::Value, Vec<String>) {
       let properties = serde_json::to_value(parameters).expect("parameters must serialize to JSON");
       let required = properties
           .as_object()
           .map(|m| m.keys().cloned().collect())
           .unwrap_or_default();
       (properties, required)
   }
   ```
   Ruby's `parameters.keys.map(&:to_s)` stringify-step (needed because
   Ruby tool params are symbol-keyed) has no Rust equivalent to carry —
   same reasoning `02_the_registry`'s Decision 6 already used for
   `transform_keys(&:to_sym)`; Rust's YAML mapping keys are already
   strings.

8. **`Backend` (model catalog) and `PromptBackend<T: Task>` (payload
   serialization) are two separate traits, not one.** Ruby/Python's
   `Base`/`Anthropic`/etc. mix two genuinely different kinds of behavior
   in one class: (a) model-table lookup and cost/usage accessors, which
   are identical in *shape* across all five backends and don't depend on
   any `Context`/`Task`, and (b) `to_payload`/`headers`/`url`, which do
   depend on a generic `Context<T>`. Rust's trait system can express (a)
   as a plain, non-generic trait with default methods (mirroring the
   `Base` mixin exactly), but (b) must be generic over `T: Task` because
   `Context<T>` is. Splitting them means `PromptBuilder<T>` can hold
   `Box<dyn PromptBackend<T>>` (a trait object — the direct Rust analogue
   of "whichever backend you pass in", matching the README's own framing)
   while the model-catalog side stays simple associated-function lookup
   that never needs dynamic dispatch:
   ```rust
   // backends/base.rs
   #[derive(Debug, Clone)]
   pub struct CostPerMillion {
       pub input: Option<f64>,
       pub output: Option<f64>,
   }

   #[derive(Debug, Clone)]
   pub struct ModelInfo {
       pub context_window: u64,
       pub cost_per_million: CostPerMillion,
       pub usage_unit: &'static str,
       pub usage_level: Option<&'static str>,
   }

   pub trait Backend {
       fn backend_name() -> &'static str where Self: Sized;
       fn models() -> HashMap<&'static str, ModelInfo> where Self: Sized;
       fn info(&self) -> &ModelInfo;

       fn validate_model(model: &str) -> Result<String, UnsupportedModelError>
       where
           Self: Sized,
       {
           let table = Self::models();
           if table.contains_key(model) {
               return Ok(model.to_string());
           }
           let mut supported: Vec<&str> = table.keys().copied().collect();
           supported.sort();
           Err(UnsupportedModelError(format!(
               "{} does not support model '{}'. Supported models: {}",
               Self::backend_name(), model, supported.join(", ")
           )))
       }

       fn context_window(&self) -> u64 { self.info().context_window }
       fn input_token_cost_per_million(&self) -> Option<f64> { self.info().cost_per_million.input }
       fn output_token_cost_per_million(&self) -> Option<f64> { self.info().cost_per_million.output }
       fn usage_unit(&self) -> &'static str { self.info().usage_unit }
       fn usage_level(&self) -> Option<&'static str> { self.info().usage_level }

       fn estimate_cost(&self, input_tokens: u64, output_tokens: u64) -> Option<f64> {
           let input_cost = self.input_token_cost_per_million()?;
           let output_cost = self.output_token_cost_per_million()?;
           Some(((input_tokens as f64 * input_cost) + (output_tokens as f64 * output_cost)) / 1_000_000.0)
       }
   }

   pub trait PromptBackend<T: crate::tasks::Task> {
       fn to_payload(&self, context: &crate::context::Context<T>, max_output_tokens: u32) -> serde_json::Value;
       fn headers(&self) -> indexmap::IndexMap<String, String>;
       fn url(&self) -> String;
   }
   ```
   `models()` builds a fresh `HashMap` on every call instead of a
   lazily-initialized static — the tables are small (3-9 entries), only
   ever consulted once per backend construction (not a hot path), and
   this avoids pulling in `once_cell`/`lazy_static` as a new dependency
   purely to memoize something this cheap.

9. **Name collision Python had to dodge (`Base.model_info` classmethod
   vs. instance method) does not apply to Rust, and needs no analogous
   workaround.** Ruby's class methods and instance methods occupy
   separate namespaces, so `Anthropic.model_info(model)` (class) and
   `anthropic_instance.model_info` (instance) never collide; Python has
   one namespace per class, forcing the port to rename the classmethod to
   `_model_info`. Rust's associated functions (`Self::models()`,
   `Self::validate_model()`) and instance methods (`self.info()`, the
   accessor methods) already live in non-overlapping call syntaxes —
   `Type::function()` vs. `value.method()` — so no rename is needed here.
   Called out only so it isn't mistaken for an oversight.

10. **`Backend::models()`/`Self: Sized` bounds mean `Backend` itself is
    never used as a trait object — only `PromptBackend<T>` is.** This is
    intentional, not a limitation worked around: nothing in
    `example.rs` needs to call `context_window()`/`estimate_cost()`
    through a `Box<dyn Backend>` — those are documented public API
    surface (per the README's "Backend instances expose...") that each
    concrete struct still exposes directly, exactly as Ruby/Python do
    (their `example.rb`/`example.py` don't call these either — they're
    part of the API contract, not the example's own smoke test).

11. **The identical `to_tools` body shared by `Ollama`/`OpenAI`/
    `OllamaCloud` in the Python/Ruby source (byte-for-byte the same
    function in all three files), and the near-identical `to_messages`
    body shared by `Ollama`/`OllamaCloud` (differing from `OpenAI` only
    in one field name, `tool_name` vs. `tool_call_id`), are factored into
    two small shared helpers in `backends/base.rs` rather than tripled
    verbatim.** This is a deliberate, minimal dedup — not a source-structure
    deviation — because the *output* stays identical to what three
    separate copy-pasted methods would produce; per this project's own
    convention against needless duplication, and unlike the trait split
    in Decision 8 (which exists because Rust's type system forces it),
    this dedup is optional but has no behavioral cost:
    ```rust
    // backends/base.rs
    pub fn function_wrapped_tools(tools: &IndexMap<String, Tool>) -> Vec<serde_json::Value> {
        tools
            .values()
            .map(|tool| {
                let (properties, required) = schema_parts(&tool.parameters);
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": {
                            "type": "object",
                            "properties": properties,
                            "required": required,
                        },
                    },
                })
            })
            .collect()
    }

    pub fn chat_style_messages(system: &str, messages: &[Message], tool_id_field: &str) -> Vec<serde_json::Value> {
        let mut result = vec![serde_json::json!({"role": "system", "content": system})];
        result.extend(messages.iter().map(|msg| {
            if msg.role == "tool_result" {
                serde_json::json!({
                    "role": "tool",
                    tool_id_field: msg.tool_use_id,
                    "content": msg.content,
                })
            } else {
                serde_json::json!({"role": msg.role, "content": msg.content})
            }
        }));
        result
    }
    ```
    `Ollama`/`OllamaCloud` call `chat_style_messages(system, messages, "tool_name")`;
    `OpenAI` calls `chat_style_messages(system, messages, "tool_call_id")`.
    `Anthropic`/`Gemini`'s `to_messages` are genuinely different shapes
    (tool results as content blocks / functionResponse parts, no
    system-message prefix) and stay as separate inherent methods, matching
    the source.

12. **`PromptBuilder<'a, T: Task>` borrows `&'a Context<T>`; it does not
    own it.** Ruby/Python's `PromptBuilder.new(context, backend)` stores
    a reference to the same `context`/`ctx` object the caller keeps using
    elsewhere (e.g., for `Config`/`Context` printing). Since
    `02_the_registry`'s Decision 7 already made `Registry` own its
    `Context<T>` by value (`registry.context`), and nothing after
    `PromptBuilder` construction needs to mutate context further in this
    step's example, `PromptBuilder` borrowing `&registry.context`
    (immutable) is the direct, zero-cost Rust equivalent — no `Rc`/`RefCell`
    needed, extending the same reasoning Decision 7 already established:
    ```rust
    pub struct PromptBuilder<'a, T: Task> {
        context: &'a Context<T>,
        backend: Box<dyn PromptBackend<T>>,
    }

    impl<'a, T: Task> PromptBuilder<'a, T> {
        pub fn new(context: &'a Context<T>, backend: Box<dyn PromptBackend<T>>) -> Self {
            Self { context, backend }
        }

        pub fn to_api_payload(&self, max_output_tokens: u32) -> serde_json::Value {
            self.backend.to_payload(self.context, max_output_tokens)
        }

        pub fn headers(&self) -> indexmap::IndexMap<String, String> {
            self.backend.headers()
        }

        pub fn url(&self) -> String {
            self.backend.url()
        }
    }
    ```

13. **`PromptBuilder.to_messages()`/`to_tools()` are dropped from the
    Rust port — not carried forward, even as a "known rough edge."**
    Python's own porting notes flag that these two zero-arg passthroughs
    aren't uniformly callable across backends already (`Anthropic`/`Gemini`
    take one arg, `OpenAI`/`Ollama`/`OllamaCloud` take two) and only ship
    because neither `example.rb` nor `example.py` ever calls them — Python
    and Ruby's duck typing lets an never-exercised method with a
    backend-dependent arity mismatch sit unnoticed. Rust's trait system
    can't paper over that the same way: putting `to_messages`/`to_tools`
    on `PromptBackend<T>` would force one arity for every implementor,
    which is false for this codebase (Decision 8/11), and Rust has no
    duck-typed "call whatever shape happens to be there" fallback. Rather
    than fabricate a fake-uniform signature that doesn't match any real
    backend, or wrap every backend's `to_messages`/`to_tools` in an
    `enum`/boxed-closure just to satisfy an interface nothing calls, this
    port omits the two methods from the public builder API entirely; each
    backend still exposes its own inherent `to_messages`/`to_tools` (used
    internally by its own `to_payload`), so no behavior is lost — only the
    unused, arity-inconsistent passthrough wrapper. Call this out
    explicitly in the README's porting notes as a real, intentional
    divergence (same treatment Decision 7 in `02_the_registry`'s plan gave
    its own divergence), not a silently dropped feature.

14. **No default arguments — `to_api_payload`/`estimate_cost` take
    required parameters.** Ruby's `max_output_tokens: 1024` keyword
    default and Python's `max_output_tokens=1024` have no Rust syntax
    equivalent; every call site in `examples/example.rs` passes `1024`
    explicitly (`builder.to_api_payload(1024)`), same treatment other
    ports in this series give missing default-argument syntax.

15. **Provider dispatch: `if`/`elif`/`raise ValueError` → `match` +
    `panic!`.** Matches this step's own unhandled-exception semantics —
    neither `example.rb` nor `example.py` catches an unsupported-provider
    error, so an uncaught Rust `panic!` with the same message
    (`"Unsupported provider for player task: {other}"`) is the direct
    equivalent, not a `Result`-returning refactor of example code that the
    source never asked for. Each branch constructs a `Box<dyn
    PromptBackend<Player>>` from the concrete backend's fallible `new`
    (`Result<Self, UnsupportedModelError>`, since model validation can
    fail), `.expect()`'d — same non-goal as above: this step isn't adding
    error-recovery UX to the example, it's porting payload assembly.

16. **New `prompts/system.md`** — copied verbatim (identical byte-for-byte
    between the Python and Ruby fixtures already, confirmed via `diff`)
    into `rust/03_prompt_builder/prompts/system.md`. This is the
    library-default system prompt; it is not the one printed in this
    step's actual output (the `.boukensha/prompts/player/system.md`
    fixture override wins, since `prompt_override.system: true` is set),
    but it must exist for `Config::PROMPTS_DIR`/`Task::system_prompt`'s
    fallback path to be exercised correctly by anyone who unsets that
    override.

## Target files (Rust)

```
week1_baseline/
  Cargo.toml                        # edit: members += "rust/03_prompt_builder"
  rust/
    03_prompt_builder/
      Cargo.toml                    # edit: package name fix + serde_json (preserve_order) added
      README.md                     # rewrite: currently a stale copy of 02's README
      prompts/
        system.md                  # new: library-default system prompt
      src/
        lib.rs                      # edit: + `pub mod {backends, prompt_builder};` + re-exports
        config.rs                   # edit: restore `Config::PROMPTS_DIR` const (dropped in 01_struct_skeleton)
        tasks/
          mod.rs, base.rs, player.rs # unchanged
        tool.rs                     # unchanged
        message.rs                  # unchanged
        context.rs                  # unchanged
        errors.rs                   # edit: + UnsupportedModelError
        registry.rs                 # unchanged
        prompt_builder.rs           # new: PromptBuilder<'a, T: Task>
        backends/
          mod.rs                   # new: `pub mod {base, anthropic, ollama, ollama_cloud, openai, gemini};` + re-exports
          base.rs                  # new: ModelInfo, CostPerMillion, Backend trait, PromptBackend<T> trait, schema_parts, function_wrapped_tools, chat_style_messages
          anthropic.rs             # new
          ollama.rs                # new
          ollama_cloud.rs          # new
          openai.rs                # new
          gemini.rs                # new
      examples/
        example.rs                  # rewrite: registers look+move, adds 3 messages, dispatches provider, builds+prints payload
```

Launcher `week1_baseline/bin/rust/03_prompt_builder` (currently missing —
`bin/python/03_prompt_builder` already exists), matching
`bin/rust/02_the_registry`'s shape:
```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../rust/03_prompt_builder"
cargo run --quiet --example example
```

`03_prompt_builder/README.md` (ported from
`python/03_prompt_builder/README.md`, replacing the current stale
02-the-registry content) keeps the "New Files" table, the "How It Works"
diagram, the backends/model-table reference tables, the system-prompt/
tool-result/tool-definition/message-role JSON comparison blocks, and the
"Considerations" closing notes verbatim (those are provider-API facts, not
language-specific), and adds Porting-notes bullets for:
- The `Backend`/`PromptBackend<T>` trait split and why (Decision 8).
- `PromptBuilder.to_messages()`/`to_tools()` dropped entirely rather than
  carried forward as an unreachable rough edge (Decision 13) — the one
  porting note that most needs to be visible, since it's a real capability
  reduction versus Ruby/Python's public surface (even though nothing calls
  it).
- `Config::PROMPTS_DIR` restored from `00_config` (dropped in
  `01_struct_skeleton`, matching Python/Ruby's own drop-and-reintroduce),
  not reinvented under a new name/shape (Decision 6).
- `serde_json` as a genuinely new dependency (not a promoted transitive
  one, unlike `indexmap` in `02_the_registry`), and why `preserve_order`
  is required, not cosmetic (Decision 3).
- The `Ollama`/`OpenAI`/`OllamaCloud` `to_tools`/`to_messages` dedup into
  shared helpers (Decision 11) — an optional simplification, not a forced
  one.
- No default arguments; explicit `1024` at call sites (Decision 14).

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`errors.rs`** — see Decision 5 above for `UnsupportedModelError`;
  `UnknownToolError` unchanged from `02_the_registry`.

- **`backends/base.rs`** — see Decisions 8 and 11 above for the full
  `Backend`/`PromptBackend<T>` traits and shared helpers.

- **A concrete backend (`backends/anthropic.rs`)**, representative of the
  one-arg-`to_messages` family (`Gemini` is the only other member):
  ```rust
  use std::collections::HashMap;

  use indexmap::IndexMap;
  use serde_json::json;

  use crate::context::Context;
  use crate::errors::UnsupportedModelError;
  use crate::message::Message;
  use crate::tasks::Task;
  use crate::tool::Tool;

  use super::base::{schema_parts, Backend, CostPerMillion, ModelInfo, PromptBackend};

  const BASE_URL: &str = "https://api.anthropic.com/v1/messages";

  pub struct Anthropic {
      api_key: String,
      model: String,
      info: ModelInfo,
  }

  impl Anthropic {
      pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, UnsupportedModelError> {
          let model = model.into();
          let validated = Self::validate_model(&model)?;
          let info = Self::models().get(validated.as_str()).cloned().expect("validated model must exist in table");
          Ok(Self { api_key: api_key.into(), model: validated, info })
      }

      fn to_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
          messages
              .iter()
              .map(|msg| {
                  if msg.role == "tool_result" {
                      json!({
                          "role": "user",
                          "content": [{
                              "type": "tool_result",
                              "tool_use_id": msg.tool_use_id,
                              "content": msg.content,
                          }],
                      })
                  } else {
                      json!({"role": msg.role, "content": msg.content})
                  }
              })
              .collect()
      }

      fn to_tools(&self, tools: &IndexMap<String, Tool>) -> Vec<serde_json::Value> {
          tools
              .values()
              .map(|tool| {
                  let (properties, required) = schema_parts(&tool.parameters);
                  json!({
                      "name": tool.name,
                      "description": tool.description,
                      "input_schema": {
                          "type": "object",
                          "properties": properties,
                          "required": required,
                      },
                  })
              })
              .collect()
      }
  }

  impl Backend for Anthropic {
      fn backend_name() -> &'static str { "Anthropic" }

      fn models() -> HashMap<&'static str, ModelInfo> {
          HashMap::from([
              ("claude-haiku-4-5", ModelInfo {
                  context_window: 200_000,
                  cost_per_million: CostPerMillion { input: Some(1.0), output: Some(5.0) },
                  usage_unit: "tokens",
                  usage_level: None,
              }),
              ("claude-haiku-4-5-20251001", ModelInfo {
                  context_window: 200_000,
                  cost_per_million: CostPerMillion { input: Some(1.0), output: Some(5.0) },
                  usage_unit: "tokens",
                  usage_level: None,
              }),
              ("claude-sonnet-4-6", ModelInfo {
                  context_window: 1_000_000,
                  cost_per_million: CostPerMillion { input: Some(3.0), output: Some(15.0) },
                  usage_unit: "tokens",
                  usage_level: None,
              }),
              ("claude-opus-4-8", ModelInfo {
                  context_window: 1_000_000,
                  cost_per_million: CostPerMillion { input: Some(5.0), output: Some(25.0) },
                  usage_unit: "tokens",
                  usage_level: None,
              }),
          ])
      }

      fn info(&self) -> &ModelInfo { &self.info }
  }

  impl<T: Task> PromptBackend<T> for Anthropic {
      fn to_payload(&self, context: &Context<T>, max_output_tokens: u32) -> serde_json::Value {
          json!({
              "model": self.model,
              "system": context.system,
              "max_tokens": max_output_tokens,
              "tools": self.to_tools(&context.tools),
              "messages": self.to_messages(&context.messages),
          })
      }

      fn headers(&self) -> IndexMap<String, String> {
          IndexMap::from([
              ("Content-Type".to_string(), "application/json".to_string()),
              ("x-api-key".to_string(), self.api_key.clone()),
              ("anthropic-version".to_string(), "2023-06-01".to_string()),
          ])
      }

      fn url(&self) -> String {
          BASE_URL.to_string()
      }
  }
  ```
  `Ollama`/`OpenAI`/`OllamaCloud` follow the same shape but call
  `chat_style_messages`/`function_wrapped_tools` from `backends::base`
  instead of defining their own `to_messages`/`to_tools` bodies (Decision
  11); `Ollama::new` takes `host: Option<String>` defaulting to
  `"http://localhost:11434"` and no API key; `Gemini` keeps its own
  `to_messages`/`to_tools` (renamed role, `functionResponse`/
  `functionDeclarations` wrapping) but otherwise follows the same
  `Backend`/`PromptBackend<T>` impl pattern.

- **`prompt_builder.rs`** — see Decision 12 above for the full type.

- **`config.rs` edit** — only the `PROMPTS_DIR` const is restored
  (Decision 6); every existing method is untouched.

- **`examples/example.rs` shape**:
  ```rust
  use std::collections::HashMap;
  use std::env;
  use std::path::Path;

  use serde_yaml_ng::Value;

  use boukensha_03_prompt_builder::backends::{Anthropic, Gemini, Ollama, OllamaCloud, OpenAI, PromptBackend};
  use boukensha_03_prompt_builder::{Config, Context, Player, PromptBuilder, Registry, Task};

  fn main() {
      if env::var("BOUKENSHA_DIR").is_err() {
          let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
              .join("../../..")
              .canonicalize()
              .expect("could not resolve repo root");
          env::set_var("BOUKENSHA_DIR", repo_root.join(".boukensha"));
      }

      let config = Config::new();
      let player_settings = config.tasks(Some("player")).unwrap_or_default();
      let system_prompt = Player::system_prompt(
          &player_settings,
          Some(&config.user_prompts_dir()),
          Some(Path::new(Config::PROMPTS_DIR)),
      );

      let ctx: Context<Player> = Context::new(system_prompt);
      let mut registry = Registry::new(ctx);

      let look_params: Value = serde_yaml_ng::from_str("{}").expect("valid parameters yaml");
      registry.tool(
          "look",
          "Look around the current room for details",
          look_params,
          |_args| "A damp stone corridor stretches north. Torches flicker on the walls.".to_string(),
      );

      let move_params: Value = serde_yaml_ng::from_str(
          "direction:\n  type: string\n  description: The direction to move\n",
      )
      .expect("valid parameters yaml");
      registry.tool(
          "move",
          "Move the player in a direction (north, south, east, west, up, down)",
          move_params,
          |args| {
              let direction = args.get("direction").cloned().unwrap_or_default();
              format!("You move {direction} into a torch-lit corridor.")
          },
      );

      registry.context.add_message(
          "user",
          "I just arrived in the dungeon. What's around me, and can you move north?",
          None,
      );
      registry.context.add_message("assistant", "Let me take a look around first.", None);
      registry.context.add_message(
          "tool_result",
          "A damp stone corridor stretches north. Torches flicker on the walls.",
          Some("toolu_01X".to_string()),
      );

      println!("=== BOUKENSHA Step 3: Prompt Builder ===");

      let provider = Player::provider(&player_settings).expect("provider is required");
      let model = Player::model(&player_settings).expect("model is required");

      let backend: Box<dyn PromptBackend<Player>> = match provider.as_str() {
          "anthropic" => Box::new(
              Anthropic::new(env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set"), &model)
                  .expect("supported model"),
          ),
          "ollama" => Box::new(Ollama::new(&model, None).expect("supported model")),
          "ollama_cloud" => Box::new(
              OllamaCloud::new(env::var("OLLAMA_API_KEY").expect("OLLAMA_API_KEY must be set"), &model)
                  .expect("supported model"),
          ),
          "openai" => Box::new(
              OpenAI::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"), &model)
                  .expect("supported model"),
          ),
          "gemini" => Box::new(
              Gemini::new(env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set"), &model)
                  .expect("supported model"),
          ),
          other => panic!("Unsupported provider for player task: {other}"),
      };

      let builder = PromptBuilder::new(&registry.context, backend);

      println!();
      println!("Config: {config}");
      println!("Provider: {provider}");
      println!("Model: {model}");
      println!(
          "{}",
          serde_json::to_string_pretty(&builder.to_api_payload(1024)).expect("payload must serialize")
      );
  }
  ```
  (`HashMap` import is unused if no tool dispatch happens in this step's
  example — drop it if the compiler flags it; `move`'s closure still needs
  `args.get("direction")` off the `&HashMap<String, String>` the registry's
  block signature requires, so it stays needed for the tool closures
  themselves even though `registry.dispatch(...)` isn't called this step.)

## Behavior parity checklist (from the Ruby/Python spec)

- [ ] `errors.rs` adds `UnsupportedModelError(String)`, `Display` renders
      the pre-formatted message, implements `std::error::Error`
- [ ] `Config::PROMPTS_DIR` restored (verbatim `00_config` shape:
      `concat!(env!("CARGO_MANIFEST_DIR"), "/prompts")`), not a new
      function; `prompts/system.md` exists with the exact fixture text
- [ ] `backends::base` defines `ModelInfo`, `CostPerMillion`, `Backend`
      trait (`backend_name`, `models`, `info`, `validate_model`,
      `context_window`, `input_token_cost_per_million`,
      `output_token_cost_per_million`, `usage_unit`, `usage_level`,
      `estimate_cost`), `PromptBackend<T: Task>` trait (`to_payload`,
      `headers`, `url`), `schema_parts`, `function_wrapped_tools`,
      `chat_style_messages`
- [ ] `Anthropic`, `Ollama`, `OllamaCloud`, `OpenAI`, `Gemini` each
      implement both `Backend` and `PromptBackend<T>`, with their own
      `MODELS` table matching the Python source's values exactly
      (context window, cost per million, usage unit/level)
- [ ] Anthropic/Gemini `to_messages` take one arg; Ollama/OpenAI/
      OllamaCloud take `(system, messages)` — arity difference preserved,
      not papered over
- [ ] System prompt: Anthropic/Gemini emit it as a top-level field
      (`system`/`systemInstruction`); Ollama/OpenAI/OllamaCloud prepend a
      `role: system` message
- [ ] Tool results: Anthropic wraps in a `user` message's `tool_result`
      content block; Ollama/OllamaCloud use `role: tool` +
      `tool_name`; OpenAI uses `role: tool` + `tool_call_id`; Gemini uses
      a `user` message's `functionResponse` part
- [ ] Tool definitions: Anthropic uses top-level `input_schema`; Ollama/
      OpenAI wrap in `{"type": "function", "function": {...}}` with
      `parameters`; Gemini wraps all tools in one `functionDeclarations`
      array
- [ ] Message roles: Anthropic/Ollama/OpenAI use `assistant`; Gemini uses
      `model`
- [ ] `PromptBuilder<'a, T: Task>` borrows `&'a Context<T>`, owns
      `Box<dyn PromptBackend<T>>`; `to_api_payload(max_output_tokens: u32)`,
      `headers()`, `url()` — no `to_messages()`/`to_tools()` passthroughs
      (Decision 13)
- [ ] `examples/example.rs` registers `look` (no params) and `move` (one
      param, with `description`), adds three messages directly via
      `registry.context.add_message(...)` (matching Ruby/Python's
      `ctx.add_message`, since `Registry` owns `Context` per
      `02_the_registry`'s Decision 7), resolves provider/model from task
      settings, builds the right backend, prints `Config`/`Provider`/
      `Model`/the full pretty-printed JSON payload
- [ ] Printed JSON payload matches the Python README's expected output
      structurally: `model`, `system`, `max_tokens`, `tools` (with `look`
      then `move`, insertion order preserved), `messages` (three entries,
      the last one the `tool_result` content block)
- [ ] Root `Cargo.toml` workspace `members` includes
      `rust/03_prompt_builder`
- [ ] `rust/03_prompt_builder/Cargo.toml` package name is
      `boukensha_03_prompt_builder`, deps add
      `serde_json = { version = "1", features = ["preserve_order"] }`
- [ ] `bin/rust/03_prompt_builder` launcher exists, mirrors
      `bin/rust/02_the_registry`'s shape
- [ ] `rust/03_prompt_builder/README.md` rewritten from its current stale
      02-the-registry content to match `python/03_prompt_builder/README.md`'s
      structure, with Rust-specific porting notes (Decisions 3, 6, 8, 11,
      13, 14)

## Open questions

None outstanding — all decided above.
