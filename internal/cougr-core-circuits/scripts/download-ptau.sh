#!/usr/bin/env bash
# Download or generate powers-of-tau for local/CI test trusted setup (development only).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KEYS="$ROOT/keys"
POWER="${PTAU_POWER:-16}"
PTAU="$KEYS/pot${POWER}_final.ptau"

mkdir -p "$KEYS"

if [[ -f "$PTAU" ]]; then
  echo "ptau already present at $PTAU"
  exit 0
fi

download_ptau() {
  local url="$1"
  echo "trying $url"
  if curl -fL --retry 2 --retry-delay 1 -o "$PTAU" "$url"; then
    return 0
  fi
  rm -f "$PTAU"
  return 1
}

URLS=(
  "https://storage.googleapis.com/zkevm/ptau/powersOfTau28_hez_final_${POWER}.ptau"
  "https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau"
)

for url in "${URLS[@]}"; do
  if download_ptau "$url"; then
    echo "saved $PTAU"
    exit 0
  fi
done

echo "download failed - generating local test ptau power=${POWER} with snarkjs (dev only)..." >&2
PTAU_0="$KEYS/pot${POWER}_0000.ptau"
PTAU_1="$KEYS/pot${POWER}_0001.ptau"
snarkjs powersoftau new bn128 "$POWER" "$PTAU_0" -v
snarkjs powersoftau contribute "$PTAU_0" "$PTAU_1" \
  --name="cougr-local" -v -e="cougr-local-$(date +%s)"
snarkjs powersoftau prepare phase2 "$PTAU_1" "$PTAU"
rm -f "$PTAU_0" "$PTAU_1"
echo "generated $PTAU"