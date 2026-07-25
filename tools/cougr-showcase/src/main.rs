//! cougr-showcase — Static gallery generator for the Cougr example catalog.
//!
//! Reads `examples/` directory structure and `examples/README.md` catalog
//! metadata, then generates a browsable HTML gallery with category/maturity
//! filtering and one detail page per example.
//!
//! Usage:  cargo run -p cougr-showcase [--output-dir <path>]

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Example {
    name: String,
    category: String,
    maturity: String, // Stable | Beta | Experimental
    focus: String,
    description: String,
    cougr_features: Vec<String>, // APIs extracted from "Cougr APIs used" section
}

// ---------------------------------------------------------------------------
// Catalog parsing
// ---------------------------------------------------------------------------

/// Strip inline bold markers like `**text**` → `text`.
fn strip_bold(s: &str) -> String {
    s.replace("**", "").trim().to_string()
}

/// Normalise a category string: lowercase, strip extra whitespace.
fn normalise_category(raw: &str) -> String {
    let s = strip_bold(raw);
    // Remove parenthetical maturity hints like "(Beta)" or "(Experimental)"
    let s = s
        .replace("(Beta)", "")
        .replace("(Experimental)", "")
        .replace("(Stable)", "");
    s.trim().to_string()
}

/// Extract maturity from a category/focus string.
fn extract_maturity(raw: &str) -> String {
    let lower = raw.to_lowercase();
    if lower.contains("stable") {
        "Stable".into()
    } else if lower.contains("beta") {
        "Beta".into()
    } else if lower.contains("experimental") {
        "Experimental".into()
    } else {
        "Stable".into() // default
    }
}

/// Parse pipe-delimited table rows from a markdown string.
fn parse_table_rows(lines: &[String]) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut header_skipped = false;

    for line in lines {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            header_skipped = false;
            continue;
        }
        // Detect separator line (|---|---|)
        if trimmed.contains("---") {
            header_skipped = true;
            continue;
        }
        if !header_skipped {
            continue; // skip header row
        }

        let cells: Vec<String> = trimmed
            .split('|')
            .skip(1) // leading empty
            .map(|s| s.trim().to_string())
            .collect();
        if cells.len() >= 3 {
            rows.push(cells);
        }
    }
    rows
}

/// Compute the maturity string based on the example name (look for keywords).
fn maturity_from_readme(example_name: &str, readme_path: &Path) -> String {
    let content = match fs::read_to_string(readme_path) {
        Ok(c) => c,
        Err(_) => return "Stable".into(),
    };
    let lower = content.to_lowercase();
    // Check for maturity classification in the README
    if lower.contains("**experimental**")
        || lower.contains("classification: experimental")
        || lower.contains("(experimental)")
    {
        "Experimental".into()
    } else if lower.contains("**(beta)**")
        || lower.contains("classification: beta")
        || lower.contains("(beta)")
    {
        "Beta".into()
    } else {
        "Stable".into()
    }
}

