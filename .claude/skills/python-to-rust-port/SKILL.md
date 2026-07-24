---
name: python-to-rust-port
description: "Port the next unported week1_baseline/python/NN_* step to week1_baseline/rust/NN_*, carrying over only the delta introduced by that Python step on top of a copy of the previous Rust step. Use when the user asks to port a boukensha step to Rust, continue the rust_port, or says things like 'port step 05 to rust', 'do the next rust port', 'port the agent loop to rust'."
---

# Python → Rust port (boukensha steps)

Ports one step of `week1_baseline/python/NN_<name>/` to
`week1_baseline/rust/NN_<name>/`. Sibling skill to
[[ruby-to-python-port]] — same discipline, different source language
and a couple of Rust-specific mechanics (crate manifests, workspace
membership, trait design).

**Core rule: Rust step N is a copy of Rust step N-1, plus only the
lines that changed between Python step N-1 and Python step N.** If a
Python file didn't change between steps, its Rust counterpart doesn't
change either — carry it forward untouched.

**Spec hierarchy: Python is the direct reference, Ruby is the ultimate
tiebreaker.** Read the Python source to know what to build (it's
closer to Rust's ahead-of-time-typed style than Ruby is), but when
Python's port and Ruby's original disagree, Ruby wins — Ruby is still
the ground-truth spec for the whole project, same precedent as
`01_struct_skeleton`/`02_the_registry`'s rust plans.

## 0. Find the step to port

If the user didn't name a step, find it:

```bash
ls week1_baseline/python       # highest NN_name here that has no bin/rust/NN_name launcher
ls week1_baseline/bin/rust
ls docs/plans/rust_port        # a missing or empty NN_name.md confirms it
cat week1_baseline/Cargo.toml  # workspace members — confirm NN_name isn't listed yet
```

Confirm with the user if more than one candidate looks unported.

## 1. Diff Python N-1 → Python N to find the real delta

```bash
diff -rq week1_baseline/python/<N-1>_prev week1_baseline/python/<N>_this
```

Read every file the diff reports as changed or new — these are the
*only* files whose behavior needs porting. Also read
`python/<N>_this/README.md` for the design spec, and skim the
corresponding `ruby/<N>_this/` files if anything about the Python
port looks like it drifted or simplified something Ruby does
differently (arity, rough edges Python carried forward on purpose,
etc.) — the rust plans call these out explicitly when they matter
(e.g. `03_prompt_builder`'s `to_messages`/`to_tools` passthrough).

## 2. Confirm the Rust baseline and branch it

`week1_baseline/rust/<N>_this/` is normally pre-scaffolded as an exact
copy of `rust/<N-1>_prev/` (including its own `Cargo.toml`, still
under the old step's package name):

```bash
diff -rq week1_baseline/rust/<N-1>_prev week1_baseline/rust/<N>_this
```

- If it reports no differences, that's your starting point — note the
  stale package name and any other carried-over identifiers you'll
  need to fix.
- If the directory doesn't exist yet, create it: `cp -r
  week1_baseline/rust/<N-1>_prev week1_baseline/rust/<N>_this`.
- If it already diverges from `<N-1>_prev` unexpectedly, stop and ask
  the user before overwriting anything.

Only edit the files whose Python counterparts changed in step 1, plus
whatever Rust-only bookkeeping is unavoidable (package name in the new
crate's `Cargo.toml`, new dependencies, workspace membership below).
Everything else stays byte-identical to `<N-1>_prev`.

## 3. Get verified real output before writing anything

Actually run the current Python step (ground truth, not just the
README) so you have a transcript to compare structurally against once
the Rust port runs:

```bash
week1_baseline/bin/python/<N>_this
```

## 4. Decide the Rust shape first (Decisions, confirmed)

Unlike the Python port — where most idiom choices were already
settled by earlier steps — Rust forces real upfront type/trait design
(ownership, trait objects vs generics, error enums vs strings, borrow
lifetimes). Work these out *before* writing the target-files list, and
write each as a numbered, confirmed decision:

- Reuse a prior step's precedent if one exists (check
  `docs/plans/rust_port/*.md` first) rather than re-deciding something
  already settled — e.g. `Registry` owning `Context` from
  `02_the_registry`, or `PROMPTS_DIR` as a `concat!(env!(...))` const
  from `00_config`.
- When a shape is genuinely ambiguous and no precedent settles it
  (e.g. a trait-object vs generic split, whether to dedupe near-
  identical backend methods into shared helpers, whether to drop an
  unreachable passthrough method entirely vs carry it forward as dead
  code), use AskUserQuestion to confirm with the user before
  implementing. Record the confirmed choice and its rationale here.
- Call out every genuine capability reduction versus the Python/Ruby
  surface (not just syntax differences) as its own decision — e.g. if
  a method has no safe Rust equivalent and gets dropped rather than
  faked, that needs to be visible, not buried.
- **New crate dependency, when Rust's `std` has no equivalent for
  something Ruby/Python's stdlib handled for free.** This project's
  default (see `ITERATIONS.md`) is stdlib-first, third-party-averse —
  it even hardcoded an OpenSSL path in `04_api_client` rather than
  pull in an HTTP library. Rust's `std` has *no* JSON, HTTP, or YAML
  support at all, so this will come up far more than it did for the
  Python port; that's still not a blank check to add whatever's
  convenient. The precedent already in this repo: `03_prompt_builder`
  added `serde_json` with the `preserve_order` feature specifically
  because tool/field insertion order is part of the observed behavior
  being ported and `std` can't serialize JSON at all, let alone
  order-preserving — not because it was easier than hand-rolling it.
  Every new crate needs its own numbered decision stating the `std`
  gap it fills and why no `std`-only workaround exists; a dependency
  that's merely more convenient than the `std` alternative doesn't
  qualify.

## 5. Write the plan doc

Create `docs/plans/rust_port/<N>_this.md`. Match the section order
already used by `docs/plans/rust_port/00_config.md` through
`03_prompt_builder.md` (note: this order differs from the
python_port plans — Decisions comes *before* Target files here,
because Rust's design decisions need to be settled before the file
list makes sense):

1. **Goal** — what this step ports, confirmation `<N>_this` currently
   equals `<N-1>_prev` (cite the `diff -rq` from step 2), which Python
   files are the only ones changing, and an explicit "Python is the
   direct reference, Ruby is the ultimate spec" line.
2. **Source files to port** — table of changed/new Python files (and
   any Ruby files worth reading directly) with their role.
3. **Runtime fixture to reuse** — `.boukensha/` at the repo root,
   shared/unchanged unless this step's diff shows otherwise.
4. **Decisions (confirmed)** — the numbered list from step 4 above.
5. **Target files (Rust)** — full file tree of `rust/<N>_this/`,
   marking `(new)`, `(edit: ...)`, or `(unchanged)`. Explicitly include:
   - root `week1_baseline/Cargo.toml` — `(edit: members +=
     "rust/<N>_this")`
   - `rust/<N>_this/Cargo.toml` — package name and any new deps
   - the `bin/rust/<N>_this` launcher (template below)
6. **Rust idiom choices (Ruby/Python concept → Rust shape)** — prose
   expansion of anything non-obvious from the Decisions list, same
   role as `ruby-to-python-port`'s "Porting notes."
7. **Behavior parity checklist** — one bullet per new
   type/trait/function/behavior, checkable against the step-3
   transcript; include the workspace-membership, package-name, and
   launcher items explicitly (they're easy to forget and this repo's
   plans always list them).
8. **Open questions** — should end up empty.

## 6. Implement

- Fix the new crate's `Cargo.toml` package name (it's stale, still
  named after `<N-1>_prev`, if the dir was copied).
- Add the crate to the root `week1_baseline/Cargo.toml` workspace
  `members` list — a Rust step is not done until this line is added;
  `cargo run` from inside the crate dir will still work without it,
  but the workspace build won't include the new step.
- Write only the files identified in step 5 as `(new)` or `(edit:
  ...)`. Match Python's *behavior* exactly — this is a port, not a
  redesign; only deviate where Rust has no safe/direct equivalent, and
  only per a Decision recorded in step 4.
- Copy non-code assets (e.g. `prompts/*.md`) byte-for-byte where
  Python's version is copied byte-for-byte.
- Add the launcher at `week1_baseline/bin/rust/<N>_this` (executable
  bit set):

```sh
#!/usr/bin/env bash

cd "$(dirname "$0")/../../rust/<N>_this"
cargo run --quiet --example example
```

## 7. Verify

```bash
chmod +x week1_baseline/bin/rust/<N>_this
cargo build --workspace   # confirms the new workspace member compiles
week1_baseline/bin/rust/<N>_this
```

Compare the printed output against the step-3 Python transcript
structurally (field names, ordering, nesting) — exact string/number
formatting differences that are just language-serializer cosmetics
(the same category of gap already accepted between Ruby and Python
reprs) don't need chasing, but note any new one in the plan doc rather
than silently ignoring it.

Finally, check off every item in the plan doc's behavior parity
checklist and clear the Open Questions section.

## Guardrails

- Python is the direct reference; Ruby is still the project's ultimate
  spec when the two disagree. Never invent behavior neither language
  has.
- Never touch `python_port` plans or files outside the current step's
  target-files list.
- Never re-derive or "clean up" files carried forward unchanged from
  the previous step, even if today's Rust idioms would shape them
  differently — consistency with the already-ported baseline matters
  more than local elegance.
- Don't skip the root `Cargo.toml` workspace-membership edit — it's a
  Rust-only step with no Python analog and is easy to forget.
- When a Rust shape is genuinely ambiguous and no earlier step's plan
  doc settles it, ask the user before implementing rather than
  guessing silently.
