// Sandbox implementation compiles inside cougr-core at src/test/mod.rs.
// This stub exists so the workspace member resolves; run tests with:
//   cargo test -p cougr-core --features testutils

/// Mirrors [`cougr_core::test::MODULE_VERSION`].
pub const MODULE_VERSION: &str = "0.1.1-sandbox";
