#!/bin/bash

set -euxo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

mv audit "audit_old_$(date -u +%Y-%m-%dT%H:%M:%S%Z)" || true
mkdir -p "$SCRIPT_DIR/audit"
oc adm node-logs "$(oc get nodes -oname)" --path=kube-apiserver/ | grep audit | while read -r log; do oc adm node-logs master1 "--path=kube-apiserver/$log" > "audit/$log"; done
