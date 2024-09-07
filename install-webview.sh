#!/bin/bash

set -e

pushd crates/power-daemon-mgr
cargo build --release
popd

pushd crates/frontend-webview
dx bundle --release
popd

sudo cp -f target/release/power-daemon-mgr /usr/bin/
sudo cp -f dist/bundle/deb/frontend_*_amd64/data/usr/bin/frontend-webview /usr/bin/power-options-webview

sudo target/release/power-daemon-mgr -v generate-files --path / --program-path /usr/bin/power-daemon-mgr

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable power-options
sudo systemctl start power-options

sudo cp -f icon.png /usr/share/icons/power-options.png

sudo cp -f install/power-options-webview.desktop /usr/share/applications/