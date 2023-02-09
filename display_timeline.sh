#!/bin/bash

set -euxo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

google-chrome --new-window --allow-file-access-from-files --user-data-dir="$SCRIPT_DIR"/chrome "$SCRIPT_DIR"/index.html
