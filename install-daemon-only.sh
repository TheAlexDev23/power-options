#!/bin/bash

set -e

pushd crates/power-daemon-mgr
cargo build --release
popd

sudo cp -f target/release/power-daemon-mgr /usr/bin/

sudo target/release/power-daemon-mgr -v generate-base-files --path / --program-path /usr/bin/power-daemon-mgr

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable power-options
sudo systemctl start power-options
