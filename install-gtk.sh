#!/bin/bash

set -e

pushd crates/power-daemon-mgr
cargo build --release
popd

pushd crates/frontend-gtk
cargo build --release
popd

sudo cp -f target/release/power-daemon-mgr /usr/bin/
sudo cp -f target/release/frontend-gtk /usr/bin/power-options-gtk

sudo target/release/power-daemon-mgr -v generate-files --path / --program-path /usr/bin/power-daemon-mgr

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable power-options
sudo systemctl start power-options

sudo cp -f static/power-options.png /usr/share/icons

echo "[Desktop Entry]
Name=Power Options Gtk
Comment=GTK Frontend for the Power Options daemon
Exec=/usr/bin/power-options-gtk
Icon=/usr/share/icons/power-options.png
Terminal=false
Type=Application
Categories=Utility;Settings;" | sudo tee /usr/share/applications/power-options-gtk.desktop > /dev/null
