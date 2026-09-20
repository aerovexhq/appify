#!/usr/bin/env bash
set -e

REPO="larvance/tauri-appify"
INSTALL_DIR="${HOME}/.local/bin"

# 1. Detect Operating System
OS="$(uname -s)"
case "$OS" in
  Linux*)   OS_TYPE="linux" ;;
  Darwin*)  OS_TYPE="darwin" ;;
  *)        echo "[!] Unsupported operating system: $OS"; exit 1 ;;
esac

# 2. Detect CPU Architecture
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64|amd64) ARCH_TYPE="x64" ;;
  arm64|aarch64) ARCH_TYPE="aarch64" ;;
  *)            echo "[!] Unsupported architecture: $ARCH"; exit 1 ;;
esac

ASSET_NAME="appify_${OS_TYPE}_${ARCH_TYPE}"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"

echo "==> Installing Appify for ${OS_TYPE}-${ARCH_TYPE}..."
mkdir -p "${INSTALL_DIR}"

TMP_FILE="$(mktemp "${INSTALL_DIR}/appify.tmp.XXXXXX")"
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_FILE"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_FILE" "$DOWNLOAD_URL"
else
  echo "[!] Error: curl or wget is required to download Appify."
  rm -f "$TMP_FILE"
  exit 1
fi

chmod +x "$TMP_FILE"
mv "$TMP_FILE" "${INSTALL_DIR}/appify"

echo "==> Successfully installed Appify to ${INSTALL_DIR}/appify!"

# Check if ~/.local/bin is on PATH
case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo ""
    echo "[!] Note: ${INSTALL_DIR} is not currently in your PATH."
    echo "    To run 'appify' from any terminal, add it by running:"
    echo "      echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
    echo "      source ~/.bashrc"
    echo ""
    ;;
esac

echo "Run 'appify --help' to get started."
