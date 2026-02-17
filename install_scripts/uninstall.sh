#!/bin/bash
set -e

# Define paths using XDG Base Directory specification
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"
BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"

PROJECT_CONFIG_DIR="$CONFIG_HOME/glucose-monitor"
PROJECT_CACHE_DIR="$CACHE_HOME/glucose-monitor"
EXTENSION_DIR="$DATA_HOME/gnome-shell/extensions/glucose-bar@pawel.com"
SYSTEMD_USER_DIR="$CONFIG_HOME/systemd/user"
BINARY_PATH="$BIN_DIR/glucose-monitor"
SERVICE_FILE="$SYSTEMD_USER_DIR/glucose-bar.service"

echo "Uninstalling Glucose Bar..."

# 1. Stop and disable the background service
# Using || true to ignore errors if service is already stopped/disabled/removed
systemctl --user stop glucose-bar.service || true
systemctl --user disable glucose-bar.service || true

# 2. Disable the GNOME extension
gnome-extensions disable glucose-bar@pawel.com || true

# 3. Remove installed files
echo "Removing binary from $BINARY_PATH"
rm -f "$BINARY_PATH"

echo "Removing service file from $SERVICE_FILE"
rm -f "$SERVICE_FILE"

echo "Removing extension from $EXTENSION_DIR"
rm -rf "$EXTENSION_DIR"

# 4. Remove config and cached data
echo "Removing config from $PROJECT_CONFIG_DIR"
rm -rf "$PROJECT_CONFIG_DIR"

echo "Removing cache from $PROJECT_CACHE_DIR"
rm -rf "$PROJECT_CACHE_DIR"

# 5. Clean up runtime data (if present)
if [ -n "$XDG_RUNTIME_DIR" ]; then
    rm -rf "${XDG_RUNTIME_DIR}/glucose-monitor/"
fi

# 6. Tell systemd to forget the service
systemctl --user daemon-reload

# 7. Restart GNOME Shell to fully unload the extension
echo "Uninstallation complete."
echo "Please restart GNOME Shell to fully unload the extension:"
echo "  On X11:  Alt+F2 -> type 'r' -> Enter"
echo "  On Wayland: Log out and log back in"
