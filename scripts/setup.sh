sudo power-daemon-mgr setup

sudo systemctl restart acpid
sudo systemctl daemon-reload
sudo systemctl enable --now power-options