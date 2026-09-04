#!/usr/bin/env bash
# Generate witness, prove, and verify one circuit (or all).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck source=lib.sh
source "$(dirname "$0")/lib.sh"
cougr_circuits_export_path
ARTIFACTS="$ROOT/artifacts"
WITNESS_NODE="${WITNESS_NODE:-node}"
KEYS="$ROOT/keys"
FIXTURES="$ROOT/fixtures"
WORK="$ROOT/work"

usage() {
  echo "usage: $0 [circuit_name|all]" >&2
  exit 1
}

prove_one() {
  local circuit="$1"
  local input="$FIXTURES/${circuit}.input.json"
  local r1cs="$ARTIFACTS/${circuit}.r1cs"
  local wasm_dir="$ARTIFACTS/${circuit}_js"
  local zkey="$KEYS/${circuit}_final.zkey"
  local vk="$KEYS/${circuit}_vk.json"
  local out="$WORK/${circuit}"

  if [[ ! -f "$input" ]]; then
    echo "missing fixture $input" >&2
    exit 1
  fi
  if [[ ! -f "$zkey" ]]; then
    echo "missing zkey $zkey - run setup.sh first" >&2
    exit 1
  fi

  mkdir -p "$out"
  echo "proving $circuit..."

  "$WITNESS_NODE" "$wasm_dir/generate_witness.js" "$wasm_dir/${circuit}.wasm" "$input" "$out/witness.wtns"
  snarkjs groth16 prove "$zkey" "$out/witness.wtns" "$out/proof.json" "$out/public.json"
  snarkjs groth16 verify "$vk" "$out/public.json" "$out/proof.json"

  echo "ok $circuit: proof verified"
}

main() {
  local target="${1:-all}"
  mkdir -p "$WORK"

  case "$target" in
    all)
      for circuit in hidden_cards fog_of_war fair_dice sealed_bid; do
        prove_one "$circuit"
      done
      ;;
    hidden_cards|fog_of_war|fair_dice|sealed_bid)
      prove_one "$target"
      ;;
    *)
      usage
      ;;
  esac
}

main "$@"