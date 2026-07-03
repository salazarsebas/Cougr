#!/usr/bin/env bash
#
# verify_hygiene.sh — Verify repository hygiene standards from issue #225
#

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"
EXIT_CODE=0

echo "=== Hygiene Verification (#225) ==="
echo ""

echo "Check 1: No tracked target/ directories"
tracked_targets=$(git -C "$REPO_ROOT" ls-files 'examples/**/target/**' 2>/dev/null || true)
if [ -z "$tracked_targets" ]; then
    echo "  PASS: No target/ files tracked"
else
    echo "  FAIL: Tracked target/ files found:"
    echo "$tracked_targets" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

echo "Check 2: No tracked .wasm files"
tracked_wasm=$(git -C "$REPO_ROOT" ls-files 'examples/**/*.wasm' 2>/dev/null || true)
if [ -z "$tracked_wasm" ]; then
    echo "  PASS: No .wasm files tracked"
else
    echo "  FAIL: Tracked .wasm files found:"
    echo "$tracked_wasm" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

echo "Check 3: No hardcoded contract IDs in README.md files"
contract_ids=$(grep -rE 'C[A-Z2-7]{55}' "$EXAMPLES_DIR"/*/README.md 2>/dev/null || true)
if [ -z "$contract_ids" ]; then
    echo "  PASS: No hardcoded contract IDs found"
else
    echo "  FAIL: Hardcoded contract IDs found:"
    echo "$contract_ids" | sed 's/^/    /'
    EXIT_CODE=1
fi
echo ""

echo "Check 4: .gitignore in every example directory"
missing_gitignore=0
for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ ! -f "$example_dir/.gitignore" ]; then
        echo "  FAIL: Missing .gitignore in $(basename "$example_dir")"
        missing_gitignore=$((missing_gitignore + 1))
        EXIT_CODE=1
    fi
done
if [ $missing_gitignore -eq 0 ]; then
    echo "  PASS: All examples have .gitignore"
fi
echo ""

echo "Check 5: .gitignore excludes target/ and *.wasm"
missing_rules=0
for gitignore in "$EXAMPLES_DIR"/*/.gitignore; do
    example_name=$(basename "$(dirname "$gitignore")")
    if ! grep -qE '^(/)?target/' "$gitignore"; then
        echo "  FAIL: examples/$example_name/.gitignore does not exclude target/"
        missing_rules=$((missing_rules + 1))
        EXIT_CODE=1
    fi
    if ! grep -qE '^\*\.wasm' "$gitignore"; then
        echo "  FAIL: examples/$example_name/.gitignore does not exclude *.wasm"
        missing_rules=$((missing_rules + 1))
        EXIT_CODE=1
    fi
done
if [ $missing_rules -eq 0 ]; then
    echo "  PASS: All .gitignore files exclude target/ and *.wasm"
fi
echo ""

echo "Check 6: .gitignore does not exclude Cargo.lock"
cargo_lock_ignored=0
for gitignore in "$EXAMPLES_DIR"/*/.gitignore; do
    example_name=$(basename "$(dirname "$gitignore")")
    if grep -qE '^Cargo\.lock$' "$gitignore"; then
        echo "  FAIL: examples/$example_name/.gitignore excludes Cargo.lock"
        cargo_lock_ignored=$((cargo_lock_ignored + 1))
        EXIT_CODE=1
    fi
done
if [ $cargo_lock_ignored -eq 0 ]; then
    echo "  PASS: No example .gitignore excludes Cargo.lock"
fi
echo ""

echo "Check 7: Root .gitignore does not exclude Cargo.lock"
if grep -qE '^Cargo\.lock$' "$REPO_ROOT/.gitignore" 2>/dev/null; then
    echo "  FAIL: Root .gitignore excludes Cargo.lock"
    EXIT_CODE=1
else
    echo "  PASS: Root .gitignore does not exclude Cargo.lock"
fi
echo ""

echo "Check 8: cargo metadata --no-deps succeeds for all examples"
metadata_failed=0
for example_dir in "$EXAMPLES_DIR"/*/; do
    example_name=$(basename "$example_dir")
    if (cd "$example_dir" && cargo metadata --no-deps --format-version 1 >/dev/null 2>&1); then
        :
    else
        echo "  FAIL: cargo metadata failed for $example_name"
        metadata_failed=$((metadata_failed + 1))
        EXIT_CODE=1
    fi
done
if [ $metadata_failed -eq 0 ]; then
    echo "  PASS: cargo metadata succeeds for all examples"
fi
echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo "=== ALL CHECKS PASSED ==="
else
    echo "=== SOME CHECKS FAILED ==="
fi

exit $EXIT_CODE