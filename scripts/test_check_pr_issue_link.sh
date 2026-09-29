#!/usr/bin/env bash
# test_check_pr_issue_link.sh
#
# Self-contained test suite for scripts/check_pr_issue_link.sh.
#
# Run:
#   bash scripts/test_check_pr_issue_link.sh
#
# Exits 0 if all cases pass, non-zero otherwise.
# Designed to run locally and in CI (no external dependencies).

set -euo pipefail

SCRIPT="$(dirname "$0")/check_pr_issue_link.sh"
PASS=0
FAIL=0

# Temp file used to pass body content; cleaned up on exit.
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

run_case() {
  local description="$1"
  local body="$2"
  local expected_exit="$3"   # 0 = should pass, 1 = should fail

  printf '%s' "$body" > "$TMPFILE"

  actual_exit=0
  GITHUB_ACTIONS=false bash "$SCRIPT" "$TMPFILE" >/dev/null 2>/dev/null \
    || actual_exit=$?

  if [[ "$actual_exit" -eq "$expected_exit" ]]; then
    echo "  PASS  $description"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $description  (expected exit $expected_exit, got $actual_exit)"
    FAIL=$((FAIL + 1))
  fi
}

# ---------------------------------------------------------------------------
# Test cases — should PASS (exit 0, reference found)
# ---------------------------------------------------------------------------

echo ""
echo "=== Cases that should PASS (reference present) ==="

run_case "lowercase 'closes #42'"         "closes #42"                                  0
run_case "lowercase 'fixes #1'"           "fixes #1"                                    0
run_case "lowercase 'resolves #999'"      "resolves #999"                               0
run_case "uppercase 'Closes #7'"          "Closes #7"                                   0
run_case "mixed case 'FIXES #10'"         "FIXES #10"                                   0
run_case "past tense 'closed #3'"         "closed #3"                                   0
run_case "past tense 'fixed #3'"          "fixed #3"                                    0
run_case "past tense 'resolved #3'"       "resolved #3"                                 0
run_case "colon separator 'Closes: #5'"   "Closes: #5"                                  0
run_case "extra whitespace 'Closes  #5'"  "Closes  #5"                                  0
run_case "cross-repo 'Closes org/repo#8'" "Closes org/repo#8"                           0
run_case "reference buried in body" \
"## What changed
Some prose here.

This implements the feature requested in Closes #123.

## Why
Because."  0

run_case "multiple references" \
"Closes #10
Fixes #20"  0

run_case "reference at end of sentence 'Fixes #55.'" \
  "Fixes #55."  0

run_case "template placeholder not stripped (real number)" \
  "Closes #380"  0

# ---------------------------------------------------------------------------
# Test cases — should FAIL (exit 1, no reference)
# ---------------------------------------------------------------------------

echo ""
echo "=== Cases that should FAIL (reference absent) ==="

run_case "empty body"                     ""                                             1
run_case "no closing keyword at all" \
"## What changed
Added some stuff."  1

run_case "keyword without issue number 'Closes'" \
  "Closes"  1

run_case "keyword with non-numeric 'Closes #abc'" \
  "Closes #abc"  1

run_case "template placeholder unfilled 'Closes #'" \
"## What changed
Closes #"  1

run_case "URL containing # but no keyword" \
  "See https://github.com/org/repo/issues/42"  1

run_case "bare issue mention '#42' without keyword" \
  "This relates to #42 from the backlog."  1

# Note: the check is line-based and does not exclude fenced code blocks;
# a closing keyword inside a code block will match. This is documented
# behaviour – it is an unlikely edge case and adding complexity to exclude
# code blocks is not worth the maintenance cost.
run_case "keyword in code block (documents that it still matches)" \
'```
closes #5
```'  0

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

echo ""
echo "Results: $PASS passed, $FAIL failed."
echo ""

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
exit 0
