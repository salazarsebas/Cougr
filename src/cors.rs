//! CORS configuration validation and dynamic origin allowlist.
//!
//! Cougr games commonly ship HTTP surfaces in front of on-chain state:
//! indexer gateways, session servers, and leaderboard APIs. Those endpoints
//! need a Cross-Origin Resource Sharing (CORS) policy that is validated
//! before it goes live and that can change at runtime as partner frontends
//! are added or removed.
//!
//! This module provides both halves of that workflow with no HTTP
//! dependency and no Soroban storage footprint (it is intended for
//! gateway/server tooling around a game contract):
//!
//! - [`CorsConfig`] models the full policy and validates it eagerly so
//!   unsafe combinations fail fast instead of leaking cross-origin access.
//! - [`OriginAllowlist`] is a runtime-mutable set of origins supporting
//!   exact entries, single-label wildcard subdomain patterns
//!   (`https://*.example.com`), and an explicit allow-all wildcard (`*`).
//!
//! # Example
//!
//! ```
//! use cougr_core::cors::{CorsConfig, OriginAllowlist};
//!
//! let mut config = CorsConfig::new();
//! config.origins.add("https://dashboard.example.com").unwrap();
//! config.origins.add("https://*.partners.example").unwrap();
//! config.validate().expect("policy is safe");
//!
//! assert!(config.origins.allows("https://dashboard.example.com"));
//! assert!(config.origins.allows("https://api.partners.example"));
//! assert!(!config.origins.allows("https://evil.example.org"));
//!
//! config.origins.remove("https://dashboard.example.com");
//! assert!(!config.origins.allows("https://dashboard.example.com"));
//! ```

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Version marker for the CORS policy module.
pub const MODULE_VERSION: &str = "0.1.0-cors";

/// The wildcard token accepted for origins, methods, and header names.
pub const WILDCARD: &str = "*";

/// Upper bound accepted by [`CorsConfig::validate`] for the preflight cache
/// lifetime. Browsers clamp this header anyway; rejecting larger values here
/// keeps configurations honest about intent.
pub const MAX_ALLOWED_MAX_AGE_SECONDS: u32 = 86_400;

/// Default preflight cache lifetime applied by [`CorsConfig::new`].
pub const DEFAULT_MAX_AGE_SECONDS: u32 = 600;

/// Request headers browsers treat as safelisted: they never need to be
/// declared in `Access-Control-Allow-Headers`.
pub const SAFELISTED_REQUEST_HEADERS: [&str; 3] = ["accept", "accept-language", "content-language"];

/// Methods applied by [`CorsConfig::new`].
pub const DEFAULT_ALLOWED_METHODS: [&str; 3] = ["GET", "HEAD", "POST"];

const MAX_ORIGIN_LENGTH: usize = 512;
const MAX_HOST_LENGTH: usize = 253;
const MAX_LABEL_LENGTH: usize = 63;
const MAX_HEADER_NAME_LENGTH: usize = 128;

/// Errors produced while parsing origins or validating a CORS policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CorsError {
    /// A required value was empty.
    EmptyValue,
    /// An origin was missing its `scheme://` prefix.
    MissingScheme,
    /// The scheme was not `http` or `https`.
    InvalidScheme(String),
    /// An origin had no host component after the scheme separator.
    MissingHost,
    /// The host component violated hostname syntax rules.
    InvalidHost(String),
    /// The port was not a decimal number in `1..=65535`.
    InvalidPort(String),
    /// An origin carried a path, query, fragment, or userinfo component.
    OriginIncludesPath(String),
    /// Credentials were enabled while a wildcard origin, method, or header
    /// was configured. Browsers reject that combination outright.
    CredentialsWithWildcard,
    /// A method was not a valid HTTP token.
    InvalidMethod(String),
    /// A header name was not a valid HTTP token.
    InvalidHeaderName(String),
    /// The preflight max age exceeded [`MAX_ALLOWED_MAX_AGE_SECONDS`].
    MaxAgeOutOfRange(u32),
}

