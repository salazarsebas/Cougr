#!/usr/bin/env bash
# Run locally the same checks pull-request CI runs, for the parts of the repo
# this branch changes. Usage:
#
#   make check        checks for what you changed (compared with upstream main)
#   make check-all    every check, whatever you changed
#   make fix          apply the automatic fixes (formatting, site sync)
#
# Each block mirrors a workflow in .github/workflows/. When a workflow changes,
# update the matching block here.

set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 1

MODE="changed"
case "${1:-}" in
  --all) MODE="all" ;;
  --fix) MODE="fix" ;;
  "") ;;
  *) echo "usage: scripts/check.sh [--all|--fix]" >&2; exit 2 ;;
esac

if [ -t 1 ]; then
  BOLD=$'\033[1m'; RED=$'\033[31m'; GREEN=$'\033[32m'; YELLOW=$'\033[33m'; RESET=$'\033[0m'
else
  BOLD=""; RED=""; GREEN=""; YELLOW=""; RESET=""
fi

PASSED=()
FAILED=()
SKIPPED=()

# run <dir> <label> <command...>: run a command in a directory and record the result.
run() {
  local dir="$1" label="$2"
  shift 2
  printf '\n%s==> %s%s\n' "$BOLD" "$label" "$RESET"
  if (cd "$dir" && "$@"); then
    PASSED+=("$label")
  else
    FAILED+=("$label")
  fi
}

# need <tool> <how to install> <label>: true if the tool exists, otherwise record a skip.
need() {
  if command -v "$1" >/dev/null 2>&1; then
    return 0
  fi
  SKIPPED+=("$3 (needs $1: $2)")
  return 1
}

# ---------------------------------------------------------------------------
# What changed

UPSTREAM_URL="https://github.com/salazarsebas/Cougr.git"
base_ref() {
  local remote
  for remote in $(git remote); do
    case "$(git remote get-url "$remote" 2>/dev/null)" in
      *salazarsebas/Cougr*|*salazarsebas/cougr*)
        if git rev-parse -q --verify "$remote/main" >/dev/null; then
          echo "$remote/main"
          return
        fi
        ;;
    esac
  done
  for ref in upstream/main origin/main main; do
    if git rev-parse -q --verify "$ref" >/dev/null; then
      echo "$ref"
      return
    fi
  done
}

CHANGED=""
if [ "$MODE" = "changed" ]; then
  REF="$(base_ref)"
  if [ -z "$REF" ]; then
    echo "${YELLOW}Could not find main to compare with, so every check runs.${RESET}"
    echo "Tip: git remote add upstream $UPSTREAM_URL && git fetch upstream"
    MODE="all"
  else
    BASE="$(git merge-base HEAD "$REF")"
    CHANGED="$(
      {
        git diff --name-only "$BASE"...HEAD
        git diff --name-only HEAD
        git ls-files --others --exclude-standard
      } | sort -u
    )"
    echo "Comparing with ${BOLD}$REF${RESET}. Changed files: $(printf '%s' "$CHANGED" | grep -c . || true)"
    case "$REF" in
      upstream/*) ;;
      *) echo "Tip: to compare with the real upstream, run: git remote add upstream $UPSTREAM_URL && git fetch upstream" ;;
    esac
  fi
fi

# touched <regex>: true in --all mode, or when a changed path matches.
touched() {
  [ "$MODE" = "all" ] && return 0
  printf '%s\n' "$CHANGED" | grep -qE "$1"
}

# ---------------------------------------------------------------------------
# make fix: automatic fixes only, then stop.

if [ "$MODE" = "fix" ]; then
  run . "cargo fmt (root)" cargo fmt
  run cli "cargo fmt (cli)" cargo fmt
  for dir in examples/*/; do
    dir="${dir%/}"
    [ -f "$dir/Cargo.toml" ] && run "$dir" "cargo fmt ($dir)" cargo fmt
  done
  run . "sync cougr-site/src" python3 cougr-site/sync.py
  echo
  echo "Done. Review the changes with 'git diff', commit them, then run 'make check'."
  [ "${#FAILED[@]}" -eq 0 ]
  exit $?
fi

# ---------------------------------------------------------------------------
# PR Sanity Check (pr-check.yml): runs on every pull request.

