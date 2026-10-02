#!/usr/bin/env bash
# check_pr_issue_link.sh
#
# Checks whether a PR body contains at least one GitHub closing-issue keyword
# followed by a valid issue reference (#N or owner/repo#N).
#
# GitHub's recognised syntax (case-insensitive):
#   close / closes / closed
#   fix   / fixes  / fixed
#   resolve / resolves / resolved
# Followed (optionally separated by whitespace) by:
#   #<number>  or  <owner>/<repo>#<number>
#
# Usage:
#   check_pr_issue_link.sh <pr-body-file>   – read body from a file
#   check_pr_issue_link.sh                  – read body from stdin
#
# Exit codes:
#   0  – at least one valid closing reference found
#   1  – no closing reference found (emits a ::notice:: annotation on GitHub Actions)
#   2  – usage error

set -euo pipefail

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Emit a GitHub Actions annotation if running inside Actions; otherwise just
# print to stderr so the test harness sees it.
emit_notice() {
  if [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; then
    echo "::notice title=Missing closing-issue reference::$1"
  else
    echo "NOTICE: $1" >&2
  fi
}

emit_error() {
  if [[ "${GITHUB_ACTIONS:-false}" == "true" ]]; then
    echo "::error title=Missing closing-issue reference::$1"
  else
    echo "ERROR: $1" >&2
  fi
}

# ---------------------------------------------------------------------------
# Input
# ---------------------------------------------------------------------------

if [[ $# -gt 1 ]]; then
  echo "Usage: $0 [<pr-body-file>]" >&2
  exit 2
fi

if [[ $# -eq 1 ]]; then
  if [[ ! -f "$1" ]]; then
    echo "File not found: $1" >&2
    exit 2
  fi
  body=$(cat "$1")
else
  body=$(cat)
fi

# ---------------------------------------------------------------------------
# Pattern
# ---------------------------------------------------------------------------
#
# Matches (case-insensitive, on any line):
#   (close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved)
#   followed by optional whitespace / colon
#   followed by an optional <owner>/<repo> prefix
#   then #<digits>
#
# The POSIX ERE pattern used with grep -iE.
#
# The leading (^|[^A-Za-z]) anchor ensures the keyword is not preceded by a
# letter, so embedded occurrences such as "prefix", "disclose", "unresolved"
# are not treated as closing references.  The anchor group is non-capturing
# for the purposes of the human-readable output produced by grep -iEo.
PATTERN='(^|[^A-Za-z])(close[sd]?|fix(e[sd])?|resolve[sd]?)[[:space:]]*:?[[:space:]]*(([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)?#[0-9]+)'

# ---------------------------------------------------------------------------
# Check
# ---------------------------------------------------------------------------

if echo "$body" | grep -iEq "$PATTERN"; then
  # Extract and echo the matched reference(s) for visibility in CI logs
  matches=$(echo "$body" | grep -iEo "$PATTERN" | tr '\n' ' ')
  echo "✅ Closing-issue reference(s) found: $matches"
  exit 0
else
  emit_error \
    "This PR body contains no closing-issue reference (e.g. 'Closes #123'). \
Add a line like 'Closes #N', 'Fixes #N', or 'Resolves #N' so Drips Wave \
payout tracking can link this PR to its issue. If no issue exists for this \
change, a maintainer may override by merging with this check failing."
  exit 1
fi
