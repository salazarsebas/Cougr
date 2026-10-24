#!/usr/bin/env bash
# check_example_workflows.sh
#
# Fails if any example directory containing a Cargo.toml (examples/*/Cargo.toml)
# is not referenced by at least one workflow in .github/workflows/*.yml.
#
# Usage:
#   bash scripts/check_example_workflows.sh [examples_dir] [workflows_dir]

set -euo pipefail

EXAMPLES_DIR="${1:-examples}"
WORKFLOWS_DIR="${2:-.github/workflows}"

if [[ ! -d "$EXAMPLES_DIR" ]]; then
  echo "::error::Examples directory not found: $EXAMPLES_DIR" >&2
  exit 1
fi

if [[ ! -d "$WORKFLOWS_DIR" ]]; then
  echo "::error::Workflows directory not found: $WORKFLOWS_DIR" >&2
  exit 1
fi

missing=0
checked=0

for cargo_toml in "$EXAMPLES_DIR"/*/Cargo.toml; do
  [[ -f "$cargo_toml" ]] || continue
  example_dir="${cargo_toml%/Cargo.toml}"
  example_name="${example_dir##*/}"
  checked=$((checked + 1))

  if ! grep -rqF "examples/$example_name/" "$WORKFLOWS_DIR"/*.yml; then
    echo "::error::Example 'examples/$example_name' is not referenced by any workflow in $WORKFLOWS_DIR" >&2
    missing=$((missing + 1))
  fi
done

if [[ "$checked" -eq 0 ]]; then
  echo "::error::No example Cargo.toml files found under $EXAMPLES_DIR" >&2
  exit 1
fi

if [[ "$missing" -ne 0 ]]; then
  echo "FAIL: $missing of $checked example(s) are missing CI workflow coverage." >&2
  exit 1
fi

echo "All $checked example crates are covered by a CI workflow."
