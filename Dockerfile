FROM docker.io/library/rust:1.97-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked


FROM docker.io/library/debian:13-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    libssl3t64 \
    libzstd1 \
    libbrotli1 \
    zlib1g \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --system --gid 986 broderapi && \
    useradd --system --uid 986 --gid broderapi --create-home broderapi

COPY --from=builder --chown=broderapi:broderapi /app/target/release/broderapi /usr/local/bin/broderapi

USER broderapi

EXPOSE 9090

ENTRYPOINT ["broderapi"]
