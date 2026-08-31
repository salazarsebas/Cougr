#!/usr/bin/env python3
"""
Cougr Showcase Gallery Generator

Reads ``examples/catalog.toml`` and each example's ``README.md`` and generates:
  - ``src/showcase/gallery.md``         — index page with client-side filtering
  - ``src/showcase/<name>.md``          — detail page per cataloged example
  - ``src/showcase/previews/<name>.svg`` — copied preview images (when present)
  - ``theme/showcase.css``              — design tokens + gallery styles
  - Updates ``src/SUMMARY.md``          — adds generated detail page entries

Architectural decision
----------------------
Integrated static generator (Python, matching the existing ``sync.py`` pattern)
that runs as a single build step before ``mdbook build``. Zero backend, zero
database — 100% static output. Design tokens are consumed directly from
``packages/tokens/tokens.json`` (the single source of truth) rather than
duplicated.

Usage::

    python3 cougr-site/generate-showcase.py
"""

import html
import json
import os
import re
import shutil
import sys

try:
    import tomllib
except ImportError:
    try:
        import tomli as tomllib  # type: ignore
    except ImportError:
        sys.exit(
            "Error: Python 3.11+ (tomllib) or the 'tomli' package is required."
        )

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

SITE_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(SITE_DIR)
EXAMPLES_DIR = os.path.join(REPO_ROOT, "examples")
CATALOG_PATH = os.path.join(EXAMPLES_DIR, "catalog.toml")
TOKENS_PATH = os.path.join(REPO_ROOT, "packages", "tokens", "tokens.json")
SHOWCASE_SRC = os.path.join(SITE_DIR, "src", "showcase")
PREVIEWS_DIR = os.path.join(SHOWCASE_SRC, "previews")
THEME_DIR = os.path.join(SITE_DIR, "theme")
SUMMARY_PATH = os.path.join(SITE_DIR, "src", "SUMMARY.md")

GITHUB_EXAMPLES = "https://github.com/salazarsebas/Cougr/tree/main/examples"
GITHUB_BLOB = "https://github.com/salazarsebas/Cougr/blob/main"

# ---------------------------------------------------------------------------
# Maturity / category mapping
# ---------------------------------------------------------------------------

# Catalog maturity values map to design-token tier colours.
MATURITY_TIER = {
    "canonical": ("stable", "Canonical"),
    "transitional": ("beta", "Transitional"),
    "experimental": ("experimental", "Experimental"),
}

CATEGORY_LABELS = {
    "arcade": "Arcade",
    "board": "Board",
    "puzzle": "Puzzle",
    "hidden-information": "Hidden Information",
    "card": "Card",
    "other": "Other",
}


# ---------------------------------------------------------------------------
# Data loading
# ---------------------------------------------------------------------------

def load_catalog():
    """Read and parse ``examples/catalog.toml``."""
    with open(CATALOG_PATH, "rb") as f:
        data = tomllib.load(f)
    return data.get("example", {})


def load_tokens():
    """Read and parse ``packages/tokens/tokens.json``."""
    with open(TOKENS_PATH, "r", encoding="utf-8") as f:
        return json.load(f)


def read_readme(name):
    """Return the raw text of ``examples/<name>/README.md`` or *None*."""
    path = os.path.join(EXAMPLES_DIR, name, "README.md")
    if not os.path.exists(path):
        return None
    with open(path, "r", encoding="utf-8") as f:
        return f.read()


# ---------------------------------------------------------------------------
# README parsing
# ---------------------------------------------------------------------------

def extract_title(readme):
    """Extract the first ``#`` heading text."""
    match = re.search(r"^#\s+(.+)$", readme, re.MULTILINE)
    return match.group(1).strip() if match else None


