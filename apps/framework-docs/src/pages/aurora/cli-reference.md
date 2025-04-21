# CLI Reference

### Installation

```
bash -i <(curl -fsSL https://fiveonefour.com/install.sh) aurora,moose
```

### Commands

### Init

Creates a data engineering project with Moose, with Aurora MCP preconfigured.

```
aurora init <project-name> <template-name> <--mcp <host>> <--location <location>> 
```

- `<name>`: name of your application
- `<template-name>`: the template you are basing your application on. Choose from an empty project (e.g. `typescript-empty`, a default project `typescript` or a pre-built project `ads-b`).
- `-mcp` <host>: Choice of which MCP host to [default: `cursor-project`] [possible values: `claude-desktop`, `cursor-global`, `cursor-project`, `windsurf-global`]
- `-location <location>`: Location of your app or service. The default is the name of the project.
- `-no-fail-already-exists`: By default, the init command fails if `location` exists, to prevent accidental reruns. This flag disables the check.

### Setup

Takes an existing data engineering project build with Moose and configures Aurora MCP for it.

```
aurora setup [path] <--mcp <host>>
```

- Path to the Moose project (defaults to current directory)
- `-mcp` <host>: Choice of which MCP host to [default: `cursor-project`] [possible values: `claude-desktop`, `cursor-global`, `cursor-project`, `windsurf-global`]

### Config

Configure Aurora settings

### Config Focus

List Aurora configured projects, allows selection of project globally configured hosts are "focused" on. No more than one Moose Project may be run at a time.

```
aurora config focus
```

### Config Keys

Updates all MCP files for projects listed in ~/.aurora/aurora-config.toml to use updated API key.

```
aurora config keys <KEY>
```

- `<KEY>`: Your Anthropic API key. If you don't have an Anthropic API key, see the Anthropic initial setup guide: https://docs.anthropic.com/en/docs/initial-setup

### Config Models

[Coming Soon]. Allows you to configure which models Aurora agents use.

### Config Tools

Toggles availability of experimental MCP tools. See Tools documentation for which tools are in `standard` and which are added in the `experimental` sets.

```
aurora config tools [EXPERIMENTAL_PREFERENCE]
```

- `[EXPERIMENTAL_PREFERENCE]`: Preference for experimental tools. 

Note, if you select `boreal-experimental`, you will need to add your ClickHouse Cloud / Boreal credentials to `mcp.json`.

### Login

[Coming soon]. Allows you to log in to Aurora with your Boreal account. Will provide secure authentication and LLM provision.

```
aurora login
```
