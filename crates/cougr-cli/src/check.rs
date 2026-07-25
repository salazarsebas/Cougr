//! `cougr check` project hygiene diagnostic command

use std::path::Path;

pub fn run_check(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Running Cougr Project Hygiene Checks ===");

    let mut violations = Vec::new();

    // Check 1: No committed build artifacts
    if root.join("target").exists() {
        violations.push("Committed target/ directory detected in repository");
    }

    // Check 2: .gitignore presence
    if !root.join(".gitignore").exists() {
        violations.push("Missing .gitignore file in project root");
    }

    // Check 3: Cargo.toml presence and description
    let cargo_toml_path = root.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        violations.push("Missing Cargo.toml in project root");
    } else if let Ok(content) = std::fs::read_to_string(&cargo_toml_path) {
        if !content.contains("description") {
            violations.push("Cargo.toml is missing 'description' metadata field");
        }
    }

    if violations.is_empty() {
        println!("  [PASS] All hygiene checks passed cleanly!");
        Ok(())
    } else {
        println!("  [FAIL] Found {} hygiene violations:", violations.len());
        for v in &violations {
            println!("    - {}", v);
        }
        std::process::exit(1);
    }
}
