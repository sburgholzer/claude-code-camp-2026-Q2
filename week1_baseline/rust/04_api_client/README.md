# 04 · The API Client (Rust port)

Behavior port of `ruby/04_api_client` / `python/04_api_client` — takes
the payload assembled by `PromptBuilder` (step 3) and sends it to the
API as one HTTP POST, parsing the raw JSON response. No tool loop yet
— just proving the round trip works. `message.rs`, `tool.rs`,
`context.rs`, `registry.rs`, `tasks/`, and every `backends/*.rs`'s
model tables are unchanged from `03_prompt_builder`; see
`../03_prompt_builder/README.md` for those.

## New Files

| File | Description |
|---|---|
| `src/client.rs` | Makes the HTTP request and parses the response |

## Updated Files

| File | Change |
|---|---|
| `src/errors.rs` | Added `ApiError` for failed HTTP requests |
| `src/config.rs` | `ConfigError::Display` wording fixed from `settings.yml` to `settings.yaml` |
| `prompts/system.md` | New default system prompt text (rewritten upstream in Ruby, unrelated to this step's own behavior) |

## How It Works

```
PromptBuilder
      ↓
Client
      ↓
POST to API endpoint (ureq + native-tls)
      ↓
Raw JSON response
```

## boukensha_04_api_client::client::Client

| Method | Description |
|---|---|
| `Client::new(&builder)` | Wraps a `PromptBuilder`, configuring a `ureq::Agent` with native-tls and `http_status_as_error(false)` |
| `call(max_output_tokens: u32) -> Result<serde_json::Value, ApiError>` | POSTs the payload and returns the parsed JSON response |

## Task Configuration

Same task-based configuration as `03_prompt_builder`:

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

## HTTP + TLS: `ureq` with `native-tls`

Rust's `std` has no HTTP client and no TLS at all — a much larger gap
than the JSON gap that justified `serde_json` in `03_prompt_builder`.
`ureq` (v3) with the `native-tls` feature was chosen over `ureq` +
rustls (default) and `reqwest`:

- **rustls** typically verifies against a bundled Mozilla root store,
  which would *silently fix* the "machine-specific TLS trust store"
  rough edge both Ruby's and Python's `04_api_client` READMEs
  deliberately document, rather than reproduce it.
- **`reqwest`** (blocking) was rejected as too heavy (hyper/tokio
  machinery even in blocking mode) for a single POST.
- **`native-tls`** uses the OS-native TLS implementation (Secure
  Transport on macOS, SChannel on Windows, OpenSSL on Linux) and the
  system trust store — the closest match to Ruby's `net/http` +
  `openssl` and Python's `ssl`-backed `urlopen`, both of which also
  defer to the platform's trust store and hit the same class of rough
  edge.

`http_status_as_error(false)` is set on the `ureq::Agent`, so a call
always returns `Ok(http::Response<Body>)` regardless of status —
mirroring Ruby's `net/http` model directly (`Net::HTTP#request` always
returns a response object; success/failure is checked *after* the
retry loop) rather than Python's exception-per-non-2xx `urllib` model.

## No Dependencies Beyond `ureq`

`serde`, `serde_yaml_ng`, `serde_json`, `dotenvy`, `dirs`, and
`indexmap` are all carried forward from earlier steps. `ureq` (with
the `native-tls` feature) is the only dependency this step adds — the
Rust-side equivalent of Ruby's `net/http`-only, no-gems approach,
forced here because `std` has no HTTP/TLS support at all (unlike
Ruby/Python, whose standard libraries both ship one).

## What the Response Looks Like

The raw response shape differs between backends — see
`ruby/04_api_client/README.md` for the full Anthropic/Ollama examples.
`client.call()` returns whatever the backend's API sends back as a
`serde_json::Value`, unprocessed. Handling `tool_use`/`tool_calls`
responses is the job of a future step (the Agent Loop).

## Considerations

**The client returns `ApiError` on failure**, not a panic — a non-2xx
response, or a network-level failure that exhausts retries, means
something went wrong (bad API key, malformed payload, server error,
dropped connection), and BOUKENSHA surfaces this explicitly via
`Result` rather than returning a confusing empty/partial response.

**Retries are limited and backed off.** `408/409/429/500/502/503/504`
responses and transient `ureq::Error`s (`Timeout`, `HostNotFound`,
`ConnectionFailed`, `Io`, `Protocol`, `Tls`/`Rustls`/`NativeTls`) are
retried up to `MAX_RETRIES` (3) times with exponential backoff
(`0.5 * 2^(attempt - 1)` seconds), matching Ruby's `client.rb` retry
constants exactly.

**A known, machine-specific TLS rough edge.** Both Ruby's and Python's
READMEs already flag that certificate lookup varies by OS. `native-tls`
defers to the platform's own trust store (Keychain on macOS, SChannel
on Windows, the system OpenSSL config on Linux) rather than bundling
its own root store, so if this machine's OS-level trust store is
misconfigured, requests can fail with a certificate-verification error
that isn't a bug in `client.rs` — it's the same category of
per-machine rough edge Ruby's own README says "you will need to update
... based on your machine's requirements" about. No code change is
made for this — matching Ruby's and Python's own choice not to bundle
a root store or relax verification (which would be a real security
regression).

