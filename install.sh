#!/usr/bin/env bash
set -e

echo "Installing cava-wallpaper..."

INSTALL_DIR="$HOME/.local/bin"
REPO_URL="https://github.com/rs-pro0/wallpaper-cava.git"
BUILD_DIR="$HOME/cava-wallpaper-src"

# Clone or update repository
if [ -d "$BUILD_DIR" ]; then
  echo "Updating existing repository..."
  cd "$BUILD_DIR"
  git pull
else
  echo "Cloning repository..."
  git clone "$REPO_URL" "$BUILD_DIR"
fi

# Build the project
cd "$BUILD_DIR"
echo "Building project..."
cargo build --release

# Install the binary
mkdir -p "$INSTALL_DIR"
cp target/release/wallpaper-cava "$INSTALL_DIR/cava-wallpaper"

# Ensure ~/.local/bin is in PATH
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
  echo "~/.local/bin added to PATH in .bashrc"
fi

echo "Installation complete."
echo ""
echo "You can now run it with:"
echo "    cava-wallpaper"