def extract_description(readme):
    """Extract a short plain-text description from the README.

    Strategy: find the ``## Purpose and pattern`` section and return its first
    paragraph.  Fall back to the first non-blockquote paragraph after the title.
    """
    # Try the "## Purpose and pattern" section.
    purpose_match = re.search(
        r"^##\s+Purpose and pattern\s*\n((?:.|\n)*?)(?=\n##\s|\Z)",
        readme,
        re.MULTILINE,
    )
    if purpose_match:
        section = purpose_match.group(1).strip()
        para = section.split("\n\n")[0].strip()
        if para:
            return _clean_markdown_inline(para)

    # Fallback: first paragraph after the title, skipping blockquotes.
    lines = readme.split("\n")
    after_title = False
    in_blockquote = False
    para_lines = []

    for line in lines:
        if line.startswith("# "):
            after_title = True
            continue
        if not after_title:
            continue
        if line.startswith(">"):
            in_blockquote = True
            continue
        if in_blockquote:
            if line.strip() == "":
                in_blockquote = False
            continue
        if line.strip():
            para_lines.append(line)
        elif para_lines:
            break

    if para_lines:
        return _clean_markdown_inline("\n".join(para_lines))
    return ""


def _clean_markdown_inline(text):
    """Strip simple inline markdown (bold, code, links) for plain-text display."""
    text = re.sub(r"\*\*([^*]+)\*\*", r"\1", text)
    text = re.sub(r"\*([^*]+)\*", r"\1", text)
    text = re.sub(r"`([^`]+)`", r"\1", text)
    text = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", r"\1", text)
    return text.strip()


def rewrite_readme_links(content, example_name):
    """Rewrite relative markdown links to absolute GitHub URLs."""
    def replacer(match):
        text = match.group(1)
        url = match.group(2)

        if url.startswith(("http", "mailto", "#")):
            return match.group(0)

        anchor = ""
        url_path = url
        if "#" in url_path:
            url_path, anchor = url_path.split("#", 1)
            anchor = "#" + anchor

        if not url_path:
            return match.group(0)

        resolved = os.path.normpath(
            os.path.join("examples", example_name, url_path)
        ).replace("\\", "/")

        return f"[{text}]({GITHUB_BLOB}/{resolved}{anchor})"

    return re.sub(r"\[([^\]]+)\]\(([^)]+)\)", replacer, content)


def strip_title(readme):
    """Remove the first ``#`` heading from README content."""
    return re.sub(r"^#\s+.+\n*", "", readme, count=1, flags=re.MULTILINE)


def validate_preview_path(name, filename):
    """Validate that *filename* is a safe relative path inside the example dir.

    Rejects absolute paths, paths containing ``..`` segments, and any path
    whose resolved real location falls outside ``EXAMPLES_DIR/<name>``. This
    prevents path-traversal via catalog.toml ``screenshot``/``preview`` fields,
    which are contributor-editable.
    """
    if not filename:
        return False

    # Reject absolute paths — os.path.join would discard the example dir.
    if os.path.isabs(filename):
        return False

    # Reject parent-directory traversals.
    parts = filename.replace("\\", "/").split("/")
    if ".." in parts:
        return False

    # Final guard: resolve the real path and confirm it's under the example dir.
    example_root = os.path.realpath(os.path.join(EXAMPLES_DIR, name))
    resolved = os.path.realpath(os.path.join(EXAMPLES_DIR, name, filename))
    if not resolved.startswith(example_root + os.sep) and resolved != example_root:
        return False

    return True


SAFE_NAME = re.compile(r"^[a-zA-Z0-9_-]+$")


def _is_under(root, path):
    """Return True if *path* resolves inside *root* (symlink-aware)."""
    root = os.path.realpath(root)
    path = os.path.realpath(path)
    try:
        return os.path.commonpath([root, path]) == root
    except ValueError:
        return False


def validate_example_name(name):
    """Validate a catalog ``name`` before using it as a filesystem path.

    Restricts names to ``[a-zA-Z0-9_-]+`` and requires the resolved example
    directory to stay under ``EXAMPLES_DIR``. Catalog ``name`` values are
    contributor-editable and otherwise feed both reads and generated writes.
    """
    if not name or not SAFE_NAME.fullmatch(name):
        return False
    example_dir = os.path.join(EXAMPLES_DIR, name)
    return _is_under(EXAMPLES_DIR, example_dir)


