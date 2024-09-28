#!/bin/bash

set -e

./install-daemon.sh

pushd ../crates/power-applet
cargo build --release
popd

sudo cp -f ../target/release/power-applet /usr/bin/power-options-tray

sudo cp -f ../icon.png /usr/share/icons/power-options-tray.png

sudo cp -f ../install/power-options-gtk.desktop /etc/xdg/autostart/

echo "If you see D-Bus related issues, please restart your system."