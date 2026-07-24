# 07 · The Boukensha.run DSL (Python port)

Behavior port of `ruby/07_the_run_dsl` — a single top-level entry
point, `boukensha.run()`, that wires together `Context`, `Registry`,
a backend, `PromptBuilder`, `Client`, `Logger`, and `Agent` so the
caller only has to describe *what* to run, not *how* to plumb it.
`message.py`, `tool.py`, `context.py`, `registry.py`, `client.py`,
`prompt_builder.py`, `agent.py`, `tasks/*.py`, and `backends/*.py` are
unchanged from `06_the_logger`; see `../06_the_logger/README.md` for
those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/run_dsl.py` | `RunDSL` — the object passed to a `register` callback; exposes only `tool()` |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/__init__.py` | Adds the top-level `run()` entry point; imports and exports `RunDSL`, `LoopError`, and all five backend classes |
| `lib/boukensha/config.py` | Restores `mud_host`/`mud_port`/`mud_username`/`mud_password` properties (removed in `06_the_logger`, reinstated here — matches the Ruby spec's own re-add at this step) |
| `lib/boukensha/errors.py` | Restores `LoopError` (same as above; not yet raised anywhere) |
| `lib/boukensha/logger.py` | Adds `turn(n)` (not yet called by `agent.py`) and `subscribe(callback)`, with `_write_log` notifying subscribers after every write |

## boukensha.run()

```python
def register_tools(dsl):
    @dsl.tool(
        "read_file",
        description="Read a file from disk",
        parameters={"path": {"type": "string", "description": "File path"}},
    )
    def read_file(path):
        return Path(path).read_text()

    @dsl.tool("list_directory", description=..., parameters={...})
    def list_directory(path):
        return ", ".join(os.listdir(path))


result = boukensha.run(
    task="Summarise lib/boukensha",
    register=register_tools,
)
```

| Option | Default | Description |
|---|---|---|
| `task` | *(required)* | The user message handed to the agent |
| `system` | task's system prompt from config | System prompt |
| `model` | task's configured model | Model name |
| `backend` | task's configured provider | `"anthropic"`, `"openai"`, `"gemini"`, `"ollama"`, or `"ollama_cloud"` |
| `api_key` | matching `*_API_KEY` env var | API key for the chosen backend; not needed for `"ollama"` |
| `ollama_host` | `"http://localhost:11434"` | Ollama base URL |
| `log` | `None` | Optional JSONL path override; by default logs go to `.boukensha/sessions/<session-id>.jsonl` |
| `max_output_tokens` | task's configured value | Per-reply output cap |
| `register` | `None` | Optional callback receiving a `RunDSL` to register tools on |

Config, system prompt, model, and backend all come from
`~/.boukensha` (or `BOUKENSHA_DIR`) via the `player` task's settings —
every previous step's manual wiring collapses into this one call.

## Before and after

**Step 6 — manual plumbing:**

```python
ctx = Context(task=Player, system=system_prompt)
registry = Registry(ctx)
backend = Anthropic(api_key=os.environ["ANTHROPIC_API_KEY"], model=model)
builder = PromptBuilder(ctx, backend)
client = Client(builder)
logger = Logger()
agent = Agent(context=ctx, registry=registry, builder=builder, client=client, logger=logger)


@registry.tool("read_file", description="Read a file", parameters={"path": {"type": "string"}})
def read_file(path):
    return Path(path).read_text()


ctx.add_message("user", "Read lib/boukensha.py")
agent.run()
```

**Step 7 — just describe what you want:**

```python
def register_tools(dsl):
    @dsl.tool("read_file", description="Read a file", parameters={"path": {"type": "string"}})
    def read_file(path):
        return Path(path).read_text()


boukensha.run(task="Read lib/boukensha.py", register=register_tools)
```

## No New Dependencies

`run_dsl.py` uses no imports at all. `requirements.txt` is unchanged
from `06_the_logger` — matches Ruby's own `Gemfile`/`Gemfile.lock`,
which are unchanged in this step too.

## Porting notes

- **`RunDSL.new(registry).instance_eval(&block)` → a `register`
  callback that receives a `RunDSL` instance directly.** Ruby's block
  is `instance_eval`'d so bare `tool "x", ...` calls resolve against
  the `RunDSL` receiver with no explicit `self`. Python has no
  `instance_eval` equivalent, and every earlier step's decorator-factory
  precedent (`Registry.tool`, see `02_the_registry`) already requires
  an explicit receiver (`@registry.tool(...)`) — so the closest
  idiomatic match is a plain function that receives the `RunDSL`
  explicitly and decorates tools onto it (`@dsl.tool(...)`), rather
  than inventing an implicit-`self` mechanism Python doesn't have.
  Confirmed with the user (three real alternatives existed: this
  callback shape, a context-manager with deferred execution on
  `__exit__`, or dropping `RunDSL` and decorating `registry` directly)
  before implementing, since no earlier plan doc had settled a
  multi-tool block pattern.
- **`RunDSL#tool` → thin proxy to `Registry.tool`, unchanged
  signature.** Since `RunDSL` exists only to narrow the DSL surface to
  one method (per Ruby's own comment), the Python class does the same:
  `RunDSL.tool` just forwards `name`/`description`/`parameters` to
  `self._registry.tool(...)` and returns its decorator, so `@dsl.tool(...)`
  behaves identically to `@registry.tool(...)`.
- **Ruby symbol `backend` (`:anthropic`, `.to_sym` on the settings
  string) → plain Python string throughout.** `example.py` since
  `03_prompt_builder` has always compared provider strings directly
  (`if provider == "anthropic"`); `boukensha.run()`'s backend dispatch
  keeps that established simplification instead of introducing a
  symbol/string distinction Python doesn't need.
- **`ensure logger&.close` → `logger = None` before the `try`, `finally:
  if logger is not None: logger.close()`.** Ruby's local variable is
  implicitly `nil` if an exception (e.g. the unknown-backend
  `ArgumentError`) fires before the `Logger.new` line is reached, and
  `&.close` no-ops on `nil`. Python raises `UnboundLocalError` on a
  never-assigned name, so `logger` is pre-bound to `None` to make the
  same "close it only if it was actually created" behavior explicit.
- **`Logger#subscribe`'s `@subscribers ||= []` lazy-init → `self._subscribers
  = []` in `__init__`.** Ruby lazily allocates the array on first
  `subscribe` call and guards the iteration with `@subscribers&.each`.
  Python has no attribute analogous to Ruby's implicit-nil-until-set
  ivars, and initializing the list eagerly in `__init__` (iterating an
  empty list is already a no-op) reproduces identical behavior without
  a lazy-init branch `__init__.py`'s existing style doesn't otherwise use.
- **`Config`'s restored `mud_*` methods and `Logger#turn`/`errors.rb`'s
  restored `LoopError` are ported even though nothing in this step's
  Ruby code calls them yet.** They're genuine re-additions in the Ruby
  diff (removed at `06_the_logger`, brought back here) rather than
  dead code the port should skip — matching the spec exactly, per the
  same principle `06_the_logger`'s README documents for the removal
  direction.

## Run Example

```bash
./week1_baseline/bin/python/07_the_run_dsl
```

This makes one or more real HTTP requests to whichever provider
`.boukensha/settings.yaml` configures (Anthropic, by default in this
repo's fixture) — it costs a small amount per model round-trip and
requires a valid API key in `.boukensha/.env`. It also writes a new
`.boukensha/sessions/<session-id>.jsonl` file.

Example output (the exact tool calls and final text are **not**
reproducible byte-for-byte — they're live model responses):

```
=== BOUKENSHA Step 7: The Boukensha.run DSL ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>


=== FINAL RESPONSE ===
## Summary of Boukensha MUD Player Assistant Framework
...
```

Verified against the real Ruby run for this same fixture (see
`docs/plans/python_port/07_the_run_dsl.md`).
