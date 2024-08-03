cargo build

sudo cp ../../target/debug/power-daemon-mgr -f /usr/bin/
sudo systemctl restart power-options

