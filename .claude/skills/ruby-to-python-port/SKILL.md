---
name: ruby-to-python-port
description: "Port the next unported week1_baseline/ruby/NN_* step to week1_baseline/python/NN_*, carrying over only the delta introduced by that Ruby step on top of a copy of the previous Python step. Use when the user asks to port a Ruby boukensha step to Python, continue the python_port, or says things like 'port step 05', 'do the next python port', 'port the agent loop to python'."
---

# Ruby → Python port (boukensha steps)

Ports one step of `week1_baseline/ruby/NN_<name>/` to
`week1_baseline/python/NN_<name>/`. This project builds the agent
step-by-step in Ruby first (the spec), then re-implements each step in
Python as a small, verifiable delta — never a from-scratch rewrite.

**Core rule: Python step N is a copy of Python step N-1, plus only the
lines that changed between Ruby step N-1 and Ruby step N.** If a Ruby
file didn't change between steps, its Python counterpart doesn't
change either — carry it forward untouched, do not "improve" it while
you're in the neighborhood.

## 0. Find the step to port

If the user didn't name a step, find it:

```bash
ls week1_baseline/ruby        # highest NN_name here that has no bin/python/NN_name launcher
ls week1_baseline/bin/python
ls docs/plans/python_port     # a missing or empty NN_name.md confirms it
```

The target is the lowest-numbered Ruby step that isn't fully ported
yet (no launcher, and/or an empty plan doc). Confirm with the user if
more than one candidate looks unported.

## 1. Diff Ruby N-1 → Ruby N to find the real delta

```bash
diff -rq week1_baseline/ruby/<N-1>_prev week1_baseline/ruby/<N>_this
```

