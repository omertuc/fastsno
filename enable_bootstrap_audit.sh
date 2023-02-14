#!/bin/bash

set -euxo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

SSH_IDENTITIY_FILE=${SSH_IDENTITIY_FILE:-"${SCRIPT_DIR}"/../bootstrap-in-place-poc/ssh-key/key}

function snossh() {
	ssh -o IdentityFile="${SSH_IDENTITIY_FILE}" -o LogLevel=ERROR -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no core@192.168.126.10 $@
}

while ! snossh ls /etc/kubernetes/bootstrap-configs/kube-apiserver-config.yaml >/dev/null; do
	echo "Waiting for kube-apiserver-config..."
	sleep 1
done

if snossh ls /etc/kubernetes/bootstrap-configs/kube-apiserver-config.yaml >/dev/null; then
	snossh cat /etc/kubernetes/bootstrap-configs/kube-apiserver-config.yaml | yq '.auditConfig.policyConfiguration.rules = [{"level": "Request"}]' >/tmp/newconf
	cat /tmp/newconf | snossh sudo tee /etc/kubernetes/bootstrap-configs/kube-apiserver-config.yaml >/dev/null
fi
