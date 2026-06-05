#!/bin/sh
# Install script for picomd
# Usage: curl -fsSL https://raw.githubusercontent.com/jondot/picomd/main/install.sh | sh
#    or: ... | sh -s -- --version v0.1.0
set -e

REPO="jondot/picomd"
BINARY="picomd"
VERSION=""

while [ $# -gt 0 ]; do
  case "$1" in
    --version|-v) VERSION="$2"; shift 2 ;;
    *) shift ;;
  esac
done

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  darwin) OS="apple-darwin" ;;
  linux)  OS="unknown-linux-gnu" ;;
  *) echo "Error: unsupported OS: $OS (desktop targets are macOS and Linux x86_64)"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Error: unsupported arch: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

# Desktop targets only: macOS (x86_64 + aarch64) and Linux x86_64.
if [ "$TARGET" = "aarch64-unknown-linux-gnu" ]; then
  echo "Error: arm64 Linux is not a published target (desktop builds are macOS + Linux x86_64)."
  exit 1
fi

if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  [ -z "$VERSION" ] && { echo "Error: could not determine latest version"; exit 1; }
fi

URL="https://github.com/${REPO}/releases/download/${VERSION}/${BINARY}-${TARGET}.tar.gz"
echo "Installing ${BINARY} ${VERSION} for ${TARGET}..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -fsSL "$URL" -o "$TMPDIR/${BINARY}.tar.gz"
tar xzf "$TMPDIR/${BINARY}.tar.gz" -C "$TMPDIR"

INSTALL_DIR="/usr/local/bin"
[ ! -w "$INSTALL_DIR" ] 2>/dev/null && INSTALL_DIR="${HOME}/.local/bin" && mkdir -p "$INSTALL_DIR"

mv "$TMPDIR/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"

echo ""
echo "  Installed ${BINARY} ${VERSION} → ${INSTALL_DIR}/${BINARY}"
echo ""

case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "  Note: ${INSTALL_DIR} is not in PATH. Add:"
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
    echo ""
    ;;
esac