def check_preview(name, entry):
    """Return the filename of the preview image if one exists, else *None*."""
    screenshot = entry.get("screenshot")
    if screenshot and validate_preview_path(name, screenshot):
        if os.path.exists(os.path.join(EXAMPLES_DIR, name, screenshot)):
            return screenshot

    preview_path = os.path.join(EXAMPLES_DIR, name, "preview.svg")
    if os.path.exists(preview_path):
        return "preview.svg"

    return None


# ---------------------------------------------------------------------------
# CSS generation
# ---------------------------------------------------------------------------

def generate_css(tokens):
    """Generate ``showcase.css`` with design tokens and gallery styles.

    Design tokens are consumed directly from ``packages/tokens/tokens.json``
    (the single source of truth mirroring ``docs/BRAND.md``) and emitted as
    CSS custom properties, following the same light/dark theming contract as
    ``packages/tokens/build.js``. Gallery-specific rules consume the tokens
    via ``var(--…)`` so the showcase never visually diverges from the docs site.
    """
    shared_decls = []
    light_decls = []
    dark_decls = []

    for tname, spec in tokens["tokens"].items():
        if "light" in spec and "dark" in spec:
            light_decls.append(f"  --{tname}: {spec['light']};")
            dark_decls.append(f"    --{tname}: {spec['dark']};")
        elif "value" in spec:
            shared_decls.append(f"  --{tname}: {spec['value']};")

    shared_block = "\n".join(shared_decls)
    light_block = "\n".join(light_decls)
    dark_block = "\n".join(dark_decls)
    dark_root_block = "\n".join(
        d.replace("    --", "  --") for d in dark_decls
    )

    return _CSS_HEADER + f"""\
:root {{
{shared_block}

{light_block}
}}

@media (prefers-color-scheme: dark) {{
  :root:not([data-theme='light']) {{
{dark_block}
  }}
}}

:root[data-theme='dark'] {{
{dark_root_block}
}}

:root[data-theme='light'] {{
{light_block}
}}

{_CSS_GALLERY}
"""


_CSS_HEADER = """\
/*
 * Cougr showcase gallery styles.
 *
 * Design tokens (the :root block) are generated from packages/tokens/tokens.json
 * — the single source of truth defined in docs/BRAND.md. Do not edit token values
 * here; change them in tokens.json (or BRAND.md) and re-run generate-showcase.py.
 *
 * Gallery-specific rules below consume the tokens via CSS custom properties so
 * the showcase never visually diverges from the docs site.
 */

/* === Design tokens === */
"""

