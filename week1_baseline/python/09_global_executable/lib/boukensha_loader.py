"""
boukensha_loader resolves which step folder to load from, then boots the REPL.
This is the "boukensha" console-script entry point installed by pyproject.toml.

Resolution order:
  1. BOUKENSHA_PATH environment variable (selects which *step* lib to load)
  2. ~/.boukensharc  (BOUKENSHA_PATH=... line, see boukensha_rc)
  3. The lib/ directory bundled inside this package (step 9 -- the latest release)

Config directory (settings.yaml, .env, system.md) is separate:
  BOUKENSHA_DIR=~/.boukensha  (default; override via env or ~/.boukensharc)

Examples:
  boukensha                                                              # uses bundled lib + ~/.boukensha
  BOUKENSHA_PATH=~/Sites/boukensha/04_api_client boukensha              # loads step 4
  BOUKENSHA_DIR=~/projects/mybot/.boukensha boukensha                   # custom config dir
  echo "BOUKENSHA_PATH=~/Sites/boukensha/08_the_repl_loop" > ~/.boukensharc && boukensha
"""
import os
import sys

import boukensha_rc

# Absolute path to this package's own bundled lib/ directory.
_BUNDLED_LIB_DIR = os.path.dirname(os.path.abspath(__file__))


def _has_boukensha_package(step_dir):
    return os.path.isfile(os.path.join(step_dir, "lib", "boukensha", "__init__.py"))


def resolve():
    """Returns the lib/ directory to prepend to sys.path for the selected
    step, or None to use the bundled default (already importable)."""
    # 1. Env var wins.
    env_path = os.environ.get("BOUKENSHA_PATH")
    if env_path:
        step_dir = os.path.abspath(os.path.expanduser(env_path))
        if _has_boukensha_package(step_dir):
            return os.path.join(step_dir, "lib")

        sys.exit(
            "boukensha: BOUKENSHA_PATH is set but no lib/boukensha package found at:\n"
            f"       {step_dir}\n"
            "       Make sure BOUKENSHA_PATH points to a step folder, e.g.:\n"
            "       BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop boukensha"
        )

    # 2. ~/.boukensharc
    rc_path = boukensha_rc.read().get("BOUKENSHA_PATH")
    if rc_path:
        step_dir = os.path.abspath(os.path.expanduser(rc_path))
        if _has_boukensha_package(step_dir):
            return os.path.join(step_dir, "lib")

        sys.exit(
            f"boukensha: ~/.boukensharc sets BOUKENSHA_PATH to {rc_path}\n"
            "       but no lib/boukensha package was found there.\n"
            "       Update ~/.boukensharc or remove it to use the bundled default."
        )

    # 3. Bundled default.
    return None


def main():
    step_lib = resolve()
    step_dir = os.path.dirname(step_lib) if step_lib else os.path.dirname(_BUNDLED_LIB_DIR)

    if os.environ.get("BOUKENSHA_DEBUG"):
        print(f"[boukensha] loading from: {step_dir}")

    if step_lib:
        sys.path.insert(0, step_lib)

    import boukensha

    if not hasattr(boukensha, "repl"):
        sys.exit(
            f"boukensha: the step at {step_dir}\n"
            "       does not support the interactive REPL (added in step 7).\n"
            "       Run its examples directly, e.g.:\n"
            f"         python {step_dir}/examples/*.py\n"
            "       Or point BOUKENSHA_PATH at step 7 or later."
        )

    boukensha.repl()


if __name__ == "__main__":
    main()
