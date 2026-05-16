#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "Resetting Armory resources..."

kubectl delete backup --all -n armory --ignore-not-found
kubectl delete volumestorage --all -n armory --ignore-not-found

kubectl apply -f "$SCRIPT_DIR/resources/"

echo "Armory resources reset complete."