_CSS_GALLERY = """\
/* === Showcase gallery === */

.showcase-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--space-5) var(--space-4);
}

/* Filter controls */
.showcase-controls {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-5);
  margin-bottom: var(--space-6);
  padding: var(--space-4);
  background: var(--color-surface);
  border-radius: var(--radius-md);
}

.filter-group {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
}

.filter-label {
  font-family: var(--font-sans);
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  margin-right: var(--space-2);
}

.filter-btn {
  font-family: var(--font-sans);
  font-size: 0.8125rem;
  font-weight: 500;
  padding: var(--space-1) var(--space-3);
  border: 1px solid var(--color-text-secondary);
  border-radius: var(--radius-full);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-btn:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}

.filter-btn.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--color-bg);
}

/* Card grid */
.showcase-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--space-5);
  list-style: none;
  padding: 0;
  margin: 0;
}

/* Card */
.showcase-card {
  background: var(--color-surface);
  border-radius: var(--radius-md);
  overflow: hidden;
  border: 1px solid transparent;
  transition: border-color 0.15s ease, transform 0.15s ease;
}

.showcase-card:hover {
  border-color: var(--color-primary);
  transform: translateY(-2px);
}

.showcase-card > a {
  text-decoration: none;
  color: inherit;
  display: block;
}

/* Card preview image */
.card-preview {
  width: 100%;
  height: 180px;
  background: var(--color-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.card-preview img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}

/* Card body */
.card-body {
  padding: var(--space-4);
}

.card-title {
  font-family: var(--font-sans);
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin: 0 0 var(--space-2) 0;
}

.card-badges {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.badge {
  display: inline-block;
  font-family: var(--font-sans);
  font-size: 0.75rem;
  font-weight: 600;
  padding: var(--space-1) var(--space-3);
  border-radius: var(--radius-full);
  text-transform: capitalize;
}

.badge-category {
  background: var(--color-surface);
  border: 1px solid var(--color-text-secondary);
  color: var(--color-text-secondary);
}

.badge-maturity {
  color: var(--color-bg);
}

.maturity-canonical {
  background: var(--color-tier-stable);
}

.maturity-transitional {
  background: var(--color-tier-beta);
}

.maturity-experimental {
  background: var(--color-tier-experimental);
}

.badge-verified {
  background: var(--color-accent);
  color: var(--color-bg);
}

/* Card features */
.card-features {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
  margin-bottom: var(--space-3);
}

.feature-tag {
  display: inline-block;
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  padding: 2px var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-surface);
}

/* Card description */
.card-desc {
  font-family: var(--font-sans);
  font-size: 0.875rem;
  line-height: 1.5;
  color: var(--color-text-secondary);
  margin: 0 0 var(--space-3) 0;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Card links */
.card-links {
  display: flex;
  gap: var(--space-3);
  font-size: 0.8125rem;
}

.card-links a {
  color: var(--color-accent);
  text-decoration: none;
}

.card-links a:hover {
  text-decoration: underline;
}

/* === Detail page === */

.showcase-detail-meta {
  margin: var(--space-4) 0 var(--space-5) 0;
  padding: var(--space-4);
  background: var(--color-surface);
  border-radius: var(--radius-md);
}

.detail-badges {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}

.detail-features {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
  margin-bottom: var(--space-3);
}

.detail-preview {
  margin: var(--space-3) 0;
}

.detail-preview img {
  max-width: 100%;
  border-radius: var(--radius-md);
}

.detail-links a {
  color: var(--color-accent);
  text-decoration: none;
  font-weight: 500;
}

.detail-links a:hover {
  text-decoration: underline;
}

.detail-contract {
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  color: var(--color-text-secondary);
  margin-top: var(--space-2);
}

/* No-results message */
.showcase-no-results {
  grid-column: 1 / -1;
  text-align: center;
  padding: var(--space-7);
  color: var(--color-text-secondary);
  font-family: var(--font-sans);
}
"""


# ---------------------------------------------------------------------------
# Page generation
# ---------------------------------------------------------------------------

