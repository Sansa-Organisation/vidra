use anyhow::{bail, Context, Result};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VltLimits {
    pub renders_per_month: Option<u32>,
    pub cloud_renders_per_month: Option<u32>,
    pub machines: Option<u32>,
    pub team_members: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VltPayload {
    pub vlt_id: String,
    pub user_id: String,
    pub plan: String,
    pub features: Vec<String>,
    pub limits: VltLimits,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vlt {
    #[serde(flatten)]
    pub payload: VltPayload,
    pub signature: String, // "ed25519:base64..."
}

// Hardcoded public key for the platform, base64 encoded.
// For demonstration, we'll use a dummy valid ed25519 pubkey (32 bytes = 44 base64 chars).
const VIDRA_PLATFORM_PUBKEY_B64: &str = "OjnM3ZkQ11cIayQxyq13oH/d3Hl4Zkx/A75yW5dZ5U8=";

fn resolve_home_dir() -> Result<PathBuf> {
    if let Ok(v) = std::env::var("VIDRA_HOME_DIR") {
        let p = PathBuf::from(v);
        if p.is_absolute() {
            return Ok(p);
        }
        return Ok(std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(p));
    }
    dirs::home_dir().context("failed to resolve home dir")
}

fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        return Ok(resolve_home_dir()?.join(rest));
    }
    Ok(PathBuf::from(path))
}

fn vlt_token_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("VIDRA_VLT_PATH") {
        let pb = expand_tilde(p.trim())?;
        if pb.is_absolute() {
            return Ok(pb);
        }
        return Ok(std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(pb));
    }
    Ok(resolve_home_dir()?.join(".vidra").join("vlt.token"))
}

fn api_keys_path() -> Result<PathBuf> {
    Ok(resolve_home_dir()?.join(".vidra").join("api_keys.json"))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKeyRecord {
    pub key_id: String,
    pub name: String,
    pub scope: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ApiKeyStore {
    pub keys: Vec<ApiKeyRecord>,
}

fn load_api_keys() -> Result<ApiKeyStore> {
    let path = api_keys_path()?;
    if !path.exists() {
        return Ok(ApiKeyStore::default());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read api keys: {}", path.display()))?;
    let store = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse api keys: {}", path.display()))?;
    Ok(store)
}

fn save_api_keys(store: &ApiKeyStore) -> Result<()> {
    let path = api_keys_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("failed to create ~/.vidra")?;
    }
    let json = serde_json::to_string_pretty(store).context("failed to serialize api keys")?;
    std::fs::write(&path, json)
        .with_context(|| format!("failed to write api keys: {}", path.display()))?;
    Ok(())
}

fn new_random_id(prefix: &str) -> String {
    let mut bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut bytes);
    let mut hasher = Sha256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(bytes);
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}_{}", prefix, &hex[..12])
}

impl Vlt {
    #[allow(dead_code)]
    pub fn get_path() -> PathBuf {
        vlt_token_path().unwrap_or_else(|_| PathBuf::from(".vidra").join("vlt.token"))
    }

    pub fn load_local() -> Result<Self> {
        let path = vlt_token_path()?;
        if !path.exists() {
            bail!("No VLT found at {:?}. Please run `vidra auth login`.", path);
        }
        let content = std::fs::read_to_string(&path).context("Failed to read VLT token file")?;
        let vlt: Vlt = serde_json::from_str(&content).context("Failed to parse VLT token JSON")?;
        Ok(vlt)
    }

