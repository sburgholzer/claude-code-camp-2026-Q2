from .agent import Agent
from .client import Client
from .config import Config
from .context import Context
from .errors import ApiError, UnknownToolError, UnsupportedModelError
from .logger import Logger
from .message import Message
from .prompt_builder import PromptBuilder
from .registry import Registry
from .tasks.player import Player
from .tool import Tool

_quiet = False
_debug = False
_config = None


def config():
    global _config
    if _config is None:
        _config = Config()
    return _config


def quiet():
    global _quiet
    _quiet = True


def loud():
    global _quiet
    _quiet = False


def is_quiet():
    return _quiet


def debug():
    global _debug
    _debug = True


def is_debug():
    return _debug


__all__ = [
    "Agent",
    "ApiError",
    "Client",
    "Config",
    "Context",
    "Logger",
    "Message",
    "Player",
    "PromptBuilder",
    "Registry",
    "Tool",
    "UnknownToolError",
    "UnsupportedModelError",
    "config",
    "quiet",
    "loud",
    "is_quiet",
    "debug",
    "is_debug",
]
