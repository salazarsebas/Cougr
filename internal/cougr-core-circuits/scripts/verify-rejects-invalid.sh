#!/usr/bin/env bash
# Tamper a fixture and expect witness generation or verification to fail.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"
cougr_circuits_export_path
WITNESS_NODE="${WITNESS_NODE:-node}"
FIXTURES="$ROOT/fixtures"
ARTIFACTS="$ROOT/artifacts"
KEYS="$ROOT/keys"
WORK="$ROOT/work/negative"

circuit="${1:-fair_dice}"
mkdir -p "$WORK"

input="$WORK/input.json"
cp "$FIXTURES/${circuit}.input.json" "$input"

node -e "
const fs = require('fs');
const p = process.argv[1];
const j = JSON.parse(fs.readFileSync(p, 'utf8'));
if (j.roll_result !== undefined) j.roll_result = '1';
if (j.next_explored_root !== undefined) j.next_explored_root = '0';
if (j.hand_commitment !== undefined) j.hand_commitment = '0';
if (j.bid_commitment !== undefined) j.bid_commitment = '0';
fs.writeFileSync(p, JSON.stringify(j, null, 2));
" "$input"

wasm_dir="$ARTIFACTS/${circuit}_js"
zkey="$KEYS/${circuit}_final.zkey"
vk="$KEYS/${circuit}_vk.json"

set +e
"$WITNESS_NODE" "$wasm_dir/generate_witness.js" "$wasm_dir/${circuit}.wasm" "$input" "$WORK/witness.wtns" 2>/dev/null
witness_status=$?
set -e

if [[ $witness_status -ne 0 ]]; then
  echo "ok $circuit: invalid input rejected at witness generation"
  exit 0
fi

snarkjs groth16 prove "$zkey" "$WORK/witness.wtns" "$WORK/proof.json" "$WORK/public.json"
if snarkjs groth16 verify "$vk" "$WORK/public.json" "$WORK/proof.json"; then
  echo "expected invalid proof to fail verification" >&2
  exit 1
fi

echo "ok $circuit: invalid proof rejected at verification"