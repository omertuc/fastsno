#!/bin/bash

set -euo pipefail


set -euo pipefail

if [ -z "${RELEASE_IMAGE:-}" ]; then
    echo "RELEASE_IMAGE is not set"
    exit 1
fi

image_references=$(oc adm release extract --from=registry.build05.ci.openshift.org/ci-ln-dfb6b7t/release:latest --file=image-references 2>/dev/null)

while read -r component; do
    <<<"$image_references" jq --arg component "$component" -r '.spec.tags[] | select(.name == $component).from.name | "sudo crictl pull \(.)"' -r
done


