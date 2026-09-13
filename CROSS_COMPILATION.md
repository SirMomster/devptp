# Cross compilation

The daemon can be built from Linux for:

- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-gnu`
- `aarch64-apple-darwin`

Install [Zig](https://ziglang.org/) and `cargo-zigbuild`, then run:

```bash
cargo install cargo-zigbuild
./scripts/build-targets.sh
```

Binaries are produced below `target/<target>/release/`.

`start_serving` and Linux `/proc` port detection are compiled only for Linux.
Windows and macOS builds remain connecting clients and return a structured IPC
error if `start_serving` is requested.
