#! /usr/bin/env nix-shell
#! nix-shell -p rsync -i bash
# shellcheck shell=bash

set -e

DEVICE=/dev/disk/by-id/usb-Garmin_FR935_Flash-0:0
MOUNTPOINT=/media/garmin

if [ "$#" -ne 1 ]; then
    echo "Usage: sync-from-device.sh <DEST_DIRECTORY>"
    exit 1
fi

dest_directory=$1

mount_device() {
    pmount -r $DEVICE $MOUNTPOINT
}

umount_device() {
    pumount $DEVICE
}

mount_device
trap umount_device EXIT

rsync --archive --verbose \
    $MOUNTPOINT/GARMIN/ACTIVITY/*.FIT "$dest_directory"
