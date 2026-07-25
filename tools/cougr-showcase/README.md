# cougr-showcase

Static gallery generator for the Cougr example catalog.

Reads `examples/` directory structure and `examples/README.md` catalog
metadata, then generates a browsable HTML gallery with category/maturity
filtering and one detail page per example.

## Usage

```bash
# Generate to default output directory (site/showcase/)
cargo run -p cougr-showcase

# Generate to a custom output directory
cargo run -p cougr-showcase -- --output-dir path/to/output
```

## Output

- `index.html` — gallery index with search and filter controls
- `{example}.html` — one detail page per example

## License

MIT
