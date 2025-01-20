---
layout: default
title: Install
nav-order: 1
---

# Install power options


## Arch Linux

There are 6 AUR packages for power-options.

- GTK: `power-options-gtk` (stable) and `power-options-gtk-git` (bleeding edge)
- Webview: `power-options-webview` (stable) and `power-options-webview-git` (bleeding edge)
- Just the daemon: `power-options-daemon` (stable) and `power-options-daemon-git` (bleeding edge)
- System Tray: `power-options-tray` (stable) and `power-options-tray-git` (bleeding edge)

## Fedora

The COPR GTK package is maintained by [@lpuv](https://github.com/lpuv)

```bash
sudo dnf copr enable leo/power-options 
sudo dnf install power-options
```

## Other distros / from source

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

- Webview: 

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

If you've installed using the AUR, your package manager should handle the
updates.

If you've installed using install scripts, simply pull the latest changes and
re-run the install scripts again and `./update.sh`. **Important, do not run
`./uninstall.sh`, `./setup.sh` or `power-daemon-mgr setup` if you
want to keep your profiles**
