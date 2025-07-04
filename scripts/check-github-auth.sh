#!/bin/bash

# Check for existing GitHub authentication in Cursor Background Agent environment
echo "🔍 Checking for existing GitHub authentication..."
echo "=================================================="

# Check common GitHub environment variables
echo "📋 GitHub Environment Variables:"
echo "GITHUB_TOKEN: ${GITHUB_TOKEN:+[SET]}"
echo "GITHUB_ACCESS_TOKEN: ${GITHUB_ACCESS_TOKEN:+[SET]}"  
echo "GH_TOKEN: ${GH_TOKEN:+[SET]}"
echo "CURSOR_GITHUB_TOKEN: ${CURSOR_GITHUB_TOKEN:+[SET]}"
echo "GITHUB_PAT: ${GITHUB_PAT:+[SET]}"
echo "GITHUB_API_TOKEN: ${GITHUB_API_TOKEN:+[SET]}"
echo ""

# Check all environment variables containing 'github' or 'token'
echo "🔍 All GitHub/Token related environment variables:"
env | grep -i github || echo "No GITHUB variables found"
echo ""
env | grep -i token || echo "No TOKEN variables found"
echo ""

# Check for git credentials
echo "🔑 Git Configuration:"
git config --global user.name 2>/dev/null && echo "Git user.name is configured" || echo "Git user.name not configured"
git config --global user.email 2>/dev/null && echo "Git user.email is configured" || echo "Git user.email not configured"
echo ""

# Check if git credential helper is configured
echo "🔐 Git Credential Helpers:"
git config --global credential.helper 2>/dev/null || echo "No global credential helper configured"
git config --list | grep credential || echo "No credential configuration found"
echo ""

# Check for GitHub CLI authentication
echo "🐙 GitHub CLI Status:"
if command -v gh &> /dev/null; then
    echo "GitHub CLI is available"
    gh auth status 2>&1 || echo "GitHub CLI not authenticated"
else
    echo "GitHub CLI not installed"
fi
echo ""

# Check common GitHub API test
echo "🌐 Testing GitHub API Access:"
if [[ -n "$GITHUB_TOKEN" ]]; then
    echo "Testing with GITHUB_TOKEN..."
    curl -s -H "Authorization: bearer $GITHUB_TOKEN" https://api.github.com/user | jq -r '.login // "Failed to authenticate"' 2>/dev/null || echo "Test failed"
elif [[ -n "$GITHUB_ACCESS_TOKEN" ]]; then
    echo "Testing with GITHUB_ACCESS_TOKEN..."
    curl -s -H "Authorization: bearer $GITHUB_ACCESS_TOKEN" https://api.github.com/user | jq -r '.login // "Failed to authenticate"' 2>/dev/null || echo "Test failed"
else
    echo "No GitHub token found to test with"
fi
echo ""

echo "✅ GitHub authentication check complete"