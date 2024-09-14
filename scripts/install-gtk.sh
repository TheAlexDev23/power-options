#!/bin/bash

set -e

./install-daemon.sh

pushd crates/frontend-gtk
cargo build --release
popd

sudo cp -f target/release/frontend-gtk /usr/bin/power-options-gtk

sudo cp -f icon.png /usr/share/icons/power-options.png

sudo cp -f install/power-options-gtk.desktop /usr/share/applications/

echo "If you see D-Bus related issues, please restart your system."