def generate_gallery_page(examples):
    """Generate the gallery index page markdown."""
    categories = sorted({e["metadata"]["category"] for e in examples})
    maturities = sorted({e["metadata"]["maturity"] for e in examples})

    # --- Filter buttons ---
    cat_buttons = [
        '<button class="filter-btn active" data-filter-type="category" '
        'data-filter-value="all">All</button>'
    ]
    for cat in categories:
        label = CATEGORY_LABELS.get(cat, cat.replace("-", " ").title())
        cat_buttons.append(
            f'<button class="filter-btn" data-filter-type="category" '
            f'data-filter-value="{html.escape(cat)}">{html.escape(label)}</button>'
        )

    mat_buttons = [
        '<button class="filter-btn active" data-filter-type="maturity" '
        'data-filter-value="all">All</button>'
    ]
    for mat in maturities:
        _, label = MATURITY_TIER.get(mat, (mat, mat.capitalize()))
        mat_buttons.append(
            f'<button class="filter-btn" data-filter-type="maturity" '
            f'data-filter-value="{html.escape(mat)}">{html.escape(label)}</button>'
        )

    # --- Cards ---
    cards = []
    for e in examples:
        meta = e["metadata"]
        name = meta["name"]
        title = e.get("title") or name
        category = meta["category"]
        maturity = meta["maturity"]
        features = meta.get("cougr_features", [])
        desc = e.get("description", "")
        preview = e.get("preview")
        verified = meta.get("verified", False)
        github_url = f"{GITHUB_EXAMPLES}/{name}"

        _, mat_label = MATURITY_TIER.get(maturity, (maturity, maturity.capitalize()))
        cat_label = CATEGORY_LABELS.get(category, category.replace("-", " ").title())

        preview_html = ""
        if preview:
            preview_html = (
                f'<div class="card-preview"><img src="previews/{name}.svg" '
                f'alt="{html.escape(name)} preview" /></div>'
            )

        features_html = "".join(
            f'<span class="feature-tag">{html.escape(f)}</span>' for f in features
        )

        verified_html = (
            '<span class="badge badge-verified">&#10003; Cougr Verified</span>'
            if verified
            else ""
        )

        desc_escaped = html.escape(desc)

        cards.append(
            f'<li class="showcase-card" data-category="{html.escape(category)}" '
            f'data-maturity="{html.escape(maturity)}">'
            f'<a href="{name}.html">'
            f"{preview_html}"
            f'<div class="card-body">'
            f'<h3 class="card-title">{html.escape(title)}</h3>'
            f'<div class="card-badges">'
            f'<span class="badge badge-category">{html.escape(cat_label)}</span>'
            f'<span class="badge badge-maturity maturity-{html.escape(maturity)}">'
            f"{html.escape(mat_label)}</span>"
            f"{verified_html}"
            f"</div>"
            f'<div class="card-features">{features_html}</div>'
            f'<p class="card-desc">{desc_escaped}</p>'
            f'<div class="card-links">'
            f'<a href="{name}.html">View Details &rarr;</a>'
            f'<a href="{github_url}">GitHub</a>'
            f"</div>"
            f"</div>"
            f"</a>"
            f"</li>"
        )

    cat_buttons_html = "\n      ".join(cat_buttons)
    mat_buttons_html = "\n      ".join(mat_buttons)
    cards_html = "\n  ".join(cards)

    return f"""\
<!-- Auto-generated by cougr-site/generate-showcase.py — do not edit by hand. -->

# Example Gallery

A browsable directory of games and demos built with Cougr. Filter by category or
maturity to find the right reference for your use case.

<div class="showcase-page">
<div class="showcase-controls">
  <div class="filter-group">
    <span class="filter-label">Category:</span>
      {cat_buttons_html}
  </div>
  <div class="filter-group">
    <span class="filter-label">Maturity:</span>
      {mat_buttons_html}
  </div>
</div>
<ul class="showcase-grid">
  {cards_html}
</ul>
</div>

<script>
(function() {{
  var activeCategory = 'all';
  var activeMaturity = 'all';
  var catBtns = document.querySelectorAll('[data-filter-type="category"]');
  var matBtns = document.querySelectorAll('[data-filter-type="maturity"]');
  var cards = document.querySelectorAll('.showcase-card');
  var grid = document.querySelector('.showcase-grid');
  var noResults = null;

  function applyFilters() {{
    var visible = 0;
    cards.forEach(function(card) {{
      var cat = card.getAttribute('data-category');
      var mat = card.getAttribute('data-maturity');
      var catMatch = activeCategory === 'all' || cat === activeCategory;
      var matMatch = activeMaturity === 'all' || mat === activeMaturity;
      if (catMatch && matMatch) {{
        card.style.display = '';
        visible++;
      }} else {{
        card.style.display = 'none';
      }}
    }});
    if (noResults) {{ noResults.remove(); noResults = null; }}
    if (visible === 0) {{
      noResults = document.createElement('p');
      noResults.className = 'showcase-no-results';
      noResults.textContent = 'No examples match the selected filters.';
      grid.appendChild(noResults);
    }}
  }}

  catBtns.forEach(function(btn) {{
    btn.addEventListener('click', function() {{
      catBtns.forEach(function(b) {{ b.classList.remove('active'); }});
      btn.classList.add('active');
      activeCategory = btn.getAttribute('data-filter-value');
      applyFilters();
    }});
  }});

  matBtns.forEach(function(btn) {{
    btn.addEventListener('click', function() {{
      matBtns.forEach(function(b) {{ b.classList.remove('active'); }});
      btn.classList.add('active');
      activeMaturity = btn.getAttribute('data-filter-value');
      applyFilters();
    }});
  }});
}})();
</script>
"""


