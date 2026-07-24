# Python Port Plan — 04 API Client

## Goal

Port the behavior of `week1_baseline/ruby/04_api_client/` to
`week1_baseline/python/04_api_client/`. The directory already existed
but was byte-identical to `python/03_prompt_builder/` (verified: `diff
-rq python/03_prompt_builder python/04_api_client` reported no
differences). This step adds `Boukensha::Client` — the piece that
actually POSTs the payload `PromptBuilder` assembles and parses the
raw JSON response. No tool-call loop yet; this just proves the round
trip works.

This is a behavior port, not a redesign — the Ruby version is the
spec. Confirmed via `diff -rq ruby/03_prompt_builder
ruby/04_api_client`: only `README.md`, `examples/example.rb`,
`lib/boukensha/{config,errors,tasks/base}.rb`, `lib/boukensha.rb`, and
`prompts/system.md` changed, plus the new `lib/boukensha/client.rb`.
`lib/boukensha/{message,tool,context,registry,prompt_builder}.rb`,
`tasks/player.rb`, `backends/*.rb`, and `Gemfile`/`Gemfile.lock` are
all unchanged — carried forward unchanged in Python too, everything
already decided in
[`03_prompt_builder.md`](03_prompt_builder.md) still applies to those
files.

**Ruby README table is stale / overstates the diff.** `ruby/04_api_client/README.md`'s
"New Files" table lists `backends/base.rb` and `tasks/{base,player}.rb`
as new, and its "Updated Files" table lists `backends/*.rb` as
updated — but `diff -rq` against `03_prompt_builder` shows
`tasks/player.rb` and everything under `backends/` are byte-identical
between the two steps. Only `tasks/base.rb` actually changed (two
message-typo fixes and a new `is_a?(Hash)` guard). This plan follows
the verified diff, not the README table — same category of
README/reality drift already documented in `02_the_registry.md`.

## Source files to port (Ruby — read these to know what to build)

| Ruby file | Role |
|---|---|
| `week1_baseline/ruby/04_api_client/README.md` | Design spec: `Client`, retry/backoff behavior, SSL handling, considerations (mostly accurate; see staleness note above for the file tables specifically) |
| `week1_baseline/ruby/04_api_client/lib/boukensha/client.rb` | `Boukensha::Client` — builds and sends the HTTP POST, retries transient errors and retryable status codes with exponential backoff, raises `ApiError` on final failure, returns parsed JSON on success |
| `week1_baseline/ruby/04_api_client/lib/boukensha/errors.rb` | Adds `Boukensha::ApiError < StandardError` alongside `UnknownToolError`/`UnsupportedModelError` |
| `week1_baseline/ruby/04_api_client/lib/boukensha/config.rb` | `PROMPTS_DIR` gains an extra `../` (see Porting Notes — treated as an unintentional upstream bug, not ported) |
| `week1_baseline/ruby/04_api_client/lib/boukensha/tasks/base.rb` | `provider`/`model` error messages fixed (`settings.yml` → `settings.yaml`); `fetch` gains a `settings.is_a?(Hash)` guard |
| `week1_baseline/ruby/04_api_client/lib/boukensha.rb` | Drops the now-redundant explicit `backends/base` require (each backend file already requires it directly — no behavior change), adds `client` require |
| `week1_baseline/ruby/04_api_client/prompts/system.md` | New default system prompt text (unrelated wording change, carried forward verbatim) |
| `week1_baseline/ruby/04_api_client/examples/example.rb` | Rewritten smoke test: registers `read_file`/`list_directory` tools, seeds one user message, resolves provider/model/backend, builds `PromptBuilder` + `Client`, prints config/provider/model/url, then the real parsed response |
| `week1_baseline/ruby/04_api_client/lib/boukensha/{message,tool,context,registry,prompt_builder}.rb`, `tasks/player.rb`, `backends/*.rb`, `Gemfile`/`Gemfile.lock` | Byte-identical to `03_prompt_builder` (verified via `diff -rq`) — carry forward as-is |

## Runtime fixture to reuse (do not duplicate)

Same `.boukensha/` fixture at the repo root as prior steps.
`settings.yaml` still sets `tasks.player.provider: anthropic`,
`tasks.player.model: claude-haiku-4-5`,
`tasks.player.prompt_override.system: true`; `.boukensha/.env` has a
real `ANTHROPIC_API_KEY` (unlike earlier steps where it was declared
empty) — this step is the first one that actually calls out over the
network, and the fixture is configured to make that call succeed. No
fixture changes needed.

## Target files to create/change (Python)

