#!/usr/bin/env python3
"""
sanitize_readme.py - Remove marketing sections and deployment artifacts from example READMEs.

Issue: #225
Usage: python3 scripts/sanitize_readme.py examples/<example>/README.md
"""

import re
import sys
import argparse
from pathlib import Path

# Sections to remove entirely (marketing/promotional content that impedes clarity)
REMOVE_SECTIONS = [
    r"## Resources\s*\n.*?(\n## |\n# |\Z)",  # Generic "Resources" with external links
    r"## Community\s*\n.*?(\n## |\n# |\Z)",
    r"## Follow us\s*\n.*?(\n## |\n# |\Z)",
    r"## Social\s*\n.*?(\n## |\n# |\Z)",
    r"## Join our community\s*\n.*?(\n## |\n# |\Z)",
    r"## Links\s*\n.*?(\n## |\n# |\Z)",
    r"## External links\s*\n.*?(\n## |\n# |\Z)",
    r"### Discord\s*\n.*?(\n### |\n## |\n# |\Z)",
    r"### Twitter\s*\n.*?(\n### |\n## |\n# |\Z)",
    r"### Telegram\s*\n.*?(\n### |\n## |\n# |\Z)",
    r"### GitHub\s*\n.*?(\n### |\n## |\n# |\Z)",
    r"### Medium\s*\n.*?(\n### |\n## |\n# |\Z)",
    r"### Blog\s*\n.*?(\n### |\n## |\n# |\Z)",
]

# Deployment output blocks to remove (within code blocks)
DEPLOYMENT_PATTERNS = [
    r"```\n# Deployed to testnet\nContract ID: [A-Z2-7]{56}\nTransaction Hash: [a-f0-9]{64}\n```",
    r"Contract ID: `C[A-Z2-7]{55}`",
    r"Transaction Hash: `[a-f0-9]{64}`",
    r"\| Contract ID\s*\| `C[A-Z2-7]{55}`\s*\|",
    r"\| Transaction Hash\s*\| `[a-f0-9]{64}`\s*\|",
    r"\| Network\s*\| Stellar (Testnet|Mainnet|Futurenet)\s*\|",
    r"\| Explorer\s*\|.*?\|",
]

# Hardcoded deployment command examples with real IDs
DEPLOYMENT_COMMAND_PATTERNS = [
    r"stellar contract invoke\s+\\\n\s+--id C[A-Z2-7]{55}\s+\\\n.*?```",
    r"stellar contract deploy\s+.*?--id C[A-Z2-7]{55}.*?```",
]


def remove_marketing_sections(content: str) -> str:
    """Remove marketing/promotional sections that impede technical clarity."""
    for pattern in REMOVE_SECTIONS:
        content = re.sub(pattern, r"\1", content, flags=re.DOTALL | re.IGNORECASE)
    return content


def sanitize_deployment_artifacts(content: str) -> str:
    """Remove hardcoded deployment identifiers and results."""
    for pattern in DEPLOYMENT_PATTERNS:
        content = re.sub(pattern, "", content, flags=re.DOTALL | re.IGNORECASE)
    
    for pattern in DEPLOYMENT_COMMAND_PATTERNS:
        content = re.sub(pattern, "```", content, flags=re.DOTALL | re.IGNORECASE)
    
    # Replace remaining hardcoded contract IDs in prose
    content = re.sub(r"C[A-Z2-7]{55}", "<CONTRACT_ID>", content)
    
    # Replace transaction hashes
    content = re.sub(r"[a-f0-9]{64}", "<TRANSACTION_HASH>", content)
    
    return content


def normalize_documentation_tone(content: str) -> str:
    """Normalize README tone to technical documentation standards."""
    # Remove excessive exclamation marks
    content = re.sub(r"!\s*!", "!", content)
    
    # Remove ALL CAPS marketing phrases
    marketing_phrases = [
        r"REVOLUTIONARY",
        r"GAME-CHANGING",
        r"CUTTING-EDGE",
        r"NEXT-GENERATION",
        r"STATE-OF-THE-ART",
        r"INDUSTRY-LEADING",
        r"WORLD-CLASS",
        r"BEST-IN-CLASS",
        r"UNPARALLELED",
        r"UNRIVALED",
        r"GROUND-BREAKING",
    ]
    for phrase in marketing_phrases:
        content = re.sub(phrase, "", content, flags=re.IGNORECASE)
    
    # Remove "Why [Product]?" sections that are pure marketing
    content = re.sub(
        r"## Why [A-Za-z]+\?\s*\n.*?(\n## |\n# |\Z)",
        r"\1",
        content,
        flags=re.DOTALL,
    )
    
    # Remove comparison tables that only market the framework
    # (keep tables that compare technical approaches)
    
    return content


def sanitize_readme(filepath: Path) -> None:
    """Main sanitization function."""
    content = filepath.read_text(encoding="utf-8")
    original = content
    
    content = remove_marketing_sections(content)
    content = sanitize_deployment_artifacts(content)
    content = normalize_documentation_tone(content)
    
    # Clean up multiple consecutive blank lines
    content = re.sub(r"\n{3,}", "\n\n", content)
    
    if content != original:
        backup = filepath.with_suffix(".md.bak")
        backup.write_text(original, encoding="utf-8")
        filepath.write_text(content, encoding="utf-8")
        print(f"  Sanitized: {filepath}")
    else:
        print(f"  No changes: {filepath}")


def main():
    parser = argparse.ArgumentParser(description="Sanitize example README.md files")
    parser.add_argument("files", nargs="+", help="README.md files to sanitize")
    args = parser.parse_args()
    
    for filepath_str in args.files:
        filepath = Path(filepath_str)
        if not filepath.exists():
            print(f"  Not found: {filepath}")
            continue
        sanitize_readme(filepath)


if __name__ == "__main__":
    main()