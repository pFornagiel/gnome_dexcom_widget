#!/bin/bash
set -e

# Define paths using XDG Base Directory specification
# Fallbacks:
# XDG_CONFIG_HOME -> $HOME/.config
# XDG_DATA_HOME   -> $HOME/.local/share
# Link: https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}" # Not standard XDG but common convention

PROJECT_CONFIG_DIR="$CONFIG_HOME/glucose-monitor"
EXTENSION_DIR="$DATA_HOME/gnome-shell/extensions/glucose-bar@pawel.com"
SYSTEMD_USER_DIR="$CONFIG_HOME/systemd/user"

echo "Installing Glucose Bar..."
echo "Configuration directory: $PROJECT_CONFIG_DIR"
echo "Extension directory: $EXTENSION_DIR"
echo "Binary directory: $BIN_DIR"

# Create directories
mkdir -p "$PROJECT_CONFIG_DIR"
mkdir -p "$BIN_DIR"
mkdir -p "$EXTENSION_DIR"
mkdir -p "$SYSTEMD_USER_DIR"

# Copy Rust binary
echo "Copying binary..."
# Assuming the script is run from project root, but let's be safe and find where we are
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if [ -f "$PROJECT_ROOT/target/release/glucose-monitor" ]; then
    cp "$PROJECT_ROOT/target/release/glucose-monitor" "$BIN_DIR/"
else
    echo "Error: Binary not found at $PROJECT_ROOT/target/release/glucose-monitor"
    echo "Please build the project first: cargo build --release"
    exit 1
fi

# Copy extension
echo "Copying extension..."
cp "$PROJECT_ROOT/extension/metadata.json" "$EXTENSION_DIR/"
cp "$PROJECT_ROOT/extension/extension.js" "$EXTENSION_DIR/"
cp -r "$PROJECT_ROOT/extension/svg" "$EXTENSION_DIR/"

# Install service
echo "Installing service..."
SERVICE_FILE="$PROJECT_ROOT/glucose-bar.service"
TARGET_SERVICE_FILE="$SYSTEMD_USER_DIR/glucose-bar.service"

cp "$SERVICE_FILE" "$TARGET_SERVICE_FILE"

# Dynamically update the EnvironmentFile path in the service file
# We escape the path for sed
ESCAPED_CONFIG_FILE=$(echo "$PROJECT_CONFIG_DIR/config" | sed 's/\//\\\//g')
sed -i "s/EnvironmentFile=.*/EnvironmentFile=$ESCAPED_CONFIG_FILE/" "$TARGET_SERVICE_FILE"

# Also update ExecStart to point to the correct binary location
ESCAPED_BIN_PATH=$(echo "$BIN_DIR/glucose-monitor" | sed 's/\//\\\//g')
sed -i "s|ExecStart=.*|ExecStart=$BIN_DIR/glucose-monitor|" "$TARGET_SERVICE_FILE"

systemctl --user daemon-reload

# Prompt for credentials if not set
if [ ! -f "$PROJECT_CONFIG_DIR/config" ]; then
    echo "Configure Dexcom credentials:"
    read -p "Dexcom Username: " username
    read -s -p "Dexcom Password: " password
    echo
    read -p "US Account? (y/n): " is_us
    
    ous="true"
    if [ "$is_us" == "y" ]; then
        ous="false"
    fi

    echo "DEXCOM_USERNAME=$username" > "$PROJECT_CONFIG_DIR/config"
    echo "DEXCOM_PASSWORD=$password" >> "$PROJECT_CONFIG_DIR/config"
    echo "DEXCOM_OUS=$ous" >> "$PROJECT_CONFIG_DIR/config"
fi

# Enable service
systemctl --user enable --now glucose-bar.service

# Enable extension
echo "Enabling extension..."
gnome-extensions enable glucose-bar@pawel.com

echo "Installation complete!"
echo "Please restart GNOME Shell (Alt+F2 -> r) to see the extension."
