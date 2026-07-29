#!/usr/bin/env python3
import os
import re
import shutil
import subprocess
import tempfile

REPO_URL = "https://github.com/salazarsebas/Cougr.git"

# Mapping from source repo path to mdBook src path
MAPPING = {
    "docs/start/build-your-first-game.md": "start/build-your-first-game.md",
    "ARCHITECTURE.md": "learn/ARCHITECTURE.md",
    "docs/PATTERNS.md": "learn/PATTERNS.md",
    "docs/ECS_CORE.md": "reference/ECS_CORE.md",
    "docs/ACCOUNT_KERNEL.md": "reference/ACCOUNT_KERNEL.md",
    "docs/STANDARDS_LAYER.md": "reference/STANDARDS_LAYER.md",
    "docs/PRIVACY_MODEL.md": "reference/PRIVACY_MODEL.md",
    "docs/FEATURE_FLAGS.md": "reference/FEATURE_FLAGS.md",
    "docs/PERFORMANCE.md": "reference/PERFORMANCE.md",
    "docs/API_CONTRACT.md": "reference/API_CONTRACT.md",
    "docs/COMPATIBILITY_PROMISES.md": "reference/COMPATIBILITY_PROMISES.md",
    "docs/MIGRATION_GUIDE.md": "reference/MIGRATION_GUIDE.md",
    "CONTRIBUTING.md": "community/CONTRIBUTING.md",
    "CHANGELOG.md": "community/CHANGELOG.md",
    "SECURITY.md": "community/SECURITY.md",
}

def sync():
    with tempfile.TemporaryDirectory() as tmpdir:
        print(f"Cloning {REPO_URL} into {tmpdir}...")
        # Use LOCAL_COUGR_PATH for fast local testing if available
        local_path = os.environ.get("LOCAL_COUGR_PATH")
        if local_path and os.path.exists(local_path):
            print(f"Using local path: {local_path}")
            repo_path = local_path
        else:
            subprocess.run(["git", "clone", "--depth", "1", REPO_URL, tmpdir], check=True)
            repo_path = tmpdir

        # Discover adr files dynamically
        adr_dir = os.path.join(repo_path, "docs/adr")
        if os.path.exists(adr_dir):
            for filename in os.listdir(adr_dir):
                if filename.endswith(".md"):
                    repo_rel = f"docs/adr/{filename}"
                    mdbook_rel = f"reference/adr/{filename}"
                    MAPPING[repo_rel] = mdbook_rel

        # Copy files and rewrite links
        for repo_rel, mdbook_rel in MAPPING.items():
            src_file = os.path.join(repo_path, repo_rel)
            if not os.path.exists(src_file):
                print(f"Warning: {repo_rel} not found in source repo.")
                continue

            with open(src_file, "r") as f:
                content = f.read()

            content = rewrite_links(content, repo_rel)

            dest_file = os.path.join("src", mdbook_rel)
            os.makedirs(os.path.dirname(dest_file), exist_ok=True)
            with open(dest_file, "w") as f:
                f.write(content)
            print(f"Synced {repo_rel} -> src/{mdbook_rel}")

def rewrite_links(content, current_file_repo_rel):
    """
    Finds markdown links and rewrites them if they point to another mapped file.
    """
    def replacer(match):
        full_match = match.group(0)
        link_text = match.group(1)
        link_url = match.group(2)

        # Ignore external links, anchors, mailto
        if link_url.startswith(("http", "mailto", "#")):
            return full_match
        
        # Split anchor from url
        anchor = ""
        if "#" in link_url:
            link_url, anchor = link_url.split("#", 1)
            anchor = "#" + anchor
            
        if not link_url:
            return full_match

        # Normalize target path relative to the repo root
        current_dir = os.path.dirname(current_file_repo_rel)
        target_repo_rel = os.path.normpath(os.path.join(current_dir, link_url))

        # Windows paths normpath fix
        target_repo_rel = target_repo_rel.replace('\\', '/')

        if target_repo_rel in MAPPING:
            # We have a mapping for the target!
            # Calculate new relative path in mdBook
            current_mdbook_rel = MAPPING[current_file_repo_rel]
            target_mdbook_rel = MAPPING[target_repo_rel]
            
            current_mdbook_dir = os.path.dirname(current_mdbook_rel)
            new_rel_path = os.path.relpath(target_mdbook_rel, current_mdbook_dir)
            new_rel_path = new_rel_path.replace('\\', '/')
            
            return f"[{link_text}]({new_rel_path}{anchor})"
            
        return full_match

    # Regex for standard markdown links: [text](url)
    pattern = re.compile(r'\[([^\]]+)\]\(([^)]+)\)')
    return pattern.sub(replacer, content)

if __name__ == "__main__":
    sync()
