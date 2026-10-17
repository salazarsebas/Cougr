#!/usr/bin/env bash
set -euo pipefail

# Fail if any examples/*/Cargo.toml directory is not referenced by any workflow in
.github/workflows/.

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

workflows_dir=".github/workflows"
missing=0

for cargo in examples/*/Cargo.tom; do
  [ -f "$cargo" ] || continue
  dir="$(dirname "$cargo")"
  name="$(basename "$dir")"
  if ! grep -qr "examples/$name/" "$workflows_dir"; then
    echo "error: example '$name' is not referenced by any workflow in $workflows_dir" >&2
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  echo "error: one or more examples are not covered by a workflow" >&2
  exit 1
fi

echo "ok: all examples are referenced by a workflow"