```
week1_baseline/python/04_api_client/
  README.md                              (rewrite: Client docs, new/updated-files tables, considerations incl. SSL rough edge, porting notes, run example)
  requirements.txt                       (unchanged — same two deps; Client is stdlib-only)
  prompts/system.md                      (new text, copied byte-for-byte from ruby/04_api_client/prompts/system.md)
  examples/example.py                    (rewrite: register read_file+list_directory, seed one user message, resolve backend, build Client, print config/provider/model/url, then the real response)
  lib/boukensha/__init__.py              (add Client, ApiError exports)
  lib/boukensha/config.py                (unchanged — PROMPTS_DIR already correct; see Decisions)
  lib/boukensha/message.py               (unchanged, already correct)
  lib/boukensha/tool.py                  (unchanged, already correct)
  lib/boukensha/context.py               (unchanged, already correct)
  lib/boukensha/registry.py              (unchanged, already correct)
  lib/boukensha/prompt_builder.py        (unchanged, already correct)
  lib/boukensha/errors.py                (add ApiError)
  lib/boukensha/client.py                (new)
  lib/boukensha/tasks/__init__.py        (unchanged)
  lib/boukensha/tasks/base.py            (edit: settings.yml→settings.yaml message fix, is_a?(Hash)→isinstance(settings, dict) guard in _fetch)
  lib/boukensha/tasks/player.py          (unchanged, already correct)
  lib/boukensha/backends/*.py            (unchanged, already correct)
```

Plus a launcher at `week1_baseline/bin/python/04_api_client`, matching
`bin/python/03_prompt_builder`'s shape (executable bit set):

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/04_api_client"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

