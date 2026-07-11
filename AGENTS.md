# devptp — Peer-to-Peer Networking Toolkit

This is a Rust workspace for **devptp**, a peer-to-peer networking toolkit built on [iroh](https://github.com/n0-computer/iroh). It provides endpoints, ping, and ticket functionality for P2P communication.

## Workspace Structure

```
devptp/
├── rdevptp/       # Rust library — iroh endpoint/ping/ticket abstractions
├── cdevptp/       # Rust library — placeholder (currently empty)
├── cli/           # Binary — "devptp" CLI tool (receiver/sender)
├── Cargo.toml     # Workspace root
└── Cargo.lock
```

## Building & Running

> **Important:** All cargo commands must be run inside the devcontainer using `devcontainer-cli`.

```bash
devcontainer up                 # Start the devcontainer
devcontainer exec cargo build --workspace   # Build everything
devcontainer exec cargo build -p devptp     # Build only the CLI binary
devcontainer exec cargo test --workspace    # Run all tests
devcontainer exec cargo run -p devptp receiver      # Run as receiver (prints ticket to stdout)
devcontainer exec cargo run -p devptp sender <ticket>  # Ping a receiver using its ticket
```

The devcontainer is configured in `.devcontainer/` with a Cargo cache volume mounted at `/usr/local/cargo`. The start script (`start-dev-container.sh`) provides a convenience wrapper for opening the container and nvim together.

## Crate Details

### `rdevptp` — Core Library

- Wraps iroh's `Endpoint`, `Router`, `Ping`, and `EndpointTicket`
- Provides `run_receiver()` which binds an endpoint, starts a ping router, and returns a ticket string + endpoint handle
- Defines `ReceiverError` / `ReceiverErrorKind` for error handling
- Uses iroh presets (`presets::N0`) for network configuration

### `cdevptp` — Placeholder Library

- Currently only has a stub `add()` function
- Intended for future C-compatible FFI bindings or additional library code

### `cli` — CLI Tool

- Binary name: `devptp`
- Two roles:
  - `receiver` — binds endpoint, prints ticket, waits for Ctrl+C
  - `sender <ticket>` — resolves ticket, pings the receiver, prints RTT
- Uses tokio runtime, tracing-subscriber for logging

## Coding Guidelines

- **Rust edition**: 2024 (as specified in Cargo.toml)
- **Error handling**: Use `anyhow::Result` in the CLI, custom error types in libraries
- **Async**: All async code uses `tokio`; prefer `#[tokio::main]` for binaries
- **Logging**: Use `tracing` in the CLI; no logging in library crates
- **Naming**: Keep existing naming conventions — `rdevptp` = Rust devptp, `cdevptp` = C devptp, `devptp` = CLI
- **Dependencies**: Only add dependencies that are already used elsewhere in the workspace or are essential
- **No MCP**, no sub-agents, no plan mode — keep it simple
- **Keep the workspace minimal** — this is a focused P2P utility, not a general-purpose framework

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `iroh` | P2P networking core (endpoint, router, presets) |
| `iroh-tickets` | Endpoint address tickets for peer discovery |
| `iroh-ping` | RTT measurement between peers |
| `futures` | Async futures utilities (in rdevptp) |
| `tokio` | Async runtime (in cli) |
| `anyhow` | Error handling (in cli) |
| `tracing-subscriber` | Logging (in cli) |
