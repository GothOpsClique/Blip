# Multi-stage Dockerfile for building the tcp-server binary
FROM rust:latest AS builder

WORKDIR /usr/src/app

# Copy workspace files from parent directory
COPY ../Cargo.toml ../Cargo.lock ./
COPY ../protocol/Cargo.toml ./protocol/Cargo.toml
COPY ../server/Cargo.toml ./server/Cargo.toml
COPY ../client/Cargo.toml ./client/Cargo.toml

# Copy source code
COPY ../protocol ./protocol
COPY ../server ./server
COPY ../client ./client

# Now fetch dependencies and build
RUN cargo fetch --locked
RUN cargo build --release --bin tcp-server --locked

FROM rust:latest AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/tcp-server /usr/local/bin/tcp-server
EXPOSE 6666
ENTRYPOINT ["/usr/local/bin/tcp-server"]

RUN cargo fetch --locked

COPY . .
RUN cargo build --release --bin tcp-server --locked

FROM rust:latest AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/tcp-server /usr/local/bin/tcp-server
EXPOSE 6666
ENTRYPOINT ["/usr/local/bin/tcp-server"]