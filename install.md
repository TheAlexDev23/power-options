---
layout: default
title: Install
nav_order: 1
---

# Install power options

## Recommended: DistroPack (GTK Frontend, Tray & Daemon)

For most users, we recommend installing via [DistroPack](https://distropack.dev), which provides packages for Debian/Ubuntu, Fedora/RHEL/Rocky, and Arch Linux with automatic handling of distribution-specific quirks.

**GTK Frontend:**
Visit [https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-gtk](https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-gtk) for installation instructions specific to your distribution.

**System Tray:**
Visit [https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-tray](https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-tray) for installation instructions specific to your distribution.

**Daemon:**
Visit [https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-daemon](https://distropack.dev/Install/Package/TheAlexDev23/power-options/power-options-daemon) for installation instructions specific to your distribution.

## Arch Linux (AUR)

For Arch Linux users, AUR packages are also available:

- GTK: `power-options-gtk` (stable) and `power-options-gtk-git` (bleeding edge)
- Webview: `power-options-webview` (stable) and `power-options-webview-git` (bleeding edge)
- System Tray: `power-options-tray` (stable) and `power-options-tray-git` (bleeding edge)
- Just the daemon: `power-options-daemon` (stable) and `power-options-daemon-git` (bleeding edge)

## Webview Frontend (Source or AUR only)

The webview frontend is currently not available on DistroPack and must be installed either from source or via AUR (for Arch Linux users).

**From AUR (Arch Linux only):**
```bash
yay -S power-options-webview  # or power-options-webview-git for bleeding edge
```

**From source:**

To build, requires dev libraries `libsoup-3.0-dev`, `libwebkit2gtk-4.1-dev` and
`libxdo-dev` (or the equivalent in your distro)
```bash
# dioxus-cli is required
cargo install dioxus-cli
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options/scripts
chmod +x *.sh
# Run as local user, will require sudo password
./install-webview.sh
# If installing for the first time
./setup.sh
# If updating
./update.sh
```

## Installing from source (Alternative method)

If you prefer to build from source or DistroPack doesn't support your distribution:

- GTK: 

To build, requires dev libraries `libgtk4-dev` `libadwaita-1-dev` (or the equivalent in
your distro)
```bash
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options/scripts
chmod +x *.sh
# Run as local user, will require sudo password
./install-gtk.sh
# If installing for the first time
./setup.sh
# If updating
./update.sh
```

- The system tray icon:

```bash
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options/scripts
chmod +x *.sh
# Run as local user, will require sudo password
./install-tray.sh
# If installing for the first time
./setup.sh
# If updating
./update.sh
```

- Just the daemon:

```bash
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options/scripts
chmod +x *.sh
# Run as local user, will require sudo password
./install-daemon.sh
# If installing for the first time
./setup.sh
# If updating
./update.sh
```

## Dependencies

For *build* dependendencies, refer to the installation guide above.

Mandatory:
- lspci
- lsusb
- acpi

Optional:
- iwlwifi compatible network card for network configuration
- Intel sound card for audio configuration
- Intel/AMD GPU for GPU configuration
- xrandr: resolution/refresh rate control
- brightnessctl: brightness control
- ifconfig: ethernet blocking
- xset: screen turn off timeout
- xautolock: system suspend timeout

Webview frontend:
- webkit2gtk
- dioxus-cli

GTK frontend:
- yad
- libadwaita

## Updating 

If you've installed using DistroPack, your package manager should handle the
updates automatically (e.g., `sudo apt update && sudo apt upgrade` for Debian/Ubuntu,
`sudo dnf update` for Fedora, or `sudo pacman -Syu` for Arch).

If you've installed using the AUR, your package manager should handle the
updates.

If you've installed using install scripts, simply pull the latest changes and
re-run the install scripts again and `./update.sh`. **Important, do not run
`./uninstall.sh`, `./setup.sh` or `power-daemon-mgr setup` if you
want to keep your profiles**