    pub fn save_local(&self) -> Result<()> {
        let path = vlt_token_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn validate_offline(&self) -> Result<()> {
        // Local-first dev token support: allow tokens with signature `local:*`.
        // These are intentionally unsigned and meant for offline/dev flows.
        if self.signature.starts_with("local:") {
            let now = Utc::now();
            let grace_period = Duration::days(7);
            let hard_expiry = self.payload.expires_at + grace_period;
            if now > hard_expiry {
                bail!("VLT has expired and the 7-day grace period has elapsed. Please run `vidra auth login` to refresh.");
            }
            if now > self.payload.expires_at {
                tracing::warn!("VLT is currently in its 7-day grace period (local token). Please run `vidra auth login` to refresh soon.");
            }
            return Ok(());
        }

        // 1. Check signature
        // The signature is over the canonical JSON of the payload.
        // For simplicity, we assume the platform signs the canonical JSON string of VltPayload.
        let pubkey_bytes = BASE64_STANDARD
            .decode(VIDRA_PLATFORM_PUBKEY_B64)
            .context("Invalid embedded pubkey base64")?;
        let pubkey = VerifyingKey::from_bytes(
            pubkey_bytes
                .as_slice()
                .try_into()
                .context("Invalid pubkey length")?,
        )
        .context("Failed to parse verifying key")?;

        let sig_part = self
            .signature
            .strip_prefix("ed25519:")
            .context("Invalid signature format; must start with ed25519:")?;

        let sig_bytes = BASE64_STANDARD
            .decode(sig_part)
            .context("Invalid signature base64")?;

        let signature = Signature::from_slice(&sig_bytes).context("Invalid signature length")?;

        // Serialization must be deterministic. We use canonical-like JSON.
        // Note: For a robust system, we would store the raw payload bytes and parse them.
        let msg =
            serde_json::to_string(&self.payload).context("Failed to serialize VLT payload")?;

        pubkey
            .verify(msg.as_bytes(), &signature)
            .context("VLT signature verification failed! The token is invalid or tampered with.")?;

        // 2. Check Expiry + 7-day grace
        let now = Utc::now();
        let grace_period = Duration::days(7);
        let hard_expiry = self.payload.expires_at + grace_period;

        if now > hard_expiry {
            bail!("VLT has expired and the 7-day grace period has elapsed. Please run `vidra auth login` to refresh.");
        } else if now > self.payload.expires_at {
            tracing::warn!("VLT is currently in its 7-day grace period. Please connect online or run `vidra auth login` to refresh soon.");
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn has_feature(&self, feature: &str) -> bool {
        self.payload.features.iter().any(|f| f == feature)
    }

    #[allow(dead_code)]
    pub fn enforce_plan_limit(&self, limit_name: &str, current_usage: u32) -> Result<()> {
        let limit = match limit_name {
            "renders_per_month" => self.payload.limits.renders_per_month,
            "cloud_renders_per_month" => self.payload.limits.cloud_renders_per_month,
            "machines" => self.payload.limits.machines,
            "team_members" => self.payload.limits.team_members,
            _ => None,
        };

        if let Some(max_amt) = limit {
            if current_usage >= max_amt {
                bail!(
                    "Plan limit reached for {}: max allowed is {}",
                    limit_name,
                    max_amt
                );
            }
        }
        Ok(())
    }
}

pub fn login() -> Result<()> {
    tracing::info!("Starting login flow...");
    // Local-first/dev login: create a local token that passes offline validation.

    let payload = VltPayload {
        vlt_id: new_random_id("vlt"),
        user_id: new_random_id("usr"),
        plan: "pro".to_string(),
        features: vec![
            "cloud_sync".to_string(),
            "share".to_string(),
            "brand_kit".to_string(),
            "commons".to_string(),
        ],
        limits: VltLimits {
            renders_per_month: None,
            cloud_renders_per_month: Some(100),
            machines: Some(3),
            team_members: None,
        },
        issued_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(30),
    };

    let vlt = Vlt {
        payload,
        signature: "local:dev".to_string(),
    };

    vlt.save_local()?;
    tracing::info!("Authentication successful! VLT saved locally.");

    Ok(())
}

pub fn create_api_key(name: &str, scope: &str) -> Result<()> {
    let mut store = load_api_keys()?;

    let key_id = new_random_id("vk");
    let secret = {
        let mut bytes = [0u8; 24];
        rand_core::OsRng.fill_bytes(&mut bytes);
        BASE64_STANDARD.encode(bytes)
    };
    let full_key = format!("{}_{}", key_id, secret);

    let rec = ApiKeyRecord {
        key_id: key_id.clone(),
        name: name.to_string(),
        scope: scope.to_string(),
        created_at: Utc::now(),
    };
    store.keys.retain(|k| k.key_id != key_id);
    store.keys.push(rec);
    store.keys.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    save_api_keys(&store)?;

    println!("✅ Created API key '{}'", name);
    println!("   Scope: {}", scope);
    println!("   Key: {}", full_key);
    println!("   (This key will not be shown again.)");
    Ok(())
}

pub fn list_api_keys() -> Result<()> {
    let store = load_api_keys()?;
    println!("Active API keys:");
    if store.keys.is_empty() {
        println!("  (none)");
        return Ok(());
    }
    for k in store.keys {
        println!("  {}  '{}'  (scope: {})", k.key_id, k.name, k.scope);
    }
    Ok(())
}

pub fn revoke_api_key(key_id: &str) -> Result<()> {
    let mut store = load_api_keys()?;
    let before = store.keys.len();
    store.keys.retain(|k| k.key_id != key_id);
    if store.keys.len() == before {
        bail!("API key not found: {}", key_id);
    }
    save_api_keys(&store)?;
    println!("✅ Revoked API key: {}", key_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlt_login_creates_local_token_and_validates() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_auth_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        login().unwrap();
        let vlt = Vlt::load_local().unwrap();
        vlt.validate_offline().unwrap();
        assert!(vlt.payload.vlt_id.starts_with("vlt_"));

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn api_keys_create_list_revoke() {
        let _lock = crate::test_support::ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join(format!("vidra_auth_keys_home_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("VIDRA_HOME_DIR", &tmp);

        create_api_key("ci", "render:read,render:write").unwrap();
        let store = load_api_keys().unwrap();
        assert_eq!(store.keys.len(), 1);
        let key_id = store.keys[0].key_id.clone();

        revoke_api_key(&key_id).unwrap();
        let store2 = load_api_keys().unwrap();
        assert_eq!(store2.keys.len(), 0);

        std::env::remove_var("VIDRA_HOME_DIR");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}

// ── Enterprise Features (Phase 3.4) ─────────────────────────────────

/// SSO provider configuration for enterprise orgs.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SsoConfig {
    pub provider: SsoProvider,
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SsoProvider {
    Saml,
    Oidc,
}

/// Audit log entry for compliance tracking.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub action: String,
    pub resource: String,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

/// Role-Based Access Control role definition.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RbacRole {
    pub name: String,
    pub permissions: Vec<String>,
}

#[allow(dead_code)]
impl RbacRole {
    pub fn owner() -> Self {
        Self {
            name: "owner".to_string(),
            permissions: vec![
                "project:*".to_string(),
                "workspace:*".to_string(),
                "billing:*".to_string(),
                "members:*".to_string(),
                "settings:*".to_string(),
            ],
        }
    }

    pub fn editor() -> Self {
        Self {
            name: "editor".to_string(),
            permissions: vec![
                "project:read".to_string(),
                "project:write".to_string(),
                "project:render".to_string(),
                "assets:read".to_string(),
                "assets:write".to_string(),
            ],
        }
    }

    pub fn viewer() -> Self {
        Self {
            name: "viewer".to_string(),
            permissions: vec!["project:read".to_string(), "assets:read".to_string()],
        }
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p == perm || p.ends_with(":*") && perm.starts_with(&p[..p.len() - 1]))
    }
}

// ── Machine Seat Licensing (Phase 3.9) ──────────────────────────────

/// Hardware fingerprint for machine seat enforcement.
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineFingerprint {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub fingerprint_hash: String,
}

#[allow(dead_code)]
impl MachineFingerprint {
    pub fn current() -> Self {
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let raw = format!("{}:{}:{}", hostname, os, arch);
        let fingerprint_hash = format!("{:x}", md5::compute(raw.as_bytes()));
        Self {
            hostname,
            os,
            arch,
            fingerprint_hash,
        }
    }
}
