#!/usr/bin/env bash
# Export verification keys to Soroban-ready JSON for contract embedding.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEYS="$ROOT/keys"
OUT="$ROOT/exported"

mkdir -p "$OUT"

for circuit in hidden_cards fog_of_war fair_dice sealed_bid; do
  vk="$KEYS/${circuit}_vk.json"
  if [[ ! -f "$vk" ]]; then
    echo "missing $vk - run setup.sh first" >&2
    exit 1
  fi
  cp "$vk" "$OUT/${circuit}_vk.json"
  echo "exported $OUT/${circuit}_vk.json"
done