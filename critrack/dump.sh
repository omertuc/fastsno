#!/bin/bash

set -euo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

SSH_IDENTITIY_FILE=${SSH_IDENTITIY_FILE:-"${SCRIPT_DIR}"/../../bootstrap-in-place-poc/ssh-key/key}

ssh -o IdentityFile="${SSH_IDENTITIY_FILE}" -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no core@192.168.126.10 sudo journalctl -u crio 2>/dev/null |
    # Find image download lines from CRI-O
    grep "Checking image" |
    # Extract the image name
    cut -d" " -f12 |
    # Remove the double quotes at the end of each line
    rev | cut -c 2- | rev |
    # Find unique lines without sorting (to preserve image appearance order and removing duplicates)
    cat -n - | sort -uk2 | sort -n | cut -f2-