Read every file the diff reports as changed or new — these are the
*only* files whose behavior needs porting. Also read
`ruby/<N>_this/README.md` for the design spec/considerations, but
treat it as documentation, not ground truth (see step 4 — READMEs in
this repo have drifted from actual behavior before, e.g.
`02_the_registry`'s README).

## 2. Confirm the Python baseline and branch it

`week1_baseline/python/<N>_this/` is normally pre-scaffolded as an
exact copy of `python/<N-1>_prev/`:

```bash
diff -rq week1_baseline/python/<N-1>_prev week1_baseline/python/<N>_this
```

- If it reports no differences, good — that's your starting point.
- If the directory doesn't exist yet, create it: `cp -r
  week1_baseline/python/<N-1>_prev week1_baseline/python/<N>_this`.
- If it already diverges from `<N-1>_prev` in ways you didn't expect,
  stop and ask the user before overwriting anything — it may be
  in-progress work.

Only edit the files inside `<N>_this` whose Ruby counterparts changed
in step 1. Everything else stays byte-identical to `<N-1>_prev`.

## 3. Get verified real output before writing anything

Don't trust the Ruby README's "Expected Output" section. Actually run
the Ruby step against the repo's real `.boukensha/` fixture and use
that transcript as ground truth:

```bash
week1_baseline/bin/ruby/<N>_this
```

If this differs from what the README claims, the plan doc must say so
explicitly and follow the real output (precedent: `02_the_registry.md`
documents exactly this kind of mismatch).

## 4. Write the plan doc first

Create `docs/plans/python_port/<N>_this.md` (same `NN_name` as the
Ruby/Python dirs) *before* writing Python code. Match the structure
already used by `docs/plans/python_port/00_config.md` through
`03_prompt_builder.md`:

1. **Goal** — one paragraph: what this step ports, confirmation that
   `<N>_this` python dir currently equals `<N-1>_prev` (cite the
   `diff -rq` from step 2), and which specific files are the only ones
   changing.
2. **Source files to port (Ruby)** — a table of the changed/new Ruby
   files and their role, plus a final row listing the unchanged files
   that "carry forward as-is."
3. **Runtime fixture to reuse** — note that `.boukensha/` at the repo
   root is shared/unchanged across steps unless this step's diff shows
   otherwise.
4. **Target files to create/change (Python)** — a full file tree of
   `python/<N>_this/`, marking each entry `(new)`, `(rewrite: ...)`, or
   `(unchanged, already correct)`. Include the `bin/python/<N>_this`
   launcher (see template below).
5. **Behavior parity checklist** — one bullet per new class/function/
   behavior, phrased so it's checkable against the verified output
   from step 3.
6. **Expected output** — the verbatim transcript from step 3, in a
   code fence, with a note on where it came from (real run, not
   README).
7. **Porting notes (Ruby idiom → Python)** — call out every place Ruby
   syntax has no direct Python equivalent (trailing blocks → decorator
   factories, optional-parens methods → deciding `@property` vs plain
   method, symbol/string key duality, class-method/instance-method
   name collisions, etc.) and *why* the chosen Python shape was picked
   — cite existing codebase precedent where one exists.
8. **Decisions** — numbered list, one per non-obvious porting call, each
   restating the rationale from the porting notes concisely enough to
   stand alone. If a decision is genuinely ambiguous (more than one
   reasonable Python shape), use AskUserQuestion to confirm with the
   user before implementing, then record the confirmed choice here.
9. **Open questions** — anything still undecided; should normally end
   up empty ("None outstanding — all decided above.").

Link back to the immediately-prior plan doc (`[N-1_prev.md](...)`) for
context that carries forward unchanged, the way each existing plan
doc does.

## 5. Implement

Write only the files identified in step 4's target-files table as
`(new)` or `(rewrite: ...)`. For each:

- Match the *behavior* of the Ruby source exactly — this is a port,
  not a redesign. Don't add error handling, validation, or structure
  the Ruby version doesn't have.
- Apply the same porting idioms already established in earlier steps'
  plan docs (decorator factories for Ruby blocks, `@property` for
  noun-named zero-arg accessors, no symbol/string transform needed in
  `dispatch`, etc.) — check `docs/plans/python_port/*.md` before
  inventing a new pattern for something already solved.
- Copy any non-code assets (e.g. `prompts/*.md`) byte-for-byte where
  the Ruby version is copied byte-for-byte.

Then add the launcher at `week1_baseline/bin/python/<N>_this`
(executable bit set), matching every existing launcher's shape:

```sh
#!/usr/bin/env bash

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/../../python/<N>_this"
"$SCRIPT_DIR/../../../.venv/bin/python" examples/example.py
```

If the Ruby step's `Gemfile` gained a new dependency, add the Python
equivalent to `python/<N>_this/requirements.txt` and note in the plan
whether `python/README.md`'s shared-venv instructions need a mention
(normally they don't — new deps just install into the same `.venv`).

**A new dependency can also be needed even when the Ruby side didn't
add one** — Ruby's stdlib may cover something Python's stdlib doesn't
(or vice versa isn't relevant here since Python is the target, but the
gap can run the other way too if the Ruby code leaned on a stdlib
`require` with no Python stdlib equivalent). This project's stated
default (see `ITERATIONS.md`) is to prefer each language's standard
library and avoid third-party packages — it even chose to hardcode an
OpenSSL path in `04_api_client` rather than pull in an HTTP gem/library.
So: only add a new pip dependency when Python's stdlib genuinely can't
do it, not for convenience, and record it as its own numbered Decision
in the plan doc stating exactly what stdlib gap it fills and why no
stdlib workaround exists — same bar as a Gemfile-mirrored dependency,
just not diff-triggered.

## 6. Verify

```bash
chmod +x week1_baseline/bin/python/<N>_this
week1_baseline/bin/python/<N>_this
```

Compare against the step-3 transcript. It should match structurally
and semantically. Known, already-accepted cosmetic gaps (Ruby symbol
reprs like `:direction` vs Python `'direction'` in `__repr__` strings)
are fine and don't need fixing — but note any *new* cosmetic diff you
find in the plan doc's Expected Output section, the way prior plans
do, rather than silently papering over it.

Finally, check off every item in the plan doc's behavior parity
checklist and clear the Open Questions section.

## Guardrails

- Ruby is always the spec. If Ruby's README and Ruby's actual runtime
  behavior disagree, the port follows the runtime, and the plan doc
  says so explicitly.
- Never touch `rust_port` plans or `python` files outside the current
  step's target-files list.
- Never re-derive or "clean up" files carried forward unchanged from
  the previous step, even if you'd write them differently today —
  consistency with the already-ported baseline matters more than
  local elegance.
- When a Ruby idiom has more than one reasonable Python shape (and no
  precedent in earlier plan docs settles it), ask the user before
  implementing rather than guessing silently.
