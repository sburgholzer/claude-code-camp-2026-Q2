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