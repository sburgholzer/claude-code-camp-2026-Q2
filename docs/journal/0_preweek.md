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

### A single agent file (e.g., AGENT.md) that references external files for context (e.g., ~/docs/*.md)

Claude first attempted to use the circlemud-world-parser to look at the world to gather information, when that tool is for us to use, not our agent. 

The second session I started did not attempt to locate that data and started with no file lookups. It seemed to have connected to my MUD pretty well, compared to Andrew's in the videos, and it was moving around the map. It tried moving around the map for around 7 minutes before it went back trying to look at other files in the repo to gather information about the world.

### Skill-based architecture: one orchestrating agent, many plug-in capabilities (e.g.,  ~/.skills)

After installing the Skill Creator plugin and creating the skill to use in this exploration, the agent was able to find the bakery, give the route it took and listed the items in the bakery and their cost, exactly what we wanted. This attempt was using Sonnet as the model.

Changed the model to Haiku and had the agent find the player's starting guild and then practice the kick skill. It took quite a while to locate the guild, but it did. However, it never even attempted to practice the kick skill. I prompted it again to do so, but somehow it did some weird things and deleted my player!

Just as Andrew did in the videos, I updated the Skill to use data/player.md and data/world.md as a memory store. To test this, since I had a brand new player, I prompted it to find the bakery again, and the Skill found the bakery and did indeed update both memory files with information.

I tested the find guild and practice kick again since we had this memory. The prompt took "player's guild" to literally mean a guild named "player's", so it kept searching for that guild and eventually got me stuck in a dungeon area in the pitch black. As it couldn't find it's way out of there, I had to login as my admin player, and teleport my player back to the temple. I clarified that "player's guild" meant look up what guild the player is in then find it. After doing that, it found the guild and did indeed practice the kick skill.

As we knew, from our world exploration tool, there is a minotaur in the Newbie Zone. The Skill was successful in getting me to the newbie zone. It did start opening doors and grates, and once again finding dark areas. So it started to go back to town to locate a light source for the dark areas. I stopped the agent at that point and gave it the prompt of the minotaur being in the "red room" of the Newbie Zone and that it should decide if it should search for it right away or level up. It initially did start with the goal of leveling up but eventually returned to locating the "red room".


### Sub-agents

Sub-agents were essentially our SKILL.md file but making it into an agent, so it worked essentially the same as Skill-based architecture.


### Overall:

- Skill based and Sub-agent based could play the MUD, but not as efficient as a player, but much more efficiently than AGENT.md.
- All architectures thus far have some big limitations from our testing and observations.



## Technical Conclusions

### A single agent file (e.g., AGENT.md) that references external files for context (e.g., ~/docs/*.md)

The agent was indeed able to move around the map, but wasn't efficient and never did find the bakery. A better prompt may have helped navigate the world better. 

Coding harnesses do tend to go off task and try to write code which we don't need our agent to do.

Due to the complexity of the world and player state data, I do not believe that if we were to simply update the markdown files, it would be sufficient for this architecture to be successful at completing goals in the MUD.

### Skill-based architecture: one orchestrating agent, many plug-in capabilities (e.g.,  ~/.skills)

Agent Skills do work, and pretty well, but we will need to have a more complex state, world and player management system in place. We weren't able to view exactly what was always happening, so we would want to have an auditable visibility of the agent for reviewing the player journey but also for token/usage reporting. The agent did keep asking us what it should do throughout the journey.

### Sub-agents

Similar to Skill-based architecture since it is SKILL.md just reworked.


### Overall
- We do need specialized memory to store map navigation and world data
- We opened a new technical use-case of if our agent should handle multiple sessions of multiple players, as this can be a co-op game, and common factor in MUDs which was forgotten in our design.
- Without having our own customized agentic loop, the agents could not perform goals efficiently. They did not have any key meta strategies or journey player strategies.

## Key Takeaway

While most of these architectures did connect and move around the world, and two were able to complete basic goals, none of these appear to be successful at completing long term tasks.