fn extract_description(readme_path: &Path) -> String {
    let content = match fs::read_to_string(readme_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    // Skip the first heading line and any frontmatter; take the first
    // substantial paragraph (up to ~300 chars).
    let mut in_frontmatter = false;
    let mut past_first_heading = false;
    let mut paragraph = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Handle YAML frontmatter
        if trimmed == "---" && !past_first_heading {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        // Skip blank lines until after first heading
        if trimmed.starts_with('#') && !past_first_heading {
            past_first_heading = true;
            continue;
        }
        if !past_first_heading {
            continue;
        }

        if trimmed.is_empty() {
            if !paragraph.is_empty() && paragraph.len() > 40 {
                break;
            }
            continue;
        }

        // Skip non-content lines
        if trimmed.starts_with("<!--")
            || trimmed.starts_with("```")
            || trimmed.starts_with('|')
        {
            continue;
        }

        // Append to paragraph
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(trimmed);

        if paragraph.len() > 300 {
            break;
        }
    }

    // Clean up markdown links in the description
    let cleaned = paragraph
        .replace("**", "")
        .replace('*', "")
        .replace('[', "")
        .replace(']', "");
    cleaned.trim().to_string()
}

/// Extract the "Cougr APIs used" section from an example's README.
/// Looks for a heading matching "Cougr APIs used" and captures bullet/table items under it.
fn extract_cougr_features(readme_path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(readme_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut features = Vec::new();
    let mut in_section = false;
    let mut section_end = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Detect "## Cougr APIs used" heading (with any prefix/formatting)
        if trimmed.to_lowercase().contains("cougr apis used")
            || trimmed.to_lowercase().contains("cougr api used")
        {
            if trimmed.starts_with('#') {
                in_section = true;
                continue;
            }
        }

        if !in_section {
            continue;
        }

        // Stop at the next heading or horizontal rule
        if trimmed.starts_with('#') && !trimmed.to_lowercase().contains("cougr apis")
            || trimmed.starts_with("---") && trimmed.len() >= 3
        {
            // Check if this is a subheading or a transition to a new section
            if trimmed.starts_with('#') {
                section_end = true;
            }
        }

        if section_end && (trimmed.starts_with('#') || trimmed.is_empty()) {
            break;
        }

        // Extract bullet items: `- ` or `* ` items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let item = trimmed
                .trim_start_matches("- ")
                .trim_start_matches("* ")
                .trim();
            // Extract just the API name before the colon if present
            let api_name = if let Some(idx) = item.find(':') {
                item[..idx].trim()
            } else {
                item
            };
            // Clean markdown backticks
            let clean = api_name.replace('`', "").trim().to_string();
            if !clean.is_empty() {
                features.push(clean);
            }
        }

        // Also extract table rows from "| API |" style tables
        if trimmed.starts_with('|') && trimmed.contains('|') {
            let cells: Vec<&str> = trimmed.split('|').collect();
            if cells.len() >= 3 {
                let api_cell = cells[1].trim();
                if !api_cell.is_empty()
                    && !api_cell.contains("---")
                    && !api_cell.eq_ignore_ascii_case("api")
                {
                    let clean = api_cell.replace('`', "").trim().to_string();
                    if !clean.is_empty() {
                        features.push(clean);
                    }
                }
            }
        }
    }

    features
}

// ---------------------------------------------------------------------------
// HTML generation
// ---------------------------------------------------------------------------

