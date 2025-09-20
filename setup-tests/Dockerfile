FROM ubuntu:24.04

# Install Ruby and other dependencies needed for agent-workflow
RUN apt-get update && apt-get install -y \
    ruby \
    ruby-dev \
    bash \
    curl \
    gnupg \
    xz-utils \
    sudo \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for testing
RUN useradd -m -s /bin/bash testuser

# Configure passwordless sudo for the testuser
RUN echo 'testuser ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers

# Create the user-project directory structure
RUN mkdir -p /workspace/user-project

# Copy the agent-workflow repository to mimic real codex environment
COPY . /workspace/user-project/agents-workflow/

# Create install markers directory with proper permissions
RUN mkdir -p /workspace/user-project/agents-workflow/.install-markers && \
    chmod 755 /workspace/user-project/agents-workflow/.install-markers && \
    mkdir -p /tmp/post-install-scripts && \
    chmod 755 /tmp/post-install-scripts && \
    chown -R testuser:testuser /workspace/user-project /tmp/post-install-scripts

# Switch to the test user
USER testuser
WORKDIR /home/testuser

# Make scripts executable
RUN find /workspace/user-project/agents-workflow -name "*.sh" -exec chmod +x {} \; && \
    chmod +x /workspace/user-project/agents-workflow/codex-setup /workspace/user-project/agents-workflow/common-* 2>/dev/null || true

# Set the working directory to the agents-workflow directory within user-project
WORKDIR /workspace/user-project/agents-workflow

# Copy the test script and make it executable
COPY --chmod=755 setup-tests/container-test.sh /container-test.sh
