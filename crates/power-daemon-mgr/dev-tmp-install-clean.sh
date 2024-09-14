cargo build

sudo rm /usr/lib/systemd/system/power-options.service
sudo cp ../../target/debug/power-daemon-mgr -f /usr/bin/
sudo power-daemon-mgr -vvv generate-base-files --path / --program-path /usr/bin/power-daemon-mgr --verbose-daemon
sudo power-daemon-mgr -vvv generate-config-files --path /
sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl restart power-options