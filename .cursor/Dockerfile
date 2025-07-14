FROM ubuntu:22.04

# Set non-interactive mode to prevent prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install system dependencies (protobuf-compiler will be installed separately with specific version)
RUN apt-get update && apt-get install -y --no-install-recommends \
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
    && rm -rf /var/lib/apt/lists/*

# Create ubuntu user early with proper sudoers permissions
RUN useradd -m -s /bin/bash ubuntu && \
    echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ubuntu && \
    chmod 0440 /etc/sudoers.d/ubuntu

# Install Docker
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/*

# Add setup for Docker-in-Docker (DinD) support - add both root and ubuntu to docker group
RUN groupadd -f docker \
    && usermod -aG docker root \
    && usermod -aG docker ubuntu

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install pnpm
RUN npm install -g pnpm@9.9.0

# Install Python 3.12 (Ubuntu 22.04 comes with Python 3.10, we need 3.12)
# Set timezone non-interactively to avoid hanging
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone \
    && apt-get update && apt-get install -y software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa -y \
    && apt-get update \
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

# Install maturin for Python-Rust bindings (with --break-system-packages for newer Python)
RUN pip install maturin==1.8.1 --break-system-packages

# Install Rust toolchain for ubuntu user (the user that actually uses the environment)
USER ubuntu
WORKDIR /home/ubuntu
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Switch back to root for remaining setup
USER root
WORKDIR /

# Set up PATH environment variables for consistent access in all contexts
ENV PATH="/home/ubuntu/.cargo/bin:/home/ubuntu/.local/bin:${PATH}"

# Verify installations
RUN node --version \
    && npm --version \
    && pnpm --version \
    && python3 --version \
    && pip --version \
    && protoc --version \
    && maturin --version

# Verify Rust installations as ubuntu user
USER ubuntu
RUN rustc --version && cargo --version
USER root

# Set working directory
WORKDIR /workspace

# Copy and run verification script
COPY scripts/verify-environment.sh /tmp/verify-environment.sh
RUN chmod +x /tmp/verify-environment.sh \
    && /tmp/verify-environment.sh

# Set up environment for ubuntu user in bashrc (for interactive shells)
USER ubuntu
RUN echo 'export PATH="/home/ubuntu/.cargo/bin:$PATH"' >> /home/ubuntu/.bashrc \
    && echo 'export PATH="/home/ubuntu/.local/bin:$PATH"' >> /home/ubuntu/.bashrc

# Switch back to root for final setup
USER root