#!/bin/bash
sudo apt-get update
sudo apt-get install -y build-essential curl
sudo apt-get install -y pkg-config libglib2.0-dev libgtk-4-dev libgtk-3-dev libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libadwaita-1-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y