#!/bin/bash

set -euxo pipefail

SCRIPT_DIR=$(dirname "$(readlink -f "$0")")

mkdir -p "$SCRIPT_DIR/timelines"

echo '[]' > "$SCRIPT_DIR"/timeline.json

now=$(jq -n 'now | todateiso8601' -r)
i=0
for jq_script_file in "$@"; do
    cat "$SCRIPT_DIR"/audit/*audit* | jq --arg now "$now" -r --slurp -f "$jq_script_file" > timelines/timeline_${i}.json
    i=$((i+1))
done

for i in {0..9}; do
    timeline="$SCRIPT_DIR/timelines/timeline_${i}.json"
    if [ -f "$timeline" ]; then
        jq -s '.[0] + .[1]' "$timeline" timeline.json | sponge timeline.json
    fi
done

AVAILABILITY_DATA="$SCRIPT_DIR/../kube-api-availability/static/data.json"
if [[ -f $AVAILABILITY_DATA ]]; then
	# Merge the AVAILABILITY_DATA JSON array with our data.js JSON array
	jq -s '.[0] + .[1]' "$AVAILABILITY_DATA" timeline.json | sponge timeline.json
fi
