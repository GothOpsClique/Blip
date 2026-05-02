# Multi-stage Dockerfile for building the tcp-server binary
FROM rust:stable AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY protocol/Cargo.toml protocol/Cargo.toml
COPY server/Cargo.toml server/Cargo.toml
COPY client/Cargo.toml client/Cargo.toml

RUN cargo fetch --locked

COPY . .
RUN cargo build --release --bin tcp-server --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/tcp-server /usr/local/bin/tcp-server
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/tcp-server"]
