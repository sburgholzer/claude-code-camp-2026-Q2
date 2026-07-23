# Week 1 Technical Documentation

## Technical Goal

I want to build a baseline agent that has all the common components for building any kind of agent. Things it should include:
- a simple agentic loop
- a tool registry along with tools
- it should be able to handle multiple backends
- it should be able to produce logs
- it should have a DSL so we can use the agent like an SDK
- it should have a global binary execution so we are able to interact with it via the CLI
- it should have an optional CLI mode
- it should manage context and compact messages when reaching the set token limit
- it should have its own configuration directory

Additional items that would be beneficial:
- a log visualizer so we can have a better way to view logs in a web browser

## Technical Uncertainty
- Is Rust going to be easy to implement an Agent in?
- How bad will it be to go from Python to Rust?
- Will I be able to use the Ruby MudManager in Rust easily?

## Technical Hypotheses 
- That Rust will be able to implement the agent
- That it will be somewhat challenging, but doable to go from Python to Rust.
- Calling another program written in another language should be fine in Rust, I don't see why not.

## Technical Observations
- As I don't use Ruby, and I run a Mac, Apple ships with Ruby 2.6.10 (at least mine did), which meant the ruby code given to us wouldn't run. I had to update, or rather install it, via homebrew.
- Python Port, via Claude Code was straight forward and easy. Just some pathing issues, but otherwise worked very well. (00_config, 01_struct_skeleton)
- Rust port went even smoother than python (mostly because pathing issues were solved.). The default prompts/system.md file path is baked into the compiled rust binary, so if it is moved from the host machine, it would break if no custom prompts in boukensha. The readme explains this. I'm still deciding what to do here, keep it like that, or bake the default prompt text into the compiled binary itself. Though on 01_struct_skeleton, this issue may have gone away... it came back in 03_prompt_builder. Keeping it as is for now, if the plan is to redistribute the rust binary, then we'd want to change it.

## Technical Conclusions
Reflecting back your education guesses from the technical uncertainty section what was the technical outcomes. Is there any new technical uncertainty that has been put aside for future exploration. Are there any next steps or technical considerations worth noting?


## Key Takeaway
In one sentence. State the most important lesson from the week.