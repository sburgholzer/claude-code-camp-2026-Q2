import os
from pathlib import Path

import dotenv
import yaml


class Config:
    # The .boukensha config directory is resolved in this order:
    #   1. BOUKENSHA_DIR environment variable (set before loading .env)
    #   2. ~/.boukensha  (default)
    DEFAULT_DIR = os.path.join(str(Path.home()), ".boukensha")

    # Default prompts shipped alongside the library code.
    PROMPTS_DIR = str(Path(__file__).resolve().parent.parent.parent / "prompts")

    def __init__(self):
        self.dir = self._resolve_dir()
        self._load_env()
        self.settings = self._load_settings()

    # ---------- tasks ------------------------------------------------------

    def tasks(self, name=None):
        all_tasks = self.dig("tasks") or {}
        return all_tasks.get(name) if name else all_tasks

    @property
    def user_prompts_dir(self):
        return os.path.join(self.dir, "prompts")

    # ---------- low-level helpers -------------------------------------------

    def dig(self, *keys):
        node = self.settings
        for key in keys:
            if isinstance(node, dict):
                node = node.get(key)
            else:
                return None
        return node

    def __repr__(self):
        return f"#<Boukensha::Config dir={self.dir} tasks={','.join(self.tasks().keys())}>"

    __str__ = __repr__

    def _resolve_dir(self):
        raw = os.environ.get("BOUKENSHA_DIR") or self.DEFAULT_DIR
        return os.path.abspath(os.path.expanduser(raw))

    def _load_env(self):
        env_file = os.path.join(self.dir, ".env")
        if os.path.exists(env_file):
            dotenv.load_dotenv(env_file)

    def _load_settings(self):
        settings_file = os.path.join(self.dir, "settings.yaml")
        if os.path.exists(settings_file):
            return yaml.safe_load(Path(settings_file).read_text()) or {}
        return {}
