#!/bin/bash

# Docker Helper Script for Cursor Dev Environment
# Handles Docker setup in containerized development environments

set -e

DOCKER_SOCK="/var/run/docker.sock"
DOCKER_DAEMON_LOG="/tmp/dockerd.log"

show_help() {
    cat << EOF
Docker Helper Script for Cursor Dev Environment

USAGE:
    docker-helper.sh [OPTIONS]

OPTIONS:
    --help              Show this help message
    --check             Check Docker daemon status and connectivity
    --start-daemon      Start Docker daemon (Docker-in-Docker mode)
    --stop-daemon       Stop Docker daemon
    --status            Show Docker daemon and socket status

EXAMPLES:
    # Check if Docker is working
    docker-helper.sh --check

    # Start Docker daemon for Docker-in-Docker
    sudo docker-helper.sh --start-daemon

    # Check status
    docker-helper.sh --status

NOTES:
    - RECOMMENDED: Mount Docker socket from host: -v /var/run/docker.sock:/var/run/docker.sock
    - ALTERNATIVE: Use --privileged mode and run --start-daemon for Docker-in-Docker
EOF
}

check_docker() {
    echo "🔍 Checking Docker setup..."
    
    # Check if Docker CLI is available
    if ! command -v docker &> /dev/null; then
        echo "❌ Docker CLI not found"
        return 1
    fi
    echo "✅ Docker CLI found: $(docker --version)"
    
    # Check if Docker socket exists
    if [ -S "$DOCKER_SOCK" ]; then
        echo "✅ Docker socket found at $DOCKER_SOCK"
    else
        echo "⚠️  Docker socket not found at $DOCKER_SOCK"
    fi
    
    # Test Docker connectivity
    if timeout 5 docker info &> /dev/null; then
        echo "✅ Docker daemon is accessible"
        echo "📊 Docker info:"
        docker info --format "table {{.ServerVersion}}\t{{.OSType}}\t{{.Architecture}}"
        return 0
    else
        echo "❌ Cannot connect to Docker daemon"
        echo "💡 Try one of these solutions:"
        echo "   - Mount Docker socket: -v /var/run/docker.sock:/var/run/docker.sock"
        echo "   - Use privileged mode and run: sudo docker-helper.sh --start-daemon"
        return 1
    fi
}

start_daemon() {
    if [ "$EUID" -ne 0 ]; then
        echo "❌ Starting Docker daemon requires root privileges"
        echo "💡 Run: sudo docker-helper.sh --start-daemon"
        return 1
    fi
    
    echo "🚀 Starting Docker daemon (Docker-in-Docker mode)..."
    
    # Check if daemon is already running
    if pgrep dockerd &> /dev/null; then
        echo "✅ Docker daemon is already running"
        return 0
    fi
    
    # Start Docker daemon in background
    echo "📝 Starting dockerd... (logs: $DOCKER_DAEMON_LOG)"
    dockerd --host=unix://$DOCKER_SOCK --storage-driver=overlay2 > "$DOCKER_DAEMON_LOG" 2>&1 &
    
    # Wait for daemon to start
    echo "⏳ Waiting for Docker daemon to start..."
    for i in {1..30}; do
        if timeout 2 docker info &> /dev/null; then
            echo "✅ Docker daemon started successfully"
            return 0
        fi
        echo -n "."
        sleep 1
    done
    
    echo ""
    echo "❌ Docker daemon failed to start within 30 seconds"
    echo "📝 Check logs: $DOCKER_DAEMON_LOG"
    return 1
}

stop_daemon() {
    if [ "$EUID" -ne 0 ]; then
        echo "❌ Stopping Docker daemon requires root privileges"
        echo "💡 Run: sudo docker-helper.sh --stop-daemon"
        return 1
    fi
    
    echo "🛑 Stopping Docker daemon..."
    
    if pgrep dockerd &> /dev/null; then
        pkill dockerd
        echo "✅ Docker daemon stopped"
    else
        echo "⚠️  Docker daemon was not running"
    fi
}

show_status() {
    echo "📊 Docker Status:"
    echo "=================="
    
    # Docker CLI version
    if command -v docker &> /dev/null; then
        echo "Docker CLI: $(docker --version)"
    else
        echo "Docker CLI: Not found"
    fi
    
    # Docker socket
    if [ -S "$DOCKER_SOCK" ]; then
        echo "Socket: ✅ $DOCKER_SOCK"
    else
        echo "Socket: ❌ $DOCKER_SOCK (not found)"
    fi
    
    # Docker daemon
    if pgrep dockerd &> /dev/null; then
        echo "Daemon: ✅ Running (PID: $(pgrep dockerd))"
    else
        echo "Daemon: ❌ Not running"
    fi
    
    # Docker connectivity
    if timeout 5 docker info &> /dev/null; then
        echo "Connectivity: ✅ Working"
    else
        echo "Connectivity: ❌ Failed"
    fi
    
    echo ""
    echo "💡 Recommendations:"
    if [ -S "$DOCKER_SOCK" ] && timeout 2 docker info &> /dev/null; then
        echo "   - Docker is working correctly! 🎉"
    elif [ ! -S "$DOCKER_SOCK" ]; then
        echo "   - Mount Docker socket: -v /var/run/docker.sock:/var/run/docker.sock"
        echo "   - Or use privileged mode: --privileged"
    else
        echo "   - Check if Docker daemon is running"
        echo "   - Try: sudo docker-helper.sh --start-daemon"
    fi
}

# Main script logic
case "${1:-}" in
    --help|-h)
        show_help
        ;;
    --check)
        check_docker
        ;;
    --start-daemon)
        start_daemon
        ;;
    --stop-daemon)
        stop_daemon
        ;;
    --status)
        show_status
        ;;
    "")
        # Default behavior - show status
        show_status
        ;;
    *)
        echo "❌ Unknown option: $1"
        echo "💡 Run 'docker-helper.sh --help' for usage information"
        exit 1
        ;;
esac