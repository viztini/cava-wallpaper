#!/usr/bin/env bash
set -e

echo "Installing cava-wallpaper..."

# Directories and repo
INSTALL_DIR="$HOME/.local/bin"
REPO_URL="https://github.com/rs-pro0/wallpaper-cava.git"
BUILD_DIR="$HOME/cava-wallpaper-src"

# Clone or update the wallpaper-cava repo
if [ -d "$BUILD_DIR" ]; then
  echo "Updating existing repository at $BUILD_DIR..."
  cd "$BUILD_DIR"
  git pull
else
  echo "Cloning repository..."
  git clone "$REPO_URL" "$BUILD_DIR"
fi

# Build the project in release mode
cd "$BUILD_DIR"
echo "Building project..."
cargo build --release

# Ensure ~/.local/bin exists
mkdir -p "$INSTALL_DIR"

# Create a wrapper script that runs from the correct directory
echo "Creating wrapper script in $INSTALL_DIR..."
cat > "$INSTALL_DIR/cava-wallpaper" <<'EOF'
#!/usr/bin/env bash
# Wrapper for wallpaper-cava to ensure it runs from its config directory

SRC_DIR="$HOME/cava-wallpaper-src"

if [ ! -d "$SRC_DIR" ]; then
  echo "Error: Source directory not found at $SRC_DIR"
  echo "Please reinstall using the install.sh script."
  exit 1
fi

cd "$SRC_DIR" || exit 1
exec ./target/release/wallpaper-cava "$@"
EOF

chmod +x "$INSTALL_DIR/cava-wallpaper"

# Ensure ~/.local/bin is in PATH
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.bashrc"
  echo "~/.local/bin has been added to your PATH in ~/.bashrc"
fi

echo ""
echo "Installation complete!"
echo "You can now run the visualizer with:"
echo "    cava-wallpaper"
echo ""
echo "To update later, simply re-run this script."
