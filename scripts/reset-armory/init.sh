#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Connecting Kind nodes to Redpanda network..."
REDPANDA_CONTAINER=$(docker ps --filter "label=com.docker.compose.service=redpanda" --format '{{.Names}}')
REDPANDA_NETWORK=$(docker inspect "$REDPANDA_CONTAINER" -f '{{range $k,$v := .NetworkSettings.Networks}}{{$k}}{{"\n"}}{{end}}' | grep -v "^$" | head -1)
for node in $(docker ps --filter "label=io.x-k8s.kind.cluster" --format '{{.Names}}'); do
    docker network connect "$REDPANDA_NETWORK" "$node" 2>/dev/null || true
done

echo "Resetting Armory resources..."
kubectl delete backup siege -n kannika-data --ignore-not-found
kubectl delete storage siege -n kannika-data --ignore-not-found
kubectl patch pvc siege -n kannika-data -p '{"metadata":{"finalizers":null}}' 2>/dev/null || true
kubectl delete pvc siege -n kannika-data --ignore-not-found
kubectl delete eventhub redpanda -n kannika-data --ignore-not-found

echo "Applying Armory resources..."
kubectl apply -f "$SCRIPT_DIR/resources/"

echo "Force killing all pods in kannika-data namespace..."
kubectl delete pods --all -n kannika-data --force --grace-period=0

echo "Armory resources reset complete."
