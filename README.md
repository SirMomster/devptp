# devptp

> **Early development:** devptp is currently an alpha release. The IPC and
> networking APIs may change between versions.

`devptp` is a small peer-to-peer development networking utility. A Linux
machine can serve selected local TCP services to a remote workstation through
an iroh peer-to-peer connection. Windows clients and Linux aarch64 systems such
as Asahi Linux can connect to a Linux serving machine.

## Features

- Peer-to-peer connectivity using iroh.
- Linux-only serving and automatic local port discovery.
- Configurable allowed ports.
- Local JSON IPC for automation and integrations.
- Linux x86_64, Linux aarch64 (including Asahi Linux), and Windows x86_64 release builds.

## Quick start

### 1. Configure the serving machine

Optionally create `.devptp.toml` in the working directory:

```toml
shared_ports = [8083, 3000]
```

Only ports in `shared_ports` that are currently listening are advertised. If no
configuration file is present, the daemon starts with no shared ports. You can
also set `DEVPTP_CONFIG` to an explicit configuration file.

### 2. Start the daemon

```bash
devptp --daemon
```

### 3. Start serving (Linux)

In another terminal, ask the daemon for a connection ticket:

```bash
devptp start-serving
```

The response is JSON containing the ticket:

```json
{"id":1,"ok":true,"result":{"ticket":"..."}}
```

### 4. Connect from a client

On the remote machine, start a daemon and connect with the ticket:

```bash
devptp --daemon
devptp connect --ticket '<ticket>'
```

The client receives and exposes the serving machine's available, configured
ports. Use `status` to inspect the connection and forwarded ports.

## CLI commands

| Command | Description |
|---|---|
| `start-serving` | **Serving daemon (Linux):** advertise local services and return an endpoint ticket |
| `connect --ticket <ticket>` | **Connecting daemon:** connect to a serving peer and create local forwarding listeners |
| `ping` | Ping the connected peer |
| `status` | Return daemon state as JSON |
| `list-forwarded-ports` | List configured, detected ports as JSON |
| `expose <port>` | Request a port exposure |
| `disconnect [--port <port>]` | Stop one forwarded port or close the peer and clean up forwarding |
| `shutdown` | Gracefully stop the daemon |

The CLI prints the raw JSON IPC response, making it suitable for scripts.

## Interactive client

Launch the cross-platform Ratatui client with:

```bash
cargo run --bin devptp-tui
```

Paste a serving ticket on the initial screen. The client owns its in-process
daemon, shows forwarded and available ports, periodically measures RTT, and
tracks application/tunnel traffic over a rolling two-minute graph. Use `r` to
reconnect with the current ticket or `c`/`e` to enter a new one. Quitting while
connected prompts before shutting down the daemon.

## Library integration

The core runtime is available to Rust frontends through the library API. Start
only the runtime when the frontend runs in-process:

```rust,no_run
let daemon = devptp::Daemon::start().await?;
let status = daemon.status().await;
// Use typed operations and render `status` in the frontend.
daemon.shutdown().await;
```

A frontend can also attach the standard IPC server to the same daemon with
`daemon.start_ipc().await?`. The CLI's `--daemon` mode continues to use
`Daemon::run()`, which starts both together.

## IPC integration

The daemon uses a current-user-only local socket and newline-delimited JSON
requests. The complete protocol, response shapes, method parameters, and
examples are documented in [`docs/ipc-client.md`](docs/ipc-client.md).

## Installation

Install the latest release on Linux x86_64 or Linux aarch64 (including Asahi Linux) with:

```bash
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/SirMomster/devptp/main/scripts/install.sh | sh
```

The installer places `devptp` in `~/.local/bin`. Set `DEVPTP_INSTALL_DIR` to
install elsewhere. Windows users can install the latest release from PowerShell:

```powershell
irm https://raw.githubusercontent.com/SirMomster/devptp/main/scripts/install.ps1 | iex
```

The PowerShell installer places `devptp.exe` in `%LOCALAPPDATA%\devptp\bin`.
Set `$env:DEVPTP_INSTALL_DIR` to install elsewhere. You can also download the
release `.zip` directly from [GitHub Releases](https://github.com/SirMomster/devptp/releases).

## Building and testing

Run Cargo commands in the development container:

```bash
devcontainer up
devcontainer exec cargo build --release
devcontainer exec cargo test
```

For local cross-compilation from Linux, install Zig and `cargo-zigbuild`, then
run:

```bash
./scripts/build-targets.sh
```

See [`CROSS_COMPILATION.md`](CROSS_COMPILATION.md) for supported targets. GitHub
releases are generated automatically when a `v*` tag is pushed.

## Project status

The current release is `0.1.2`. The networking and IPC layers are implemented. The daemon is intentionally
small: Linux handles service discovery and serving, while Windows is a
connecting client. Additional forwarding policies can be added without
changing the JSON IPC transport.

## License

This project is licensed under the [MIT License](LICENSE).
