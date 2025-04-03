# Tool reference

### Analytics Engineering Agent

### Read your moose project and data

- `read_moose_project`
    - Reads `moose.config.toml` to get configuration details about Moose project and infrastructure.
- `read_file`
    - Gives access to moose primitives in the moose project directory (Workflows, Data Models, Streaming Functions, APIs).
- `read_clickhouse_tables`
    - Read queries from clickhouse.
- `read_redpanda_topic`
    - Read from redpanda.

### Interact with your moose project and data

- `create_egress_api`
    - Creates an egress API from Moose Clickhouse. Can utilize type safe parameters.
- `test_egress_api`
    - Tests said apis.

### Experimental tools

> Note, these are experimental tools, if you want to use them, toggle the experimental features flag in your config file.
> 
- `write_spec`
    - Generate or update a specification for a data-intensive feature
- `write_workflow`
    - Write a script for use in Moose Workflows.
- `write_data_model`
    - Generate a Moose data model file and write it to the project's datamodels directory
- `run_workflowt`
    - Runs said scripts.
- `write_stream_function`
    - Generate a Moose stream processing function using an LLM and write it to the project's functions directory
- `write_and_run_temp_script`
    - Creates and runs a temporary script, usually for sampling purposes.