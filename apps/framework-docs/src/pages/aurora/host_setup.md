# Host Setup Guide

| Host | Support | Best for |
| --- | --- | --- |
| Claude Desktop | ✅ - Dynamic config management through aurora CLI | Exploring your data and generating visualizations |
| Cursor: Global | ✅ - Dynamic config management through aurora CLI | Almost always better to use project-based configuration |
| Cursor: Project Based | ✅ - Dynamic config management through aurora CLI | Creating code in your data engineering project or around it |
| Windsurf: Global | ✅ - Dynamic config management through aurora CLI | Almost always better to use project-based configuration |
| Others | ⚠️ - Try at your own risk, JSON config below | Living on the edge. Tell us what you are using at aurora@fiveonefour.com |

Our opinion on what each host is best for is based on our experience as of March 27th 2025 (this stuff changes quickly)!

### Claude Desktop

Quick Links

- 🔗 Install Claude Desktop: https://claude.ai/download
- 🔗 Claude Desktop MCP reference: https://docs.anthropic.com/en/docs/agents-and-tools/mcp

To create a new project (for details, see CLI Reference)

```
aurora init <PROJECT-NAME> <TEMPLATE-NAME> --mcp claude-desktop
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup --mcp claude-desktop
```

This will look to the directory Claude stores its MCP configuration (`~/Library/Application Support/Claude/claude_desktop_config.json`):

- If there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

Note: you will have to restart Claude Desktop for the MCP to be picked up. If it was successful, you will see the Aurora tools listed beside the main chat box.

Common issues—if Claude Desktop doesn't pick up Aurora MCP:

- Try restart Claude Desktop
- Refer to Claude Desktop MCP settings (`Claude > Settings > Developer`)
- Refer to Claude Desktop MCP documentation: https://docs.anthropic.com/en/docs/agents-and-tools/mcp

Common issues—if the MCP isn't giving you access to data

- Ensure the local development server is running (`moose dev`)

### Cursor Global MCP

To create a new project (for details, see CLI Reference)

```
aurora init <PROJECT-NAME> <TEMPLATE-NAME> --mcp cursor-global
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup --mcp cursor-global
```

This will look to the directory Cursor stores its MCP configuration (`~/.cursor/mcp.json`):

- If there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

You will have to enable the MCP server in the Cursor MCP settings (`Cursor > Settings > Cursor Settings > MCP`). You may then need to refresh the MCP server for it to work.

Common issues—if Cursor doesn't allow "🟢" the MCP:

- Try disable and enable the MCP again
- Try refresh the MCP
- See Cursor MCP documentation: https://docs.cursor.com/chat/tools#mcp

Common issues—if the MCP isn't giving you access to data

- Ensure the local development server is running (`moose dev`)

### Cursor Project Based MCP (preferred)

To create a new project (for details, see CLI Reference)

```
aurora init <PROJECT-NAME> <TEMPLATE-NAME> --mcp cursor-project
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup
```

You don't need to use the `--mcp` flag since `cursor-project` is our default option.

This will look within the project for where Cursor stores its MCP configuration (`<path/to/project-root>/.cursor/mcp.json`):

- If there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

You will have to enable the MCP server in the Cursor MCP settings (`Cursor > Settings > Cursor Settings > MCP`). You may then need to refresh the MCP server for it to work.

Common issues—if Cursor doesn't 🟢 the MCP:

- Try disable and enable the MCP again
- Try refresh the MCP
- See Cursor MCP documentation: https://docs.cursor.com/chat/tools#mcp

Common issues—if the MCP isn't giving you access to data

- Ensure the local development server is running (`moose dev`)

#### Cursor FAQ:

- I have the MCPs running, I can see them enabled and I can see the little green dot, but Cursor isn't using any of my tools?
    - Cursor will use the tools if the chat bar is set to agent mode (see the bottom left of the chat bar). Ensure you are in agent mode.

### Windsurf Global MCP

To create a new project (for details, see CLI Reference)

```
aurora init <PROJECT-NAME> <TEMPLATE-NAME> --mcp windsurf-global
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup --mcp windsurf-global
```

This will look to the directory Windsurf stores its MCP configuration (`~/.codeium/windsurf/mcp_config.json`):

- If there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

You will have to enable the MCP server in the Cascade MCP settings (`Windsurf > Settings > Windsurf Settings > Cascade`). You may then need to refresh the MCP server for it to work.

### MCP.json 

Note, this will create an MCP outside of the management of the Aurora CLI (changes to `focus`, `tools`, etc. will not be propagated to this MCP).

Standard tooling, local only (note, you only need node or python, not both):

```JSON
{
  "mcpServers": {
    "aurora": {
      "args": [
        "@514labs/aurora-mcp@latest",
        "/path/to/moose/project"
      ],
      "command": "npx",
      "env": {
        "ANTHROPIC_API_KEY": "<key>",
        "MOOSE_PATH": "/path/to/.moose/bin/moose",
        "NODE_PATH": "/path/to/.nvm/versions/node/v22.14.0/bin/node",
        "PYTHON_PATH": "/path/to/.pyenv/shims/python"
      }
    }
  }
}
```
Experimental tooling, local only (note, you only need node or python, not both):

```JSON
{
  "mcpServers": {
    "aurora": {
      "args": [
        "@514labs/aurora-mcp@latest",
        "--experimental",
        "/path/to/moose/project"
      ],
      "command": "npx",
      "env": {
        "ANTHROPIC_API_KEY": "<key>",
        "MOOSE_PATH": "/path/to/.moose/bin/moose",
        "NODE_PATH": "/path/to/.nvm/versions/node/v22.14.0/bin/node",
        "PYTHON_PATH": "/path/to/.pyenv/shims/python"
      }
    }
  }
}
```

Experimental tooling, local development and remote Boreal—requires creation of read only credentials for Boreal managed Clickhouse:

```JSON
{
  "mcpServers": {
    "aurora": {
      "args": [
        "@514labs/aurora-mcp@latest",
        "--experimental",
        "--boreal-experimental",
        "/path/to/moose/project"
      ],
      "command": "npx",
      "env": {
        "ANTHROPIC_API_KEY": "<key>",
        "BOREAL_CLICKHOUSE_DATABASE": "",
        "BOREAL_CLICKHOUSE_HOST": "",
        "BOREAL_CLICKHOUSE_PASSWORD": "",
        "BOREAL_CLICKHOUSE_PORT": "",
        "BOREAL_CLICKHOUSE_USER": "",
        "MOOSE_PATH": "/path/to/.moose/bin/moose",
        "NODE_PATH": "/path/to/.nvm/versions/node/v22.14.0/bin/node",
        "PYTHON_PATH": "/path/to/.pyenv/shims/python"
      }
    }
  }
}
```

Query to create read only credentials:

```SQL
CREATE USER username IDENTIFIED BY 'password' SETTINGS PROFILE 'readonly'
GRANT SHOW TABLES, SELECT ON db+name.* TO username
```
