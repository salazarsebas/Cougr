#!/usr/bin/env python3
"""Validate examples/catalog.toml against the filesystem and schema rules.

Checks:
  1. catalog.toml is valid TOML
  2. Every entry has all required fields (name, category, maturity, cougr_features)
  3. category is one of the allowed values
  4. maturity is canonical or transitional
  5. Every entry's name matches a directory under examples/
  6. cougr_features is a non-empty list
  7. If screenshot is set, the file exists
  8. All 10 canonical examples from the standard are present
  9. Every example directory with a Cargo.toml has a catalog entry
     (skips non-directories and directories without a Cargo.toml)
"""

import os
import sys

# tomllib is part of the standard library from Python 3.11 onward. Fail with a
# clear message instead of a ModuleNotFoundError traceback on older versions.
if sys.version_info < (3, 11):
    sys.exit(
        "validate_catalog.py requires Python 3.11 or newer "
        f"(this interpreter is {sys.version.split()[0]}), because it uses the "
        "standard-library tomllib module."
    )

import tomllib

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = sys.argv[1] if len(sys.argv) > 1 else os.path.join(REPO_ROOT, "examples", "catalog.toml")
EXAMPLES_DIR = os.path.join(REPO_ROOT, "examples")

VALID_CATEGORIES = {"arcade", "board", "puzzle", "hidden-information", "card", "other"}
VALID_MATURITIES = {"canonical", "transitional"}
REQUIRED_CANONICAL = [
    "spawn_and_move",
    "tic_tac_toe",
    "session_arena",
    "hidden_hand",
    "fog_explorer",
    "dice_duel",
    "blind_auction",
    "snake",
    "battleship",
    "guild_arena",
]

errors = []


def fail(msg):
    errors.append(msg)


def check_example_dirs_in_catalog(entries):
    """Every example directory with a Cargo.toml must have a catalog entry."""
    catalogued = {entry.get("name") for entry in entries.values()}
    if not os.path.isdir(EXAMPLES_DIR):
        return
    for name in sorted(os.listdir(EXAMPLES_DIR)):
        example_dir = os.path.join(EXAMPLES_DIR, name)
        if not os.path.isdir(example_dir):
            continue
        if not os.path.isfile(os.path.join(example_dir, "Cargo.toml")):
            continue
        if name not in catalogued:
            fail(f'example directory "examples/{name}" is not in the catalog')


if not os.path.isfile(CATALOG):
    fail(f"catalog.toml not found at {CATALOG}")
    for e in errors:
        print(f"FAIL: {e}")
    sys.exit(1)

with open(CATALOG, "rb") as f:
    try:
        data = tomllib.load(f)
    except Exception as e:
        fail(f"Invalid TOML in catalog.toml: {e}")
        for e in errors:
            print(f"FAIL: {e}")
        sys.exit(1)

# TOML [example.<name>] sections are parsed as a nested "example" table
example_table = data.get("example", {})
entries = {("example." + k): v for k, v in example_table.items()}
if not entries:
    fail("No [example.*] sections found in catalog.toml")

# Check all required canonical examples are present
canonical_names = set()
for key, entry in entries.items():
    if entry.get("maturity") == "canonical":
        canonical_names.add(entry.get("name"))

for cn in REQUIRED_CANONICAL:
    if cn not in canonical_names:
        fail(f"Missing required canonical example: {cn}")

for key, entry in entries.items():
    name = entry.get("name")

    # Check required fields
    for field in ["name", "category", "maturity", "cougr_features"]:
        if field not in entry:
            fail(f"{key}: missing required field \"{field}\"")

    if not name:
        continue

    example_dir = os.path.join(EXAMPLES_DIR, name)

    # Check example directory exists
    if not os.path.isdir(example_dir):
        fail(f'{key}: example directory "{name}" does not exist at examples/{name}')

    category = entry.get("category")
    if category and category not in VALID_CATEGORIES:
        fail(f'{key}: invalid category "{category}" (must be one of: {", ".join(sorted(VALID_CATEGORIES))})')

    maturity = entry.get("maturity")
    if maturity and maturity not in VALID_MATURITIES:
        fail(f'{key}: invalid maturity "{maturity}" (must be canonical or transitional)')

    cougr_features = entry.get("cougr_features")
    if cougr_features is not None:
        if not isinstance(cougr_features, list):
            fail(f"{key}: cougr_features must be a list")
        elif len(cougr_features) == 0:
            fail(f"{key}: cougr_features must be a non-empty list")

    screenshot = entry.get("screenshot")
    if screenshot:
        screenshot_path = os.path.join(REPO_ROOT, screenshot)
        if not os.path.isfile(screenshot_path):
            fail(f'{key}: screenshot file "{screenshot}" not found')

check_example_dirs_in_catalog(entries)

if errors:
    for e in errors:
        print(f"FAIL: {e}")
    sys.exit(1)

print("All catalog validation checks passed")