No changes to `week1_baseline/python/README.md` — same shared root
`.venv`, no new dependency to install (`Client` is stdlib-only, same
as Ruby's `net/http`-only, no-gems approach).

## Behavior parity checklist (from the real Ruby output)

- [x] `ApiError` — plain `Exception` subclass, no custom fields, added
      alongside `UnknownToolError`/`UnsupportedModelError` in `errors.py`
- [x] `Client.RETRYABLE_STATUS_CODES` — `{408, 409, 429, 500, 502, 503, 504}`
- [x] `Client.TRANSIENT_ERRORS` — tuple of `EOFError`,
      `ConnectionResetError`, `ConnectionRefusedError`,
      `socket.timeout`, `TimeoutError`, `ssl.SSLError`,
      `socket.gaierror`, `urllib.error.URLError` (see Porting Notes for
      the class-for-class mapping and why `URLError` is included)
- [x] `Client.MAX_RETRIES = 3`, `Client.BASE_RETRY_DELAY = 0.5`,
      backoff `0.5 * 2 ** (attempt - 1)` seconds
- [x] `Client(builder)` stores the builder; `call(max_output_tokens=1024)`
      builds the request from `builder.url`/`builder.headers`/
      `builder.to_api_payload(...)`, retries transient errors and
      retryable HTTP statuses up to `MAX_RETRIES` times with backoff,
      raises `ApiError` with the same message shape as Ruby on final
      failure, returns the parsed JSON `dict` on success
- [x] `tasks/base.py._fetch` returns `None` for non-dict `settings`
      instead of raising; `provider`/`model` `ValueError` messages say
      `settings.yaml`, not `settings.yml`
- [x] `Config.PROMPTS_DIR` stays pointed at `04_api_client/prompts`
      (does **not** reproduce Ruby's extra-`../` regression — see
      Decisions)
- [x] `examples/example.py` registers `read_file` (one `path` param)
      and `list_directory` (one `path` param) via `@registry.tool(...)`,
      seeds a single user message ("What files are in the current
      directory?"), resolves `provider`/`model` from
      `Player.provider/model`, builds the matching `Backends.*`
      instance, builds `PromptBuilder` + `Client`, prints `Config`,
      `Provider`, `Model`, `Sending request to {url}...`, then `Raw
      response:` followed by `json.dumps(response, indent=2)`
- [x] Printed request payload is byte-identical to Ruby's for the same
      fixture — confirmed structurally (both real runs report
      `"input_tokens": 695` for the assembled prompt, and both got a
      `tool_use` response naming `list_directory` with `{"path": "."}`)

Expected output (verified by actually running
`./week1_baseline/bin/ruby/04_api_client` from the repo root against
the real `.boukensha/` fixture — this step makes a real, billed
Anthropic API call, confirmed with the user before running):

```
=== BOUKENSHA Step 4: API Client ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Sending request to https://api.anthropic.com/v1/messages...

Raw response:
{
  "model": "claude-haiku-4-5-20251001",
  "id": "msg_011CdL38VtzgyQ5FTivGxvQZ",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "tool_use",
      "id": "toolu_01A6sjnqbv2V3nv6UF1M35Bz",
      "name": "list_directory",
      "input": { "path": "." },
      "caller": { "type": "direct" }
    }
  ],
  "stop_reason": "tool_use",
  "stop_sequence": null,
  "stop_details": null,
  "usage": {
    "input_tokens": 695,
    "cache_creation_input_tokens": 0,
    "cache_read_input_tokens": 0,
    "cache_creation": { "ephemeral_5m_input_tokens": 0, "ephemeral_1h_input_tokens": 0 },
    "output_tokens": 53,
    "service_tier": "standard",
    "inference_geo": "not_available"
  }
}
```

Python's own verified run (`./week1_baseline/bin/python/04_api_client`,
with `SSL_CERT_FILE=/etc/ssl/cert.pem` — see Decisions/Porting Notes
for why that env var was needed on this machine) produced the same
shape with `"input_tokens": 695` and the same `list_directory`/`{"path":
"."}` tool call, differing only in `id`/`output_tokens` (53 vs 53 —
actually identical this run) the way any two independent live model
calls can. **The exact response body is not expected to reproduce
byte-for-byte** across runs or languages — only the request payload
(and therefore `input_tokens`) is expected to match, which it did.

## Porting notes (Ruby idiom → Python)

- **Ruby's response-object HTTP model → Python's exception-based HTTP
  model.** `Net::HTTP#request` always returns a response object, even
  for a 4xx/5xx status; `client.rb` checks
  `response.is_a?(Net::HTTPSuccess)` itself, after its own retry loop
  inspects `response.code`. Python's `urllib.request.urlopen` instead
  *raises* `urllib.error.HTTPError` (itself a response-like object
  with `.code`/`.read()`) for any non-2xx status. `client.py` catches
  `HTTPError` specifically, applies the same
  `RETRYABLE_STATUS_CODES`-against-`.code` check, and raises `ApiError`
  with `.read()` as the body when retries are exhausted — same
  decision logic, forced into exception-based control flow instead of
  a post-loop `if` check.
- **`TRANSIENT_ERRORS` — one Ruby class doesn't map to exactly one
  Python class.** `Net::OpenTimeout`/`Net::ReadTimeout`/`Timeout::Error`
  all collapse to Python's `socket.timeout` (identical to `TimeoutError`
  since Python 3.10) — Ruby distinguishes open vs. read timeouts,
  Python's `urllib` doesn't expose that distinction without a lower-
  level `http.client` rewrite, so both collapse to the same catch.
  `SocketError` (Ruby's DNS/hostname-resolution failure class) maps to
  `socket.gaierror`. One entry has **no** Ruby equivalent at all:
  `urllib.error.URLError`. `urlopen` wraps most
  connection-establishment failures (DNS lookup, connection refused,
  connect-time timeout) in a `URLError` whose `.reason` holds the
  actual underlying exception, rather than letting that exception
  propagate directly the way Ruby's `Net::HTTP` does. Rather than
  unwrap `.reason` and re-match on the specific cause, `client.py`
  treats `URLError` itself as transient and retries it — Ruby's own
  list is already just "these are the flavors of network hiccup,
  retry them all the same way" without finer-grained handling per
  cause, so treating the wrapper class as transient preserves that
  same granularity rather than adding precision Ruby's own port
  doesn't have either.
- **`settings.is_a?(Hash)` guard → `isinstance(settings, dict)` guard**
  in `tasks/base.py`'s `_fetch`. Direct one-line port. Without it,
  Python's `settings.get(key)` on a non-dict `settings` (e.g. `None`)
  raises `AttributeError`, the same way Ruby's `settings[key]` would
  raise `NoMethodError` without the guard — this is a real robustness
  fix carried over, not a stylistic change.
- **`Client`/`ApiError` exported from the top-level `boukensha` package**
  (`__init__.py`), unlike `backends/*`, which stayed in their own
  sub-package per `03_prompt_builder`'s Decision 1 (to avoid dumping
  generically-named classes like `Base`/`OpenAI` into the flat
  namespace). `Client` and `ApiError` don't have that name-crowding
  problem, so they follow the majority pattern used by `Config`,
  `Context`, `PromptBuilder`, etc.
