#!/bin/bash

# Function to check if running as root
check_root() {
    if [ "$(id -u)" -ne 0 ]; then
        echo "This script must be run as root. Please use sudo or run as root."
        exit 1
    fi
}

# Function to remove files and directories
remove_files_and_dirs() {
    # Remove directory
    if [ -d "/etc/power-options" ]; then
        echo "Removing /etc/power-options directory..."
        rm -rf /etc/power-options
    fi

    # Remove application files
    echo "Removing application files..."
    rm -f /usr/share/applications/power-options-*

    # Remove icon
    echo "Removing icon..."
    rm -f /usr/share/icons/power-options.png

    # Remove daemon binaries
    echo "Removing executables..."
    rm -f /usr/bin/power-daemon-mgr
    if [ -d "/usr/bin/power-options-gtk" ]; then
        rm -f /usr/bin/power-options-gtk
    fi
    if [ -d "/usr/bin/power-options-webview" ]; then
        rm -f /usr/bin/power-options-webview
    fi

    # Remove additional files
    echo "Removing additional configuration files..."
    rm -f /etc/acpi/events/power-options
    rm -f /usr/lib/udev/rules.d/85-power-daemon.rules
    rm -f /usr/share/dbus-1/system.d/power-daemon.conf
}

# Function to disable and remove systemd service
remove_systemd_service() {
    SERVICE_NAME="power-options.service"
    SERVICE_PATH="/lib/systemd/system/${SERVICE_NAME}"
    if [ -f "$SERVICE_PATH" ]; then
        echo "Stopping and disabling ${SERVICE_NAME}..."
        systemctl stop "$SERVICE_NAME"
        systemctl disable "$SERVICE_NAME"
        echo "Removing ${SERVICE_NAME}..."
        rm -f "$SERVICE_PATH"
        echo "Reloading systemd daemon..."
        systemctl daemon-reload
    else
        echo "${SERVICE_NAME} not found. Skipping..."
    fi
}

# Main execution
check_root
remove_files_and_dirs
remove_systemd_service
echo "Cleanup completed successfully."