#!/usr/bin/env sh
set -euo pipefail

REPO="loknopf/Tickr"
VERSION="latest"
BIN_DIR=""

usage() {
    cat <<'EOF'
Usage: install.sh [-r owner/repo] [-v version] [-b bin_dir]

Options:
  -r, --repo      GitHub repo (default: loknopf/Tickr)
  -v, --version   Version (default: latest)
  -b, --bin-dir   Install directory (default: ~/.local/bin or /usr/local/bin)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        -r|--repo)
            REPO="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -b|--bin-dir)
            BIN_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
done

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    Darwin)
        TARGET_OS="apple-darwin"
        ;;
    *)
        echo "Unsupported OS: $OS" >&2
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    *)
        echo "Unsupported CPU architecture: $ARCH" >&2
        exit 1
        ;;
esac

TARGET="$TARGET_ARCH-$TARGET_OS"
EXT="tar.gz"

if [ -z "$BIN_DIR" ]; then
    if [ "$(id -u)" -eq 0 ]; then
        BIN_DIR="/usr/local/bin"
    else
        BIN_DIR="$HOME/.local/bin"
    fi
fi

mkdir -p "$BIN_DIR"

if [ "$VERSION" = "latest" ]; then
    API_URL="https://api.github.com/repos/$REPO/releases/latest"
else
    VERSION="${VERSION#v}"
    API_URL="https://api.github.com/repos/$REPO/releases/tags/v$VERSION"
fi

JSON="$(curl -fsSL "$API_URL")"
TAG="$(printf "%s" "$JSON" | grep -m1 '"tag_name"' | cut -d '"' -f4)"
if [ -z "$TAG" ]; then
    echo "Unable to resolve release tag." >&2
    exit 1
fi

VERSION="${TAG#v}"
ASSET="tickr-$VERSION-$TARGET.$EXT"
URL="$(printf "%s" "$JSON" | grep -m1 "browser_download_url.*$ASSET" | cut -d '"' -f4)"

if [ -z "$URL" ]; then
    echo "Asset not found: $ASSET" >&2
    exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

ARCHIVE="$TMP_DIR/$ASSET"

curl -fsSL "$URL" -o "$ARCHIVE"

( cd "$TMP_DIR" && tar -xzf "$ARCHIVE" )

if [ ! -f "$TMP_DIR/tickr" ]; then
    echo "tickr binary not found in archive." >&2
    exit 1
fi

if command -v install >/dev/null 2>&1; then
    install -m 0755 "$TMP_DIR/tickr" "$BIN_DIR/tickr"
else
    chmod +x "$TMP_DIR/tickr"
    mv "$TMP_DIR/tickr" "$BIN_DIR/tickr"
fi

echo "Installed tickr to $BIN_DIR/tickr"