fn generate_index_html(examples: &[Example]) -> String {
    let categories = collect_categories(examples);
    let maturities = vec!["Stable", "Beta", "Experimental"];
    let cards: String = examples
        .iter()
        .map(|ex| {
            let maturity_color = maturity_color(&ex.maturity);
            format!(
                r#"<a href="{name}.html" class="example-card" data-category="{cat}" data-maturity="{mat}">
      <div class="card-header">
        <span class="card-name">{name}</span>
        <span class="maturity-badge" style="background:{mcolor}">{mat}</span>
      </div>
      <div class="card-category">{cat}</div>
      <div class="card-desc">{desc}</div>
      <div class="card-footer">
        <span class="card-focus">{focus}</span>
        <span class="card-source">View on GitHub →</span>
      </div>
    </a>"#,
                name = html_escape(&ex.name),
                cat = html_escape(&ex.category),
                mat = html_escape(&ex.maturity),
                mcolor = maturity_color,
                desc = html_escape(&truncate(&ex.description, 150)),
                focus = html_escape(&ex.focus),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let filter_buttons: String = categories
        .iter()
        .map(|cat| {
            format!(
                r#"<button class="filter-btn" data-filter="category:{cat}">{cat}</button>"#,
                cat = html_escape(cat)
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    let maturity_buttons: String = maturities
        .iter()
        .map(|m| {
            let color = maturity_color(m);
            format!(
                r#"<button class="filter-btn maturity-filter" data-filter="maturity:{m}" style="--badge-color:{color}">{m}</button>"#,
                m = html_escape(m),
                color = color,
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    let total = examples.len();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Cougr Showcase — Example Gallery</title>
  <style>
    :root {{
      --bg: #0b0d11;
      --surface: #14171c;
      --surface-hover: #1a1e25;
      --border: #262a31;
      --text: #e8eaed;
      --text-muted: #8b92a0;
      --accent: #4f8cff;
      --accent-dim: rgba(79,140,255,0.12);
      --radius: 12px;
      --stable: #2ea043;
      --beta: #d29922;
      --experimental: #da3633;
    }}
    * {{ margin:0; padding:0; box-sizing:border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
      min-height: 100vh;
    }}
    .container {{ max-width:1200px; margin:0 auto; padding:0 24px; }}
    header {{
      padding: 48px 0 32px;
      border-bottom: 1px solid var(--border);
      margin-bottom: 32px;
    }}
    header h1 {{
      font-size: 2.2rem;
      font-weight: 700;
      letter-spacing: -0.03em;
      margin-bottom: 8px;
    }}
    header h1 span {{ color: var(--accent); }}
    header p {{ color: var(--text-muted); font-size: 1.05rem; }}
    .stats {{ margin-top: 12px; font-size: 0.9rem; color: var(--text-muted); }}
    .filters {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      margin-bottom: 28px;
      align-items: center;
    }}
    .filters-label {{
      font-size: 0.85rem;
      color: var(--text-muted);
      margin-right: 8px;
      font-weight: 500;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}
    .filter-btn {{
      background: var(--surface);
      border: 1px solid var(--border);
      color: var(--text);
      padding: 6px 16px;
      border-radius: 20px;
      cursor: pointer;
      font-size: 0.85rem;
      font-weight: 500;
      transition: all 0.2s ease;
    }}
    .filter-btn:hover {{
      background: var(--surface-hover);
      border-color: var(--accent);
    }}
    .filter-btn.active {{
      background: var(--accent-dim);
      border-color: var(--accent);
      color: var(--accent);
    }}
    .maturity-filter {{ border-left: 3px solid var(--badge-color, var(--border)); }}
    .filter-btn.clear-btn {{
      color: var(--text-muted);
      font-size: 0.8rem;
      border-color: transparent;
    }}
    .filter-btn.clear-btn:hover {{ color: var(--text); }}
    .search-box {{
      width: 100%;
      max-width: 400px;
      padding: 10px 16px;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      color: var(--text);
      font-size: 0.95rem;
      margin-bottom: 24px;
      outline: none;
      transition: border-color 0.2s;
    }}
    .search-box:focus {{ border-color: var(--accent); }}
    .search-box::placeholder {{ color: var(--text-muted); }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
      gap: 16px;
      padding-bottom: 64px;
    }}
    .example-card {{
      display: block;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 20px;
      text-decoration: none;
      color: inherit;
      transition: all 0.25s ease;
      position: relative;
      overflow: hidden;
    }}
    .example-card::before {{
      content: '';
      position: absolute;
      top: 0; left: 0; right: 0;
      height: 2px;
      background: var(--border);
      transition: background 0.25s ease;
    }}
    .example-card:hover {{
      background: var(--surface-hover);
      border-color: var(--accent);
      transform: translateY(-2px);
      box-shadow: 0 8px 24px rgba(0,0,0,0.3);
    }}
    .example-card:hover::before {{ background: var(--accent); }}
    .card-header {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 8px;
    }}
    .card-name {{
      font-size: 1.1rem;
      font-weight: 600;
      letter-spacing: -0.02em;
    }}
    .maturity-badge {{
      font-size: 0.7rem;
      font-weight: 600;
      padding: 2px 8px;
      border-radius: 10px;
      color: #fff;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}
    .card-category {{
      font-size: 0.8rem;
      color: var(--accent);
      font-weight: 500;
      margin-bottom: 8px;
    }}
    .card-desc {{
      font-size: 0.88rem;
      color: var(--text-muted);
      line-height: 1.5;
      margin-bottom: 12px;
    }}
    .card-footer {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      font-size: 0.8rem;
    }}
    .card-focus {{ color: var(--text-muted); }}
    .card-source {{
      color: var(--accent);
      opacity: 0;
      transition: opacity 0.2s ease;
    }}
    .example-card:hover .card-source {{ opacity: 1; }}
    .no-results {{
      grid-column: 1 / -1;
      text-align: center;
      padding: 48px;
      color: var(--text-muted);
    }}
    footer {{
      text-align: center;
      padding: 32px 0;
      border-top: 1px solid var(--border);
      color: var(--text-muted);
      font-size: 0.85rem;
    }}
    footer a {{ color: var(--accent); text-decoration: none; }}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>Cougr <span>Showcase</span></h1>
      <p>Browse the full catalog of {total} on-chain game examples built with Cougr-Core.</p>
      <div class="stats">{total} examples · Filter by category or maturity</div>
    </header>

    <input type="text" class="search-box" id="search" placeholder="Search examples…" oninput="applyFilters()">

    <div class="filters">
      <span class="filters-label">Category</span>
      {filter_buttons}
      <span class="filters-label" style="margin-left:16px;">Maturity</span>
      {maturity_buttons}
      <button class="filter-btn clear-btn" onclick="clearFilters()">Clear all</button>
    </div>

    <div class="grid" id="gallery">
      {cards}
    </div>

    <footer>
      Powered by <a href="https://github.com/salazarsebas/Cougr">Cougr</a> &mdash; MIT License
    </footer>
  </div>

  <script>
    let activeFilters = [];

    document.querySelectorAll('.filter-btn:not(.clear-btn)').forEach(btn => {{
      btn.addEventListener('click', () => {{
        const filter = btn.dataset.filter;
        btn.classList.toggle('active');
        if (btn.classList.contains('active')) {{
          activeFilters.push(filter);
        }} else {{
          activeFilters = activeFilters.filter(f => f !== filter);
        }}
        applyFilters();
      }});
    }});

    function applyFilters() {{
      const search = document.getElementById('search').value.toLowerCase();
      const cards = document.querySelectorAll('.example-card');
      let visible = 0;

      cards.forEach(card => {{
        const category = card.dataset.category;
        const maturity = card.dataset.maturity;
        const name = card.querySelector('.card-name').textContent.toLowerCase();
        const desc = card.querySelector('.card-desc').textContent.toLowerCase();

        const matchesSearch = !search || name.includes(search) || desc.includes(search);

        let matchesFilter = true;
        if (activeFilters.length > 0) {{
          matchesFilter = activeFilters.every(f => {{
            if (f.startsWith('category:')) return category === f.replace('category:', '');
            if (f.startsWith('maturity:')) return maturity === f.replace('maturity:', '');
            return false;
          }});
        }}

        if (matchesSearch && matchesFilter) {{
          card.style.display = 'block';
          visible++;
        }} else {{
          card.style.display = 'none';
        }}
      }});

      const noResults = document.getElementById('no-results');
      if (visible === 0) {{
        if (!noResults) {{
          const msg = document.createElement('div');
          msg.id = 'no-results';
          msg.className = 'no-results';
          msg.textContent = 'No examples match your filters. Try adjusting your criteria.';
          document.getElementById('gallery').appendChild(msg);
        }}
      }} else {{
        if (noResults) noResults.remove();
      }}
    }}

    function clearFilters() {{
      activeFilters = [];
      document.querySelectorAll('.filter-btn.active').forEach(b => b.classList.remove('active'));
      document.getElementById('search').value = '';
      applyFilters();
    }}
  </script>
</body>
</html>"#,
        total = total,
        filter_buttons = filter_buttons,
        maturity_buttons = maturity_buttons,
        cards = cards,
    )
}

fn generate_detail_html(ex: &Example, all_examples: &[Example]) -> String {
    let maturity_color = match ex.maturity.as_str() {
        "Stable" => "#2ea043",
        "Beta" => "#d29922",
        "Experimental" => "#da3633",
        _ => "#8b92a0",
    };

    let other_examples: String = all_examples
        .iter()
        .filter(|e| e.name != ex.name)
        .take(6)
        .map(|e| {
            format!(
                r#"<a href="{name}.html" class="related-card">
      <div class="related-name">{name}</div>
      <div class="related-cat">{cat}</div>
    </a>"#,
                name = html_escape(&e.name),
                cat = html_escape(&e.category),
            )
        })
        .collect::<Vec<_>>()
        .join("\n      ");

    let safe_desc = html_escape(&ex.description);
    let safe_focus = html_escape(&ex.focus);
    let safe_name = html_escape(&ex.name);
    let safe_category = html_escape(&ex.category);
    let safe_maturity = html_escape(&ex.maturity);
    let github_url = format!(
        "https://github.com/salazarsebas/Cougr/tree/main/examples/{}",
        ex.name
    );

    // Render cougr_features as a list
    let features_html: String = if ex.cougr_features.is_empty() {
        String::new()
    } else {
        let items: String = ex
            .cougr_features
            .iter()
            .map(|f| format!("<li>{}</li>", html_escape(f)))
            .collect::<Vec<_>>()
            .join("\n        ");
        format!(
            r#"<div class="detail-features">
      <h2>Cougr APIs Used</h2>
      <ul>
        {items}
      </ul>
    </div>"#,
            items = items
        )
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{safe_name} — Cougr Showcase</title>
  <style>
    :root {{
      --bg: #0b0d11;
      --surface: #14171c;
      --surface-hover: #1a1e25;
      --border: #262a31;
      --text: #e8eaed;
      --text-muted: #8b92a0;
      --accent: #4f8cff;
      --accent-dim: rgba(79,140,255,0.12);
      --radius: 12px;
    }}
    * {{ margin:0; padding:0; box-sizing:border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.6;
    }}
    .container {{ max-width:800px; margin:0 auto; padding:0 24px; }}
    nav {{
      padding: 20px 0;
      border-bottom: 1px solid var(--border);
      margin-bottom: 32px;
    }}
    nav a {{
      color: var(--accent);
      text-decoration: none;
      font-size: 0.9rem;
      font-weight: 500;
      transition: opacity 0.2s;
    }}
    nav a:hover {{ opacity: 0.8; }}
    nav a::before {{ content: '← '; }}
    .detail-header {{
      margin-bottom: 32px;
    }}
    .detail-header h1 {{
      font-size: 2rem;
      font-weight: 700;
      letter-spacing: -0.03em;
      margin-bottom: 12px;
    }}
    .detail-meta {{
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      align-items: center;
    }}
    .detail-meta .badge {{
      font-size: 0.75rem;
      font-weight: 600;
      padding: 4px 12px;
      border-radius: 10px;
      color: #fff;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }}
    .detail-meta .category-tag {{
      font-size: 0.85rem;
      color: var(--accent);
      font-weight: 500;
      background: var(--accent-dim);
      padding: 4px 12px;
      border-radius: 6px;
    }}
    .detail-meta .focus-tag {{
      font-size: 0.85rem;
      color: var(--text-muted);
    }}
    .detail-description {{
      font-size: 1.05rem;
      color: var(--text);
      line-height: 1.8;
      margin-bottom: 24px;
      padding: 24px;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: var(--radius);
    }}
    .detail-features {{
      margin-bottom: 32px;
      padding: 20px 24px;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: var(--radius);
    }}
    .detail-features h2 {{
      font-size: 1rem;
      font-weight: 600;
      margin-bottom: 12px;
      color: var(--accent);
      letter-spacing: -0.01em;
    }}
    .detail-features ul {{
      list-style: none;
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }}
    .detail-features li {{
      background: var(--accent-dim);
      color: var(--accent);
      font-size: 0.82rem;
      font-weight: 500;
      padding: 4px 12px;
      border-radius: 6px;
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', 'Roboto Mono', monospace;
    }}
    .detail-actions {{
      display: flex;
      gap: 12px;
      margin-bottom: 48px;
    }}
    .btn {{
      display: inline-flex;
      align-items: center;
      padding: 10px 24px;
      border-radius: 8px;
      font-size: 0.9rem;
      font-weight: 600;
      text-decoration: none;
      transition: all 0.2s ease;
    }}
    .btn-primary {{
      background: var(--accent);
      color: #fff;
      border: none;
    }}
    .btn-primary:hover {{ background: #3a78e8; transform: translateY(-1px); }}
    .btn-secondary {{
      background: var(--surface);
      color: var(--text);
      border: 1px solid var(--border);
    }}
    .btn-secondary:hover {{ background: var(--surface-hover); border-color: var(--accent); }}
    .related-section h2 {{
      font-size: 1.3rem;
      font-weight: 600;
      margin-bottom: 16px;
      letter-spacing: -0.02em;
    }}
    .related-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
      gap: 12px;
      margin-bottom: 64px;
    }}
    .related-card {{
      display: block;
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 16px;
      text-decoration: none;
      color: inherit;
      transition: all 0.2s ease;
    }}
    .related-card:hover {{
      background: var(--surface-hover);
      border-color: var(--accent);
      transform: translateY(-1px);
    }}
    .related-name {{ font-weight: 600; font-size: 0.95rem; margin-bottom: 4px; }}
    .related-cat {{ font-size: 0.8rem; color: var(--text-muted); }}
    footer {{
      text-align: center;
      padding: 32px 0;
      border-top: 1px solid var(--border);
      color: var(--text-muted);
      font-size: 0.85rem;
    }}
    footer a {{ color: var(--accent); text-decoration: none; }}
  </style>
</head>
<body>
  <div class="container">
    <nav><a href="index.html">Showcase</a></nav>

    <div class="detail-header">
      <h1>{safe_name}</h1>
      <div class="detail-meta">
        <span class="badge" style="background:{maturity_color}">{safe_maturity}</span>
        <span class="category-tag">{safe_category}</span>
        <span class="focus-tag">{safe_focus}</span>
        <span class="focus-tag">· cougr-core</span>
      </div>
    </div>

    <div class="detail-description">
      {safe_desc}
    </div>

    {features_html}

    <div class="detail-actions">
      <a href="{github_url}" class="btn btn-primary" target="_blank">View Source on GitHub</a>
      <a href="index.html" class="btn btn-secondary">Back to Gallery</a>
    </div>

    <div class="related-section">
      <h2>Other examples you might like</h2>
      <div class="related-grid">
        {other_examples}
      </div>
    </div>

    <footer>
      Powered by <a href="https://github.com/salazarsebas/Cougr">Cougr</a> &mdash; MIT License
    </footer>
  </div>
</body>
</html>"#,
        safe_name = safe_name,
        maturity_color = maturity_color,
        safe_maturity = safe_maturity,
        safe_category = safe_category,
        safe_focus = safe_focus,
        safe_desc = safe_desc,
        features_html = features_html,
        github_url = github_url,
        other_examples = other_examples,
    )
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut truncated = s[..max].to_string();
    if let Some(last_space) = truncated.rfind(' ') {
        truncated.truncate(last_space);
    }
    truncated.push('…');
    truncated
}

fn collect_categories(examples: &[Example]) -> Vec<String> {
    let mut cats: Vec<String> = examples
        .iter()
        .map(|e| e.category.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    cats.sort();
    cats
}

fn maturity_color(maturity: &str) -> &str {
    match maturity {
        "Stable" => "#2ea043",
        "Beta" => "#d29922",
        "Experimental" => "#da3633",
        _ => "#8b92a0",
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    // Determine project root — walk up from manifest dir or CWD
    let project_root = find_project_root();
    let examples_dir = project_root.join("examples");
    let catalog_path = examples_dir.join("README.md");
    let out_dir = parse_args();

    // Read catalog
    let catalog_content = fs::read_to_string(&catalog_path)
        .unwrap_or_else(|e| panic!("Cannot read catalog at {:?}: {}", catalog_path, e));

    let lines: Vec<String> = catalog_content.lines().map(|l| l.to_string()).collect();

    // Parse tables
    let raw_rows = parse_table_rows(&lines);

    // Build example list
    let mut examples: Vec<Example> = Vec::new();

    for cells in &raw_rows {
        let name = cells[0].trim().to_string();
        // Remove backticks if present (some rows have `name` backticked)
        let name = name.trim_matches('`').to_string();
        let category = normalise_category(&cells[1]);
        let maturity_from_focus = extract_maturity(&cells[2]);
        let focus = strip_bold(&cells[2]);

        // Build full path
        let example_dir = examples_dir.join(&name);
        let readme_path = example_dir.join("README.md");

        let (description, cougr_features) = if example_dir.exists() {
            let desc = extract_description(&readme_path);
            let features = extract_cougr_features(&readme_path);
            (desc, features)
        } else {
            (String::new(), Vec::new())
        };

        let maturity = maturity_from_readme(&name, &readme_path);

        // If the README didn't give explicit maturity, use focus-column hint
        // Transitional examples without explicit maturity marker default to Beta
        let maturity = if maturity == "Stable" && maturity_from_focus != "Stable" {
            maturity_from_focus
        } else if maturity == "Stable" && maturity_from_focus == "Stable"
            && focus.to_lowercase().contains("transitional")
        {
            "Beta".into()
        } else {
            maturity
        };

        examples.push(Example {
            name,
            category,
            maturity,
            focus,
            description,
            cougr_features,
        });
    }

    // Sort examples alphabetically
    examples.sort_by(|a, b| a.name.cmp(&b.name));

    // Create output directory
    fs::create_dir_all(&out_dir)
        .unwrap_or_else(|e| panic!("Cannot create output dir {:?}: {}", out_dir, e));

    // Generate index page
    let index_html = generate_index_html(&examples);
    let index_path = out_dir.join("index.html");
    fs::write(&index_path, &index_html)
        .unwrap_or_else(|e| panic!("Cannot write {:?}: {}", index_path, e));
    println!("Wrote {}", index_path.display());

    // Generate detail pages
    for ex in &examples {
        let detail_html = generate_detail_html(ex, &examples);
        let detail_path = out_dir.join(format!("{}.html", ex.name));
        fs::write(&detail_path, &detail_html)
            .unwrap_or_else(|e| panic!("Cannot write {:?}: {}", detail_path, e));
        println!("Wrote {}", detail_path.display());
    }

    println!(
        "\n✨ Showcase generated: {} examples → {} pages",
        examples.len(),
        examples.len() + 1
    );
}

/// Parse CLI args: optional `--output-dir <path>` (default: ./site/showcase)
fn parse_args() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    match args.len() {
        1 => PathBuf::from("site/showcase"),
        3 if args[1] == "--output-dir" => PathBuf::from(&args[2]),
        3 if args[1] == "-o" => PathBuf::from(&args[2]),
        _ => {
            eprintln!("Usage: cougr-showcase [--output-dir <path>]");
            std::process::exit(1);
        }
    }
}

/// Walk up from CWD to find the project root (where the workspace Cargo.toml lives).
fn find_project_root() -> PathBuf {
    let mut cwd = std::env::current_dir().expect("Cannot determine current directory");
    loop {
        if cwd.join("Cargo.toml").exists() {
            // Verify it's the workspace root by checking for [workspace]
            let content = fs::read_to_string(cwd.join("Cargo.toml")).ok();
            if let Some(c) = content {
                if c.contains("[workspace]") {
                    return cwd;
                }
            }
        }
        if !cwd.pop() {
            panic!("Could not find workspace root (Cargo.toml with [workspace])");
        }
    }
}
