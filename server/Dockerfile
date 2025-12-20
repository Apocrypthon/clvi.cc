FROM rust:1.77 as builder
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY server/Cargo.toml server/
COPY server/src server/src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release -p server

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/server /usr/local/bin/server

ENV RUST_LOG=info
ENV SERVER_PORT=3000

EXPOSE 3000

CMD ["server"]
