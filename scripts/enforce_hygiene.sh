#!/usr/bin/env bash
#
# enforce_hygiene.sh - Enforce repository hygiene standards across examples/
#
# Issue: #225
# Run from repository root.
#

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"
FAILED=0

echo "=== Cougr Examples Hygiene Enforcement (#225) ==="
echo "Repository root: $REPO_ROOT"
echo ""

# =============================================================================
# 1. REMOVE COMMITTED target/ DIRECTORIES AND .wasm ARTIFACTS
# =============================================================================
echo "[1/5] Removing committed build artifacts..."

# Remove tracked target/ directories
find "$EXAMPLES_DIR" -type d -name "target" | while read -r target_dir; do
    echo "  Removing: ${target_dir#$REPO_ROOT/}"
    rm -rf "$target_dir"
done

# Remove any .wasm files
find "$EXAMPLES_DIR" -type f -name "*.wasm" | while read -r wasm_file; do
    echo "  Removing: ${wasm_file#$REPO_ROOT/}"
    rm -f "$wasm_file"
done

echo "  Done."
echo ""

# =============================================================================
# 2. ENSURE .gitignore IN EVERY EXAMPLE DIRECTORY
# =============================================================================
echo "[2/5] Ensuring .gitignore files..."

GITIGNORE_CONTENT='target/
**/*.rs.bk
*.wasm

.idea/
.vscode/
*.swp
*.swo

.DS_Store
Thumbs.db
'

for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ -d "$example_dir" ]; then
        gitignore_file="$example_dir/.gitignore"
        example_name=$(basename "$example_dir")
        
        if [ ! -f "$gitignore_file" ]; then
            echo "  Creating: examples/$example_name/.gitignore"
            printf '%s' "$GITIGNORE_CONTENT" > "$gitignore_file"
        else
            # Ensure target/ and *.wasm are in the existing .gitignore
            needs_update=false
            if ! grep -q "^target/" "$gitignore_file" 2>/dev/null; then
                needs_update=true
            fi
            if ! grep -q "^\\*\\.wasm" "$gitignore_file" 2>/dev/null; then
                needs_update=true
            fi
            
            if [ "$needs_update" = true ]; then
                echo "  Updating: examples/$example_name/.gitignore"
                {
                    echo ""
                    echo "# Added by hygiene enforcement (#225)"
                    grep -q "^target/" "$gitignore_file" 2>/dev/null || echo "target/"
                    grep -q "^\\*\\.wasm" "$gitignore_file" 2>/dev/null || echo "*.wasm"

                } >> "$gitignore_file"
            else
                echo "  OK: examples/$example_name/.gitignore"
            fi
        fi
    fi
done

echo "  Done."
echo ""

# =============================================================================
# 3. CORRECT Cargo.toml DESCRIPTION FIELDS
# =============================================================================
echo "[3/5] Correcting Cargo.toml descriptions..."

# Description mapping: directory_name -> correct_description
declare -A DESCRIPTIONS=(
    ["spawn_and_move"]="Canonical Cougr starter: ECS spawn and movement with observed events"
    ["tic_tac_toe"]="Turn-based tic-tac-toe with rich components and Address ownership"
    ["trading_card_game"]="Two-player trading card game with atomic batch turns and session keys"
    ["battleship"]="Battleship with commit-reveal hidden state using Merkle proofs"
    ["chess"]="Chess with on-chain move validation using Cougr ECS"
    ["asteroids"]="Arcade asteroid shooter with entity-heavy movement and collisions"
    ["snake"]="Snake arcade game with growth mechanics and collision rules"
    ["pong"]="Minimal competitive Pong loop demonstrating ECS fundamentals"
    ["tetris"]="Tetris puzzle with piece rotation and board clearing"
    ["rock_paper_scissors"]="Commit-reveal hidden choice game"
    ["flappy_bird"]="Reflex arcade game with tight tick-loop updates"
    ["space_invaders"]="Wave shooter with formation movement and tick systems"
    ["pac_man"]="Maze action with grid navigation and adversarial movement"
    ["guild_arena"]="Account patterns: social recovery and multi-device gameplay"
    ["proof_of_hunt"]="ZK proof verification and x402 premium actions"
    ["treasure_hunt"]="Merkle map commitments with fog-of-war proof-gated discovery"
    ["angry_birds"]="Projectile physics and destructible-state gameplay"
    ["arkanoid"]="Paddle collision and brick lifecycle management"
    ["bomberman"]="Grid action with tile updates and timed hazards"
    ["geometry_dash"]="Deterministic timing and obstacle progression"
    ["murdoku"]="Puzzle with ephemeral ECS validation and creator registry"
    ["pokemon_mini"]="Turn-based combat sequencing and match state transitions"
    ["tap_battle"]="Lightweight casual competitive action resolution"
)

for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ ! -d "$example_dir" ]; then
        continue
    fi
    
    example_name=$(basename "$example_dir")
    cargo_toml="$example_dir/Cargo.toml"
    
    if [ ! -f "$cargo_toml" ]; then
        echo "  Warning: No Cargo.toml in $example_name"
        continue
    fi
    
    # Check if description exists and what it currently says
    current_desc=$(grep -E "^description\s*=" "$cargo_toml" 2>/dev/null | sed 's/.*=\s*"\(.*\)".*/\1/' || true)
    
    if [ -n "${DESCRIPTIONS[$example_name]+x}" ]; then
        correct_desc="${DESCRIPTIONS[$example_name]}"
        
        if [ "$current_desc" != "$correct_desc" ]; then
            echo "  Fixing: $example_name"
            echo "    Was: ${current_desc:-<missing>}"
            echo "    Now: $correct_desc"
            
            if grep -q "^description" "$cargo_toml"; then
                # Replace existing description
                sed -i.bak "s/^description\s*=.*/description = \"$correct_desc\"/" "$cargo_toml"
                rm -f "$cargo_toml.bak"
            else
                # Add description after [package] header
                sed -i.bak "/^\[package\]/a description = \"$correct_desc\"" "$cargo_toml"
                rm -f "$cargo_toml.bak"
            fi
        else
            echo "  OK: $example_name"
        fi
    else
        echo "  Warning: No description mapping for $example_name"
    fi
