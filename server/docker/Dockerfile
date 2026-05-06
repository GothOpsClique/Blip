# Multi-stage Dockerfile for building the tcp-server binary
FROM rust:slim AS builder

WORKDIR /usr/src/app

# Copy Cargo workspace manifests and source folders from repository root
COPY Cargo.toml Cargo.lock ./
COPY protocol ./protocol
COPY server ./server
COPY client ./client

# Fetch dependencies and build the tcp-server binary
RUN cargo fetch --locked
RUN cargo build --release --bin tcp-server --locked

FROM rust:slim AS runtime

COPY --from=builder /usr/src/app/target/release/tcp-server /usr/local/bin/tcp-server
EXPOSE 6666
ENTRYPOINT ["/usr/local/bin/tcp-server"]