impl core::fmt::Display for CorsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CorsError::EmptyValue => write!(f, "value must not be empty"),
            CorsError::MissingScheme => write!(f, "origin is missing a scheme:// prefix"),
            CorsError::InvalidScheme(scheme) => write!(
                f,
                "unsupported origin scheme {scheme:?}: expected http or https"
            ),
            CorsError::MissingHost => write!(f, "origin is missing a host"),
            CorsError::InvalidHost(host) => write!(f, "invalid origin host {host:?}"),
            CorsError::InvalidPort(port) => write!(f, "invalid origin port {port:?}"),
            CorsError::OriginIncludesPath(raw) => write!(
                f,
                "origin {raw:?} must not include a path, query, fragment, or userinfo"
            ),
            CorsError::CredentialsWithWildcard => write!(
                f,
                "wildcard values cannot be combined with allow_credentials"
            ),
            CorsError::InvalidMethod(method) => write!(f, "invalid HTTP method {method:?}"),
            CorsError::InvalidHeaderName(header) => write!(f, "invalid header name {header:?}"),
            CorsError::MaxAgeOutOfRange(max_age) => write!(
                f,
                "max_age_seconds {max_age} exceeds limit of {MAX_ALLOWED_MAX_AGE_SECONDS}"
            ),
        }
    }
}

fn is_origin_delimiter(c: char) -> bool {
    matches!(c, '/' | '?' | '#' | '@')
}

/// A parsed browser origin (`scheme://host[:port]`) in normalized form.
///
/// Schemes and hostnames are lowercased, and explicit default ports are
/// folded away (`http://host:80` compares equal to `http://host`), so two
/// origins written differently still match per the URL equality rules used
/// by browsers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Origin {
    scheme: String,
    host: String,
    port: Option<u16>,
}

impl Origin {
    /// Parse and normalize a raw origin string.
    pub fn parse(raw: &str) -> Result<Self, CorsError> {
        if raw.is_empty() || raw.trim().is_empty() {
            return Err(CorsError::EmptyValue);
        }
        if raw.chars().any(char::is_whitespace) {
            return Err(CorsError::InvalidHost(raw.to_string()));
        }
        if raw.len() > MAX_ORIGIN_LENGTH {
            return Err(CorsError::InvalidHost(raw.to_string()));
        }

        let (scheme_raw, authority) = raw.split_once("://").ok_or(CorsError::MissingScheme)?;
        let scheme = validate_scheme(scheme_raw)?;
        if authority.is_empty() {
            return Err(CorsError::MissingHost);
        }
        if authority.chars().any(is_origin_delimiter) {
            return Err(CorsError::OriginIncludesPath(raw.to_string()));
        }

        let (host_raw, port_raw) = split_host_port(authority)?;
        let host = validate_host(host_raw)?;
        let port = resolve_port(&scheme, port_raw)?;

        Ok(Origin { scheme, host, port })
    }

    /// Lowercased scheme of the origin.
    pub fn scheme(&self) -> &str {
        &self.scheme
    }

