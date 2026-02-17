# Glucose Bar — Install & Uninstall

## Prerequisites

- GNOME Shell 45+
- Rust toolchain (`cargo`)
- A Dexcom CGM account

## Install

1. **Build the daemon**

   ```bash
   cd glucose-monitor
   cargo build --release
   cd ..
   ```

2. **Run the installer**

   ```bash
   chmod +x install.sh
   ./install.sh
   ```

   You'll be prompted for your Dexcom credentials on first install.

3. **Restart GNOME Shell**

   - **X11:** `Alt+F2` → type `r` → Enter
   - **Wayland:** Log out and log back in

The glucose reading and trend arrow icon should now appear in the top bar.

## Uninstall

```bash
chmod +x uninstall.sh
./uninstall.sh
```

This removes the binary, extension, systemd service, config, and cached data.