def generate_detail_page(example):
    """Generate a detail page for a single example."""
    meta = example["metadata"]
    name = meta["name"]
    title = example.get("title") or name
    category = meta["category"]
    maturity = meta["maturity"]
    features = meta.get("cougr_features", [])
    readme = example.get("readme_content", "")
    preview = example.get("preview")
    verified = meta.get("verified", False)
    testnet_contract = meta.get("testnet_contract")
    github_url = f"{GITHUB_EXAMPLES}/{name}"

    _, mat_label = MATURITY_TIER.get(maturity, (maturity, maturity.capitalize()))
    cat_label = CATEGORY_LABELS.get(category, category.replace("-", " ").title())

    # Badges
    badges = [
        f'<span class="badge badge-category">{html.escape(cat_label)}</span>',
        f'<span class="badge badge-maturity maturity-{html.escape(maturity)}">'
        f"{html.escape(mat_label)}</span>",
    ]
    if verified:
        badges.append(
            '<span class="badge badge-verified">&#10003; Cougr Verified</span>'
        )
    badges_html = "\n  ".join(badges)

    # Features
    features_html = "".join(
        f'<span class="feature-tag">{html.escape(f)}</span>' for f in features
    )

    # Preview
    preview_html = ""
    if preview:
        preview_html = (
            f'<div class="detail-preview"><img src="previews/{name}.svg" '
            f'alt="{html.escape(name)} preview" /></div>'
        )

    # Contract
    contract_html = ""
    if testnet_contract:
        contract_html = (
            f'<div class="detail-contract">Testnet contract: '
            f"<code>{html.escape(testnet_contract)}</code></div>"
        )

    # Links
    links_html = (
        f'<div class="detail-links"><a href="{github_url}">'
        f"View source on GitHub &rarr;</a></div>"
    )

    # Rewrite README links and strip the title
    readme = rewrite_readme_links(readme, name)
    readme = strip_title(readme)

    return f"""\
<!-- Auto-generated by cougr-site/generate-showcase.py — do not edit by hand. -->

# {title}

<div class="showcase-detail-meta">
  <div class="detail-badges">
  {badges_html}
  </div>
  <div class="detail-features">{features_html}</div>
  {preview_html}
  {links_html}
  {contract_html}
</div>

---

{readme.strip()}
"""


# ---------------------------------------------------------------------------
# Preview images
# ---------------------------------------------------------------------------

def copy_previews(examples):
    """Copy preview images to ``src/showcase/previews/``."""
    if os.path.exists(PREVIEWS_DIR):
        shutil.rmtree(PREVIEWS_DIR)
    os.makedirs(PREVIEWS_DIR, exist_ok=True)

    count = 0
    for e in examples:
        preview = e.get("preview")
        if not preview:
            continue
        name = e["metadata"]["name"]
        if not validate_preview_path(name, preview):
            print(f"Warning: rejecting unsafe preview path for '{name}': {preview}")
            continue
        src = os.path.join(EXAMPLES_DIR, name, preview)
        if os.path.exists(src):
            ext = os.path.splitext(preview)[1] or ".svg"
            dst = os.path.join(PREVIEWS_DIR, f"{name}{ext}")
            shutil.copy2(src, dst)
            count += 1
    return count


# ---------------------------------------------------------------------------
# SUMMARY.md update
# ---------------------------------------------------------------------------

_START_MARKER = "<!-- showcase-detail-start -->"
_END_MARKER = "<!-- showcase-detail-end -->"


