use anyhow::{Context, Result, bail};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// ── Auth mode ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthMode {
    None,
    Iap,
    Bearer,
}

impl AuthMode {
    pub fn from_env() -> Self {
        match std::env::var("CURIO_SERVICE_AUTH_MODE")
            .ok()
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("iap") => AuthMode::Iap,
            Some("bearer") => AuthMode::Bearer,
            _ => AuthMode::None,
        }
    }
}

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub mode: AuthMode,
    /// Expected audience for IAP JWTs (`/projects/<n>/global/backendServices/<id>`)
    pub iap_audience: Option<String>,
    /// Service account email that Pub/Sub uses for OIDC push auth
    pub pubsub_sa_email: Option<String>,
    /// Cloud Run service URL used as the OIDC audience for the pubsub path
    pub service_url: Option<String>,
    /// Static bearer token (mode == Bearer only)
    pub bearer_token: Option<String>,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            mode: AuthMode::from_env(),
            iap_audience: std::env::var("CURIO_IAP_AUDIENCE")
                .ok()
                .filter(|s| !s.is_empty()),
            pubsub_sa_email: std::env::var("CURIO_PUBSUB_SA_EMAIL")
                .ok()
                .filter(|s| !s.is_empty()),
            service_url: std::env::var("CURIO_SERVICE_URL")
                .ok()
                .filter(|s| !s.is_empty()),
            bearer_token: std::env::var("CURIO_SERVICE_BEARER_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
        }
    }
}

// ── Principal ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum PrincipalKind {
    Human,
    ServiceAccount,
    System,
}

#[derive(Debug, Clone)]
pub struct VerifiedPrincipal {
    pub email: String,
    pub kind: PrincipalKind,
}

// ── JWKS cache ─────────────────────────────────────────────────────────────

const JWKS_TTL_SECS: u64 = 300;

struct JwksCacheEntry {
    jwks_json: String,
    fetched_at: std::time::Instant,
}

#[derive(Clone)]
pub struct JwksCache {
    inner: Arc<Mutex<HashMap<String, JwksCacheEntry>>>,
    http: reqwest::Client,
}

impl Default for JwksCache {
    fn default() -> Self {
        Self::new()
    }
}

impl JwksCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            http: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        let now = std::time::Instant::now();
        {
            let guard = self.inner.lock().await;
            if let Some(entry) = guard.get(url)
                && now.duration_since(entry.fetched_at).as_secs() < JWKS_TTL_SECS
            {
                return Ok(entry.jwks_json.clone());
            }
        }
        let jwks_json = self
            .http
            .get(url)
            .send()
            .await
            .context("JWKS fetch failed")?
            .text()
            .await
            .context("JWKS body read failed")?;
        {
            let mut guard = self.inner.lock().await;
            guard.insert(
                url.to_string(),
                JwksCacheEntry {
                    jwks_json: jwks_json.clone(),
                    fetched_at: now,
                },
            );
        }
        Ok(jwks_json)
    }
}

// ── AuthState ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AuthState {
    pub config: AuthConfig,
    pub cache: JwksCache,
}

impl AuthState {
    pub fn from_env() -> Self {
        Self {
            config: AuthConfig::from_env(),
            cache: JwksCache::new(),
        }
    }
}

// ── JWT internal types ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GoogleClaims {
    email: String,
    sub: String,
}

#[derive(Debug, Deserialize)]
struct RawJwk {
    kid: Option<String>,
    kty: String,
    // EC fields (crv is parsed by serde but we use x/y directly)
    #[allow(dead_code)]
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
    // RSA fields
    n: Option<String>,
    e: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawJwkSet {
    keys: Vec<RawJwk>,
}

const IAP_JWKS_URL: &str = "https://www.gstatic.com/iap/verify/public_key-jwk";
const GOOGLE_OIDC_JWKS_URL: &str = "https://www.googleapis.com/oauth2/v3/certs";

fn decoding_key_for_jwk(jwk: &RawJwk) -> Result<DecodingKey> {
    match jwk.kty.as_str() {
        "EC" => {
            let x = jwk.x.as_deref().context("EC JWK missing x")?;
            let y = jwk.y.as_deref().context("EC JWK missing y")?;
            DecodingKey::from_ec_components(x, y).context("EC DecodingKey failed")
        }
        "RSA" => {
            let n = jwk.n.as_deref().context("RSA JWK missing n")?;
            let e = jwk.e.as_deref().context("RSA JWK missing e")?;
            DecodingKey::from_rsa_components(n, e).context("RSA DecodingKey failed")
        }
        other => bail!("Unsupported JWK kty: {}", other),
    }
}

fn find_key<'a>(keyset: &'a RawJwkSet, kid: &str) -> Option<&'a RawJwk> {
    keyset
        .keys
        .iter()
        .find(|k| k.kid.as_deref() == Some(kid))
        .or_else(|| keyset.keys.first())
}

async fn verify_google_jwt(
    token: &str,
    jwks_url: &str,
    alg: Algorithm,
    audience: &str,
    cache: &JwksCache,
) -> Result<GoogleClaims> {
    let header = decode_header(token).context("JWT header decode failed")?;
    let kid = header.kid.as_deref().unwrap_or("");

    let jwks_json = cache.get(jwks_url).await?;
    let keyset: RawJwkSet = serde_json::from_str(&jwks_json).context("JWKS parse failed")?;
    let jwk = find_key(&keyset, kid).context("No matching JWK found")?;
    let decoding_key = decoding_key_for_jwk(jwk)?;

    let mut validation = Validation::new(alg);
    validation.set_audience(&[audience]);

    let data = decode::<GoogleClaims>(token, &decoding_key, &validation)
        .context("JWT validation failed")?;
    Ok(data.claims)
}

// ── Public verifiers ───────────────────────────────────────────────────────

/// Verify a Google Cloud IAP JWT (ES256).
pub async fn verify_iap_jwt(
    token: &str,
    audience: &str,
    cache: &JwksCache,
) -> Result<VerifiedPrincipal> {
    let claims = verify_google_jwt(token, IAP_JWKS_URL, Algorithm::ES256, audience, cache).await?;
    let kind = if claims.sub.contains(':') {
        PrincipalKind::Human
    } else {
        PrincipalKind::ServiceAccount
    };
    Ok(VerifiedPrincipal {
        email: claims.email,
        kind,
    })
}

/// Verify a Pub/Sub OIDC push JWT (RS256). Checks that the token's email
/// matches `expected_sa_email` and the audience matches `audience`.
pub async fn verify_pubsub_oidc(
    token: &str,
    expected_sa_email: &str,
    audience: &str,
    cache: &JwksCache,
) -> Result<VerifiedPrincipal> {
    let claims = verify_google_jwt(
        token,
        GOOGLE_OIDC_JWKS_URL,
        Algorithm::RS256,
        audience,
        cache,
    )
    .await?;
    if claims.email != expected_sa_email {
        bail!(
            "OIDC token email {} does not match expected {}",
            claims.email,
            expected_sa_email
        );
    }
    Ok(VerifiedPrincipal {
        email: claims.email,
        kind: PrincipalKind::ServiceAccount,
    })
}

/// Verify a static bearer token.
pub fn verify_bearer(token: &str, expected: &str) -> Result<VerifiedPrincipal> {
    if !expected.is_empty() && token == expected {
        Ok(VerifiedPrincipal {
            email: "system@curio.local".to_string(),
            kind: PrincipalKind::System,
        })
    } else {
        bail!("Bearer token invalid")
    }
}
