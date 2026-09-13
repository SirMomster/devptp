#!/usr/bin/env sh
set -eu

REPO="${DEVPTP_REPO:-SirMomster/devptp}"
INSTALL_DIR="${DEVPTP_INSTALL_DIR:-$HOME/.local/bin}"
OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}:${ARCH}" in
  Linux:x86_64|Linux:amd64)
    ASSET="devptp-linux-x86_64.tar.gz"
    ;;
  Linux:aarch64|Linux:arm64)
    ASSET="devptp-linux-aarch64.tar.gz"
    ;;
  *)
    echo "Unsupported platform: ${OS}/${ARCH}" >&2
    echo "Supported platforms: Linux x86_64 and Linux aarch64 (including Asahi Linux)" >&2
    exit 1
    ;;
esac

command -v curl >/dev/null 2>&1 || {
  echo "curl is required" >&2
  exit 1
}
command -v tar >/dev/null 2>&1 || {
  echo "tar is required" >&2
  exit 1
}

URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

printf 'Downloading devptp for %s/%s...\n' "$OS" "$ARCH"
curl --fail --location --silent --show-error "$URL" -o "$TMP_DIR/$ASSET"
mkdir -p "$INSTALL_DIR"
tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"
install -m 0755 "$TMP_DIR/devptp" "$INSTALL_DIR/devptp"

printf 'Installed devptp to %s/devptp\n' "$INSTALL_DIR"
case ":${PATH}:" in
  *:"$INSTALL_DIR":*) ;;
  *) printf 'Add %s to PATH if needed.\n' "$INSTALL_DIR" ;;
esac
