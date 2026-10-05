# Multi-stage Dockerfile for engram-server
# Build: docker build -t engram-server .
# Run:   docker run -v engram-data:/data -p 3100:3100 engram-server --transport http

# Pin builder to bookworm to match the runtime glibc (2.36).
# rust:latest drifts to newer base images and can link GLIBC_2.39+
# which crashes on debian:bookworm-slim at startup.
# Pinned by index digest (2026-10-05: rust:1-bookworm = Rust 1.99.0); bump digest deliberately.
FROM rust:1-bookworm@sha256:59037199c44290f2befcdd58dcc540164763fc296950255aaefeef096a1866b0 AS builder

WORKDIR /build
COPY . .

RUN cargo build --release --bin engram-server --bin engram-cli \
    && strip target/release/engram-server \
    && strip target/release/engram-cli

# Pinned by index digest (2026-10-05); bump together with the builder base.
FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251

# Install ca-certificates and curl for container health check
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Create dedicated non-root system user and secure data volume directory
RUN groupadd -g 10001 engram \
    && useradd -u 10001 -g engram -s /sbin/nologin -d /data engram \
    && mkdir -p /data \
    && chown -R engram:engram /data \
    && chmod 0700 /data

COPY --from=builder /build/target/release/engram-server /usr/local/bin/
COPY --from=builder /build/target/release/engram-cli /usr/local/bin/

ENV ENGRAM_DB_PATH=/data/memories.db \
    ENGRAM_HTTP_PORT=3100 \
    ENGRAM_HTTP_BIND_ADDRESS=127.0.0.1

USER engram:engram
WORKDIR /data
VOLUME /data
EXPOSE 3100

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD curl -f http://127.0.0.1:3100/health || exit 1

ENTRYPOINT ["engram-server"]
