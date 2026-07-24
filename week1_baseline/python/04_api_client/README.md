# 04 · The API Client (Python port)

Behavior port of `ruby/04_api_client` — takes the payload assembled by
`PromptBuilder` and sends it to the API. One HTTP POST, one response.
No tool loop yet — just proving the round trip works.
`prompt_builder.py`, `backends/*.py`, `message.py`, `tool.py`,
`context.py`, `registry.py`, and `tasks/player.py` are unchanged from
`03_prompt_builder`; see `../03_prompt_builder/README.md` for those.

## New Files

| File | Description |
|---|---|
| `lib/boukensha/client.py` | Makes the HTTP request and parses the response |

## Updated Files

| File | Change |
|---|---|
| `lib/boukensha/errors.py` | Added `ApiError` for failed HTTP requests |
| `lib/boukensha/tasks/base.py` | `_fetch` now guards against non-dict `settings`; `provider`/`model` error messages fixed from `settings.yml` to `settings.yaml` |
| `prompts/system.md` | New default system prompt text (rewritten upstream in Ruby, unrelated to this step's own behavior) |

## How It Works

```
PromptBuilder
      ↓
Client
      ↓
POST to API endpoint
      ↓
Raw JSON response
```

## boukensha.client.Client

| Method | Description |
|---|---|
| `call(max_output_tokens=1024)` | POSTs the payload and returns the parsed JSON response |

## Task Configuration

Same task-based configuration as prior steps:

```yaml
tasks:
  player:
    provider: anthropic
    model: claude-haiku-4-5
    prompt_override:
      system: true
```

When `prompt_override.system` is true, Boukensha reads
`.boukensha/prompts/player/system.md`. Otherwise it falls back to this
step's shipped `prompts/system.md`.

## No Dependencies

`Client` uses Python's standard `urllib.request`, `urllib.error`,
`ssl`, and `socket` modules. No third-party HTTP library
(`requests`, `httpx`, ...) is added — matching Ruby's own
`net/http`-only, no-gems approach for this step. `requirements.txt`
is unchanged from `03_prompt_builder`.

## What the Response Looks Like

The raw response shape differs between backends — see
`ruby/04_api_client/README.md` for the full Anthropic/Ollama examples.
`client.call()` returns whatever the backend's API sends back as a
plain `dict`, unprocessed. Handling `tool_use`/`tool_calls` responses
is the job of a future step (the Agent Loop).

## Considerations

**The client raises `ApiError` on failure.** A non-2xx response, or a
network-level failure that exhausts retries, means something went
wrong — bad API key, malformed payload, server error, dropped
connection. BOUKENSHA surfaces this explicitly rather than returning a
confusing `None` or partial response.

**Retries are limited and backed off.** `408/409/429/500/502/503/504`
responses and transient network errors (connection reset/refused, DNS
failure, TLS error, timeout, unexpected EOF) are retried up to
`MAX_RETRIES` (3) times with exponential backoff
(`0.5 * 2 ** (attempt - 1)` seconds), matching Ruby's `client.rb`
exactly.

**TLS verification is on by default.** `urllib.request.urlopen`
already verifies certificates for `https://` URLs out of the box via
Python's default SSL context — there's no Python equivalent needed for
Ruby's explicit `http.use_ssl = ...` / `verify_mode = VERIFY_PEER`
lines, since Python's stdlib HTTP client only ever operates over TLS
when the URL scheme says so, and always verifies unless told not to.

**A known, machine-specific TLS rough edge.** Ruby's README already
flags that `net/http`'s default certificate lookup varies by OS. The
Python side has its own version of the same problem: the official
python.org macOS installer ships a Python build that doesn't
automatically pick up the macOS system keychain's trust store, so
`urlopen` can fail with `CERTIFICATE_VERIFY_FAILED: unable to get
local issuer certificate` even though the same request works fine from
Ruby, curl, or a Homebrew-installed Python on the identical machine.
This is not a bug in `client.py` — it's a per-machine OpenSSL
trust-store wiring issue, exactly the category of thing Ruby's own
README says "you will need to update ... based on your machine's
requirements" about. If you hit this:
- Run `/Applications/Python <version>/Install Certificates.command`
  (ships with the python.org installer), **or**
- Point `SSL_CERT_FILE` at a valid PEM bundle for one run, e.g.
  `SSL_CERT_FILE=/etc/ssl/cert.pem bin/python/04_api_client` on macOS,
  **or**
- Use a Python distribution that already links the OS trust store
  (Homebrew's `python3`, `pyenv` built against the system OpenSSL,
  etc).

No code change is made for this — matching Ruby's own choice to leave
`ca_file` unset and let the platform's default resolution work,
documenting the rough edge instead of hardcoding a path.

## Porting notes

- **Ruby's response-object HTTP model → Python's exception-based HTTP
  model.** `Net::HTTP#request` always returns a response object, even
  for a 4xx/5xx status — Ruby's `client.rb` checks
  `response.is_a?(Net::HTTPSuccess)` itself after the retry loop.
  Python's `urllib.request.urlopen` instead *raises*
  `urllib.error.HTTPError` for any non-2xx status (it's a subclass of
  the response object with a `.code`/`.read()`), so `client.py` catches
  `HTTPError` explicitly, checks the same `RETRYABLE_STATUS_CODES` set
  against `.code`, and raises `ApiError` with `.read()` as the body
  when retries are exhausted — same decision logic, different control
  flow shape forced by the language.
- **`TRANSIENT_ERRORS` class-for-class mapping:**

  | Ruby | Python |
  |---|---|
  | `EOFError` | `EOFError` (built-in, same name) |
  | `Errno::ECONNRESET` | `ConnectionResetError` |
  | `Errno::ECONNREFUSED` | `ConnectionRefusedError` |
  | `Net::OpenTimeout` / `Net::ReadTimeout` / `Timeout::Error` | `socket.timeout`, `TimeoutError` |
  | `OpenSSL::SSL::SSLError` | `ssl.SSLError` |
  | `SocketError` | `socket.gaierror` |
  | *(no single Ruby equivalent)* | `urllib.error.URLError` |

  `URLError` is included because `urlopen` wraps most
  connection-establishment failures (DNS lookup, connection refused,
  connect-time timeout) in a `URLError` whose `.reason` holds the
  underlying cause, rather than raising that underlying exception
  directly. Rather than unwrap `.reason` and match on the specific
  cause, `client.py` treats `URLError` itself as transient — Ruby's own
  list is already just "these are network hiccups, retry them" without
  distinguishing by exact cause, so this keeps the same granularity.
- **`fetch`'s new `settings.is_a?(Hash)` guard → `isinstance(settings,
  dict)` guard** in `tasks/base.py`'s `_fetch`. Direct port of Ruby's
  new defensive check; Python's `dict.get` would otherwise raise
  `AttributeError` on a non-dict `settings` the same way Ruby's
  `settings[key]` would raise `NoMethodError` without the guard.
- **`Client`/`ApiError` exported from the top-level `boukensha`
  package** (`lib/boukensha/__init__.py`), unlike `backends/*` which
  stayed sub-packaged (see `03_prompt_builder`'s Decision 1). `Client`
  and `ApiError` are single, distinctively-named classes — the
  crowding concern that kept `Base`/`OpenAI`/etc. out of the flat
  namespace doesn't apply here.
- **`Config.PROMPTS_DIR` unchanged from `03_prompt_builder`.** Ruby's
  `config.rb` picked up an extra `..` in this step
  (`File.expand_path("../../../prompts", __dir__)`), which resolves to
  a nonexistent `ruby/prompts` directory instead of
  `04_api_client/prompts` — contradicted by its own updated comment
  ("shipped alongside this step") and invisible in the real fixture
  output only because `prompt_override.system: true` and the override
  file exist, so the default-prompts path is never actually exercised.
  Confirmed with the user this is an unintentional upstream regression,
  not a deliberate change: Python's `PROMPTS_DIR` keeps its already-
  correct `parent.parent.parent` (3 hops from `lib/boukensha/config.py`
  to `04_api_client/`) rather than reproducing the bug.
- **`read_file`/`list_directory` tool bodies** use `pathlib.Path.read_text()`
  and `os.listdir()` + a `startswith(".")` filter — direct equivalents
  of Ruby's `File.read` and `Dir.entries(...).reject { |f|
  f.start_with?(".") }`. `os.listdir` doesn't return `.`/`..` in the
  first place (unlike `Dir.entries`, which does and then filters them
  out), so the filter only needs to exclude other dotfiles; no sorting
  is applied, matching Ruby's unsorted, filesystem-order listing.

## Run Example

```bash
./week1_baseline/bin/python/04_api_client
```

This makes a real HTTP request to whichever provider `.boukensha/settings.yaml`
configures (Anthropic, by default in this repo's fixture) — it costs a
small amount and requires a valid API key in `.boukensha/.env`.

Example output (the exact response body is **not** reproducible
byte-for-byte — it's a live model response, and the model may choose a
different tool call, or none, from one run to the next):

```
=== BOUKENSHA Step 4: API Client ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Sending request to https://api.anthropic.com/v1/messages...

Raw response:
{
  "model": "claude-haiku-4-5-20251001",
  "id": "msg_011CdL3XuwkK4otCHh3oeTyN",
  "type": "message",
  "role": "assistant",
  "content": [
    {
      "type": "tool_use",
      "id": "toolu_01QUE8B8VyoTQ2TzEEmjQh8Y",
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
    "cache_creation": {
      "ephemeral_5m_input_tokens": 0,
      "ephemeral_1h_input_tokens": 0
    },
    "output_tokens": 53,
    "service_tier": "standard",
    "inference_geo": "not_available"
  }
}
```

The `input_tokens` count (695) matched exactly between the verified
Ruby run and this Python run for the same fixture and prompt — strong
structural confirmation that the two languages build byte-identical
request payloads, even though the response content itself varies
run to run.