    /// Lowercased host of the origin (IPv6 literals keep their brackets).
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Explicit non-default port, if any.
    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

fn validate_scheme(scheme_raw: &str) -> Result<String, CorsError> {
    if scheme_raw.is_empty() {
        return Err(CorsError::MissingScheme);
    }
    let lowered = scheme_raw.to_ascii_lowercase();
    if lowered != "http" && lowered != "https" {
        return Err(CorsError::InvalidScheme(lowered));
    }
    Ok(lowered)
}

/// Split `host[:port]`, honoring bracketed IPv6 literals.
fn split_host_port(authority: &str) -> Result<(&str, Option<&str>), CorsError> {
    if let Some(rest) = authority.strip_prefix('[') {
        let closing = rest
            .find(']')
            .ok_or_else(|| CorsError::InvalidHost(authority.to_string()))?;
        let inner = &rest[..closing];
        if inner.is_empty() {
            return Err(CorsError::InvalidHost(authority.to_string()));
        }
        let remainder = &rest[closing + 1..];
        if remainder.is_empty() {
            return Ok((authority.get(..closing + 2).unwrap_or(authority), None));
        }
        let port = remainder
            .strip_prefix(':')
            .ok_or_else(|| CorsError::InvalidHost(authority.to_string()))?;
        return Ok((
            authority.get(..closing + 2).unwrap_or(authority),
            Some(port),
        ));
    }

    match authority.rfind(':') {
        None => Ok((authority, None)),
        Some(index) => {
            let host = &authority[..index];
            let port = &authority[index + 1..];
            if host.contains(':') {
                // Bare (unbracketed) IPv6 literal - reject as a host error.
                return Err(CorsError::InvalidHost(authority.to_string()));
            }
            if host.is_empty() || port.is_empty() {
                return Err(CorsError::InvalidPort(port.to_string()));
            }
            Ok((host, Some(port)))
        }
    }
}

fn parse_port(port_raw: &str) -> Result<u16, CorsError> {
    if port_raw.is_empty() || !port_raw.bytes().all(|b| b.is_ascii_digit()) {
        return Err(CorsError::InvalidPort(port_raw.to_string()));
    }
    // Reject leading zeros to keep serialization canonical.
    if port_raw.len() > 1 && port_raw.starts_with('0') {
        return Err(CorsError::InvalidPort(port_raw.to_string()));
    }
    match port_raw.parse::<u16>() {
        Ok(port) if port != 0 => Ok(port),
        _ => Err(CorsError::InvalidPort(port_raw.to_string())),
    }
}

fn default_port_for(scheme: &str) -> u16 {
    if scheme == "https" {
        443
    } else {
        80
    }
}

/// Parse an optional textual port and fold default ports away.
fn resolve_port(scheme: &str, port_raw: Option<&str>) -> Result<Option<u16>, CorsError> {
    match port_raw {
        None => Ok(None),
        Some(raw) => {
            let port = parse_port(raw)?;
            if port == default_port_for(scheme) {
                Ok(None)
            } else {
                Ok(Some(port))
            }
        }
    }
}

fn validate_host(host_raw: &str) -> Result<String, CorsError> {
    let lowered = host_raw.to_ascii_lowercase();
    if lowered.is_empty() || lowered.len() > MAX_HOST_LENGTH {
        return Err(CorsError::InvalidHost(host_raw.to_string()));
    }

    if let Some(inner) = lowered
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        // Bracketed IPv6 literal: sanity-check hex digits and separators.
        let plausible = !inner.is_empty()
            && inner
                .bytes()
                .all(|b| b.is_ascii_hexdigit() || b == b':' || b == b'.');
        if plausible {
            return Ok(lowered);
        }
        return Err(CorsError::InvalidHost(host_raw.to_string()));
    }

    for label in lowered.split('.') {
        if label.is_empty() || label.len() > MAX_LABEL_LENGTH {
            return Err(CorsError::InvalidHost(host_raw.to_string()));
        }
        let valid_label = label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            && !label.starts_with('-')
            && !label.ends_with('-');
        if !valid_label {
            return Err(CorsError::InvalidHost(host_raw.to_string()));
        }
    }
    Ok(lowered)
}

