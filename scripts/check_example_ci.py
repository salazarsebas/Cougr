#!/usr/bin/env python3
"""Verify every Rust example is referenced by a root GitHub Actions workflow."""

from pathlib import Path


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    examples = sorted((root / "examples").glob("*/Cargo.toml"))
    workflows = (root / ".github" / "workflows").glob("*.yml")
    workflow_text = "\n".join(path.read_text(encoding="utf-8") for path in workflows)
    uncovered = [
        cargo_toml.parent.name
        for cargo_toml in examples
        if f"examples/{cargo_toml.parent.name}/" not in workflow_text
    ]

    if uncovered:
        print("Examples missing a CI workflow reference:")
        for name in uncovered:
            print(f"  - {name}")
        return 1

    print(f"All {len(examples)} Rust examples are referenced by CI workflows.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
