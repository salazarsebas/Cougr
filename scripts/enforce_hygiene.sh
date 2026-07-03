#!/usr/bin/env bash
# enforce_hygiene.sh — Enforce repository hygiene standards across examples/ (#225)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
EXAMPLES_DIR="$REPO_ROOT/examples"

echo "=== Cougr Examples Hygiene Enforcement (#225) ==="

while IFS= read -r target_dir; do
    rm -rf "$target_dir" || true
done < <(find "$EXAMPLES_DIR" -type d -name "target")
while IFS= read -r wasm_file; do
    rm -f "$wasm_file" || true
done < <(find "$EXAMPLES_DIR" -type f -name "*.wasm")

GITIGNORE_CONTENT='# Build artifacts
target/
*.wasm

# Soroban
.soroban/
test_snapshots/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db
'

for example_dir in "$EXAMPLES_DIR"/*/; do
    [ -d "$example_dir" ] || continue
    gitignore_file="$example_dir/.gitignore"
    if [ ! -f "$gitignore_file" ]; then
        printf '%s' "$GITIGNORE_CONTENT" > "$gitignore_file"
    else
        grep -qE '^Cargo\.lock$' "$gitignore_file" && sed -i.bak '/^Cargo\.lock$/d' "$gitignore_file" && rm -f "$gitignore_file.bak"
        grep -qE '^(/)?target/' "$gitignore_file" || printf '\ntarget/\n' >> "$gitignore_file"
        grep -qE '^\*\.wasm' "$gitignore_file" || printf '*.wasm\n' >> "$gitignore_file"
    fi
done

python3 - "$EXAMPLES_DIR" <<'PY'
import re, sys
from pathlib import Path
examples_dir = Path(sys.argv[1])
descriptions = {
    "ai_dungeon_master_arena": "AI Dungeon Master Arena with x402-paid actions and stellar-zk run proofs",
    "angry_birds": "Angry Birds turn-based physics puzzle using Cougr-Core ECS on Stellar Soroban",
    "arkanoid": "Arkanoid on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "asteroids": "Arcade asteroid shooter with entity movement and collision systems",
    "battleship": "Battleship with hidden board using commit-reveal pattern on Stellar Soroban",
    "blind_auction": "Canonical Cougr ZK example: sealed bids via cougr_core::circuits::sealed_bid",
    "bomberman": "Grid action game with tile updates and timed hazards on Stellar Soroban",
    "checkers": "On-chain Checkers game example using Cougr ECS on Soroban",
    "chess": "Verifiable Chess with ZK move validation using Cougr-Core on Stellar Soroban",
    "connect_four": "Connect Four on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "cross_asset_racing_league": "Cross-asset racing league with payment-driven gameplay and stellar-zk anti-cheat",
    "dice_duel": "Canonical Cougr ZK example: fair dice via cougr_core::circuits::fair_dice",
    "flappy_bird": "Flappy Bird on-chain game example using cougr-core ECS framework",
    "fog_explorer": "Canonical Cougr ZK example: fog of war via cougr_core::circuits::fog_of_war",
    "geometry_dash": "Geometry Dash on-chain game example using cougr-core ECS framework",
    "guild_arena": "PvP arena game with guild-based social recovery and multi-device support using Cougr-Core",
    "guild_treasury_wars": "Guild Treasury Wars with DAO-governed factions and stellar-zk commitments",
    "hidden_hand": "Canonical Cougr ZK example: hidden card deals via cougr_core::circuits::hidden_cards",
    "memory_match": "Memory Match card game on-chain using Cougr-Core ECS framework on Stellar Soroban",
    "minesweeper": "Minesweeper on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "murdoku": "Murdoku puzzle registry and creator contract using cougr-core ECS framework",
    "pac_man": "Pac-Man on-chain game example using Cougr-Core for Stellar/Soroban",
    "pokemon_mini": "Pokémon Mini on-chain game example using cougr-core ECS framework",
    "pong": "Pong on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "proof_of_hunt": "Proof-of-Hunt Soroban example with stellar-zk proof validation and x402 premium actions",
    "reversi": "Reversi on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "rock_paper_scissors": "Rock Paper Scissors with commit-reveal using Poseidon2 hashing on Stellar Soroban",
    "session_arena": "Canonical Cougr session UX example: approve once, play frictionlessly, renew on expiry",
    "shadow_draft_card_game": "Shadow Draft card game with hidden-hand draft gameplay and stellar-zk card validation",
    "snake": "Snake on-chain game example using cougr-core ECS framework",
    "space_invaders": "Space Invaders on-chain game example using cougr-core",
    "spawn_and_move": "Canonical Cougr starter: spawn a player entity and move it around a 2D world on Stellar Soroban",
    "sudoku": "Sudoku puzzle on-chain using Cougr-Core ECS framework on Stellar Soroban",
    "tap_battle": "Tap Battle on-chain game with passkey authentication using cougr-core",
    "tetris": "Tetris puzzle game with piece rotation and board clearing on Stellar Soroban",
    "tic_tac_toe": "Tic Tac Toe on-chain game using Cougr-Core ECS framework on Stellar Soroban",
    "tower_defense": "Tower defense on-chain game example using cougr-core ECS framework",
    "trading_card_game": "Two-player trading card game with atomic batch turns and session keys",
    "treasure_hunt": "Treasure Hunt Soroban example using Merkle map commitments and sparse fog-of-war",
}
for example_dir in sorted(examples_dir.iterdir()):
    if not example_dir.is_dir(): continue
    cargo = example_dir / "Cargo.toml"
    if not cargo.exists(): continue
    text = cargo.read_text(encoding="utf-8")
    changed = False
    if example_dir.name == "trading_card_game" and 'name = "tower_defense"' in text:
        text = text.replace('name = "tower_defense"', 'name = "trading_card_game"', 1)
        changed = True
    desc = descriptions.get(example_dir.name)
    if desc:
        m = re.search(r'^description\s*=\s*"(.*)"\s*$', text, re.MULTILINE)
        cur = m.group(1) if m else None
        if cur != desc:
            if m:
                text = re.sub(r'^description\s*=.*$', f'description = "{desc}"', text, count=1, flags=re.MULTILINE)
            else:
                text = text.replace("[package]\n", f'[package]\ndescription = "{desc}"\n', 1)
            changed = True
    if example_dir.name == "murdoku" and "[workspace]" not in text:
        text = text.rstrip() + "\n\n[workspace]\n"
        changed = True
    if changed:
        cargo.write_text(text, encoding="utf-8")
PY

TIC_TAC_TOE_README="$EXAMPLES_DIR/tic_tac_toe/README.md"
if [ -f "$TIC_TAC_TOE_README" ] && grep -q '&gt;' "$TIC_TAC_TOE_README" 2>/dev/null; then
    git -C "$REPO_ROOT" show main:examples/tic_tac_toe/README.md > "$TIC_TAC_TOE_README"
fi
for readme in "$EXAMPLES_DIR"/*/README.md; do
    [ -f "$readme" ] && python3 "$REPO_ROOT/scripts/sanitize_readme.py" "$readme"
done

grep -qE '^Cargo\.lock$' "$REPO_ROOT/.gitignore" 2>/dev/null && sed -i.bak '/^Cargo\.lock$/d' "$REPO_ROOT/.gitignore" && rm -f "$REPO_ROOT/.gitignore.bak"

"$REPO_ROOT/scripts/verify_hygiene.sh"