<div align="center">
    <img src="icon.png" width=120>
    <h1>
        Power Options
    </h1>
</div>

All-In-One blazingly fast Linux GUI Application for simple and advanced power
management on any device.

Power Options can be a drop in replacement for any power-saving application,
including TLP, auto-cpufreq, cpupower-gui, etc. Power Options covers all of the
most common power saving settings and adds additional ones not present in any other application.

Upon install, Power Options will analyze the system and intelligently generate a wide range
of profiles based on the findings. These profile are greatly optimized and shouldn't
require intervention, unless the user wants more control.

Most applications only allow two profiles: Battery Profile and AC profile. This
is suboptimal for ocasions where one might want greater control. This is what
led to the creation of the profile system that this program uses:
- The user can have as many profiles as they please
- The user chooses which profiles will be selected for Battery and AC.
- The user can temporarily override the profile selection to another one until
  they remove that override.
- The user may set a persistent override that will be kept across reboots.

## Available Frontends/Interfaces

One can simply install the daemon and edit the configuration files manually
as those are written in TOML. But the biggest strength of this program
are the GUI interfaces it provides, as almost any other power saving tool does not
have one and resorts to configuration files.

### Native GTK frontend

Simple, lightweight and native with a simple interface. Recommended for most
users.

![gtk-slideshow](static/gtk-slideshow.gif)

### WebKit frontend

More advanced options and greater control. Not as lightweight. Recommended for
advanced users and users who are looking to use power-options in tandem with
another power management solution.

![webview-slideshow](static/webview-slideshow.gif)

## Features

Power options was made based on a recollection of all the tips and
recommendations from the biggest linux wikis and guides as well as other power
saving applications. Some examples include:
- https://wiki.archlinux.org/title/Power_management
- https://wiki.gentoo.org/wiki/Power_management/Guide
- https://en.wikipedia.org/wiki/Power_management
- https://github.com/supplantr/ftw
- https://github.com/linrunner/TLP
- https://github.com/AdnanHodzic/auto-cpufreq
- https://github.com/vagnum08/cpupower-gui

Power Option includes the following features:
- More profile types than alternatives
- Can smartly generate profiles by analyzing the user's system.
- CPU Options
- Individual CPU Core Options. Most power saving tools lack this option and was
  one of the main motivations for this project.
- Screen Options
- Options for disabling radio components (e.g Bluetooth, WiFi, NFC)
- Network Options. Allows WAY greater control than alternative applications, but
  does require network driver reload. Limited to Intel network cards that use iwlwifi.
- ASPM Options
- PCI Options
- USB Options
- SATA Options
- Kernel Options

## Installation


- GTK: 

AUR: `power-options-gtk` or `power-options-gtk-git` for bleeding edge

```bash
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options
chmod +x ./install-gtk.sh
# Run as local user, will require sudo password
./install-gtk.sh
```

- Webview: 

AUR: `power-options-webview` or `power-options-webview-git` for bleeding edge

```bash
# dioxus-cli is required
cargo install dioxus-cli
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options
chmod +x ./install-webview.sh
# Run as local user, will require sudo password
./install-webview.sh
```

- Just the daemon:

AUR: `power-options-daemon` or `power-options-daemon-git` for bleeding edge

```bash
git clone https://github.com/TheAlexDev23/power-options/ --depth=1
cd power-options
chmod +x ./install-webview.sh
# Run as local user, will require sudo password
./install-daemon-only.sh
```

## Dependencies

Mandatory:
- zsh
- lspci
- lsusb
- acpi

Optional:
- iwlwifi compatible network card for network configuration
- xrandr: resolution/refresh rate control
- brightnessctl: brightness control
- ifconfig: ethernet blocking

Webview frontend:
- webkit2gtk
- dioxus-cli

GTK frontend:
- yad
- libadwaita

## Limitations
- Network configuration only works on intel cards and cards that use iwlwifi
- Resolution and refresh rate control only works on X11 (other options should
  work though).
- Settings for resolution and refresh rate control are only available on the
  webview frontend.

## Acknowledgements
- Arch Linux Wiki (https://wiki.archlinux.org)
- TLP (https://github.com/linrunner/TLP)
- Open Source Figma Icon set (https://www.figma.com/community/file/1250041133606945841/8-000-free-icons-open-source-icon-set)