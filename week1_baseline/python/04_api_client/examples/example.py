import json
import os
import sys
from pathlib import Path

LIB_DIR = Path(__file__).resolve().parent.parent / "lib"
sys.path.insert(0, str(LIB_DIR))

from boukensha.backends.anthropic import Anthropic  # noqa: E402
from boukensha.backends.gemini import Gemini  # noqa: E402
from boukensha.backends.ollama import Ollama  # noqa: E402
from boukensha.backends.ollama_cloud import OllamaCloud  # noqa: E402
from boukensha.backends.openai import OpenAI  # noqa: E402
from boukensha.client import Client  # noqa: E402
from boukensha.config import Config  # noqa: E402
from boukensha.context import Context  # noqa: E402
from boukensha.prompt_builder import PromptBuilder  # noqa: E402
from boukensha.registry import Registry  # noqa: E402
from boukensha.tasks.player import Player  # noqa: E402

# Override the config directory so the example works from the repo root.
# In real usage a user's ~/.boukensha is picked up automatically.
os.environ.setdefault(
    "BOUKENSHA_DIR", str(Path(__file__).resolve().parents[4] / ".boukensha")
)

config = Config()
player_settings = config.tasks("player")
system_prompt = Player.system_prompt(
    player_settings,
    user_prompts_dir=config.user_prompts_dir,
    default_prompts_dir=Config.PROMPTS_DIR,
)

ctx = Context(task=Player, system=system_prompt)
registry = Registry(ctx)


@registry.tool(
    "read_file",
    description="Read the contents of a file from disk",
    parameters={"path": {"type": "string", "description": "The file path to read"}},
)
def read_file(path):
    return Path(path).read_text()


@registry.tool(
    "list_directory",
    description="List files in a directory",
    parameters={"path": {"type": "string", "description": "The directory path to list"}},
)
def list_directory(path):
    return "\n".join(f for f in os.listdir(path) if not f.startswith("."))


ctx.add_message("user", "What files are in the current directory?")

print("=== BOUKENSHA Step 4: API Client ===")
provider = Player.provider(player_settings)
model = Player.model(player_settings)

if provider == "anthropic":
    backend = Anthropic(api_key=os.environ["ANTHROPIC_API_KEY"], model=model)
elif provider == "ollama":
    backend = Ollama(model=model)
elif provider == "ollama_cloud":
    backend = OllamaCloud(api_key=os.environ["OLLAMA_API_KEY"], model=model)
elif provider == "openai":
    backend = OpenAI(api_key=os.environ["OPENAI_API_KEY"], model=model)
elif provider == "gemini":
    backend = Gemini(api_key=os.environ["GEMINI_API_KEY"], model=model)
else:
    raise ValueError(f"Unsupported provider for player task: {provider}")

builder = PromptBuilder(ctx, backend)
client = Client(builder)

print()
print(f"Config: {config}")
print(f"Provider: {provider}")
print(f"Model: {model}")
print(f"Sending request to {builder.url}...")
print()

response = client.call()
print("Raw response:")
print(json.dumps(response, indent=2))
