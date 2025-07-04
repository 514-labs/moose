FROM ubuntu:22.04

# Install system dependencies
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

# Install Docker
RUN curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get update \
    && apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin \
    && rm -rf /var/lib/apt/lists/*

# Add setup for Docker-in-Docker (DinD) support
RUN groupadd -f docker \
    && usermod -aG docker root

# Install Node.js 20.x
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs

# Install pnpm
RUN npm install -g pnpm@9.9.0

# Install Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && echo 'source $HOME/.cargo/env' >> ~/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Python 3.12 (Ubuntu 22.04 comes with Python 3.10, we need 3.12)
RUN apt-get update && apt-get install -y software-properties-common \
    && add-apt-repository ppa:deadsnakes/ppa -y \
    && apt-get update \
    && apt-get install -y python3.12 python3.12-dev python3.12-venv python3.12-distutils \
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

# Install maturin for Python-Rust bindings
RUN pip install maturin==1.8.1

# Verify installations
RUN node --version \
    && npm --version \
    && pnpm --version \
    && rustc --version \
    && cargo --version \
    && python3 --version \
    && pip --version \
    && protoc --version

# Set working directory
WORKDIR /workspace

# Copy and run verification script
COPY scripts/verify-environment.sh /tmp/verify-environment.sh
RUN chmod +x /tmp/verify-environment.sh \
    && /tmp/verify-environment.sh

# Set up environment
RUN echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc \
    && echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc