# Host setup guide

| Host | Support | Best for |
| --- | --- | --- |
| Claude Desktop | ✅ - dynamic config management through aurora CLI | Exploring your data and generating visualizations |
| Cursor: Global | ✅ - dynamic config management through aurora CLI | Almost always better to use project-based configuration |
| Cursor: Project Based | ✅ - dynamic config management through aurora CLI | Creating code in your data engineering project or around it. |
| Others | ⚠️ - Try at your own risk, JSON config below | Living on the edge. Tell us what you are using at aurora@fiveonefour.com |

Our opinion on what each host is best for is based on our experience as of 27 March 2025 (this stuff changes quickly)!

### Claude Desktop

Quick links

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

- if there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

Note: you will have to restart Claude Desktop for the MCP to be picked up. If it was successful, you will see the Aurora tools listed beside the main chat box.

Common issues—if Claude Desktop doesn't pick up Aurora MCP:

- try restart Claude Desktop
- refer to Claude Desktop MCP settings (`Claude > Settings > Developer`)
- refer to Claude Desktop MCP documentation: https://docs.anthropic.com/en/docs/agents-and-tools/mcp

Common issues—if the MCP isn't giving you access to data

- ensure the local development server is running (`moose dev`)

### Cursor Global MCP

To create a new project (for details, see CLI Reference)

```
aurora init <TEMPLATE-NAME> --mcp cursor-global
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup --mcp cursor-global
```

This will look to the directory Cursor stores its MCP configuration (`~/.cursor/mcp.json`):

- if there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

You will have to enable the MCP server in the Cursor MCP settings (`Cursor > Settings > Cursor Settings > MCP`). You may then need to refresh the MCP server for it to work.

Common issues—if Cursor doesn't 🟢 the MCP:

- try disable and enable the MCP again
- try refresh the MCP
- See Cursor MCP documentation: https://docs.cursor.com/chat/tools#mcp

Common issues—if the MCP isn't giving you access to data

- ensure the local development server is running (`moose dev`)

### Cursor Project Based MCP (preferred)

To create a new project (for details, see CLI Reference)

```
aurora init <TEMPLATE-NAME> --mcp cursor-project
```

To configure the MCP for an existing project (for details, see CLI Reference)

```
aurora setup
```

(You don't need to use the `--mcp` flag since `cursor-project` is our default option.

This will look within the project for where Cursor stores its MCP configuration (`<path/to/project-root>/.cursor/mcp.json`):

- if there isn't a file there, it will create a config file and add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there, and there is no configuration, it will add the appropriate configuration JSON that allows it to use Aurora MCP
- If there is an existing file there with other MCPs, it will add the `"aurora"` JSON object to configure the aurora MCP

It will also list the project directory in the `~/.aurora/config.toml` file such that any changes to your MCP preferences (`aurora config`) will be propagated to this project.

You will have to enable the MCP server in the Cursor MCP settings (`Cursor > Settings > Cursor Settings > MCP`). You may then need to refresh the MCP server for it to work.

Common issues—if Cursor doesn't 🟢 the MCP:

- try disable and enable the MCP again
- try refresh the MCP
- See Cursor MCP documentation: https://docs.cursor.com/chat/tools#mcp

Common issues—if the MCP isn't giving you access to data

- ensure the local development server is running (`moose dev`)

### Cursor FAQ:

- I have the MCPs running, I can see them enabled and I can see the little green dot, but Cursor isn't using any of my tools?
    - Cursor will use the tools if the chat bar is set to agent mode (see the bottom left of the chat bar). Ensure you are in agent mode.

### Experimental: BYO Host

```
{
    "mcpServers": {
        "aurora": {
            "command": "npx",
            "args": [
                "@514labs/aurora-mcp",
                "/path/to/your/moose/project"
            ],
            "env": {
                "ANTHROPIC_API_KEY": "<API-Key>"
            }
        }
    }
}
```

This may give you access to the Aurora MCP in the tools of your choice, but it is unsupported by our config management CLI, so changes to preferences from `aurora config`will not be applied.

If you want to use experimental tools, use the following:

```
{
    "mcpServers": {
        "aurora": {
            "command": "npx",
            "args": [
                "@514labs/aurora-mcp",
                "--experimental",
                "/path/to/your/moose/project"
            ],
            "env": {
                "ANTHROPIC_API_KEY": "<API-Key>"
            }
        }
    }
}
```