## Porting notes

- **Ruby's response-object HTTP model → `ureq`'s `http_status_as_error(false)`
  mode.** Since Ruby is the ultimate spec and its shape (check success/
  failure *after* the retry loop, from a response object you always
  get) maps more directly onto this `ureq` mode than fighting
  Python's exception-per-status `urllib` model would, this is the more
  faithful port target — see "HTTP + TLS" above.
- **Transient-error classification maps Ruby's `TRANSIENT_ERRORS` list
  onto `ureq::Error` variants at the same granularity Python's own
  port already chose** (treat broad connection failures as transient
  without unwrapping to the exact OS cause): `Timeout(_)` and
  `HostNotFound` (~ Ruby's `SocketError`), `ConnectionFailed`, `Io(_)`
  (~ `Errno::ECONNRESET`/`ECONNREFUSED`/`EOFError`), `Protocol(_)` (~
  `EOFError` on a truncated read), and `Tls(_)`/`Rustls(_)`/
  `NativeTls(_)` (~ `OpenSSL::SSL::SSLError`).
- **Non-transient, non-status `ureq::Error` variants** (`BadUri`,
  `InvalidProxyUrl`, `TooManyRedirects`, cookie/decode/json-parsing
  errors, etc.) **are wrapped into `ApiError` on the spot**, rather
  than left to propagate as a raw `ureq::Error` — Rust's single-
  `Result`-type public API has no clean equivalent to "let an
  arbitrary unrelated exception type propagate" without leaking
  `ureq::Error` into `Client::call`'s signature. Behaviorally
  unobservable for this step's fixture: a fixed, hardcoded HTTPS URL
  with no proxy and no redirects never triggers any of these variants.
- **`RequestBuilder` is rebuilt fresh each retry attempt** from
  URL/headers/body cached once before the loop, since `ureq` consumes
  its builder on `.send()`. Ruby/Python build one request/body object
  before the loop and reuse it across retries; Rust caches the same
  three pieces of data and reconstructs the builder per attempt — no
  behavior difference, headers and body are byte-identical on every
  attempt either way.
- **`ApiError(pub String)` tuple struct**, following the exact pattern
  already used for `UnsupportedModelError(pub String)` in
  `03_prompt_builder`'s `errors.rs`.
- **`settings.yml` → `settings.yaml` wording fix** lives in
  `config.rs`'s `ConfigError::Display` impl
  (`tasks.{task}.{key} is required in settings.yaml`), not in
  `tasks/base.rs`, since `03_prompt_builder` centralized that message
  there.
- **`_fetch`'s new `settings.is_a?(Hash)`/`isinstance(settings, dict)`
  guard needs no Rust change.** `Task::fetch` in `tasks/base.rs`
  already does `settings.as_mapping().and_then(|m| m.get(key))`, which
  already returns `None` for a non-mapping `settings` value — Rust's
  type system made this class of bug unreachable from the start.
- **`Config::PROMPTS_DIR` unchanged.** Ruby's `config.rb` picked up an
  extra `..` in this step, resolving to a nonexistent path — an
  unintentional upstream regression, confirmed with the user during
  the Python port, not reproduced. Rust's `Config::PROMPTS_DIR`
  (`concat!(env!(...), "/prompts")`) already points at
  `04_api_client/prompts` correctly and needs no change.
- **`read_file`/`list_directory` tool closures use
  `std::fs::read_to_string`/`std::fs::read_dir` with `.unwrap_or_else(|e|
  panic!(...))` on I/O failure**, matching Ruby's/Python's uncaught-
  exception-on-missing-file behavior. `list_directory` filters
  dotfiles via `!name.starts_with('.')` and does not sort —
  `std::fs::read_dir` yields OS/filesystem order, matching Python's
  `os.listdir`.

## Run Example

```bash
./week1_baseline/bin/rust/04_api_client
```

This makes a real HTTP request to whichever provider
`.boukensha/settings.yaml` configures (Anthropic, by default in this
repo's fixture) — it costs a small amount and requires a valid API key
in `.boukensha/.env`.

Example output (the exact response body is **not** reproducible
byte-for-byte — it's a live model response, and the model may choose a
different tool call, or none, from one run to the next):

```
=== BOUKENSHA Step 4: API Client ===

Config: #<Boukensha::Config dir=/.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Sending request to https://api.anthropic.com/v1/messages...

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
      "input": { "path": "." }
    }
  ],
  "stop_reason": "tool_use",
  "usage": {
    "input_tokens": 695,
    "output_tokens": 53
  }
}
```

`input_tokens: 695` matched exactly against the verified Ruby and
Python transcripts captured for this step, for the same fixture and
prompt — strong structural confirmation that all three languages build
byte-identical request payloads, even though the response content
itself varies run to run.
