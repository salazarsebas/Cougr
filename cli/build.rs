//! Rebuild the CLI whenever an embedded template changes.
//!
//! `rust-embed` bakes `templates/` into the binary through a derive macro,
//! which Cargo cannot see as an input on its own. Without this, editing a
//! template leaves a stale copy compiled into `cougr`.

fn main() {
    println!("cargo:rerun-if-changed=templates");
}
