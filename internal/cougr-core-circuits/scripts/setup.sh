#!/usr/bin/env bash
# Trusted setup for test keys (development only — not for production).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"
cougr_circuits_export_path
ARTIFACTS="$ROOT/artifacts"
KEYS="$ROOT/keys"

mkdir -p "$KEYS"

POWER="${PTAU_POWER:-16}"
PTAU="$KEYS/pot${POWER}_final.ptau"
if [[ ! -f "$PTAU" ]]; then
  "$(dirname "$0")/download-ptau.sh"
fi

if ! command -v snarkjs >/dev/null 2>&1; then
  echo "snarkjs required for setup" >&2
  exit 1
fi

for circuit in hidden_cards fog_of_war fair_dice sealed_bid; do
  r1cs="$ARTIFACTS/${circuit}.r1cs"
  if [[ ! -f "$r1cs" ]]; then
    echo "missing $r1cs — run compile.sh first" >&2
    exit 1
  fi
  echo "setup $circuit (test pot12)..."
  snarkjs groth16 setup "$r1cs" "$PTAU" "$KEYS/${circuit}_0000.zkey"
  snarkjs zkey contribute "$KEYS/${circuit}_0000.zkey" "$KEYS/${circuit}_final.zkey" \
    --name="cougr-test" -v -e="$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')"
  snarkjs zkey export verificationkey "$KEYS/${circuit}_final.zkey" "$KEYS/${circuit}_vk.json"
done