# devptp

> **Early development:** devptp is currently an alpha release. The IPC and
> networking APIs may change between versions.

`devptp` is a small peer-to-peer development networking utility. A Linux
machine can serve selected local TCP services to a remote workstation through
an iroh peer-to-peer connection. Windows and macOS clients can connect to a
Linux serving machine without performing local port discovery.

## Features

- Peer-to-peer connectivity using iroh.
- Linux-only serving and automatic local port discovery.
- Configurable allowed ports.
- Local JSON IPC for automation and integrations.
- Linux x86_64, Windows x86_64, and macOS Apple Silicon release builds.

## Quick start

### 1. Configure the serving machine

Create `.devptp.toml` in the working directory:

```toml
shared_ports = [8083, 3000]
```

Only ports in `shared_ports` that are currently listening are advertised.

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
| `start-serving` | Start Linux serving and return an endpoint ticket |
| `connect --ticket <ticket>` | Connect to a serving peer |
| `ping` | Ping the connected peer |
| `status` | Return daemon state as JSON |
| `list-forwarded-ports` | List configured, detected ports as JSON |
| `expose <port>` | Request a port exposure |
| `disconnect` | Close the peer and clean up forwarding |
| `shutdown` | Gracefully stop the daemon |

The CLI prints the raw JSON IPC response, making it suitable for scripts.

## IPC integration

The daemon uses a current-user-only local socket and newline-delimited JSON
requests. The complete protocol, response shapes, method parameters, and
examples are documented in [`docs/ipc-client.md`](docs/ipc-client.md).

## Installation

Install the latest release on Linux x86_64 or macOS Apple Silicon with:

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

The current release is `0.1.0-alpha.1`. The networking and IPC layers are implemented. The daemon is intentionally
small: Linux handles service discovery and serving, while Windows and macOS
are connecting clients. Additional forwarding policies can be added without
changing the JSON IPC transport.

## License

This project is licensed under the [MIT License](LICENSE).
