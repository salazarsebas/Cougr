#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

fail() {
  echo "FAIL: $1"
  exit 1
}

echo "=== verify_hygiene.sh (#225) ==="

if grep -q '^Cargo\.lock$' .gitignore 2>/dev/null; then
  fail "root .gitignore must not ignore Cargo.lock (examples are applications)"
fi

if [ -n "$(git ls-files 'examples/**/target/**')" ]; then
  fail "tracked target/ artifacts found"
fi

if [ -n "$(git ls-files 'examples/**/*.wasm')" ]; then
  fail "tracked .wasm artifacts found"
fi

if rg -q 'C[A-Z2-7]{55}' examples/*/README.md 2>/dev/null; then
  fail "hardcoded contract IDs in example READMEs"
fi

for d in examples/*/; do
  [ -f "${d}.gitignore" ] || fail "missing .gitignore in $(basename "$d")"
  grep -q '^target/' "${d}.gitignore" || fail "target/ not ignored in $(basename "$d")"
  if grep -q '^Cargo\.lock$' "${d}.gitignore" 2>/dev/null; then
    fail "Cargo.lock must not be gitignored in $(basename "$d")"
  fi
done

for d in examples/*/; do
  (cd "$d" && cargo metadata --no-deps >/dev/null 2>&1) || fail "cargo metadata failed in $(basename "$d")"
done

echo "ALL CHECKS PASSED"