done

echo "  Done."
echo ""

# =============================================================================
# 4. SANITIZE README.md FILES
# =============================================================================
echo "[4/5] Sanitizing README.md files..."

PYTHON_SCRIPT="$REPO_ROOT/scripts/sanitize_readme.py"

for readme in "$EXAMPLES_DIR"/*/README.md; do
    if [ ! -f "$readme" ]; then
        continue
    fi
    
    example_name=$(basename "$(dirname "$readme")")
    echo "  Processing: $example_name/README.md"
    
    # Run Python sanitizer
    python3 "$PYTHON_SCRIPT" "$readme"
    
    # Additional sed-based cleanup for edge cases
    sed -i.bak -E 's/--network (testnet|mainnet|futurenet)/--network <NETWORK>/g' "$readme"
    sed -i.bak -E 's/--source [a-zA-Z0-9_]+/--source <ACCOUNT>/g' "$readme"
    sed -i.bak -E 's/🚀 ?//g; s/🔥 ?//g; s/⭐ ?//g; s/💎 ?//g; s/🎮 ?//g; s/🎯 ?//g; s/✨ ?//g; s/⚡ ?//g; s/💪 ?//g' "$readme"
    
    rm -f "$readme.bak"
done

echo "  Done."
echo ""

# =============================================================================
# 5. VERIFY cargo metadata --no-deps
# =============================================================================
echo "[5/5] Verifying cargo metadata for all examples..."

for example_dir in "$EXAMPLES_DIR"/*/; do
    if [ ! -d "$example_dir" ]; then
        continue
    fi
    
    example_name=$(basename "$example_dir")
    echo -n "  Checking: $example_name ... "
    
    if (cd "$example_dir" && cargo metadata --no-deps --format-version 1 >/dev/null 2>&1); then
        echo "OK"
    else
        echo "FAILED"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "=== Summary ==="
echo "Build artifacts removed: ✓"
echo ".gitignore files ensured: ✓"
echo "Cargo.toml descriptions corrected: ✓"
echo "README.md files sanitized: ✓"

if [ $FAILED -eq 0 ]; then
    echo "cargo metadata verification: ALL PASSED ✓"
    echo ""
    echo "Definition of done verification:"
    echo "  git ls-files 'examples/**/target/**'  # Should return nothing"
    echo "  grep -rE 'C[A-Z2-7]{55}' examples/*/README.md || echo 'No contract IDs found ✓'"
    echo ""
    echo "Next steps:"
    echo "  1. Review: git diff"
    echo "  2. Stage:  git add examples/"
    echo "  3. Commit: git commit -m 'chore(examples): enforce repository hygiene standards (#225)'"
    echo "  4. Push:   git push origin <branch>"
else
    echo "cargo metadata verification: $FAILED FAILED ⚠"
    echo "Please review the failing examples manually."
fi