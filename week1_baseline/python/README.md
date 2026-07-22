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
