FROM base-image:latest

RUN mkdir -p /home/vscode/.cargo

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# The base image already provides Rust. Updating rustup here can fail on
# mounted/overlay filesystems while replacing rust-analyzer components.
RUN cargo binstall --no-confirm --locked tree-sitter-cli

WORKDIR /home/vscode/
