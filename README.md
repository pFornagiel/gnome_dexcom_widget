# Dexcom Glucose Widget for GNOME Shell

A GNOME Shell widget extension that displays real-time Dexcom glucose readings in your top bar. Uses a Rust backend service to fetch data using [dexrs](https://github.com/makors/dexrs).

The widget displays glucose levels and trend arrows fetched from Dexcom Share API, with added delta from the last recorded reading. If there were no readings in 10 minutes, the displayed value is marked as _Stale_.

The extension was vibe-coded in one evening as a simple and small personal utility that does not rely on additional integrations like Nightscout. The installation shell scripts are also provided with a small `README` explaining how to set up the extension.

If anyone finds this useful, I am glad that claude brought some value to the society.

<p align="center">
  <img width="50%"  alt="image" src="https://github.com/user-attachments/assets/ff46b681-8007-46b7-a227-14640f1fede6" />
</p>

## Prerequisites

Before installing, ensure you have the following requirements:

- **GNOME Shell**: Version 45 or later.
- **Rust Toolchain**: Required to build the backend service.
- **System Dependencies**:
  - `pkg-config`
  - `openssl` (devel/headers)
  - `git`

## Installation

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/pawel/dextop_linux.git
    cd dextop_linux
    ```

2.  **Build the backend**:
    ```bash
    cd glucose-monitor
    cargo build --release
    cd ..
    ```

3.  **Run the install script**:
    ```bash
    chmod +x install_scripts/install.sh
    ./install_scripts/install.sh
    ```

    The script will:
    - Install the `glucose-monitor` binary to `~/.local/bin` (or `$XDG_BIN_HOME`).
    - Prompt you for your Dexcom credentials.
    - Create a systemd user service `glucose-bar.service`.
    - Install and enable the GNOME extension.

4.  **Restart GNOME Shell**:
    - **X11**: Press `Alt+F2`, type `r`, and hit Enter.
    - **Wayland**: Log out and log back in.

## Configuration

Your credentials are stored in:
- `~/.config/glucose-monitor/config` (or `$XDG_CONFIG_HOME/glucose-monitor/config`)

You can edit this file manually if needed:

```ini
DEXCOM_USERNAME=your_username
DEXCOM_PASSWORD=your_password
DEXCOM_OUS=boolean # true for non-US accounts, false for US
```

## Uninstallation

To remove the extension and backend service entirely:

```bash
chmod +x install_scripts/uninstall.sh
./install_scripts/uninstall.sh
```

## Troubleshooting

### Installation fails
Make sure `~/.local/bin` is in your `$PATH`.

### No data appearing
Check the service status logs:

```bash
systemctl --user status glucose-bar
journalctl --user -u glucose-bar -f
```

## License

[MIT](LICENSE)

## Disclaimer:

>*I am utilizing the Dexcom Share API solely for personal use and do not have any affiliation with Dexcom, Inc. This application is not endorsed, sponsored, or managed by Dexcom, and I do not take any responsibility for the accuracy or performance of the data provided through the API. Furthermore, I do not profit from, or claim ownership of, any Dexcom-related content or services. All trademarks, logos, and brand names belong to their respective owners.*
