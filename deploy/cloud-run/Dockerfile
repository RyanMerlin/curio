FROM rust:1.90-bookworm AS builder

WORKDIR /workspace

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . /workspace

WORKDIR /workspace/curio-rs
RUN cargo build --release --locked --bin curio --bin curio-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    openssh-client \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root runtime user (UID 1001, GID 0 for OpenShift/Cloud Run compatibility)
RUN useradd -r -u 1001 -g root curio

# Copy only the compiled binaries — no source, no secrets, no repo content.
COPY --from=builder /workspace/curio-rs/target/release/curio /usr/local/bin/curio
COPY --from=builder /workspace/curio-rs/target/release/curio-service /usr/local/bin/curio-service

ENV CURIO_BINARY=/usr/local/bin/curio \
    CURIO_SERVICE_PROVIDER_BACKEND=gemini \
    CURIO_SERVICE_BIND_ADDR=0.0.0.0:8080 \
    CURIO_SERVICE_READ_ONLY=false

# Ensure writable temp dir for git credential files and worktree cache
RUN mkdir -p /tmp/curio && chown 1001:0 /tmp/curio

USER 1001

EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/curio-service"]