run . "PR sanity: cargo fmt --check" cargo fmt --check
run . "PR sanity: no committed build artifacts" bash -c '
  if git ls-files | grep -E "(^|/)target/|\.wasm$"; then
    echo "Remove target/ directories and *.wasm files from the commit."
    exit 1
  fi'
run . "PR sanity: no merge conflict markers" bash -c '
  if git grep -nE "^(<<<<<<<|=======|>>>>>>>)( |$)" -- . ":(exclude)*.md"; then
    echo "Resolve the merge conflict markers above."
    exit 1
  fi'

# ---------------------------------------------------------------------------
# Core CI (core.yml): library validation and MSRV.

if touched '^(src|internal|tests|benches|cli)/|^Cargo\.(toml|lock)$|^\.github/workflows/core\.yml$'; then
  run . "core: clippy" cargo clippy --all-targets --all-features -- -D warnings
  run . "core: tests" cargo test --workspace --features testutils
  run . "core: build" cargo build
  run . "core: hygiene check (cougr-cli)" cargo run -q -p cougr-cli -- check --path .

  MSRV="$(grep -oE 'rust-toolchain@[0-9]+\.[0-9]+(\.[0-9]+)?' .github/workflows/core.yml | head -1 | cut -d@ -f2)"
  if [ -n "$MSRV" ]; then
    if rustup toolchain list 2>/dev/null | grep -q "^$MSRV"; then
      run . "core: MSRV $MSRV" cargo "+$MSRV" check --workspace --all-features
    else
      SKIPPED+=("core: MSRV $MSRV (install with: rustup toolchain install $MSRV --target wasm32v1-none)")
    fi
  fi
fi

# CLI CI (cli.yml)
if touched '^cli/|^\.github/workflows/cli\.yml$'; then
  run cli "cli: cargo fmt --check" cargo fmt --check
  run cli "cli: clippy" cargo clippy --all-targets --all-features -- -D warnings
  run cli "cli: tests" cargo test
  run cli "cli: build" cargo build
fi

# Cargo Deny (cargo-deny.yml)
if touched '(^|/)Cargo\.(toml|lock)$|^deny\.toml$|^\.github/workflows/cargo-deny\.yml$'; then
  if cargo deny --version >/dev/null 2>&1; then
    run . "cargo deny check" cargo deny check
  else
    SKIPPED+=("cargo deny check (install with: cargo install --locked cargo-deny)")
  fi
fi

# ---------------------------------------------------------------------------
# Examples. Each example with its own workflow uses example_reusable.yml.
# The four ZK examples use circuit_examples.yml instead.

CIRCUIT_EXAMPLES="hidden_hand fog_explorer dice_duel blind_auction"
is_circuit_example() {
  case " $CIRCUIT_EXAMPLES " in *" $1 "*) return 0 ;; esac
  return 1
}

stellar_build() {
  local dir="$1" name="$2"
  if need stellar "https://developers.stellar.org/docs/tools/cli/install-cli" "$name: stellar contract build"; then
    run "$dir" "$name: stellar contract build" stellar contract build
  fi
}

