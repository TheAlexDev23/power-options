cargo build

sudo cp ../../target/debug/power-daemon-mgr -f /usr/bin/
sudo power-daemon-mgr generate-files --path / --program-path /usr/bin/power-daemon-mgr
sudo systemctl daemon-reload
sudo systemctl restart power-options