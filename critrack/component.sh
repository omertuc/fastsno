#!/bin/bash

set -euo pipefail

if [ -z "${RELEASE_IMAGE:-}" ]; then
    echo "RELEASE_IMAGE is not set"
    exit 1
fi

image_references=$(oc adm release extract --from="$RELEASE_IMAGE" --file=image-references 2>/dev/null)

while read -r image; do
    <<<"$image_references" jq --arg image "$image" -r '.spec.tags[] | select(.from.name == $image).name' -r
done

