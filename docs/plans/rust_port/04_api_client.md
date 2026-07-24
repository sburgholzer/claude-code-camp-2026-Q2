# 04 · The API Client (Rust port)

## Goal

Ports `python/04_api_client` (and its Ruby ground truth,
`ruby/04_api_client`) into `rust/04_api_client`. This step takes the
payload assembled by `PromptBuilder` (step 3) and sends it to the API
as one HTTP POST, parsing the raw JSON response. No tool loop yet —
just proving the round trip works.

`rust/04_api_client` currently equals `rust/03_prompt_builder` byte
for byte (confirmed via `diff -rq rust/03_prompt_builder
rust/04_api_client` — no output), still under the stale
`boukensha_03_prompt_builder` package name.

Diffing `python/03_prompt_builder` → `python/04_api_client` (ignoring
`__pycache__`) shows the only real changes are:

- `lib/boukensha/client.py` (new)
- `lib/boukensha/errors.py` (added `ApiError`)
- `lib/boukensha/tasks/base.py` (`_fetch`'s dict guard; `settings.yml`
  → `settings.yaml` in error messages)
- `lib/boukensha/__init__.py` (export `Client`/`ApiError`)
- `examples/example.py` (read_file/list_directory tools, new user
  message, calls `Client.call`)
- `prompts/system.md` (new default prompt text)
- `README.md`

Python is the direct reference; Ruby (`ruby/04_api_client`) is the
ultimate spec where the two disagree — used here for `client.rb`'s
retry/response-object model and `tasks/base.rb`'s error-message
wording.

## Source files to port

| File | Role |
|---|---|
| `python/04_api_client/lib/boukensha/client.py` | Reference for retry loop, status/transient-error classification |
| `ruby/04_api_client/lib/boukensha/client.rb` | Ground truth: response-object-first model, exact retry constants |
| `python/04_api_client/lib/boukensha/errors.py` | `ApiError` addition |
| `ruby/04_api_client/lib/boukensha/tasks/base.rb` | `settings.yaml` wording (Ruby already fixed; Python matches) |
| `python/04_api_client/examples/example.py` | New tools (`read_file`, `list_directory`), new user message, `Client` wiring |
| `python/04_api_client/prompts/system.md` | New default prompt text (copy byte-for-byte) |

## Runtime fixture to reuse

`.boukensha/` at the repo root (`../​.boukensha` from
`week1_baseline/`), unchanged: `settings.yaml` configures
`tasks.player` for `anthropic`/`claude-haiku-4-5` with
`prompt_override.system: true`, and `.env` holds a real
`ANTHROPIC_API_KEY`. Verified Python transcript (this session, live
call):

```
=== BOUKENSHA Step 4: API Client ===

Config: #<Boukensha::Config dir=.../.boukensha tasks=player>
Provider: anthropic
Model: claude-haiku-4-5
Sending request to https://api.anthropic.com/v1/messages...

Raw response:
{
  "model": "claude-haiku-4-5-20251001",
  ...
  "content": [
    { "type": "text", "text": "..." },
    { "type": "tool_use", "name": "list_directory", "input": {"path": "."}, "caller": {"type": "direct"}, ... }
  ],
  "stop_reason": "tool_use",
  ...
  "usage": { "input_tokens": 695, ... }
}
```

`input_tokens: 695` is the structural anchor — Rust's request payload
must produce the same token count for the same fixture/prompt (already
confirmed byte-identical between Ruby and Python in the Python step's
README).

## Decisions (confirmed)

1. **HTTP + TLS crate: `ureq` (v3) with the `native-tls` feature,
   configured via `TlsProvider::NativeTls`.** Rust's `std` has no HTTP
   client and no TLS at all — a much larger gap than the JSON gap that
   justified `serde_json` in step 3. Confirmed with the user (asked via
   AskUserQuestion) over three alternatives:
   - `ureq` + rustls (default): rejected because rustls typically
     verifies against a bundled Mozilla root store, which would
     *silently fix* the "machine-specific TLS trust store" rough edge
     both Ruby's and Python's step-4 READMEs deliberately document
     (e.g. python.org's macOS installer not picking up the system
     keychain) rather than reproduce it.
   - Raw `TcpStream` + `native-tls`, hand-rolled HTTP/1.1 framing:
     rejected — hand-writing status-line/header/content-length/chunked
     body parsing for one call site is real protocol-correctness
     surface, not just "inconvenient", and this project's
     dependency-averse default doesn't require reinventing what a
     11-year-old widely-used crate already gets right.
   - `reqwest` (blocking): rejected as too heavy (hyper/tokio machinery
     even in blocking mode) for a single POST.
   `native-tls` uses the OS-native TLS implementation (Secure
   Transport on macOS, SChannel on Windows, OpenSSL on Linux) and the
   system trust store — the closest match to Ruby's `net/http` +
   `openssl` and Python's `ssl`-backed `urlopen`, both of which also
   defer to the platform's trust store and hit the same class of rough
   edge.
