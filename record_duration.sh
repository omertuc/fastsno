#!/bin/bash

set -euxo pipefail

if [ -z ${KUBECONFIG+x} ]; then
        echo "Please set KUBECONFIG"
        exit 1
fi

filename="duration.txt"


start=$(date +%s)
until [ "$(oc get clusterversion -o jsonpath='{.items[*].status.conditions[?(@.type=="Available")].status}')" == "True" ]
do
  sleep 20
done

end=$(date +%s)
runtime=$((end-start))
echo "cluster installation took: $((runtime/60)) minutes" >> $filename
