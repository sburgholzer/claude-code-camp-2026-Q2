# Explore Agent Architectures

The largest confusion tech professionals have is applying the correct agent solution because many solutions appear to overlap responsibilities.

We will explore multiple agent architectures to determine fit for our agent workload.

## 1. An agent file with referenced files (e.g. AGENT.md, @~/docs/*.MD)

The simplest agent is creating an "agent file" and possibly importing other files that are read conditionally when needed.

We should attempt to create an agent file and see if it can connect to the MUD and complete a simple goal: "Can you find the bakery and tell me whats on the menu"

We want to use the smallest and least intelligent model and scale up.

### Technical Observations

I used Haiku 4.5 and the first attempt it attempted to use circlemud-world-parser to look at the world, which is not what we wanted. I exited that session and started a new session.

The new session had much better progress. It was using only the AGENT.md and data/player.md and data/world.md files. 

It was using python to connect to the MUD and play the game. It appeared successful, but was hard to always tell.

From what I was able to tell, it was able to connect fairly often (if not always), and move around the map. It never did find the bakery. It kept trying for around 7 minutes and around 10k tokens before it asked to start looking at other files  to locate the bakery. As we do not want the agent to use the world data, I ended the session there, and similar to Andrew in his video, stopped attempting this method at that time.


### Technical Conclusions

The agent was able to move around the map, but not efficiently and never did find the bakery. 

We could potentially write a better prompt to navigate the world better for what we are trying to do. 

Coding harnesses tend to go off task and try to write code which we do not need our agent to do.
Coding harnesses at least at this specific architecture stage do not appear to be a good fit.

We are justified to build our own MUD SDK to connect to the MUD since clearly the agent wants to mange the connection via script and execute common commands over the port.

If we had an MCP server for our MUD SDK then maybe we could drive the agent better at this architectural level.

Due to the complexity of the world and player state data, I do not believe that simply updating the markdown files will be sufficient for this architecture to be successful at completing goals in the MUD.

> Use coding harnesses for coding, and for specialized agents make your own loop.

## 2. Agent Skills driven by main agent, e.g., ~/.skills

A very common way to drive specific functionality is via Agent Skills, which is an open format for agents adopted by many coding harnesses and agent SDKs.

We should create a skill that has its own script to help it connect to a MUD, we should attempt to have it manage it's own data.


### Technical Observations

Skill Creator Skill used this prompt "I want a skill that can play a mud that is running on localhost:4000 it is tbaMUD a variation of CircleMude I have a player already created: dummy / helloworld. Can you create that skill and create a script to manage the telnet connect and issue command commands." and created a python script. It ran the script first to make sure it worked, and it ran into an issue and attempted to fix the issue and ran again. It worked, but found a couple of bugs that it fixed. It then verified the script worked and went back to work on writing the skill.

Unlike Andrew, my Claude Code put the skill in the 02_agent_skills, but I still had to do the .claude/skills directory set up like him.

This actually did find the bakery, gave me the route from my last location (which was the Inn), and listed the bakery items and their cost. It did stop executing once it found the bakery.

Switched to Haiku model and tried to find the player's starting guild and practice kick. It took a while, but it found the Guild, but unlike Andrew, mine did not actually attempt to practice the kick, it gave the location, full path, guild areas, and info about the kick skill and that the practice yard is where you'd practice and learn the kick ability. I never did get it to practice kick. In fact somehow it deleted my player!

After we made changes to the skill to add data/player.md and data/world.md, and switched back to Haiku, I tested finding the bakery again, and it found it and updated both files.

I tested the find guild and practice kick again as well. It got me stuck in a dungeon area in the pitch black. I had to use my admin character to teleport my player character back to the temple. Then from there clarified to Claude Code to look at the player's guild, go there and practice kick and it worked.

Attempted to find the minotaur. The Skill got me to the newbie zone and explored, but couldn't find anything. It kept opening doors and grates, and finding dark areas. So it went back to town to locate a light source. I stopped it there and gave it the same command as Andrew to locate the minotaur. My Claude Code seemed to go to start leveling up. I died to a baby dragon. After that, it went back to try to find the Red Room.

A Real player would have held the goal, and been more productive expecting it to the boss of the level, and progressively leveling up and exploring, not simply trying to find the end boss.

It did update the world and player stat, just not in real time. That made it hard to observe what it knows has changed. It should have been collecting observations to explore later, but instead would go back and just brute force not appearing to reason its journey pathing.

Claude Code's agentic loop is a good driver, but if they were to update Claude Code, we would have no idea how the agentic loop would be affected.

At scale, it could have a hard time managing the state of just markdown files for memory. A dynamic adaptive task management system would most likely be best option.

Example: Goal: Defat the Massive Minotaur in the Newbie Zone north of town

Before I find the Newbie Zone and leave the town, do I need to prepare?
- collect information from NPCs for my goal?
- can I obtain any resources?
- any training I need to do?

I should find the Newbie Zone.
- while on path, was there anything of interest that should warrant a detour? Would this spawn a sidequest?
- Explorer Mode:
  - Focused: Stay on main quest
  - Curious: Consider sidequests while on main quest, especially if could save backtracking or provide an advantage or resources
  - Aloof: Do all sidequests, and not worry too quickly about main quest progression

I have found the Newbie Zone.
- Risk Mode:
  - Bold: Try and push exploration to find your end goal, and try to run past high level mobs, or run away, try and push fighting stronger mobs to level up faster, and take more risks.
  - Balanced: <something inbetween>
  - Scared: Don't progress exploration where mobs are higher level or I am at a risk of dying. Take the time to be in a safe area and heal. If hungry and thirsty or risk of losing money, backtrack to town always, have plenty of resources.
  - There can be high level mobs that are not a risk like Town guards, context is key, if we are in a forest of monsters than mobs are higher risk.

## Technical Conclusions

Agent Skills does work, and pretty well, but we will need a much more complex state, world and player management. We really need to have auditable visibility of the agent for reporting token/usage and to review the player journey. A custom agentic loop would help solve this. We want an agent that acts, and spends less time asking "what should it do."

We probably should be defining a Player Persona, which describes how the player likes to play, based on a mix of modes e.g., Risk Mode, Exploration Mode, etc.

When we enter a goal, we should see a goal decomposition/planning, so we can see how it will reason the goal.

## 3a. Filesystem Subagent driven by a coding harness

### Technical Observations

Worked the same as 02_agent_skills, just as an agent, and it was able to login as the new second user.

I used the same prompt as Andrew to play both players, but it did not spawn two agents to play both. 

## 3b. Subagent SDK

### Technical Observations

It kept editing main.py for some reason.. not sure why.

My claude make an agents/play_mud_agent.py and a main.py, the main.py is how you interact with the agent, and you have to pass it in a goal... It did give me the hunger status with the prompt andrew used for both players, though it also showed thirst, which I'm not mad about.