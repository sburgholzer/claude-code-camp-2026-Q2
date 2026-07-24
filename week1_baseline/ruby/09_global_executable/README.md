# Step 8 — Global Executable

Package BOUKENSHA as a gem so the `boukensha` command works from anywhere on your machine.

## What this step adds

- `boukensha.gemspec` — declares the gem: name, version, which files to include, and the `bin/boukensha` executable
- `bin/boukensha` — the shebang script that becomes the global command
- `lib/boukensha_loader.rb` — resolves *which step folder* to load from, then boots the REPL
- `lib/boukensha.rb` + `lib/boukensha/` — step 7's lib, bundled as the default

## Install

```bash
cd 09_global_executable
gem build boukensha.gemspec
gem install boukensha-0.9.0.gem
```

After that, `boukensha` is on your `$PATH` and works from any directory.

## Switching steps with BOUKENSHA_PATH

The loader resolves in this order:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | `BOUKENSHA_PATH` env var | `BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop boukensha` |
| 2 | `~/.boukensharc` file | `echo "BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop" > ~/.boukensharc` |
| 3 | Bundled default | just run `boukensha` |

`BOUKENSHA_PATH` must point to a step folder that contains `lib/boukensha.rb`.

## Persistent config with ~/.boukensharc

`~/.boukensharc` can set `BOUKENSHA_PATH` and/or `BOUKENSHA_DIR` so you don't
have to export them in every shell session:

```
# ~/.boukensharc
BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop
BOUKENSHA_DIR=~/projects/mybot/.boukensha
```

Blank lines and `#` comments are ignored. An environment variable always
overrides the matching rc value.

Legacy format: a `~/.boukensharc` containing just a bare path (no `=`) is
still read as `BOUKENSHA_PATH`, so existing rc files keep working:

```bash
echo ~/Sites/boukensha/07_the_repl_loop > ~/.boukensharc
```

`BOUKENSHA_DIR` picks the config directory (`settings.yaml`, `.env`,
prompt overrides) and resolves in this order:

| Priority | Source | Example |
|----------|--------|---------|
| 1 | `BOUKENSHA_DIR` env var | `BOUKENSHA_DIR=~/projects/mybot/.boukensha boukensha` |
| 2 | `~/.boukensharc` file | `BOUKENSHA_DIR=~/projects/mybot/.boukensha` line |
| 3 | `~/.boukensha` default | just run `boukensha` |

## Running a specific step

```bash
# step 7 (interactive REPL)
BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop boukensha

# step 6 doesn't have a REPL — loader tells you how to run it
BOUKENSHA_PATH=~/Sites/boukensha/06_the_run_dsl boukensha
# => boukensha: the step at .../06_the_run_dsl does not support the interactive REPL
#    Run its examples directly, e.g.: ruby .../06_the_run_dsl/examples/*.rb
```

## Debug mode

```bash
BOUKENSHA_DEBUG=1 boukensha
# => [boukensha] loading from: /path/to/step
```

## The key idea

The gem is just a **wrapper and a default**. All the teaching material stays in the numbered step folders exactly as it was. The gem doesn't copy or symlink anything — it just knows where to look.
