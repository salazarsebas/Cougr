use std::process::Command;

fn cougr_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cougr")
}

fn run_cougr(args: &[&str]) -> std::process::Output {
    Command::new(cougr_bin())
        .args(args)
        .output()
        .expect("failed to run cougr binary")
}

#[cfg(test)]
fn add_unknown_piece_fails_with_hint() {
    todo!()
}
