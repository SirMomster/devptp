#!/usr/bin/env bash
set -euo pipefail

# Cross-build from Linux. Install Zig and cargo-zigbuild first:
#   cargo install cargo-zigbuild
#
# The resulting binaries are written to target/<triple>/release/devptp.
targets=(
  x86_64-unknown-linux-gnu
  x86_64-pc-windows-gnu
  aarch64-apple-darwin
)

rustup target add "${targets[@]}"

for target in "${targets[@]}"; do
  echo "Building ${target}"
  cargo zigbuild --release --target "${target}"
done
