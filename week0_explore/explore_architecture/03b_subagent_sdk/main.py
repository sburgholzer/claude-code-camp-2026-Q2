#!/usr/bin/env python3
"""Orchestrator entry point: drives the SDK-defined `play-mud` subagent.

Unlike `03a_subagent_sdk`, nothing here is discovered from `.claude/agents/`
on disk — the subagent is declared in code (`agents/play_mud_agent.py`) via
`AgentDefinition` and registered directly on `ClaudeAgentOptions`. A thin
orchestrator dispatches incoming goals to it through the `Task` tool, which
keeps the door open for registering additional subagents here later without
changing how goals are issued.

Usage:
    python3 main.py "find the bakery and tell me what's on the menu"
"""

import argparse
import asyncio
import re
import shlex
from pathlib import Path

from claude_agent_sdk import (
    AssistantMessage,
    ClaudeAgentOptions,
    ClaudeSDKClient,
    HookMatcher,
    ResultMessage,
    TextBlock,
    ToolResultBlock,
    ToolUseBlock,
)

from agents.play_mud_agent import MUD_ROOT, PLAY_MUD_AGENT

# Matches shell operators that chain multiple commands in one Bash call, so
# each chained piece can be validated on its own.
_SEGMENT_SPLIT_RE = re.compile(r"&&|\|\||[;|]")


def _deny(reason: str) -> dict:
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }


def _resolves_within_root(path_str: str, root: Path) -> bool:
    candidate = Path(path_str)
    if not candidate.is_absolute():
        candidate = Path.cwd() / candidate
    try:
        candidate.resolve().relative_to(root)
    except ValueError:
        return False
    return True


async def _restrict_bash(tool_input: dict) -> dict:
    """Only allow `python3 <script under MUD_ROOT>` Bash calls.

    `AgentDefinition.tools` can't express the `Bash(python3 *)` command-prefix
    filter that `.claude/agents/play-mud.md`'s frontmatter used in 03a, so
    this hook enforces the equivalent (and stricter: the script itself must
    live under this project directory, not just be a python3 invocation).
    """
    command = tool_input.get("command", "")
    segments = [s.strip() for s in _SEGMENT_SPLIT_RE.split(command) if s.strip()]
    if not segments:
        return _deny("Empty Bash command.")

    for segment in segments:
        try:
            tokens = shlex.split(segment)
        except ValueError as exc:
            return _deny(f"Could not parse command segment {segment!r}: {exc}")
        if not tokens:
            return _deny("Empty command segment.")

        program = tokens[0]
        if program not in ("python3", "python"):
            return _deny(f"Only python3/python invocations are allowed; got {program!r}.")

        script_tokens = [t for t in tokens[1:] if not t.startswith("-")]
        if not script_tokens:
            return _deny("Expected a script path argument.")
        script_path = script_tokens[0]
        if not _resolves_within_root(script_path, MUD_ROOT):
            return _deny(f"Script {script_path!r} is outside the allowed directory {MUD_ROOT}.")

    return {}


async def _restrict_file_path(tool_name: str, tool_input: dict) -> dict:
    """Only allow Read/Edit calls whose `file_path` lives under MUD_ROOT."""
    file_path = tool_input.get("file_path")
    if not file_path:
        return _deny(f"{tool_name} call is missing a file_path.")
    if not _resolves_within_root(file_path, MUD_ROOT):
        return _deny(f"{tool_name} path {file_path!r} is outside the allowed directory {MUD_ROOT}.")
    return {}


async def _force_foreground_dispatch(tool_input: dict) -> dict:
    """Force `Agent` dispatch calls to run in the foreground.

    A backgrounded dispatch returns control to the orchestrator (and ends its
    turn, which is what `receive_response()` waits for) before the subagent
    has actually run, so this script would exit having streamed nothing of
    the subagent's work. This demo is only useful if the dispatch is
    observable end-to-end, so pin it to synchronous execution regardless of
    what the model requests.
    """
    if tool_input.get("run_in_background") is False:
        return {}
    forced_input = dict(tool_input)
    forced_input["run_in_background"] = False
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "updatedInput": forced_input,
        }
    }


async def restrict_tools_to_repo(input_data, tool_use_id, context):
    """PreToolUse hook: keep every tool the play-mud subagent has access to
    (Bash, Read, Edit) scoped to files/scripts under this project directory,
    and keep subagent dispatch synchronous so output stays observable."""
    tool_name = input_data.get("tool_name")
    tool_input = input_data.get("tool_input", {})

    if tool_name == "Bash":
        return await _restrict_bash(tool_input)
    if tool_name in ("Read", "Edit"):
        return await _restrict_file_path(tool_name, tool_input)
    if tool_name == "Agent":
        return await _force_foreground_dispatch(tool_input)
    return {}


def build_options() -> ClaudeAgentOptions:
    return ClaudeAgentOptions(
        agents={"play-mud": PLAY_MUD_AGENT},
        allowed_tools=["Agent", "Bash", "Read", "Edit"],
        permission_mode="bypassPermissions",
        hooks={
            "PreToolUse": [
                HookMatcher(matcher="Bash|Read|Edit|Agent", hooks=[restrict_tools_to_repo])
            ]
        },
        cwd=str(MUD_ROOT),
        system_prompt=(
            "You are a thin orchestrator. For any request about playing, "
            "exploring, or automating the MUD, dispatch it to the 'play-mud' "
            "subagent via the Agent tool rather than handling it yourself. "
            "Always dispatch it in the foreground (run_in_background=False) "
            "and wait for its full result before replying — never tell the "
            "user you'll report back later."
        ),
    )


def _render_message(message) -> None:
    prefix = "[subagent]" if getattr(message, "parent_tool_use_id", None) else "[main]"
    if isinstance(message, AssistantMessage):
        for block in message.content:
            if isinstance(block, TextBlock):
                print(f"{prefix} {block.text}")
            elif isinstance(block, ToolUseBlock):
                print(f"{prefix} -> {block.name} {block.input}")
            elif isinstance(block, ToolResultBlock):
                print(f"{prefix} <- {block.content}")
    elif isinstance(message, ResultMessage):
        cost = f"${message.total_cost_usd:.4f}" if message.total_cost_usd else "n/a"
        print(f"\n--- done: {message.num_turns} turns, cost {cost} ---")


async def run(goal: str) -> None:
    options = build_options()
    async with ClaudeSDKClient(options=options) as client:
        await client.query(goal)
        async for message in client.receive_response():
            _render_message(message)


def main() -> None:
    parser = argparse.ArgumentParser(description="Drive the SDK-defined play-mud subagent.")
    parser.add_argument(
        "goal",
        help="Natural-language goal for the orchestrator, e.g. "
        "\"find the bakery and tell me what's on the menu\"",
    )
    args = parser.parse_args()
    asyncio.run(run(args.goal))


if __name__ == "__main__":
    main()