def update_summary(examples):
    """Insert generated detail-page entries into ``SUMMARY.md``."""
    with open(SUMMARY_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    entries = []
    for e in examples:
        name = e["metadata"]["name"]
        title = e.get("title") or name
        entries.append(f"  - [{title}](showcase/{name}.md)")
    entries_block = "\n".join(entries)

    if _START_MARKER in content and _END_MARKER in content:
        pattern = re.compile(
            re.escape(_START_MARKER) + r".*?" + re.escape(_END_MARKER),
            re.DOTALL,
        )
        content = pattern.sub(
            f"{_START_MARKER}\n{entries_block}\n  {_END_MARKER}",
            content,
        )
    else:
        gallery_line = "  - [Example Gallery](showcase/gallery.md)"
        if gallery_line in content:
            replacement = (
                gallery_line
                + "\n  "
                + _START_MARKER
                + "\n"
                + entries_block
                + "\n  "
                + _END_MARKER
            )
            content = content.replace(gallery_line, replacement)
        else:
            print("Warning: could not find gallery line in SUMMARY.md — "
                  "detail pages will not appear in navigation.")

    with open(SUMMARY_PATH, "w", encoding="utf-8") as f:
        f.write(content)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    if not os.path.exists(CATALOG_PATH):
        sys.exit(f"Error: catalog not found at {CATALOG_PATH}")
    if not os.path.exists(TOKENS_PATH):
        sys.exit(f"Error: tokens not found at {TOKENS_PATH}")

    catalog = load_catalog()
    tokens = load_tokens()

    if not catalog:
        print("Warning: catalog is empty — nothing to generate.")
        return

    # Build the examples list.
    examples = []
    for key, meta in sorted(catalog.items()):
        name = meta.get("name", key)
        if not validate_example_name(name):
            print(f"Warning: rejecting unsafe example name '{name}', skipping.")
            continue
        example_dir = os.path.join(EXAMPLES_DIR, name)

        if not os.path.isdir(example_dir):
            print(f"Warning: example directory '{name}' not found, skipping.")
            continue

        readme = read_readme(name)
        if not readme:
            print(f"Warning: no README.md for '{name}', skipping.")
            continue

        safe_meta = dict(meta)
        safe_meta["name"] = name
        examples.append({
            "metadata": safe_meta,
            "title": extract_title(readme),
            "description": extract_description(readme),
            "preview": check_preview(name, safe_meta),
            "readme_content": readme,
        })

    if not examples:
        print("Warning: no valid examples found.")
        return

    print(f"Generating showcase for {len(examples)} examples...")

    # CSS
    css = generate_css(tokens)
    os.makedirs(THEME_DIR, exist_ok=True)
    css_path = os.path.join(THEME_DIR, "showcase.css")
    with open(css_path, "w", encoding="utf-8") as f:
        f.write(css)
    print(f"  + {os.path.relpath(css_path, SITE_DIR)}")

    # Gallery page
    gallery_md = generate_gallery_page(examples)
    gallery_path = os.path.join(SHOWCASE_SRC, "gallery.md")
    with open(gallery_path, "w", encoding="utf-8") as f:
        f.write(gallery_md)
    print(f"  + {os.path.relpath(gallery_path, SITE_DIR)}")

    # Detail pages
    os.makedirs(SHOWCASE_SRC, exist_ok=True)
    showcase_root = os.path.realpath(SHOWCASE_SRC)
    for e in examples:
        detail_md = generate_detail_page(e)
        name = e["metadata"]["name"]
        detail_path = os.path.join(SHOWCASE_SRC, f"{name}.md")
        resolved = os.path.realpath(detail_path)
        if not _is_under(showcase_root, resolved) and resolved != showcase_root:
            print(f"Warning: rejecting unsafe detail path for '{name}', skipping.")
            continue
        with open(detail_path, "w", encoding="utf-8") as f:
            f.write(detail_md)
        print(f"  + {os.path.relpath(detail_path, SITE_DIR)}")

    # Preview images
    preview_count = copy_previews(examples)
    if preview_count:
        print(f"  + src/showcase/previews/ ({preview_count} images)")

    # SUMMARY.md
    update_summary(examples)
    print("  + src/SUMMARY.md (detail entries inserted)")

    print(f"\nDone: {len(examples)} examples, {preview_count} previews.")


if __name__ == "__main__":
    main()
