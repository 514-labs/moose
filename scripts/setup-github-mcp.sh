#!/bin/bash

# Setup GitHub MCP with automatic token detection
echo "🔧 Setting up GitHub MCP for Cursor Background Agent..."

# Check for existing GitHub authentication
GITHUB_TOKEN_TO_USE=""

if [[ -n "$GITHUB_TOKEN" ]]; then
    echo "✅ Found GITHUB_TOKEN environment variable"
    GITHUB_TOKEN_TO_USE="$GITHUB_TOKEN"
elif [[ -n "$CURSOR_GITHUB_TOKEN" ]]; then
    echo "✅ Found CURSOR_GITHUB_TOKEN environment variable"
    GITHUB_TOKEN_TO_USE="$CURSOR_GITHUB_TOKEN"
elif [[ -n "$GH_TOKEN" ]]; then
    echo "✅ Found GH_TOKEN environment variable"
    GITHUB_TOKEN_TO_USE="$GH_TOKEN"
else
    echo "⚠️  No GitHub token found in environment variables"
    echo "    You'll need to configure one manually in .cursor/mcp.json"
    GITHUB_TOKEN_TO_USE="your_github_token_here"
fi

# Create the MCP configuration
cat > .cursor/mcp.json << EOF
{
  "mcpServers": {
    "github-mcp": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "$GITHUB_TOKEN_TO_USE"
      }
    }
  }
}
EOF

if [[ "$GITHUB_TOKEN_TO_USE" == "your_github_token_here" ]]; then
    echo ""
    echo "🔑 Manual Setup Required:"
    echo "   1. Create a GitHub Personal Access Token:"
    echo "      - Go to GitHub Settings → Developer settings → Personal access tokens"
    echo "      - Generate token with 'repo' and 'read:discussion' scopes"
    echo "   2. Edit .cursor/mcp.json and replace 'your_github_token_here' with your token"
    echo ""
else
    echo "✅ GitHub MCP configured successfully with auto-detected token"
    echo "   Background agents can now access GitHub PR comments and issues!"
fi

echo "📋 Configuration saved to .cursor/mcp.json"