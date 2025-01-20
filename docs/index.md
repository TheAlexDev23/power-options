---
layout: default
title: Power Options
---

# Power Options

Power Options is a comprehensive power management tool for Linux, designed to provide greater control over power-saving features and system profiles. It includes both a daemon and multiple GUI frontends for ease of use. It is very feature complete and provides significantly more options and control over alternatives.

Power Options generates profiles manually based on your system so that you don't have to touch anything if you don't want to. For a list of default configurations please refer to [this page](./defaults.md)

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
- System Sleep Options (suspend, screen turn off)
- CPU Options
- Individual CPU Core Options. Most power saving tools lack this option and was
  one of the main motivations for this project.
- Screen Options
- Options for disabling radio components (e.g Bluetooth, WiFi, NFC)
- Network Options. Allows WAY greater control than alternative applications, but
  does require network driver reload. Limited to Intel network cards that use
  iwlwifi.
- ASPM Options
- PCI Options
- USB Options
- SATA Options
- Kernel Options
- Firmware settings
- Audio Options
- GPU Options
- Intel Running Average Power Limit (RAPL) settings

## Available Frontends/Interfaces

### Native GTK Frontend

Simple, lightweight, and native with a straightforward interface. Recommended for most users.

![GTK Slideshow](./static/gtk-slideshow.gif)

### WebKit Frontend

Offers more advanced options and greater control. Recommended for advanced users and those looking to use Power Options alongside other power management solutions.

![Webview Slideshow](./static/webview-slideshow.gif)

## Installation

Refer to the [install page](./install.md)