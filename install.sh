#!/bin/bash

set -e

VERSION="0.1.0"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" = "darwin" ]; then
  if [ "$ARCH" = "x86_64" ]; then
    BINARY_URL="https://github.com/amioranza/sm2env/releases/download/v${VERSION}/sm2env-v${VERSION}-x86_64-apple-darwin.tar.gz"
  elif [ "$ARCH" = "arm64" ]; then
    echo "ARM64 macOS binary not available yet. Using x86_64 binary with Rosetta."
    BINARY_URL="https://github.com/amioranza/sm2env/releases/download/v${VERSION}/sm2env-v${VERSION}-x86_64-apple-darwin.tar.gz"
  else
    echo "Unsupported architecture: $ARCH"
    exit 1
  fi
elif [ "$OS" = "linux" ]; then
  if [ "$ARCH" = "x86_64" ]; then
    BINARY_URL="https://github.com/amioranza/sm2env/releases/download/v${VERSION}/sm2env-v${VERSION}-x86_64-linux.tar.gz"
  else
    echo "Unsupported architecture: $ARCH"
    exit 1
  fi
else
  echo "Unsupported operating system: $OS"
  exit 1
fi

# Create a temporary directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Download the binary
echo "Downloading sm2env v${VERSION}..."
curl -L "$BINARY_URL" -o "$TMP_DIR/sm2env.tar.gz"

# Extract the binary
echo "Extracting..."
tar -xzf "$TMP_DIR/sm2env.tar.gz" -C "$TMP_DIR"

# Install the binary
echo "Installing to /usr/local/bin/sm2env..."
sudo mv "$TMP_DIR/v${VERSION}/sm2env" /usr/local/bin/
sudo chmod +x /usr/local/bin/sm2env

echo "sm2env v${VERSION} has been installed to /usr/local/bin/sm2env"
echo "Run 'sm2env --help' to get started."