/// True when `host` ends with `suffix` and adds exactly one extra label
/// (`api.example.com` matches `.example.com`; `a.b.example.com` does not).
fn wildcard_matches_host(host: &str, suffix: &str) -> bool {
    match host.strip_suffix(suffix) {
        Some(labels) => !labels.is_empty() && !labels.contains('.'),
        None => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AllowEntry {
    /// Allow every origin.
    Any,
    /// Allow one exact origin.
    Exact(Origin),
    /// Allow `scheme://label.suffix[:port]` where `label` is exactly one
    /// additional hostname label (wildcards never span dots).
    WildcardSubdomain {
        scheme: String,
        suffix: String,
        port: Option<u16>,
    },
}

impl AllowEntry {
    fn matches(&self, origin: &Origin) -> bool {
        match self {
            AllowEntry::Any => true,
            AllowEntry::Exact(expected) => expected == origin,
            AllowEntry::WildcardSubdomain {
                scheme,
                suffix,
                port,
            } => {
                if &origin.scheme != scheme || &origin.port != port {
                    return false;
                }
                wildcard_matches_host(&origin.host, suffix)
            }
        }
    }
}

/// A runtime-mutable allowlist of origins.
///
/// Entries can be added and removed while a server is live, which makes this
/// type the dynamic half of [`CorsConfig`]. Three entry shapes are supported:
///
/// - exact origins: `https://api.example.com`
/// - single-label wildcard subdomains: `https://*.example.com`
/// - the global wildcard: `*`
///
/// Matching is case-insensitive for scheme and host and treats explicit
/// default ports as equal to their omission.
#[derive(Clone, Debug, Default)]
pub struct OriginAllowlist {
    entries: Vec<(String, AllowEntry)>,
}

impl OriginAllowlist {
    /// Create an empty allowlist (all origins denied).
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an origin pattern to the allowlist.
    pub fn add(&mut self, pattern: &str) -> Result<(), CorsError> {
        let entry = Self::parse_pattern(pattern)?;
        self.entries.push((pattern.to_string(), entry));
        Ok(())
    }

    /// Remove an origin pattern previously added with the same shape.
    ///
    /// Malformed patterns are reported as errors; a well-formed pattern that
    /// is not present returns `Ok(false)`.
    pub fn remove(&mut self, pattern: &str) -> Result<bool, CorsError> {
        let entry = Self::parse_pattern(pattern)?;
        let before = self.entries.len();
        self.entries.retain(|(_, existing)| existing != &entry);
        Ok(self.entries.len() != before)
    }

    /// Remove every entry, returning to deny-by-default.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Check whether an origin string is allowed right now.
    ///
    /// Unparseable origins are never allowed.
    pub fn allows(&self, origin: &str) -> bool {
        match Origin::parse(origin) {
            Ok(parsed) => self.entries.iter().any(|(_, entry)| entry.matches(&parsed)),
            Err(_) => false,
        }
    }

    /// Number of entries currently registered.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no entries are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The registered patterns, in insertion order.
    pub fn patterns(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|(pattern, _)| pattern.clone())
            .collect()
    }

    /// Whether the list contains the global `*` wildcard.
    pub fn is_wildcard_all(&self) -> bool {
        self.entries
            .iter()
            .any(|(_, entry)| matches!(entry, AllowEntry::Any))
    }

    fn parse_pattern(pattern: &str) -> Result<AllowEntry, CorsError> {
        if pattern.is_empty() {
            return Err(CorsError::EmptyValue);
        }
        if pattern == WILDCARD {
            return Ok(AllowEntry::Any);
        }

        let (scheme_raw, authority) = pattern.split_once("://").ok_or(CorsError::MissingScheme)?;
        let scheme = validate_scheme(scheme_raw)?;
        if authority.is_empty() {
            return Err(CorsError::MissingHost);
        }
        if authority.chars().any(is_origin_delimiter) {
            return Err(CorsError::OriginIncludesPath(pattern.to_string()));
        }

        if let Some(stripped) = authority.strip_prefix("*.") {
            // `stripped` is a plain hostname with an optional port.
            let (host_part, port_raw) = match stripped.rfind(':') {
                Some(index) => (&stripped[..index], Some(&stripped[index + 1..])),
                None => (stripped, None),
            };
            let suffix_host = validate_host(host_part)?;
            let port = resolve_port(&scheme, port_raw)?;
            return Ok(AllowEntry::WildcardSubdomain {
                scheme,
                suffix: format!(".{suffix_host}"),
                port,
            });
        }

        if authority.starts_with('*') {
            return Err(CorsError::InvalidHost(authority.to_string()));
        }

        let origin = Origin::parse(pattern)?;
        Ok(AllowEntry::Exact(origin))
    }
}

/// A CORS policy for HTTP gateways in front of a game contract.
///
/// Construct via [`CorsConfig::new`] (deny-by-default with the common simple
/// methods preconfigured) or [`CorsConfig::empty`], mutate fields, then call
/// [`CorsConfig::validate`] before installing the policy. Validation rejects
/// unsafe combinations such as credentials together with any wildcard value.
#[derive(Clone, Debug)]
pub struct CorsConfig {
    /// Dynamic origin allowlist consulted for every request.
    pub origins: OriginAllowlist,
    /// Methods permitted on cross-origin requests.
    pub allowed_methods: Vec<String>,
    /// Non-safelisted request headers permitted on preflight requests.
    pub allowed_headers: Vec<String>,
    /// Response headers exposed to cross-origin clients.
    pub exposed_headers: Vec<String>,
    /// Whether credentials (cookies, TLS client certs) may be sent.
    pub allow_credentials: bool,
    /// Preflight cache lifetime advertised to browsers.
    pub max_age_seconds: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CorsConfig {
    /// A policy preconfigured with [`DEFAULT_ALLOWED_METHODS`] and
    /// [`DEFAULT_MAX_AGE_SECONDS`], allowing no origins yet.
    pub fn new() -> Self {
        Self {
            origins: OriginAllowlist::new(),
            allowed_methods: DEFAULT_ALLOWED_METHODS
                .iter()
                .map(|method| (*method).to_string())
                .collect(),
            allowed_headers: Vec::new(),
            exposed_headers: Vec::new(),
            allow_credentials: false,
            max_age_seconds: DEFAULT_MAX_AGE_SECONDS,
        }
    }

    /// A fully empty policy: no methods, no headers, zero max age.
    pub fn empty() -> Self {
        Self {
            origins: OriginAllowlist::new(),
            allowed_methods: Vec::new(),
            allowed_headers: Vec::new(),
            exposed_headers: Vec::new(),
            allow_credentials: false,
            max_age_seconds: 0,
        }
    }

    /// Validate the whole policy, failing fast on unsafe or malformed input.
    pub fn validate(&self) -> Result<(), CorsError> {
        if self.max_age_seconds > MAX_ALLOWED_MAX_AGE_SECONDS {
            return Err(CorsError::MaxAgeOutOfRange(self.max_age_seconds));
        }
        if self.allow_credentials && self.origins.is_wildcard_all() {
            return Err(CorsError::CredentialsWithWildcard);
        }
        if self.allow_credentials
            && (self.allowed_methods.iter().any(|m| m == WILDCARD)
                || self.allowed_headers.iter().any(|h| h == WILDCARD)
                || self.exposed_headers.iter().any(|h| h == WILDCARD))
        {
            return Err(CorsError::CredentialsWithWildcard);
        }

        for method in &self.allowed_methods {
            validate_method(method)?;
        }
        for header in self
            .allowed_headers
            .iter()
            .chain(self.exposed_headers.iter())
        {
            validate_header_name(header)?;
        }
        Ok(())
    }

    /// Evaluate a preflight (OPTIONS) request against this policy.
    ///
    /// Re-runs [`CorsConfig::validate`] on every call and returns
    /// [`PreflightDecision::denied`] when the policy is unsafe or malformed,
    /// including after runtime mutations that introduce invalid combinations
    /// (for example, credentials together with a wildcard origin).
    ///
    /// Safelisted request headers ([`SAFELISTED_REQUEST_HEADERS`]) never need
    /// explicit configuration; every other requested header must be listed in
    /// `allowed_headers` for the preflight to pass.
    pub fn evaluate_preflight(
        &self,
        origin: &str,
        request_method: &str,
        request_headers: &[&str],
    ) -> PreflightDecision {
        if self.validate().is_err() {
            return PreflightDecision::denied();
        }
        if !self.origins.allows(origin) || !self.method_allowed(request_method) {
            return PreflightDecision::denied();
        }

        let mut granted_headers: Vec<&str> = Vec::new();
        for header in request_headers {
            let header_allowed = SAFELISTED_REQUEST_HEADERS
                .iter()
                .any(|safe| safe.eq_ignore_ascii_case(header))
                || self
                    .allowed_headers
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(header));
            if !header_allowed {
                return PreflightDecision::denied();
            }
            granted_headers.push(header);
        }

        let allow_origin = if self.origins.is_wildcard_all() && !self.allow_credentials {
            WILDCARD.to_string()
        } else {
            origin.to_string()
        };

        PreflightDecision {
            allowed: true,
            allow_origin: Some(allow_origin),
            allow_methods: Some(self.allowed_methods.join(", ")),
            allow_headers: Some(granted_headers.join(", ")),
            allow_credentials: self.allow_credentials,
            max_age_seconds: self.max_age_seconds,
        }
    }

    /// Value for the `Access-Control-Allow-Origin` response header on an
    /// actual (non-preflight) request, or `None` when the origin is denied.
    ///
    /// Re-runs [`CorsConfig::validate`] on every call and returns `None` when
    /// the policy is unsafe or malformed.
    pub fn response_allow_origin(&self, origin: &str) -> Option<String> {
        if self.validate().is_err() {
            return None;
        }
        if !self.origins.allows(origin) {
            return None;
        }
        if self.origins.is_wildcard_all() && !self.allow_credentials {
            Some(WILDCARD.to_string())
        } else {
            Some(origin.to_string())
        }
    }

    fn method_allowed(&self, method: &str) -> bool {
        self.allowed_methods
            .iter()
            .any(|allowed| allowed == method || allowed == WILDCARD)
    }
}

/// Outcome of evaluating a preflight request against a [`CorsConfig`].
///
/// When [`PreflightDecision::allowed`] is `false`, every header field is
/// `None`: the caller must not emit CORS response headers at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreflightDecision {
    /// Whether the preflight may proceed.
    pub allowed: bool,
    /// `Access-Control-Allow-Origin` value (echoed origin or `*`).
    pub allow_origin: Option<String>,
    /// `Access-Control-Allow-Methods` value.
    pub allow_methods: Option<String>,
    /// `Access-Control-Allow-Headers` value (echoed requested headers).
    pub allow_headers: Option<String>,
    /// `Access-Control-Allow-Credentials` flag.
    pub allow_credentials: bool,
    /// `Access-Control-Max-Age` value.
    pub max_age_seconds: u32,
}

