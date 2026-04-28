#!/usr/bin/env bash
set -euo pipefail

BIN_NAME="envx"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

cargo build --release

mkdir -p "$INSTALL_DIR"
install_path="$INSTALL_DIR/$BIN_NAME"
cp "$TARGET_DIR/release/$BIN_NAME" "$install_path"
chmod +x "$install_path"

echo "Installed $BIN_NAME to $install_path"
