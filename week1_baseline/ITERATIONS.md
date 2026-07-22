# What is the goal for Week 1?

[Referenced here in Week 1 Journal](docs/journal/1_week1.md)


_Rest is taken from the Iterations Walkthrough video_

## What should the baseline agent be able to do?
It should be able to play the MUD, though we will have to give it specific commands.

## What will it not be able to do?
It will have poor perception since it doesn't have a way to manage memory, decision making, or be token effective.

## Technical Design Considerations
- We will use REST APIS directly. This design choice is made so that during this bootcamp, I as the student, can understand how simple it is to interact with managed APIs and how much they vary. It also allows me to understand how the entire agentic workflow works without any abstraction.
- Some SDKs, even official ones, do not expose all features, so REST APIs will allow us to have full access to feature sets.
- The agent is already given in Ruby, and the videos do show how to port it to Python. I as the student, have the ability to port it to any language of my choice.
- I must use the Ruby MudManager to interact with the MUD.
- I should attempt to use the Standard Library (STDs) as much as possible and avoid introducing third party libraries.

### What should we not use?
- I should avoid using Agent SDKs as they already implement features I will be implementing by scratch. They also could limit my ability to implement exactly what I need in my agent.
    - e.g. don't use OpenRouter, Amazon, Strands, CoreAgent, LangChain, etc.
- I shouldn't be using the coding harness to drive the agent as it is not purposed for our agent task.

## Explain Structure Approach

The `ruby/` folder contains each step-by-step iteration for agent.

