#! /usr/bin/env nix-shell
#! nix-shell -p gpsbabel -i bash
# shellcheck shell=bash

if [ "$#" -ne 1 ]; then
    echo "Usage: fit2gpx.sh <FIT_FILE>"
    exit 1
fi

fit_file=$1
gpsbabel -i garmin_fit -f "$fit_file" -o gpx -F "${fit_file%.*}.gpx"
