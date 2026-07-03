#!/usr/bin/env python3
"""Sanitize example README.md files for issue #225."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CONTRACT_ID_RE = re.compile(r"C[A-Z2-7]{55}")
TX_HASH_RE = re.compile(r"\b[a-f0-9]{64}\b")
PROMO_EMOJI_RE = re.compile(
    r"[\U0001F300-\U0001FAFF\U00002600-\U000027BF]"
    r"|✅|✓|🔗|🟢|🔴|🟡|⚡|💎|⭐|🔥|🚀|🎮|🎯|✨|💪|📦|🏆|🔧"
)

DEPLOYMENT_PATTERNS = [
    re.compile(r"^\|\s*\*?\*?Contract ID\*?\*?\s*\|.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\|\s*\*?\*?Transaction Hash\*?\*?\s*\|.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\|\s*\*?\*?Explorer\*?\*?\s*\|.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\|\s*\*?\*?Network\*?\*?\s*\|\s*Stellar (Testnet|Mainnet|Futurenet).*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\|\s*Testnet\s*\|.*C[A-Z2-7]{55}.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\*\*✅ Successfully Deployed.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^>\s*🔗.*$", re.MULTILINE),
    re.compile(r"^>\s*\*\*Contract ID:\*\*.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^CONTRACT_ID=\"C[A-Z2-7]{55}\".*$", re.MULTILINE),
    re.compile(r"^\*\*Contract ID\*\*:\s*`C[A-Z2-7]{55}`.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\*\*Explorer Link\*\*:.*C[A-Z2-7]{55}.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^### this the deployed testnet link.*$", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\*\*Deployed Contract:\*\*.*$", re.MULTILINE | re.IGNORECASE),
]


def sanitize_readme(filepath: Path) -> bool:
    original = filepath.read_text(encoding="utf-8")
    content = original

    for pattern in DEPLOYMENT_PATTERNS:
        content = pattern.sub("", content)

    content = CONTRACT_ID_RE.sub("<CONTRACT_ID>", content)

    lines = []
    for line in content.splitlines(keepends=True):
        if re.search(r"(transaction|hash|deploy|tx)", line, re.IGNORECASE):
            line = TX_HASH_RE.sub("<TRANSACTION_HASH>", line)
        lines.append(line)
    content = "".join(lines)

    content = re.sub(r"--id\s+C[A-Z2-7]{55}", "--id <CONTRACT_ID>", content)
    content = re.sub(
        r"--network\s+(testnet|mainnet|futurenet)\b",
        "--network <NETWORK>",
        content,
        flags=re.IGNORECASE,
    )

    cleaned_lines = []
    for line in content.splitlines(keepends=True):
        if line.lstrip().startswith("#") or re.match(r"^>\s", line) or re.match(r"^\|\s*[^|]+\|\s*[🟢✅🔗]", line):
            line = PROMO_EMOJI_RE.sub("", line)
            line = re.sub(r"\s{2,}", " ", line)
        cleaned_lines.append(line)
    content = "".join(cleaned_lines)
    content = re.sub(r"\n{3,}", "\n\n", content)

    if content != original:
        filepath.write_text(content, encoding="utf-8")
        return True
    return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="+")
    args = parser.parse_args()
    for filepath_str in args.files:
        filepath = Path(filepath_str)
        if not filepath.exists():
            print(f"  Not found: {filepath}", file=sys.stderr)
            continue
        print(f"  {'Sanitized' if sanitize_readme(filepath) else 'No changes'}: {filepath}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())