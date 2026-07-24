# Python Port

One-time setup shared by every `python/NN_*` step. There is a single virtual
environment for the whole port line — created once at the root of the repo,
not per-step — so each new step only adds its own `requirements.txt` on top
of it.

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r python/00_config/requirements.txt
```

(Run from the repo root, sibling to `week0_explore/` and `week1_baseline/`.)

Each step's `bin/python/<step>` launcher assumes this venv already exists
and calls its interpreter directly (`.venv/bin/python`), the same way the
Ruby steps assume `bundle exec` has already resolved gems from the
`Gemfile` sitting next to the code.

When a later step adds new dependencies, install its `requirements.txt`
into the same `.venv` rather than creating a new one.

## macOS: fix TLS certificate errors before running steps that call an API

Starting at `04_api_client`, steps make real HTTPS requests. The
python.org macOS installer's Python build doesn't link into the macOS
system keychain trust store, so `urllib`/`ssl` can fail with:

```
ssl.SSLCertVerificationError: [SSL: CERTIFICATE_VERIFY_FAILED] certificate verify failed: unable to get local issuer certificate
```

This is a one-time, per-machine fix, not a bug in the port (a venv
shares its base interpreter's OpenSSL default paths, so this isn't
scoped away by being in `.venv` either). Fix it once for whichever
Python 3.x framework build the `.venv` was created from:

```bash
"/Applications/Python 3.12/Install Certificates.command"
```

(Adjust the version number to match. This installs/upgrades `certifi`
into that Python framework's own site-packages and symlinks its CA
bundle into the framework's default cert path — every venv built from
that interpreter picks it up automatically.)

See `04_api_client/README.md`'s "Considerations" section for the full
explanation and alternative one-off fixes (`SSL_CERT_FILE=...`, or
using a Python distribution that already links the system trust
store).
