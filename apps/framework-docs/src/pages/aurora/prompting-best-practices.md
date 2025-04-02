# Prompting best practices

There are many guides to prompting well, but the below are suggestions we have for working with Aurora's Agentic MCPs.

- It is easier for LLMs to do one thing at a time, we suggest asking questions: (1) introducing the context to the model; (2) inquiring to find interesting analytics; (3) asking for the creation of egress APIs with the MCP tools; and (4) asking for the creation of the front-end off the back of the API that was created.
- Giving the LLM the correct folder as context can improve the probability of success (e.g. when you are building an egress API, throw the `apis` folder in as context).