impl PreflightDecision {
    /// The canonical denied decision: no CORS headers may be emitted.
    pub fn denied() -> Self {
        Self {
            allowed: false,
            allow_origin: None,
            allow_methods: None,
            allow_headers: None,
            allow_credentials: false,
            max_age_seconds: 0,
        }
    }
}

/// Validate an RFC 7230 token (methods and header names share the grammar).
fn validate_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_HEADER_NAME_LENGTH
        && value.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || matches!(
                    b,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

fn validate_method(method: &str) -> Result<(), CorsError> {
    if method == WILDCARD || validate_token(method) {
        Ok(())
    } else {
        Err(CorsError::InvalidMethod(method.to_string()))
    }
}

fn validate_header_name(name: &str) -> Result<(), CorsError> {
    if validate_token(name) {
        Ok(())
    } else {
        Err(CorsError::InvalidHeaderName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_parse_normalizes_case_and_default_ports() {
        let origin = Origin::parse("HTTPS://Example.COM:443").unwrap();
        assert_eq!(origin.scheme(), "https");
        assert_eq!(origin.host(), "example.com");
        assert_eq!(origin.port(), None);
        assert_eq!(origin, Origin::parse("https://example.com").unwrap());
    }

    #[test]
    fn test_parse_keeps_non_default_ports() {
        let origin = Origin::parse("http://localhost:3000").unwrap();
        assert_eq!(origin.port(), Some(3000));
        assert_ne!(origin, Origin::parse("http://localhost").unwrap());
    }

    #[test]
    fn test_parse_rejects_malformed_origins() {
        let cases = [
            "",
            "   ",
            "example.com",
            "://host",
            "ftp://host",
            "https:///path",
            "https://host/path",
            "https://host?q=1",
            "https://host#frag",
            "https://user@host",
            "https://host:0",
            "https://host:65536",
            "https://host:abc",
            "https://host:",
            "https://ho st:80",
            "https://bad..host",
            "https://-bad.host",
            "https://bad-.host",
            "https://[::1",
        ];
        for case in cases {
            assert!(
                Origin::parse(case).is_err(),
                "expected rejection of {case:?}"
            );
        }
    }

    #[test]
    fn test_parse_accepts_ipv6_literal_with_port() {
        let origin = Origin::parse("http://[2001:db8::1]:8080").unwrap();
        assert_eq!(origin.host(), "[2001:db8::1]");
        assert_eq!(origin.port(), Some(8080));
        assert_eq!(
            origin.port(),
            Origin::parse("http://[2001:DB8::1]:8080").unwrap().port()
        );
    }

    #[test]
    fn test_allowlist_exact_and_dynamic_mutation() {
        let mut allowlist = OriginAllowlist::new();
        assert!(allowlist.is_empty());
        assert!(!allowlist.allows("https://a.example"));

        allowlist.add("https://a.example").unwrap();
        allowlist.add("https://B.Example").unwrap();
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.allows("https://a.example"));
        assert!(allowlist.allows("https://b.example"));
        assert_eq!(
            allowlist.patterns(),
            vec![
                "https://a.example".to_string(),
                "https://B.Example".to_string()
            ]
        );

        assert!(allowlist.remove("https://a.example").unwrap());
        assert!(!allowlist.remove("https://a.example").unwrap());
        assert!(!allowlist.allows("https://a.example"));
    }

    #[test]
    fn test_allowlist_wildcard_subdomain_matches_single_label_only() {
        let mut allowlist = OriginAllowlist::new();
        allowlist.add("https://*.example.com").unwrap();

        assert!(allowlist.allows("https://api.example.com"));
        assert!(allowlist.allows("https://API.Example.COM"));
        assert!(!allowlist.allows("https://example.com"));
        assert!(!allowlist.allows("https://a.b.example.com"));
        assert!(!allowlist.allows("https://api.example.com:8443"));
        assert!(!allowlist.allows("http://api.example.com"));
        assert!(!allowlist.allows("https://evilexample.com"));
        assert!(!allowlist.allows("not an origin"));
    }

    #[test]
    fn test_allowlist_wildcard_subdomain_with_port() {
        let mut allowlist = OriginAllowlist::new();
        allowlist.add("https://*.dev.example:8443").unwrap();
        assert!(allowlist.allows("https://one.dev.example:8443"));
        assert!(!allowlist.allows("https://one.dev.example"));
    }

    #[test]
    fn test_allowlist_global_wildcard() {
        let mut allowlist = OriginAllowlist::new();
        allowlist.add("*").unwrap();
        assert!(allowlist.is_wildcard_all());
        assert!(allowlist.allows("https://anything.example"));
        assert!(allowlist.allows("http://other.test:9999"));

        allowlist.clear();
        assert!(allowlist.is_empty());
        assert!(!allowlist.is_wildcard_all());
    }

    #[test]
    fn test_allowlist_rejects_bad_patterns() {
        let mut allowlist = OriginAllowlist::new();
        assert_eq!(allowlist.add(""), Err(CorsError::EmptyValue));
        assert_eq!(allowlist.add("example.com"), Err(CorsError::MissingScheme));
        assert_eq!(
            allowlist.add("https://*/path"),
            Err(CorsError::OriginIncludesPath("https://*/path".to_string()))
        );
        assert!(allowlist.add("https://*.example.com/path").is_err());
        assert!(allowlist.add("https://*evil.example.com").is_err());
        assert!(allowlist.add("https://example.com:99999").is_err());
        assert_eq!(allowlist.remove("*"), Ok(false));
        assert!(allowlist.is_empty());
    }

    #[test]
    fn test_default_config_is_deny_by_default_but_valid() {
        let config = CorsConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.origins.is_empty());
        assert_eq!(config.max_age_seconds, DEFAULT_MAX_AGE_SECONDS);
        assert!(config.method_allowed("GET"));
        assert!(!config.method_allowed("DELETE"));
    }

    #[test]
    fn test_empty_config_has_nothing_enabled() {
        let config = CorsConfig::empty();
        assert!(config.validate().is_ok());
        assert!(config.allowed_methods.is_empty());
        assert_eq!(config.max_age_seconds, 0);
        assert!(!config.method_allowed("GET"));
    }

    #[test]
    fn test_validate_rejects_credentials_with_wildcards() {
        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.origins.add("*").unwrap();
        assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));

        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.allowed_methods = vec![WILDCARD.to_string()];
        assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));

        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.allowed_headers = vec![WILDCARD.to_string()];
        assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));

        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.origins.add("https://app.example").unwrap();
        assert!(config.validate().is_ok());

        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.exposed_headers = vec![WILDCARD.to_string()];
        assert_eq!(config.validate(), Err(CorsError::CredentialsWithWildcard));
    }

    #[test]
    fn test_validate_rejects_bad_methods_headers_and_max_age() {
        let mut config = CorsConfig::new();
        config.allowed_methods = vec!["GET".to_string(), "BAD METHOD".to_string()];
        assert!(matches!(
            config.validate(),
            Err(CorsError::InvalidMethod(_))
        ));

        let mut config = CorsConfig::new();
        config.allowed_methods = vec![String::new()];
        assert!(matches!(
            config.validate(),
            Err(CorsError::InvalidMethod(_))
        ));

        let mut config = CorsConfig::new();
        config.allowed_headers = vec!["X Game Token".to_string()];
        assert!(matches!(
            config.validate(),
            Err(CorsError::InvalidHeaderName(_))
        ));

        let mut config = CorsConfig::new();
        config.exposed_headers = vec!["X-Game-Token".to_string()];
        assert!(config.validate().is_ok());

        let mut config = CorsConfig::new();
        config.max_age_seconds = MAX_ALLOWED_MAX_AGE_SECONDS + 1;
        assert_eq!(
            config.validate(),
            Err(CorsError::MaxAgeOutOfRange(MAX_ALLOWED_MAX_AGE_SECONDS + 1))
        );

        let mut config = CorsConfig::new();
        config.max_age_seconds = MAX_ALLOWED_MAX_AGE_SECONDS;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_preflight_decision_echoes_origin_and_granted_headers() {
        let mut config = CorsConfig::new();
        config.origins.add("https://app.example").unwrap();
        config.allowed_methods = vec!["GET".to_string(), "POST".to_string()];
        config.allowed_headers = vec!["X-Game-Token".to_string()];

        let decision =
            config.evaluate_preflight("https://app.example", "POST", &["X-GAME-TOKEN", "Accept"]);
        assert!(decision.allowed);
        assert_eq!(
            decision.allow_origin.as_deref(),
            Some("https://app.example")
        );
        assert_eq!(decision.allow_methods.as_deref(), Some("GET, POST"));
        assert_eq!(
            decision.allow_headers.as_deref(),
            Some("X-GAME-TOKEN, Accept")
        );
        assert!(!decision.allow_credentials);
        assert_eq!(decision.max_age_seconds, DEFAULT_MAX_AGE_SECONDS);
    }

    #[test]
    fn test_preflight_denied_on_unknown_method_or_header() {
        let mut config = CorsConfig::new();
        config.origins.add("https://app.example").unwrap();
        config.allowed_headers = vec!["X-Game-Token".to_string()];

        let decision = config.evaluate_preflight("https://app.example", "DELETE", &[]);
        assert!(!decision.allowed);
        assert_eq!(decision.allow_origin, None);

        let decision = config.evaluate_preflight("https://app.example", "POST", &["X-Not-Allowed"]);
        assert!(!decision.allowed);

        let decision = config.evaluate_preflight("https://intruder.example", "GET", &[]);
        assert!(!decision.allowed);

        let denied = PreflightDecision::denied();
        assert_eq!(denied, decision);
    }

    #[test]
    fn test_preflight_wildcard_origin_uses_star_without_credentials() {
        let mut config = CorsConfig::new();
        config.origins.add("*").unwrap();
        let decision = config.evaluate_preflight("https://anywhere.example", "GET", &[]);
        assert!(decision.allowed);
        assert_eq!(decision.allow_origin.as_deref(), Some("*"));
    }

    #[test]
    fn test_evaluate_preflight_denies_invalid_policy_without_prior_validate() {
        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.origins.add("*").unwrap();

        let decision = config.evaluate_preflight("https://attacker.example", "GET", &[]);
        assert!(!decision.allowed);
        assert_eq!(decision, PreflightDecision::denied());
    }

    #[test]
    fn test_response_allow_origin_denies_invalid_policy_without_prior_validate() {
        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.origins.add("*").unwrap();

        assert_eq!(
            config.response_allow_origin("https://attacker.example"),
            None
        );
    }

    #[test]
    fn test_evaluate_preflight_denies_after_runtime_mutation_to_invalid_policy() {
        let mut config = CorsConfig::new();
        config.allow_credentials = true;
        config.origins.add("https://app.example").unwrap();
        config.validate().unwrap();

        let decision = config.evaluate_preflight("https://app.example", "GET", &[]);
        assert!(decision.allowed);

        config.origins.add("*").unwrap();
        let decision = config.evaluate_preflight("https://attacker.example", "GET", &[]);
        assert_eq!(decision, PreflightDecision::denied());
    }

    #[test]
    fn test_response_allow_origin_variants() {
        let mut config = CorsConfig::new();
        assert_eq!(config.response_allow_origin("https://app.example"), None);

        config.origins.add("https://app.example").unwrap();
        assert_eq!(
            config.response_allow_origin("https://app.example"),
            Some("https://app.example".to_string())
        );

        config.origins.clear();
        config.origins.add("*").unwrap();
        assert_eq!(
            config.response_allow_origin("https://app.example"),
            Some("*".to_string())
        );
    }

    #[test]
    fn test_display_messages_are_informative() {
        let message = CorsError::InvalidScheme("ftp".to_string()).to_string();
        assert!(message.contains("ftp"));
        let message = CorsError::MaxAgeOutOfRange(99_999).to_string();
        assert!(message.contains("86400"));
        let message = CorsError::InvalidHost("bad host".to_string()).to_string();
        assert!(message.contains("bad host"));
    }

    #[test]
    fn test_wildcard_helper_requires_single_label() {
        assert!(wildcard_matches_host("api.example.com", ".example.com"));
        assert!(!wildcard_matches_host("a.b.example.com", ".example.com"));
        assert!(!wildcard_matches_host("example.com", ".example.com"));
        assert!(!wildcard_matches_host("evilexample.com", ".example.com"));
    }

    #[test]
    fn test_module_version_marker() {
        assert_eq!(MODULE_VERSION, "0.1.0-cors");
    }
}
