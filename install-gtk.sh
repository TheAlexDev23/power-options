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

sudo target/release/power-daemon-mgr -v generate-base-files --path / --program-path /usr/bin/power-daemon-mgr

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable power-options
sudo systemctl start power-options

sudo cp -f icon.png /usr/share/icons/power-options.png

sudo cp -f install/power-options-gtk.desktop /usr/share/applications/

echo "If you see D-Bus related issues, please restart your system."