- **`Config.PROMPTS_DIR` — Ruby regressed, Python didn't follow it.**
  Ruby's `config.rb` changed `File.expand_path("../../prompts",
  __dir__)` (correct, resolves to the step's own `prompts/`) to
  `File.expand_path("../../../prompts", __dir__)` (one extra `../`,
  resolves to a nonexistent `ruby/prompts`) in this step, while its own
  comment still says "Default prompts shipped alongside this step" —
  the comment and the new code disagree, which is the signature of an
  unintentional regression rather than a deliberate relocation. It's
  invisible in the real fixture output only because
  `prompt_override.system: true` and the override file both exist, so
  `default_prompts_dir` is built but never actually read. Confirmed
  with the user (AskUserQuestion) before implementing: Python's
  `PROMPTS_DIR` keeps its existing, already-correct
  `parent.parent.parent` computation (still resolving to
  `04_api_client/prompts`) rather than reproducing the bug — this is
  the one place this step's Python port deliberately does *not* match
  Ruby's literal behavior, and it's called out explicitly here and in
  the README rather than silently diverging.
- **`read_file`/`list_directory` tool bodies.** `Path(path).read_text()`
  is the direct equivalent of Ruby's `File.read(path)`. `os.listdir(path)`
  plus a `not f.startswith(".")` filter matches Ruby's
  `Dir.entries(path).reject { |f| f.start_with?(".") }` — `os.listdir`
  simply never returns `.`/`..` in the first place (`Dir.entries` does,
  then filters them via the same dot-prefix check), so the net set of
  names filtered is identical. No sorting is applied on either side;
  both return filesystem-order listings.
- **A real, machine-specific TLS verification failure surfaced during
  verification, unrelated to the port's own code.** Running
  `bin/python/04_api_client` on this machine initially raised
  `ssl.SSLCertVerificationError: unable to get local issuer
  certificate` — the python.org macOS installer's Python build doesn't
  wire into the macOS system keychain trust store the way this
  machine's Ruby installation apparently does (Ruby's real run
  succeeded with no special handling). This is the Python-specific
  instance of the exact rough edge Ruby's own README already documents
  under "OpenSSL Certificate" ("You will need to update the code based
  on your machines requirements"). Per that same precedent, no code
  change was made in `client.py` — verification of this step's output
  used `SSL_CERT_FILE=/etc/ssl/cert.pem` (macOS's own system cert
  bundle) as a one-off environment variable for the test invocation
  only. The rough edge and three fix options are documented in the
  README's Considerations section instead of being papered over with a
  hardcoded path or a relaxed verification mode (which would be a real
  security regression, not a fix).
- `message.py`, `tool.py`, `context.py`, `registry.py`,
  `prompt_builder.py`, `tasks/player.py`, `backends/*.py` need **no
  edits** — already correct in the current `python/04_api_client/` tree
  (verified identical to `03_prompt_builder`'s versions, matching the
  Ruby side being byte-identical too, aside from the one `config.rb`
  regression this plan deliberately doesn't port).

## Decisions

1. **Run the real Ruby step for a verified transcript, confirmed with
   the user first** — this step is the first one to make a real,
   billed network call (the fixture's `.env` has a live
   `ANTHROPIC_API_KEY`), so unlike prior steps' "just run it" default,
   this required an explicit go-ahead before executing.

2. **`Config.PROMPTS_DIR` is fixed in Python, not reproduced as a bug**
   — confirmed with the user (AskUserQuestion) after finding Ruby's
   `config.rb` picked up an extra `../` that resolves to a nonexistent
   directory and contradicts its own updated comment. See Porting
   Notes for the full reasoning. This is the one deliberate,
   documented behavior divergence from literal Ruby in this step.

3. **`Client`/`ApiError` join the flat `boukensha` namespace** rather
   than a sub-package, since — unlike `backends/*` — neither name is
   generic enough to risk crowding/colliding in `__init__.py`.

4. **`TRANSIENT_ERRORS` includes `urllib.error.URLError` as a whole**
   rather than unwrapping `.reason` to match Ruby's per-cause classes
   individually. See Porting Notes for why this preserves, rather than
   changes, Ruby's own granularity.

5. **No dependency added; `Client` is stdlib-only** (`urllib.request`,
   `urllib.error`, `ssl`, `socket`, `json`, `time`) — matches Ruby's
   `net/http`-only, no-gems approach and this project's stated
   stdlib-first default (`ITERATIONS.md`).

6. **The macOS TLS trust-store issue is documented, not coded around**
   — matches Ruby's own precedent of leaving `ca_file` unset and
   telling the user to adjust for their machine, rather than
   hardcoding a path or disabling verification (which would be a
   genuine security regression, unlike Ruby's issue which never
   required disabling verification either).

7. **`bin/python/04_api_client` launcher** — added per the repo's
   `bin/<language>/<step>` convention, matching
   `bin/python/03_prompt_builder`'s shape.

## Open questions

None outstanding — all decided above.
