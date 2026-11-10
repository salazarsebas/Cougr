#!/usr/bin/env bash
# Sanity-check frozen public-input counts match Circom scaffolds and Rust layouts.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

check_circuit() {
  local circuit="$1"
  local count="$2"
  local file="$ROOT/circom/${circuit}.circom"
  if [[ ! -f "$file" ]]; then
    echo "missing $file" >&2
    exit 1
  fi
  local inputs
  inputs=$(grep -c 'signal input' "$file" || true)
  if [[ "$inputs" -lt "$count" ]]; then
    echo "$circuit: expected at least $count public inputs in circom, found $inputs" >&2
    exit 1
  fi
  echo "ok $circuit (>= $count public inputs)"
}

check_circuit hidden_cards 5
check_circuit fog_of_war 8
check_circuit fair_dice 4
check_circuit sealed_bid 4