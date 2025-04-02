# CLI reference

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
- `-mcp` <host>: Choice of which MCP host to [default: `cursor-project`] [possible values: `claude-desktop`, `cursor-global`, `cursor-project`]
- `-location <location>`: Location of your app or service. The default is the name of the project.
- `-no-fail-already-exists`: By default, the init command fails if `location` exists, to prevent accidental reruns. This flag disables the check.

### Setup

Takes an existing data engineering project build with Moose and configures Aurora MCP for it.

```
aurora setup [path] <--mcp <host>>
```

- Path to the Moose project (defaults to current directory)
- `-mcp` <host>: Choice of which MCP host to [default: `cursor-project`] [possible values: `claude-desktop`, `cursor-global`, `cursor-project`]

### Config

Configure Aurora settings

### Config focus

List Aurora configured projects, allows selection of project globally configured hosts are "focused" on. No more than one Moose Project may be run at once.

```
aurora config focus
```

### Config keys

Updates all MCP files for projects listed in ~/.aurora/aurora-config.toml to use updated API key.

```
aurora config keys <KEY>
```

- `<KEY>`: Your Anthropic API key. If you don't have an Anthropic API key, see the Anthropic initial setup guide: https://docs.anthropic.com/en/docs/initial-setup

### Config Models

[Coming Soon] will allow you to configure which models Aurora agents use.

### Config tools

Toggles availability of experimental MCP tools. See Tools documentation for which tools are in `standard` and which are added in the `experimental`set.

```
aurora config tools [EXPERIMENTAL_PREFERENCE]
```

- `[EXPERIMENTAL_PREFERENCE]`: Preference for experimental tools [default: `standard`] [possible values: `standard`, `experimental`]

### login

[Coming soon]. Allows you to log in to Aurora with your Boreal account. Will provide secure authentication and LLM provision.

```
aurora login
```