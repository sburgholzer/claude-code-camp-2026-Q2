"""
boukensha_rc parses ~/.boukensharc, a small persistent config file that can
set BOUKENSHA_PATH and/or BOUKENSHA_DIR so you don't have to export them in
every shell session.

Format -- one KEY=VALUE per line, "#" comments and blank lines ignored:

  BOUKENSHA_PATH=~/Sites/boukensha/07_the_repl_loop
  BOUKENSHA_DIR=~/projects/mybot/.boukensha

Legacy format: a file containing just a bare path (no "=" on any
non-comment line) is treated as BOUKENSHA_PATH, e.g.:

  echo ~/Sites/boukensha/07_the_repl_loop > ~/.boukensharc
"""
import os

PATH = os.path.expanduser("~/.boukensharc")


def read():
    """Returns a dict of the keys set in ~/.boukensharc, e.g.
    {"BOUKENSHA_PATH": "...", "BOUKENSHA_DIR": "..."}.
    Returns {} if the file doesn't exist or is empty.
    """
    if not os.path.exists(PATH):
        return {}

    with open(PATH) as f:
        raw_lines = f.readlines()

    lines = [line.strip() for line in raw_lines]
    lines = [line for line in lines if line and not line.startswith("#")]
    if not lines:
        return {}

    if not any("=" in line for line in lines):
        # Legacy format: the whole file is a bare BOUKENSHA_PATH value.
        return {"BOUKENSHA_PATH": " ".join(lines).strip()}

    config = {}
    for line in lines:
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        config[key.strip()] = value.strip()
    return config
