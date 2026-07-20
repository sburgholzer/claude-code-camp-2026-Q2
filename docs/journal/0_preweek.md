# Preweek Technical Documentation

## Technical Goal
_This week I am sticking with Andrew's intended goal, unless as I go through his videos a separate goal comes to mind._

The technical goal of Preweek (which is to Explore) is to determine how well do Agent Architectures fit our business use-case.

[Ref 1] Ways to Build Agents, From Zero-Code to Full Custom
- A single agent file (e.g., AGENT.md) that references external files for context (e.g., ~/docs/*.md)
- Skill-based architecture: one orchestrating agent, many plug-in capabilities (e.g.,  ~/.skills)
- A coding agent SDK that spawns sub-agents capable of reading, writing, and executing files (e.g., ~/subagents) or by using Filesystem Subagents
- A visual workflow platform that connects agents, tools, and triggers (e.g., n8n)
- A general-purpose agent SDK with plug-and-play components (e.g., LangChain, CrewAI)
- Using the model provider's SDK directly (e.g., Anthropic SDK, OpenAI SDK, AWS Bedrock SDK) and writing your own agent loop
- Calling the LLM's REST API directly and writing your own agent loop from scratch
  - The model decides what happens next, but your code can intercept and guide those decisions
  - Code-driven orchestration: the program dictates the steps, the LLM is just a tool it uses

## Technical Uncertainty
- To be quite honest the overall technical uncertainty is while I have heard of essentially all of the [Ref 1] methods to build Agents, I personally haven't fully built one out. I am familiar with the concepts, but don't have much experience to understand why one would be better than another for a use case, though I can make an educated guess with low confidence.
- Given [Ref 1], I'm uncertain if anything that isn't the last two methods will work for our use case


## Technical Hypotheses 
- Given how we are trying to play a MUD game, I highly doubt some of the simpler (Zero-code) methods will be of much use to us.
- A coding agent SDK could maybe work since it spawns sub-agents and can execute files, and maybe save files as a way to save information to remember.
- I honestly think the using the model provider's SDK directly or calling the LLM's REST API directly would be the two most likely candidates for what we are trying to do.

## Technical Observations

## Technical Conclusions


## Key Takeaway
