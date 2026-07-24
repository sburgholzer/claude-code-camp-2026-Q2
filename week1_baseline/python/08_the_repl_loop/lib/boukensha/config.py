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

    # ---------- MUD connection ---------------------------------------------

    @property
    def mud_host(self):
        return self.dig("mud", "host") or "localhost"

    @property
    def mud_port(self):
        return self.dig("mud", "port") or 4000

    @property
    def mud_username(self):
        return self.dig("mud", "username")

    @property
    def mud_password(self):
        return self.dig("mud", "password")

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
        # 1. Explicit override
        if os.environ.get("BOUKENSHA_DIR"):
            return os.path.abspath(os.path.expanduser(os.environ["BOUKENSHA_DIR"]))

        # 2. .boukensha in the current working directory
        cwd_dir = os.path.join(os.getcwd(), ".boukensha")
        if os.path.isdir(cwd_dir):
            return cwd_dir

        # 3. ~/.boukensha default
        return os.path.abspath(os.path.expanduser(self.DEFAULT_DIR))

    def _load_env(self):
        env_file = os.path.join(self.dir, ".env")
        if os.path.exists(env_file):
            dotenv.load_dotenv(env_file)

    def _load_settings(self):
        settings_file = os.path.join(self.dir, "settings.yaml")
        if os.path.exists(settings_file):
            return yaml.safe_load(Path(settings_file).read_text()) or {}
        return {}
