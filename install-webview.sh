#!/bin/bash

set -e

pushd crates/power-daemon-mgr
cargo build --release
popd

pushd crates/frontend-webview
dx build --release
popd

sudo cp -f target/release/power-daemon-mgr /usr/bin/
sudo cp -f target/release/frontend /usr/bin/power-options-webview

sudo target/release/power-daemon-mgr -v generate-base-files --path / --program-path /usr/bin/power-daemon-mgr

sudo mkdir /usr/lib/power-options-webview
sudo cp -r crates/frontend-webview/assets /usr/lib/power-options-webview

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable power-options
sudo systemctl start power-options

sudo cp -f icon.png /usr/share/icons/power-options.png

sudo cp -f install/power-options-webview.desktop /usr/share/applications/