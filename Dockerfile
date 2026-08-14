# Multi-stage Dockerfile for engram-server
# Build: docker build -t engram-server .
# Run:   docker run -v engram-data:/data -p 3100:3100 engram-server --transport http

# Pin builder to bookworm to match the runtime glibc (2.36).
# rust:latest drifts to newer base images and can link GLIBC_2.39+
# which crashes on debian:bookworm-slim at startup.
FROM rust:1-bookworm AS builder

WORKDIR /build
COPY . .

RUN cargo build --release --bin engram-server --bin engram-cli \
    && strip target/release/engram-server \
    && strip target/release/engram-cli

FROM debian:bookworm-slim

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
