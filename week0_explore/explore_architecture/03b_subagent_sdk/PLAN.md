# Plan: Convert 03b to an `AgentDefinition`-based subagent (Claude Agent SDK)

## Current state

`03b_subagent_sdk` is currently an exact copy of `03a_subagent_sdk`: a
**filesystem-based** subagent. The MUD-playing agent is defined in
`.claude/agents/play-mud.md` (frontmatter + prompt body) and is discovered and
loaded automatically by the Claude Code CLI at session start. There is no SDK
code in this directory yet — it's driven entirely by the coding harness
reading that markdown file off disk.

Per `docs/explore_architectures.md`, `03b` is meant to be the next
architecture variant: drive the same MUD-playing subagent via the **Claude
Agent SDK** directly, defining the agent in code with `AgentDefinition`
instead of relying on the harness to discover a `.claude/agents/*.md` file.

## Goal

Write a standalone Python script that uses the Claude Agent SDK
(`claude_agent_sdk`) to run an orchestrator query which dispatches to a
`play-mud` subagent declared via `AgentDefinition` — same capabilities as
`play-mud.md` (description, system prompt, tool access), just declared in
Python instead of on disk.

## Steps

1. **Add dependency**
   - Add `claude-agent-sdk` (Python package) to a new `requirements.txt` in
     `03b_subagent_sdk/`.
   - Confirm `ANTHROPIC_API_KEY` (or whatever auth the SDK expects) is
     available in the shell env — I will not hardcode a key anywhere.

2. **Create `agents/play_mud_agent.py`** (or inline in the main script)
   - Port the content of `.claude/agents/play-mud.md` into an
     `AgentDefinition(...)`:
     - `description` ← the frontmatter `description` field (used for
       subagent routing/dispatch).
     - `prompt` ← the full markdown body (Players, Default server, Usage,
       Playing, Memory & Long-Term Goals, Troubleshooting sections) as the
       subagent's system prompt.
     - `tools` ← `["Bash"]` (equivalent to the `Bash(python3 *)` restriction
       in frontmatter; the SDK's `AgentDefinition.tools` doesn't support the
       finer-grained `Bash(python3 *)` command-prefix restriction, so I'll
       note that as a behavior change and, if needed, enforce the
       `python3`-only constraint via a `PreToolUse` hook/callback instead).
     - `model` ← leave unset to inherit the orchestrator's model, matching
       today's default behavior (no `model:` was pinned in the `.md` file).
   - Replace the hardcoded `MUD_ROOT` path in the prompt with one computed
     from the script's own location (`Path(__file__).resolve().parent`),
     so it stays correct if this directory moves — the current `.md` file
     even points at the wrong sibling dir (`03_subagent_sdk`, missing the
     `a`/`b` suffix), which this rewrite fixes.

3. **Create `main.py`** — the orchestrator entry point
   - Use `ClaudeAgentOptions(agents={"play-mud": <AgentDefinition>}, ...)`
     with `ClaudeSDKClient` (or the `query()` one-shot helper) from
     `claude_agent_sdk`.
   - Accept a goal/prompt on the CLI (e.g. `python3 main.py "find the bakery
     and tell me what's on the menu"`) and stream the orchestrator's
     response plus any subagent tool calls to stdout, so behavior is
     observable the same way the coding-harness runs were in
     `explore_architectures.md`.
   - Keep `scripts/mudctl.py` untouched — it's the actual MUD driver and is
     reused as-is via `Bash` tool calls from inside the subagent.

4. **Remove/retire the filesystem definition**
   - Delete `.claude/agents/play-mud.md` (and the now-empty `.claude/agents/`
     dir) from `03b_subagent_sdk` only, since the whole point of this
     variant is *not* loading subagents from the filesystem.
   - Leave `03a_subagent_sdk` untouched as the filesystem-driven baseline for
     comparison.

5. **Smoke test**
   - Run `python3 main.py "look around and tell me what you see"` against
     the local MUD (`localhost:4000`) to confirm the SDK dispatches to the
     `play-mud` subagent, it shells out to `mudctl.py`, and output streams
     back — mirroring the manual verification already done for 02/03a in
     `explore_architectures.md`.

## Open questions for you

- **Language**: Python (matches `mudctl.py`) or would you rather this be
  TypeScript (`@anthropic-ai/claude-agent-sdk`)?
A: We want this to be python.
- **Orchestrator shape**: a single top-level `query()` call that always
  routes to `play-mud`, or a thin main agent that can pick between multiple
  future subagents (closer to a "real" multi-agent setup)?
A: I'm not sure. Choose the best for me.
- **Tool restriction**: OK with enforcing "only `python3 mudctl.py`" via a
  hook/callback (since `AgentDefinition.tools` can't express the
  `Bash(python3 *)` prefix filter that frontmatter `tools:` could), or is
  plain `Bash` access acceptable for this exploration?
A: only allow scripts within the 03b_subagent_sdk folder.

Nothing has been installed, deleted, or written outside this plan file yet —
this is for your review before I touch any code.



