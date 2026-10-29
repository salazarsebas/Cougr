#!/usr/bin/env bash
# Full Cougr Circom pipeline: compile → setup → prove → verify.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS="$ROOT/scripts"
# shellcheck source=lib.sh
source "$SCRIPTS/lib.sh"
cougr_circuits_export_path

export PATH="${CIRCOM_PATH:-}:$PATH"
export CIRCOM="${CIRCOM:-circom}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_cmd "$CIRCOM"
require_cmd snarkjs
require_cmd bun

"$SCRIPTS/validate-layouts.sh"
bun run fixtures
"$SCRIPTS/compile.sh"
"$SCRIPTS/download-ptau.sh"
"$SCRIPTS/setup.sh"
"$SCRIPTS/prove.sh" all
"$SCRIPTS/verify-rejects-invalid.sh" fair_dice
"$SCRIPTS/export-vk.sh"
bun "$SCRIPTS/export-soroban-artifacts.mjs"

echo "circom pipeline complete"