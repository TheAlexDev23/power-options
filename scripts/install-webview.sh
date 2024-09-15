#!/bin/bash

set -e

./install-daemon.sh

pushd ../crates/frontend-webview
dx build --release
popd

sudo cp -f ../target/release/frontend /usr/bin/power-options-webview

sudo mkdir -p /usr/lib/power-options-webview
sudo cp -r ../crates/frontend-webview/assets /usr/lib/power-options-webview

sudo cp -f ../icon.png /usr/share/icons/power-options-webview.png

sudo cp -f ../install/power-options-webview.desktop /usr/share/applications/

echo "If you see D-Bus related issues, please restart your system."