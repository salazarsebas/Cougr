//! `cougr doctor` toolchain diagnostics command

use std::process::Command;

pub struct DoctorCheck {
    pub name: &'static str,
    pub passed: bool,
    pub fix_cmd: &'static str,
}

pub fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cougr Toolchain Diagnostics (`cougr doctor`) ===");

    let mut checks = Vec::new();

    // Check 1: rustc
    let rustc_ok = Command::new("rustc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    checks.push(DoctorCheck {
        name: "Rust compiler (rustc)",
        passed: rustc_ok,
        fix_cmd: "Install Rust via https://rustup.rs",
    });

    // Check 2: wasm32-unknown-unknown target
    let target_ok = Command::new("rustup").args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-unknown-unknown") || String::from_utf8_lossy(&o.stdout).contains("wasm32v1-none"))
        .unwrap_or(false);
    checks.push(DoctorCheck {
        name: "WASM compilation target (wasm32-unknown-unknown / wasm32v1-none)",
        passed: target_ok,
        fix_cmd: "rustup target add wasm32-unknown-unknown",
    });

    // Check 3: stellar CLI
    let stellar_ok = Command::new("stellar").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    checks.push(DoctorCheck {
        name: "Stellar CLI (stellar)",
        passed: stellar_ok,
        fix_cmd: "cargo install --locked soroban-cli",
    });

    let mut passed_count = 0;
    for check in &checks {
        if check.passed {
            println!("  [PASS] {}", check.name);
            passed_count += 1;
        } else {
            println!("  [FAIL] {}", check.name);
            println!("         Fix: {}", check.fix_cmd);
        }
    }

    println!("\nSummary: {}/{} checks passed.", passed_count, checks.len());
    if passed_count < checks.len() {
        std::process::exit(1);
    }
    Ok(())
}
