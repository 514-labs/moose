FROM ubuntu:22.04

# Set non-interactive mode to prevent prompts
ENV DEBIAN_FRONTEND=noninteractive

# Set timezone early to prevent clock skew issues
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Work around potential clock skew in build environment for apt
RUN printf 'Acquire::Check-Valid-Until "false";\nAcquire::Check-Date "false";\n' > /etc/apt/apt.conf.d/99no-check-valid-until

# Install system dependencies (protobuf-compiler will be installed separately with specific version)
# Use apt options to handle potential clock skew issues
RUN apt-get update -o Acquire::Check-Valid-Until=false -o Acquire::Check-Date=false && apt-get install -y --no-install-recommends \
    curl \
    wget \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    libpq-dev \
    unzip \
    zip \
    ca-certificates \
    gnupg \
    lsb-release \
    python3-dev \
    python3-pip \
    python3-venv \
    apt-transport-https \
    software-properties-common \
    sudo \
    kmod \
    iptables \
    fuse-overlayfs \
    && rm -rf /var/lib/apt/lists/*

# Create ubuntu user early with proper sudoers permissions
RUN useradd -m -s /bin/bash ubuntu && \
    echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ubuntu && \
    chmod 0440 /etc/sudoers.d/ubuntu

# Install Docker CLI and Daemon (for Cursor background agent)
# Primary approach: Docker socket mounting from host
# Fallback: Docker-in-Docker with privileged mode if needed
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update -o Acquire::Check-Valid-Until=false -o Acquire::Check-Date=false \
    && apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/*

# Setup docker group and add users
RUN groupadd -f docker \
    && usermod -aG docker root \
    && usermod -aG docker ubuntu

# Create necessary directories for Docker-in-Docker
RUN mkdir -p /var/lib/docker \
    && mkdir -p /var/run/docker \
    && chmod 755 /var/lib/docker \
    && chmod 755 /var/run/docker

# Set up Docker daemon configuration for better compatibility
RUN mkdir -p /etc/docker \
    && echo '{\
  "storage-driver": "vfs",\
  "log-driver": "json-file",\
  "log-opts": {\
    "max-size": "10m",\
    "max-file": "3"\
  },\
  "experimental": false,\
  "debug": false\
}' > /etc/docker/daemon.json

# Install Docker Compose standalone for maximum compatibility
RUN curl -SL https://github.com/docker/compose/releases/download/v2.24.1/docker-compose-linux-x86_64 -o /usr/local/bin/docker-compose \
    && chmod +x /usr/local/bin/docker-compose

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install pnpm
RUN npm install -g pnpm@9.9.0

# Install Python 3.12 (Ubuntu 22.04 comes with Python 3.10, we need 3.12)
RUN apt-get update -o Acquire::Check-Valid-Until=false -o Acquire::Check-Date=false && apt-get install -y software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa -y \
    && apt-get update -o Acquire::Check-Valid-Until=false -o Acquire::Check-Date=false \
    && apt-get install -y python3.12 python3.12-dev python3.12-venv \
    && rm -rf /var/lib/apt/lists/*

# Make python3.12 the default python3
RUN update-alternatives --install /usr/bin/python3 python3 /usr/bin/python3.12 1

# Install pip for Python 3.12
RUN curl -sS https://bootstrap.pypa.io/get-pip.py | python3.12

# Install additional Python packages needed for development
RUN python3.12 -m pip install --break-system-packages virtualenv setuptools wheel

# Install Protocol Buffers compiler (version 24.4 to match CI)
RUN PROTOC_VERSION=24.4 \
    && PROTOC_ZIP=protoc-$PROTOC_VERSION-linux-x86_64.zip \
    && curl -OL https://github.com/protocolbuffers/protobuf/releases/download/v$PROTOC_VERSION/$PROTOC_ZIP \
    && unzip -o $PROTOC_ZIP -d /usr/local bin/protoc \
    && unzip -o $PROTOC_ZIP -d /usr/local 'include/*' \
    && rm -f $PROTOC_ZIP \
    && chmod +x /usr/local/bin/protoc

# Switch to ubuntu user for remaining installations
USER ubuntu
WORKDIR /home/ubuntu

# Install maturin for Python-Rust bindings (with --break-system-packages for newer Python)
RUN pip install maturin==1.8.1 --break-system-packages

# Install Rust toolchain for ubuntu user (the user that actually uses the environment)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set up PATH environment variables for consistent access in all contexts
ENV PATH="/home/ubuntu/.cargo/bin:/home/ubuntu/.local/bin:${PATH}"

# Set up environment for ubuntu user in bashrc (for interactive shells)
RUN echo 'export PATH="/home/ubuntu/.cargo/bin:$PATH"' >> /home/ubuntu/.bashrc \
    && echo 'export PATH="/home/ubuntu/.local/bin:$PATH"' >> /home/ubuntu/.bashrc

# Verify installations as ubuntu user
RUN node --version \
    && npm --version \
    && pnpm --version \
    && python3 --version \
    && pip --version \
    && protoc --version \
    && maturin --version \
    && rustc --version \
    && cargo --version

# Set working directory
WORKDIR /workspace

# Copy and run verification script (switch to root temporarily for this)
USER root
COPY scripts/verify-environment.sh /tmp/verify-environment.sh
COPY .cursor/docker-helper.sh /usr/local/bin/docker-helper.sh
RUN chmod +x /tmp/verify-environment.sh \
    && chmod +x /usr/local/bin/docker-helper.sh \
    && /tmp/verify-environment.sh

# Switch back to ubuntu user as the default user for the environment
USER ubuntu
WORKDIR /workspace

# USAGE NOTES:
# 1. RECOMMENDED: Mount Docker socket from host
#    docker run -v /var/run/docker.sock:/var/run/docker.sock <image>
# 2. ALTERNATIVE: Use privileged mode for Docker-in-Docker  
#    docker run --privileged <image> 
#    then run: sudo /usr/local/bin/docker-helper.sh --start-daemon
# 3. Check Docker setup: /usr/local/bin/docker-helper.sh --status
# 4. The daemon.json is configured to use vfs storage driver by default for better compatibility
# 5. Docker helper script will try overlay2 first, then fall back to vfs if needed