for dir in examples/*/; do
  dir="${dir%/}"
  name="${dir#examples/}"
  [ -f "$dir/Cargo.toml" ] || continue

  if is_circuit_example "$name"; then
    touched "^$dir/|^src/circuits/|^internal/cougr-core-circuits/|^\.github/workflows/circuit_examples\.yml$" || continue
    run "$dir" "$name: cargo test" cargo test
    stellar_build "$dir" "$name"
    continue
  fi

  # Only examples that have a CI workflow. Markdown-only changes do not trigger it.
  grep -qE "working_directory: *$dir *$" .github/workflows/*.yml 2>/dev/null || continue
  if [ "$MODE" != "all" ]; then
    printf '%s\n' "$CHANGED" | grep -E "^$dir/" | grep -vqE '\.md$' \
      || touched '^\.github/workflows/example_reusable\.yml$' \
      || continue
  fi
  run "$dir" "$name: cargo fmt --check" cargo fmt --check
  run "$dir" "$name: clippy" cargo clippy --all-targets --all-features -- -D warnings
  run "$dir" "$name: cargo test" cargo test
  stellar_build "$dir" "$name"
done

# Circuit Examples (circuit_examples.yml): circuits module
if touched '^(src/circuits|internal/cougr-core-circuits)/|^examples/(hidden_hand|fog_explorer|dice_duel|blind_auction)/|^\.github/workflows/circuit_examples\.yml$'; then
  run . "circuits: module tests" cargo test -p cougr-core --features testutils -- circuits integration_circuits
  run . "circuits: validate layouts" bash internal/cougr-core-circuits/scripts/validate-layouts.sh
fi

# Circom Pipeline (circom.yml): heavy, needs circom and bun.
if touched '^internal/cougr-core-circuits/|^src/circuits/generated\.rs$|^\.github/workflows/circom\.yml$'; then
  if need circom "https://docs.circom.io/getting-started/installation/" "circom pipeline" \
    && need bun "https://bun.sh" "circom pipeline"; then
    run internal/cougr-core-circuits "circom: install" bun install --frozen-lockfile
    run internal/cougr-core-circuits "circom: pipeline" bun run pipeline
    run . "circom: integration tests" cargo test --test integration_circuits --features testutils
  fi
fi

# ---------------------------------------------------------------------------
# Docs and site (cougr-site.yml, docs-site.yml)

SITE_PATHS='^(cougr-site|docs)/|^examples/catalog\.toml$|^examples/.*/(README\.md|preview\.svg)$|^packages/tokens/tokens\.json$|^(ARCHITECTURE|CONTRIBUTING|CHANGELOG|SECURITY)\.md$|^\.github/workflows/(cougr-site|docs-site)\.yml$'
if touched "$SITE_PATHS"; then
  run . "site: sync cougr-site/src" python3 cougr-site/sync.py
  run . "site: cougr-site/src is committed" bash -c '
    if [ -n "$(git status --porcelain -- cougr-site/src)" ]; then
      git status --short -- cougr-site/src
      echo "cougr-site/src differs from its source docs. Commit the files above (make fix regenerates them)."
      exit 1
    fi'
  if need mdbook "cargo install mdbook --version 0.4.43" "site: mdbook"; then
    if touched '^(cougr-site|docs)/|^\.github/workflows/docs-site\.yml$'; then
      run cougr-site "site: mdbook test" mdbook test
    fi
    # generate-showcase.py rewrites two tracked files. Keep the contributor's copies.
    keep="$(mktemp -d)"
    cp cougr-site/src/SUMMARY.md "$keep/SUMMARY.md" 2>/dev/null
    cp cougr-site/src/showcase/gallery.md "$keep/gallery.md" 2>/dev/null
    run . "site: generate showcase" python3 cougr-site/generate-showcase.py
    # cougr-site.yml builds without mdbook-linkcheck (it is optional in book.toml).
    # A local linkcheck built for another mdbook version would fail the build, so stub it out.
    printf '#!/bin/sh\ncat >/dev/null\n' >"$keep/mdbook-linkcheck"
    chmod +x "$keep/mdbook-linkcheck"
    run cougr-site "site: mdbook build" env PATH="$keep:$PATH" mdbook build
    # docs-site.yml builds with mdbook 0.4.43 and mdbook-linkcheck 0.7.7, which rejects broken links.
    if touched '^(cougr-site|docs)/|^\.github/workflows/docs-site\.yml$'; then
      if mdbook --version 2>/dev/null | grep -q 'v0\.4\.' && command -v mdbook-linkcheck >/dev/null 2>&1; then
        run cougr-site "site: link check" mdbook build
      else
        SKIPPED+=("site: link check (needs mdbook 0.4.43 and mdbook-linkcheck 0.7.7: cargo install mdbook --version 0.4.43 --locked && cargo install mdbook-linkcheck --version 0.7.7 --locked)")
      fi
    fi
    [ -f "$keep/SUMMARY.md" ] && cp "$keep/SUMMARY.md" cougr-site/src/SUMMARY.md
    [ -f "$keep/gallery.md" ] && cp "$keep/gallery.md" cougr-site/src/showcase/gallery.md
    rm -rf "$keep"
  fi
fi

# Example Catalog (catalog.yml)
if touched '^examples/catalog.*\.toml$|^scripts/validate_catalog\.(py|sh)$|^\.github/workflows/catalog\.yml$'; then
  run . "catalog: validate examples/catalog.toml" python3 scripts/validate_catalog.py
fi

