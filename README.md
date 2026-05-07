# Blip

A swag chat server and client written in Rust.

## Development

### Prerequisites

- Rust toolchain installed
- Optional: `podman` or `docker` for container builds

### Build locally

From the repository root:

```bash
cargo build --package tcp-server --bin tcp-server
cargo build --package tcp-client --bin tcp-client
```

### Run locally

Start the server:

```bash
cargo run --package tcp-server --bin tcp-server -- -- localhost:6666
```

Then start one or more clients in other terminals:

```bash
cargo run --package tcp-client --bin tcp-client -- localhost:6666
```

### Container build with Podman

From the repository root:

```bash
podman build -f server/docker/Dockerfile . --target runtime -t blip-server
```

Run the container:

```bash
podman run --rm -p 6666:6666 blip-server
```

Or with compose:

```bash
podman compose -f compose.yml up
```

Then connect with the client as above.

### Tests

Run the workspace tests with:

```bash
cargo test --workspace --all-targets
```
