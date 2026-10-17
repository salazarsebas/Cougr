//! Error-path coverage for `cougr add unknown-piece`.

use std::process::Command;

#[test]
fn addunknown_piece_fails_with_hint() {
    let dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_cougr"))
        .args(["add", "not-a-real-piece", "--path"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not-a-real-piece"),
        "stderr did not mention the piece name: {stderr}"
    );
    assert!(
        stderr.contains("cougr add --list"),
        "stderr did not include the hint: {stderr}"
    );
}
