#!/usr/bin/env bash
# Compile Cougr game circuits. Requires circom on PATH (or CIRCOM_PATH).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CIRCOM_DIR="$ROOT/circom"
OUT_DIR="$ROOT/artifacts"
CIRCOM_BIN="${CIRCOM:-circom}"

DECK_SIZE="${DECK_SIZE:-52}"
HAND_SIZE="${HAND_SIZE:-5}"

if ! command -v "$CIRCOM_BIN" >/dev/null 2>&1; then
  echo "circom not found — set CIRCOM or install from https://github.com/iden3/circom" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

# Parameterize hidden_cards.circom main component instantiation
sed -i.bak -E "s/HiddenCards\([0-9]+, *[0-9]+\)/HiddenCards(${DECK_SIZE}, ${HAND_SIZE})/g" "$CIRCOM_DIR/hidden_cards.circom" && rm -f "$CIRCOM_DIR/hidden_cards.circom.bak"

for circuit in hidden_cards fog_of_war fair_dice sealed_bid; do
  echo "compiling $circuit..."
  "$CIRCOM_BIN" "$CIRCOM_DIR/${circuit}.circom" \
    -l "$ROOT/node_modules/circomlib/circuits" \
    -l "$CIRCOM_DIR" \
    --r1cs --wasm --sym -o "$OUT_DIR"
done

echo "artifacts written to $OUT_DIR"