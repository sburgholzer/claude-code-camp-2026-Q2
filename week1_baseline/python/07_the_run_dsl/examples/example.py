import os
import sys
from pathlib import Path

LIB_DIR = Path(__file__).resolve().parent.parent / "lib"
sys.path.insert(0, str(LIB_DIR))

# Override the config directory so the example works from the repo root.
# In real usage a user's ~/.boukensha is picked up automatically.
os.environ.setdefault(
    "BOUKENSHA_DIR", str(Path(__file__).resolve().parents[4] / ".boukensha")
)

import boukensha  # noqa: E402

# Config is loaded automatically inside boukensha.run() — system prompt, model,
# and API key all come from ~/.boukensha (or BOUKENSHA_DIR) by default.
# You can still override any of them as keyword arguments if you want.

print("=== BOUKENSHA Step 7: The Boukensha.run DSL ===")
print()
print(f"Config: {boukensha.config()}")
print()

base_dir = Path(__file__).resolve().parent.parent


def register_tools(dsl):
    @dsl.tool(
        "read_file",
        description="Read the contents of a file from disk",
        parameters={"path": {"type": "string", "description": "The file path to read"}},
    )
    def read_file(path):
        return (base_dir / path).read_text()

    @dsl.tool(
        "list_directory",
        description="List the files in a directory",
        parameters={"path": {"type": "string", "description": "The directory path to list"}},
    )
    def list_directory(path):
        return ", ".join(f for f in os.listdir(base_dir / path) if not f.startswith("."))


result = boukensha.run(
    task="Read the README.md file and summarise what this MUD player assistant framework can do.",
    register=register_tools,
)

print()
print("=== FINAL RESPONSE ===")
print(result)
