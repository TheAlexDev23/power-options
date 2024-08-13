cargo build

sudo cp ../../target/debug/power-daemon-mgr -f /usr/bin/
sudo power-daemon-mgr -vvv generate-files --path / --program-path /usr/bin/power-daemon-mgr --verbose-daemon
sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl restart power-options