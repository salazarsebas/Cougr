//! Integration coverage for the CORS policy module.
//!
//! Exercises the public surface from outside the crate the way a gateway
//! server would: build a policy, validate it, mutate the origin allowlist at
//! runtime, and evaluate preflight and actual requests.

use cougr_core::cors::{
    CorsConfig, CorsError, OriginAllowlist, PreflightDecision, MAX_ALLOWED_MAX_AGE_SECONDS,
    MODULE_VERSION,
};

fn base_config() -> CorsConfig {
    let mut config = CorsConfig::new();
    config.origins.add("https://app.cougr.test").unwrap();
    config.allowed_methods = vec!["GET".to_string(), "POST".to_string()];
    config.allowed_headers = vec!["X-Game-Token".to_string()];
    config.exposed_headers = vec!["X-Game-Version".to_string()];
    config
}

#[test]
fn validated_policy_accepts_legitimate_preflight() {
    let config = base_config();
    config.validate().expect("baseline policy must validate");

    let decision = config.evaluate_preflight("https://app.cougr.test", "POST", &["X-Game-Token"]);
    assert!(decision.allowed);
    assert_eq!(
        decision.allow_origin.as_deref(),
        Some("https://app.cougr.test")
    );
    assert_eq!(decision.allow_methods.as_deref(), Some("GET, POST"));
    // Safelisted headers ride along without configuration.
    let with_safe = config.evaluate_preflight(
        "https://app.cougr.test",
        "GET",
        &["accept-language", "X-Game-Token"],
    );
    assert!(with_safe.allowed);
}

#[test]
fn validation_rejects_unsafe_and_malformed_policies() {
    let mut config = base_config();
    config.allow_credentials = true;
    config.origins.add("*").unwrap();
    assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));

    let mut config = base_config();
    config.allowed_methods.push("BROWSE".to_string());
    config.allowed_methods.push("not a token".to_string());
    assert!(matches!(
        config.validate(),
        Err(CorsError::InvalidMethod(_))
    ));

    let mut config = base_config();
    config.allowed_headers.push("X Bad Header".to_string());
    assert!(matches!(
        config.validate(),
        Err(CorsError::InvalidHeaderName(_))
    ));

    let mut config = base_config();
    config.max_age_seconds = MAX_ALLOWED_MAX_AGE_SECONDS + 1;
    assert!(matches!(
        config.validate(),
        Err(CorsError::MaxAgeOutOfRange(_))
    ));
}

#[test]
fn origin_allowlist_is_dynamic_at_runtime() {
    let mut allowlist = OriginAllowlist::new();

    assert!(!allowlist.allows("https://one.example"));
    allowlist.add("https://one.example").unwrap();
    allowlist.add("https://*.partners.example").unwrap();
    assert_eq!(allowlist.len(), 2);

    assert!(allowlist.allows("https://one.example"));
    assert!(allowlist.allows("https://ONE.Example"));
    assert!(allowlist.allows("https://shop.partners.example"));
    assert!(!allowlist.allows("https://a.b.partners.example"));
    assert!(!allowlist.allows("http://one.example"));

    assert!(allowlist.remove("https://one.example").unwrap());
    assert!(!allowlist.allows("https://one.example"));
    assert!(allowlist.allows("https://shop.partners.example"));

    allowlist.clear();
    assert!(allowlist.is_empty());
    assert!(!allowlist.allows("https://shop.partners.example"));
}

#[test]
fn default_port_normalization_matches_browser_semantics() {
    let mut allowlist = OriginAllowlist::new();
    allowlist.add("http://gateway.example:80").unwrap();
    allowlist.add("https://secure.example").unwrap();

    assert!(allowlist.allows("http://gateway.example"));
    assert!(allowlist.allows("https://secure.example:443"));
    assert!(!allowlist.allows("https://gateway.example"));
}

#[test]
fn preflight_denies_disallowed_requests_without_headers() {
    let config = base_config();

    for (origin, method, headers) in [
        ("https://intruder.example", "GET", &[][..]),
        ("https://app.cougr.test", "DELETE", &[][..]),
        ("https://app.cougr.test", "GET", &["X-Sneaky"][..]),
        ("garbage", "GET", &[][..]),
    ] {
        let decision = config.evaluate_preflight(origin, method, headers);
        assert_eq!(decision, PreflightDecision::denied());
        assert!(!decision.allowed);
        assert_eq!(decision.allow_origin, None);
    }
}

#[test]
fn evaluate_methods_deny_unvalidated_unsafe_policy() {
    let mut config = CorsConfig::new();
    config.allow_credentials = true;
    config.origins.add("*").unwrap();

    let decision = config.evaluate_preflight("https://attacker.example", "GET", &[]);
    assert_eq!(decision, PreflightDecision::denied());
    assert_eq!(
        config.response_allow_origin("https://attacker.example"),
        None
    );
}

#[test]
fn evaluate_methods_deny_after_runtime_mutation_breaks_validation() {
    let mut config = base_config();
    config.allow_credentials = true;
    config.validate().unwrap();

    assert!(
        config
            .evaluate_preflight("https://app.cougr.test", "GET", &[])
            .allowed
    );

    config.allowed_methods.push("*".to_string());
    assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));
    assert_eq!(
        config.evaluate_preflight("https://app.cougr.test", "GET", &[]),
        PreflightDecision::denied()
    );
    assert_eq!(config.response_allow_origin("https://app.cougr.test"), None);
}

#[test]
fn wildcard_origin_policy_reflects_credentials_mode() {
    let mut config = CorsConfig::new();
    config.origins.add("*").unwrap();
    config.validate().unwrap();

    let decision = config.evaluate_preflight("https://anywhere.example", "GET", &[]);
    assert_eq!(decision.allow_origin.as_deref(), Some("*"));

    // Credentials + `*` is rejected before the policy can go live...
    config.allow_credentials = true;
    assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));

    // ...so an exact-origin credentials policy echoes the origin instead.
    let mut strict = CorsConfig::new();
    strict.origins.add("https://player.example").unwrap();
    strict.allow_credentials = true;
    strict.validate().unwrap();
    let decision = strict.evaluate_preflight("https://player.example", "GET", &[]);
    assert_eq!(
        decision.allow_origin.as_deref(),
        Some("https://player.example")
    );
    assert!(decision.allow_credentials);

    // Actual responses follow the same rule.
    assert_eq!(
        strict.response_allow_origin("https://player.example"),
        Some("https://player.example".to_string())
    );
    assert_eq!(strict.response_allow_origin("https://other.example"), None);
}

#[test]
fn module_version_marker_exposed() {
    assert_eq!(MODULE_VERSION, "0.1.0-cors");
}