# Design Tokens (design-tokens.yml)
if touched '^packages/tokens/|^docs/BRAND\.md$|^\.github/workflows/design-tokens\.yml$'; then
  run packages/tokens "tokens: build --check" node build.js --check
  run packages/tokens "tokens: build" node build.js
  run . "tokens: dist is not committed" bash -c '
    if git ls-files --error-unmatch packages/tokens/dist >/dev/null 2>&1; then
      echo "packages/tokens/dist is generated and must not be committed."
      exit 1
    fi'
fi

# ---------------------------------------------------------------------------
# JavaScript packages, clients and studio

npm_ci() {
  local dir="$1"
  if [ -f "$dir/package-lock.json" ]; then
    run "$dir" "$dir: npm ci" npm ci --no-audit --no-fund
  fi
}

if touched '^packages/sdk-events/|^\.github/workflows/sdk-events\.yml$' && need npm "https://nodejs.org" "packages/sdk-events"; then
  npm_ci packages/sdk-events
  run packages/sdk-events "sdk-events: typecheck" npm run typecheck
  run packages/sdk-events "sdk-events: test" npm test
  run packages/sdk-events "sdk-events: build" npm run build
fi
for pkg in sdk-passkey sdk-session; do
  if touched "^packages/$pkg/|^\.github/workflows/$pkg\.yml$" && need npm "https://nodejs.org" "packages/$pkg"; then
    run "packages/$pkg" "$pkg: test" npm test
  fi
done

# clients/<dir> -> workflow file
for pair in checkpoint-web:checkpoint-client storageworld-web:storageworld-client web:turn-based-client; do
  client="${pair%%:*}"
  wf="${pair##*:}"
  if touched "^clients/$client/|^\.github/workflows/$wf\.yml$" && need npm "https://nodejs.org" "clients/$client"; then
    npm_ci "clients/$client"
    run "clients/$client" "clients/$client: test" npm test
    run "clients/$client" "clients/$client: build" npm run build
  fi
done

if touched '^clients/spectator/|^packages/sdk-events/|^\.github/workflows/spectator-client\.yml$' && need npm "https://nodejs.org" "clients/spectator"; then
  npm_ci packages/sdk-events
  run packages/sdk-events "spectator: build sdk-events" npm run build
  run clients/spectator "spectator: install" npm install --no-audit --no-fund
  run clients/spectator "spectator: typecheck" npm run typecheck
  run clients/spectator "spectator: test" npm test
  run clients/spectator "spectator: build" npm run build
fi

if touched '^clients/godot/|^\.github/workflows/godot-checkpoint-client\.yml$'; then
  run clients/godot/checkpoint_vectors "godot: rust vectors" cargo test -- --nocapture
  if need godot "https://godotengine.org/download (4.4.1)" "godot: GDScript vectors"; then
    run clients/godot "godot: GDScript vectors" godot --headless --path . --script res://addons/cougr_checkpoint/tests/run_tests.gd
  fi
fi

if touched '^studio/|^packages/tokens/tokens\.json$|^\.github/workflows/(studio|cougr-studio)\.yml$' && need npm "https://nodejs.org" "studio"; then
  npm_ci studio
  run studio "studio: npm test" npm test
  run packages/tokens "studio: build design tokens" node build.js
  run studio "studio: validate fixtures" node tools/validate-fixtures.js
  run studio "studio: node --test" node --test
fi

# ---------------------------------------------------------------------------
# Summary

echo
echo "${BOLD}Summary${RESET}"
for item in "${PASSED[@]+"${PASSED[@]}"}"; do echo "  ${GREEN}pass${RESET}  $item"; done
for item in "${SKIPPED[@]+"${SKIPPED[@]}"}"; do echo "  ${YELLOW}skip${RESET}  $item"; done
for item in "${FAILED[@]+"${FAILED[@]}"}"; do echo "  ${RED}FAIL${RESET}  $item"; done

if [ "${#SKIPPED[@]}" -gt 0 ]; then
  echo
  echo "${YELLOW}Skipped checks still run in CI. Install the tool shown to run them here.${RESET}"
fi
if [ "${#FAILED[@]}" -gt 0 ]; then
  echo
  echo "${RED}${#FAILED[@]} check(s) failed.${RESET} CI will fail the same way. Fix them before pushing."
  echo "Formatting and site sync can be fixed automatically with: make fix"
  exit 1
fi
echo
echo "${GREEN}All checks passed.${RESET} Read your diff once more (git diff), then push."