2. **Disable `http_status_as_error` on the `ureq::Agent` config, always
   getting `Ok(http::Response<Body>)`.** This mirrors Ruby's
   `client.rb` model directly (`Net::HTTP#request` always returns a
   response object; success/failure is checked *after* the retry loop
   via `response.is_a?(Net::HTTPSuccess)`), rather than Python's
   exception-per-non-2xx `urllib` model. Since Ruby is the ultimate
   spec and its shape maps more directly onto `ureq`'s
   `http_status_as_error(false)` mode than fighting an
   exception-shaped API would, this is the more faithful port target.
3. **`ApiError(pub String)` tuple struct in `errors.rs`**, following the
   exact pattern already used for `UnsupportedModelError(pub String)`
   in `03_prompt_builder`'s `errors.rs` — no new error-design decision
   needed, just consistency with the established style.
4. **Transient-error classification maps Ruby's `TRANSIENT_ERRORS` list
   onto `ureq::Error` variants at the same granularity Python's own
   port already chose** (treat `URLError`/broad connection failures as
   transient without unwrapping to the exact OS cause — see Python's
   README porting note). Rust's `is_transient` matches:
   `Timeout(_)`, `HostNotFound` (DNS, ~ Ruby's `SocketError`),
   `ConnectionFailed`, `Io(_)` (~ Ruby's `Errno::ECONNRESET`/
   `ECONNREFUSED`/`EOFError`), `Protocol(_)` (~ `EOFError` on a
   truncated read), and `Tls(_)`/`Rustls(_)`/`NativeTls(_)` (~
   `OpenSSL::SSL::SSLError`). Retryable HTTP status codes are the same
   literal set (`408 409 429 500 502 503 504`), `MAX_RETRIES = 3`,
   `BASE_RETRY_DELAY = 0.5` with the same `0.5 * 2^(attempt-1)` backoff.
5. **Non-transient, non-status `ureq::Error` variants (`BadUri`,
   `InvalidProxyUrl`, `TooManyRedirects`, cookie/decode/json-parsing
   errors, etc.) are wrapped into `ApiError` on the spot rather than
   left to propagate as a raw `ureq::Error`.** This is a capability
   reduction versus Ruby/Python, where an *unlisted* exception simply
   propagates uncaught (the process crashes with a backtrace) — Rust's
   single-`Result`-type public API doesn't have a clean equivalent to
   "let an arbitrary unrelated exception type propagate" without
   leaking `ureq::Error` into `Client::call`'s signature. Behaviorally
   unobservable for this step's fixture: a fixed, hardcoded HTTPS URL
   with no proxy and no redirects never triggers any of these variants.
6. **`RequestBuilder` is rebuilt fresh each retry attempt from
   URL/headers/body cached once before the loop**, since `ureq`
   consumes its builder on `.send()`. Ruby/Python build one
   request/body object before the loop and reuse it across retries;
   Rust caches the same three pieces of data and reconstructs the
   builder per attempt. No behavior difference — headers and body are
   byte-identical on every attempt either way.
7. **`_fetch`'s new `settings.is_a?(Hash)`/`isinstance(settings, dict)`
   guard needs no Rust change.** `Task::fetch` in
   `03_prompt_builder`'s `tasks/base.rs` already does
   `settings.as_mapping().and_then(|m| m.get(key))`, which already
   returns `None` for a non-mapping `settings` value — Rust's type
   system made this class of bug unreachable from the start. Recorded
   here as a capability note, not a Decision requiring a code change.
8. **`settings.yml` → `settings.yaml` wording fix.** Ruby's and
   Python's `tasks/base.{rb,py}` `provider`/`model` error messages
   changed from `settings.yml` to `settings.yaml` in this step. In
   Rust, that message lives in `config.rs`'s `ConfigError::Display`
   impl (`tasks.{task}.{key} is required in settings.yml`), not in
   `tasks/base.rs` (which only constructs the error, per
   `03_prompt_builder`'s Decision to centralize the message there) — so
   the one-word fix applies to `config.rs`, not `tasks/base.rs`.
9. **`Config::PROMPTS_DIR` unchanged.** Same reasoning as the Python
   port: Ruby's `config.rb` picked up an extra `..` in this step,
   resolving to a nonexistent path — an unintentional upstream
   regression, confirmed by the user during the Python port, not
   reproduced. Rust's `Config::PROMPTS_DIR` (`concat!(env!(...),
   "/prompts")`, 00_config's precedent) already points at
   `04_api_client/prompts` correctly and needs no change.
10. **`read_file`/`list_directory` tool closures use
    `std::fs::read_to_string`/`std::fs::read_dir` with `.expect(...)`
    on I/O failure**, matching Ruby's/Python's uncaught-exception-on-
    missing-file behavior (`File.read`/`Path.read_text` raise; nothing
    catches them). `list_directory` filters dotfiles via
    `!name.starts_with('.')` and does not sort — `std::fs::read_dir`
    yields OS/filesystem order, matching Python's `os.listdir` (which,
    unlike Ruby's `Dir.entries`, never yields `.`/`..` to begin with).

## Target files (Rust)

```
week1_baseline/Cargo.toml                       (edit: members += "rust/04_api_client")
week1_baseline/rust/04_api_client/
  Cargo.toml                                    (edit: package name → boukensha_04_api_client; + ureq dep)
  src/
    lib.rs                                      (edit: pub mod client; export Client, ApiError)
    client.rs                                    (new)
    errors.rs                                    (edit: + ApiError)
    config.rs                                    (edit: settings.yml → settings.yaml in ConfigError::Display)
    tasks/base.rs                                (unchanged — fetch() already guards non-mapping settings)
    context.rs, message.rs, prompt_builder.rs,
    registry.rs, tool.rs, tasks/mod.rs,
    tasks/player.rs, backends/*.rs               (unchanged)
  prompts/system.md                              (edit: copy new text byte-for-byte)
  examples/example.rs                            (edit: read_file/list_directory tools, new message, Client wiring)
week1_baseline/bin/rust/04_api_client            (new launcher)
```

## Rust idiom choices (Ruby/Python concept → Rust shape)

- **`Client<'a, T: Task>`** borrows `&'a PromptBuilder<'a, T>` and owns
  a `ureq::Agent`, mirroring Ruby's `Client.new(builder)` /
  `@builder` ivar — `PromptBuilder` itself already borrows `Context`,
  so `Client` borrowing `PromptBuilder` keeps the same ownership chain
  as `03_prompt_builder`.
- **`call(&self, max_output_tokens: u32) -> Result<serde_json::Value,
  ApiError>`** — keyword-optional `max_output_tokens: 1024` in
  Ruby/Python becomes a required positional `u32` in Rust (no default
  args in the language); the example always passes `1024` explicitly,
  identical to both Ruby's and Python's actual call sites (neither
  exercises the default).
- **Retry loop**: `loop { ... }` with a `u32 attempts` counter,
  structurally identical to Ruby's `loop do ... end` — checks
  retryable status first (sleep + `continue`), then a transient
  `Err` (sleep, loop back), then success-or-`ApiError` after the loop,
  same three-way branch as `client.rb`.
- **Backoff**: `Duration::from_secs_f64(0.5 * 2f64.powi(attempt as i32
  - 1))` fed to `std::thread::sleep`, the direct analog of Ruby's
  `sleep(BASE_RETRY_DELAY * (2**(attempt - 1)))`.

## Behavior parity checklist

- [x] `errors.rs`: `ApiError(pub String)` added, exported from `lib.rs`
- [x] `config.rs`: `ConfigError::Display` wording is `settings.yaml`
      (not `settings.yml`)
- [x] `client.rs`: `RETRYABLE_STATUS_CODES = [408, 409, 429, 500, 502,
      503, 504]`, `MAX_RETRIES = 3`, `BASE_RETRY_DELAY = 0.5`
- [x] `client.rs`: retryable status → sleep + retry; exhausted retries
      on a bad status → `ApiError` with attempt count + status + body
- [x] `client.rs`: transient `ureq::Error` → sleep + retry; exhausted →
      `ApiError` with attempt count + error
- [x] `client.rs`: success (2xx) → parsed `serde_json::Value` returned
- [x] `Cargo.toml`: package renamed `boukensha_04_api_client`; `ureq`
      (native-tls feature) added
- [x] root `Cargo.toml`: `rust/04_api_client` added to workspace
      `members`
- [x] `bin/rust/04_api_client` launcher added, executable
- [x] `examples/example.rs`: `read_file`/`list_directory` tools, single
      user message "What files are in the current directory?", prints
      Config/Provider/Model/URL lines then the raw pretty-printed JSON
      response
- [x] `prompts/system.md` copied byte-for-byte from
      `python/04_api_client/prompts/system.md`
- [x] `cargo build --workspace` succeeds
- [x] `bin/rust/04_api_client` runs against the live fixture and
      produces a structurally equivalent response (same top-level
      shape: `model`, `id`, `type`, `role`, `content[]`, `stop_reason`,
      `usage.input_tokens`). Verified: `input_tokens: 695` matched
      exactly against the Python transcript captured for this step,
      same as the byte-identical-payload confirmation already noted in
      `python/04_api_client/README.md`.

## Open questions

(none)