### Considerations
- Some manual adjustments will be needed since the original code did not exist in a ruby sub-folder
- AI affected the handwritten code, so we will identify parts that should be rewritten but we may leave intact as to not disturb future layers.
- The videos will port the code over to Python, I may then port over to Rust (tbd). In both cases, need to ensure the MudManager Ruby version works with Ruby, Python and Rust. (I'm porting to Python as I know it more than Ruby, so porting that to Rust would be easier, and the videos will help with the Python port)

## Baseline MUD Agent

The baseline MUD agent is a fully working MUD agent that can connect to a tbaMUD server, log in as a character, and control it through natural language.

**What the baseline gives you:**
- A persistent TCP session to the MUD server that stays connected across the agent's tool calls
    - technically the MudManager is persisting the connection
- Five interchangeable LLM backends (Anthropic, OpenAI, Gemini, Ollama, Ollama Cloud) behind one normalized request/response shape, configured per-task in `settings.yaml`
    - Andrew implements all 5 backends, I may have them all coded in to have them there for reference, but I will personally be using just Anthropic for this bootcamp.
- MUD tools covering every core action: movement, combat, perception, inventory, magic, and communication
    - MudManager implements specific actions, but there are actions missing, e.g., Thief commands, rest commands, etc. I will need to consider solving these at some point, either end of week 1 or in week 2.
- A standard tool library for file I/O and shell commands so the agent can also read/write local state
    - These tools are simply mirrors to the MudManager tools and likely need reworking (which will occur in week 1)
- A multi-turn REPL so you can have a back-and-forth conversation with the agent while it plays
- Full conversation history carried across turns so the agent remembers what it has seen and done
    - This is the sessions log files, but consider we can load previous conversations since we don't implement those features in the agent.
- Colored structured logging of every API call, tool dispatch, and response
    - Technically there is a bit of coloring, but the web browser logger provides more information

**What it does not yet have** (to be added in later iterations):
- Long-term memory beyond the current conversation window
- A world model or map built from exploration
- Goal planning, tactical reasoning, or autonomous behavior
- Character progression tracking or strategy


- For each of our steps we will often have a class for each, e.g. Configuration will have config, REPL will have rebl.rb, etc.

### 0 Configuration

`Boukensha::Config` and ~/.boukensha directory stores all of our configuration data including secrets, prompts, logging (aka sessions), and settings file.

We have an env var called `BOUKENSHA_DIR` that lets us override its default location which is in the user's home directory.

We do use .dotenv standard for storing our secrets and we do need to include the dotenv library.

> If we are building an agent that can be deployed on multiple servers, having a configuration directory seems appropriate.

### 1 The Struct Skeleton

Define `Boukensha::Tool`, `Boukensha::Message`, and `Boukensha::Context` as plain data containers. No logic yet, just the shapes.

We are defining the main data structures to pass around data.

### 2 The Tool Registry

The tool Registry is responsible for managing a data table of possible tools, and also dispatching tools when called. In other words, it matches a prompt call to an appropriate tool.

> It is discovered that at some point the AI regressed the implementation and Context is still responsible for managing tools which is not correct and the tools[] will need to be moved to the Tool Registry

### 3 The Prompt Builder

Since we are calling multiple backends via direct REST API requests, we need to know exactly their schema structure. As such, we need to build those expected structures.

Additionally, we need the prompt builder to normalizer their responses into a single standard that our agent can understand.

> We have to consider the thinking option models, some models have thinking turned on by default, whereas others do not. Some models also cannot turn off thinking. There are other parameters we can fine tune, but we didn't spend much time exploring them in the video.

### 4 The API Client

The API Client is simply a low-level http-server making a direct API call to the REST API.

> We end up hardcoding the exact OpenSSL path, which changes based on the OS (Windows, Mac or Linux). A third party http-server like HTTPParty or Faraday would solve this, but it will abstract more and make it harder to see the moving parts. We would also have to use a library, so we just fix the code for where we run it.

### 5 The Agent Loop

`Boukensha::Agent` - the core agentic loop. Calls the API, checks `stop_reason`, dispatches tool calls back into the registry, appends results to the context, and repeats until `end_turn` or `MAX_ITERATIONS` is hit. Adds `Boukensha::Errors` (`LoopError`, `ApiError`) and wires everything together in `Boukensha.run`.

Also brings the OpenAI, Gemini, and Ollama Cloud backends online alongside Anthropic and Ollama - each implements `parse_response` to convert its raw reply into one normalized `{stop_reason:, content:}` shape so `Agent` never has to know which provider it's talking to.

> It was mentioned earlier we need to normalize the responses in the prompt backend and so it occurs here. It is believed the normalization implementation is done within the prompt builder and the provider backends.

### 6 The Logger

We create a logger which will record the logs of a session in ~/.boukensha/sessions/<data>-<session_id>.jsonl

> There is a log_viz app, which is a simple sintra app, to visualize the session. In the future it may be a good idea to port it to typescript and have it show realtime logs instead of having to refresh the page to get updated information.

It is made sure that the data stored is exactly which model, which provider and cost, trying to uplift as much information on each call for details reporting, and also allowing us to mid conversation to switch agents (despite lacking commands to do so in the CLI)

### 7 The Run DSL

Up to this point, there are multiple classes needed to create instances of, and it becomes a mess of code so it is time to implement a single .run command to abstract away the complexity and give us a SDK like interface to our agent.

`Boukensha::RunDSL` - the object `self` becomes inside a `Boukensha.run { }` block. Exposes a single `tool` method so callers can register ad-hoc tools inline alongside the task, keeping the DSL surface small and the main `Boukensha.run` signature clean.

### 8 The REPL Loop

It lets us have an interactive loop from the terminal.

`Boukensha::Repl` - an interactive session that stays alive across turns. Reads user input, runs the agent, prints the reply, and loops back to the prompt. A single `Context` is shared across all turns so the agent sees full conversation history. Built-in command: `/quiet`, `/loud`, `/clear` (wipe history, keep tools), `/exit` / `/quit`, `/help`. Adds `Boukensha::VERSON`.

### 9 Global Executable

Lets us call `boukensah` anywhere in our terminal to start using our agent.

> Here a .boukensharc file gets introduced which allows us to set the configuration path and the current gem path for boukensha binary to load and we end up having to carry that code in future steps.

Packages everything as an installable gem so the `boukensha` command is available anywhere on the machine. Adds `boukensha.gemspec`, `bin/boukensha`, and `lib/boukensha_loader.rb`. The loader resolves which step folder to use in priority oder: `BOUKENSHA_PATH` env var -> `~/.boukensharc` file -> bundled default. `BOUKENSHA_DEBUG=1` prints the resolved path on startup.

```sh
cd 09_global_executable
gem build boukensha.gemspec
gem install boukensha-0.9.0.gem

BOUKENSHA_PATH=~/Sites/boukensha/09_global_executable boukensah
```

Each step from here on ships its own gem the same way (`gem build boukensha.gemspec && gem install boukensha-<version>.gem`) - point `BOUKENSHA_PATH` at whichever step folder you want to run.

> This step was skipped in the video for Python port

### 10 Standard Tool Library - MCP Host

We are implementing a mapping of tools for the agent from the MudManager.
HOwever when we went to port the code to Python, the python app had no way of accessing the MudManager Ruby version, so we ended up implementing a MCP.

> The MCP implementation is a 2 hour video. It was worth watching, but not doing. It was recommended copying over the MudManger and the 10_standard_tool_library from Andrew's (omenking) repo.

> We end up adding the MCP server within MudManger so it is a single gem

> Also due to the major code changes here, we end up having to carry forward code, which makes the ruby step more involved.

### 11 Terminal UI

TUI is just a nicer REPL, so it has advanced display features within the terminal.

> We use Charm's BubbleTea for the TUI in Ruby, AI thinks BubbleTea is not available in Python and so it ends up using Texual. Though, since there is log_viz, we don't really need a TUI anymore, but in the original implementation of the agent while Andrew was developing this bootcamp, he had the TUI and didn't develop the log_viz till later, this is still here.

### 12 Context Management

There is no auto-compacting when you call an LLM directly - you're responsible for the context window. This step adds proper token tracking, visual warnings, and automatic compaction on top of the MCP-host tool model and TUI carried forward from steps 10-11.

> There should be settings exposed to increase the 600 e.g. 60,000 max token limit, as that is a very low amount but we never tested in Week 1, but it probably can be adjusted.