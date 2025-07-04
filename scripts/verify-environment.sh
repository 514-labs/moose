#!/bin/bash

# Cursor Background Agent Environment Verification Script
# This script verifies that all required dependencies are installed and working

set -euo pipefail

echo "🔍 Verifying Cursor Background Agent Environment..."
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if command exists and print version
check_command() {
    local cmd="$1"
    local version_flag="${2:---version}"
    local name="${3:-$cmd}"
    
    if command -v "$cmd" &> /dev/null; then
        local version
        version=$($cmd $version_flag 2>&1 | head -n 1)
        echo -e "${GREEN}✓${NC} $name: $version"
        return 0
    else
        echo -e "${RED}✗${NC} $name: Not found"
        return 1
    fi
}

# Function to check Node.js version specifically
check_node_version() {
    if command -v node &> /dev/null; then
        local version
        version=$(node --version)
        local major_version
        major_version=$(echo "$version" | cut -d'.' -f1 | sed 's/v//')
        
        if [ "$major_version" -ge 20 ]; then
            echo -e "${GREEN}✓${NC} Node.js: $version (>= 20.0.0)"
            return 0
        else
            echo -e "${RED}✗${NC} Node.js: $version (< 20.0.0, upgrade required)"
            return 1
        fi
    else
        echo -e "${RED}✗${NC} Node.js: Not found"
        return 1
    fi
}

# Function to check Python version specifically
check_python_version() {
    if command -v python3 &> /dev/null; then
        local version
        version=$(python3 --version 2>&1)
        local version_number
        version_number=$(echo "$version" | grep -o '[0-9]\+\.[0-9]\+')
        local major_version
        major_version=$(echo "$version_number" | cut -d'.' -f1)
        local minor_version
        minor_version=$(echo "$version_number" | cut -d'.' -f2)
        
        if [ "$major_version" -eq 3 ] && [ "$minor_version" -ge 12 ]; then
            echo -e "${GREEN}✓${NC} Python: $version (>= 3.12.0)"
            return 0
        else
            echo -e "${RED}✗${NC} Python: $version (< 3.12.0, upgrade required)"
            return 1
        fi
    else
        echo -e "${RED}✗${NC} Python: Not found"
        return 1
    fi
}

# Function to check protoc version specifically
check_protoc_version() {
    if command -v protoc &> /dev/null; then
        local version
        version=$(protoc --version 2>&1)
        if echo "$version" | grep -q "24.4"; then
            echo -e "${GREEN}✓${NC} Protocol Buffers: $version (matches CI version)"
            return 0
        else
            echo -e "${YELLOW}⚠${NC} Protocol Buffers: $version (expected 24.4)"
            return 0  # Warning, not failure
        fi
    else
        echo -e "${RED}✗${NC} Protocol Buffers: Not found"
        return 1
    fi
}

# Function to check pnpm version specifically
check_pnpm_version() {
    if command -v pnpm &> /dev/null; then
        local version
        version=$(pnpm --version)
        if echo "$version" | grep -q "9.9.0"; then
            echo -e "${GREEN}✓${NC} pnpm: $version (matches project version)"
            return 0
        else
            echo -e "${YELLOW}⚠${NC} pnpm: $version (expected 9.9.0)"
            return 0  # Warning, not failure
        fi
    else
        echo -e "${RED}✗${NC} pnpm: Not found"
        return 1
    fi
}

# Track overall status
OVERALL_STATUS=0

echo "🌐 System Information:"
echo "OS: $(lsb_release -d | cut -f2-)"
echo "Architecture: $(uname -m)"
echo

echo "🔧 Core Dependencies:"
check_command "git" "--version" "Git" || OVERALL_STATUS=1
check_command "curl" "--version" "curl" || OVERALL_STATUS=1
check_command "wget" "--version" "wget" || OVERALL_STATUS=1
check_command "unzip" "-v" "unzip" || OVERALL_STATUS=1
check_command "zip" "-v" "zip" || OVERALL_STATUS=1
echo

echo "🦀 Rust Toolchain:"
check_command "rustc" "--version" "Rust Compiler" || OVERALL_STATUS=1
check_command "cargo" "--version" "Cargo" || OVERALL_STATUS=1
check_command "maturin" "--version" "Maturin" || OVERALL_STATUS=1
echo

echo "🌐 Node.js Environment:"
check_node_version || OVERALL_STATUS=1
check_command "npm" "--version" "npm" || OVERALL_STATUS=1
check_command "npx" "--version" "npx" || OVERALL_STATUS=1
check_pnpm_version || OVERALL_STATUS=1
echo

echo "🐍 Python Environment:"
check_python_version || OVERALL_STATUS=1
check_command "pip" "--version" "pip" || OVERALL_STATUS=1
echo

echo "📋 Protocol Buffers:"
check_protoc_version || OVERALL_STATUS=1
echo

echo "🏗️ Build Tools:"
check_command "gcc" "--version" "GCC" || OVERALL_STATUS=1
check_command "make" "--version" "Make" || OVERALL_STATUS=1
check_command "pkg-config" "--version" "pkg-config" || OVERALL_STATUS=1
echo

echo "� Docker (Required for Moose Development):"
check_command "docker" "--version" "Docker" || OVERALL_STATUS=1
check_command "docker-compose" "--version" "Docker Compose" || OVERALL_STATUS=1
echo

echo "� Project Verification:"
if [ -f "pnpm-workspace.yaml" ]; then
    echo -e "${GREEN}✓${NC} pnpm workspace configuration found"
else
    echo -e "${RED}✗${NC} pnpm workspace configuration not found"
    OVERALL_STATUS=1
fi

if [ -f "Cargo.toml" ]; then
    echo -e "${GREEN}✓${NC} Cargo workspace configuration found"
else
    echo -e "${RED}✗${NC} Cargo workspace configuration not found"
    OVERALL_STATUS=1
fi

if [ -f "apps/framework-cli/Cargo.toml" ]; then
    echo -e "${GREEN}✓${NC} Framework CLI Cargo.toml found"
else
    echo -e "${RED}✗${NC} Framework CLI Cargo.toml not found"
    OVERALL_STATUS=1
fi

if [ -f "packages/protobuf/infrastructure_map.proto" ]; then
    echo -e "${GREEN}✓${NC} Protocol buffer definitions found"
else
    echo -e "${RED}✗${NC} Protocol buffer definitions not found"
    OVERALL_STATUS=1
fi

echo
echo "=================================================="
if [ $OVERALL_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Environment verification passed!${NC}"
    echo "All required dependencies are installed and ready for development."
else
    echo -e "${RED}❌ Environment verification failed!${NC}"
    echo "Some dependencies are missing or have incorrect versions."
    echo "Please check the .cursor/Dockerfile for installation requirements"
fi

echo
echo "🚀 Next Steps:"
echo "1. Ensure Docker is running (required for 'moose dev')"
echo "2. Run 'pnpm install --frozen-lockfile' to install project dependencies"
echo "3. Run 'cargo build' to build the Rust CLI"
echo "4. Run 'pnpm build' to build TypeScript packages"
echo "5. Run 'moose dev' to start development with Docker containers"
echo "6. Check .cursor/Dockerfile for complete environment setup"

exit $OVERALL_STATUS