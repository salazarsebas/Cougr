#!/usr/bin/env bash
# validate_catalog.sh
#
# Validates examples/catalog.toml against the filesystem and schema rules.
# Delegates to validate_catalog.py (the canonical implementation).
# Run from the repository root.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

exec python3 "$REPO_ROOT/scripts/validate_catalog.py"
