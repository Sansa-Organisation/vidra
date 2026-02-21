use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::path::{PathBuf, Path};
use ed25519_dalek::{Verifier, VerifyingKey, Signature};
use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

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

impl Vlt {
    pub fn get_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".vidra").join("vlt.token")
    }

    pub fn load_local() -> Result<Self> {
        let path = Self::get_path();
        if !path.exists() {
            bail!("No VLT found at {:?}. Please run `vidra auth login`.", path);
        }
        let content = std::fs::read_to_string(&path)
            .context("Failed to read VLT token file")?;
        let vlt: Vlt = serde_json::from_str(&content)
            .context("Failed to parse VLT token JSON")?;
        Ok(vlt)
    }

    pub fn save_local(&self) -> Result<()> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn validate_offline(&self) -> Result<()> {
        // 1. Check signature
        // The signature is over the canonical JSON of the payload.
        // For simplicity, we assume the platform signs the canonical JSON string of VltPayload.
        let pubkey_bytes = BASE64_STANDARD.decode(VIDRA_PLATFORM_PUBKEY_B64)
            .context("Invalid embedded pubkey base64")?;
        let pubkey = VerifyingKey::from_bytes(
            pubkey_bytes.as_slice().try_into().context("Invalid pubkey length")?
        ).context("Failed to parse verifying key")?;

        let sig_part = self.signature.strip_prefix("ed25519:")
            .context("Invalid signature format; must start with ed25519:")?;
        
        let sig_bytes = BASE64_STANDARD.decode(sig_part)
            .context("Invalid signature base64")?;
        
        let signature = Signature::from_slice(&sig_bytes)
            .context("Invalid signature length")?;

        // Serialization must be deterministic. We use canonical-like JSON.
        // Note: For a robust system, we would store the raw payload bytes and parse them.
        let msg = serde_json::to_string(&self.payload)
            .context("Failed to serialize VLT payload")?;

        pubkey.verify(msg.as_bytes(), &signature)
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

    pub fn has_feature(&self, feature: &str) -> bool {
        self.payload.features.iter().any(|f| f == feature)
    }

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
                bail!("Plan limit reached for {}: max allowed is {}", limit_name, max_amt);
            }
        }
        Ok(())
    }
}

pub fn login() -> Result<()> {
    tracing::info!("Starting browser-based login flow...");
    tracing::info!("Opening browser to https://vidra.dev/auth/login...");
    
    // In a real implementation we would:
    // 1. Spawns browser
    // 2. Opens local server on random port
    // 3. Waits for callback with the token payload
    // 
    // For now, we simulate receiving a valid token by signing a dummy one.
    
    use ed25519_dalek::SigningKey;

    // Generate a temporary signing key to mock a token
    let mock_bytes = [7u8; 32];
    let signing_key = SigningKey::from_bytes(&mock_bytes);

    let payload = VltPayload {
        vlt_id: "vlt_mock_12345".to_string(),
        user_id: "usr_mock_001".to_string(),
        plan: "pro".to_string(),
        features: vec!["cloud_sync".to_string(), "share".to_string(), "brand_kit".to_string(), "commons_premium".to_string()],
        limits: VltLimits {
            renders_per_month: None,
            cloud_renders_per_month: Some(100),
            machines: Some(3),
            team_members: None,
        },
        issued_at: Utc::now(),
        expires_at: Utc::now() + Duration::days(30),
    };

    let msg = serde_json::to_string(&payload)?;
    use ed25519_dalek::Signer;
    let signature_bytes = signing_key.sign(msg.as_bytes());
    
    let signature_b64 = BASE64_STANDARD.encode(signature_bytes.to_bytes());
    let sig_string = format!("ed25519:{}", signature_b64);

    let vlt = Vlt {
        payload,
        signature: sig_string,
    };

    vlt.save_local()?;
    tracing::info!("Authentication successful! VLT saved locally.");
    
    Ok(())
}

pub fn create_api_key(name: &str, scope: &str) -> Result<()> {
    // Generate an API key prefix
    let key = format!("vk_live_mock_{}...", BASE64_STANDARD.encode(name).to_lowercase());
    println!("Successfully created API key '{}' with scopes: [{}]", name, scope);
    println!("Key: {}", key);
    Ok(())
}

pub fn list_api_keys() -> Result<()> {
    println!("Active API keys:");
    println!("  vk_live_mock_default_... (Scope: all)");
    Ok(())
}

pub fn revoke_api_key(key_id: &str) -> Result<()> {
    println!("Successfully revoked API key: {}", key_id);
    Ok(())
}

// ── Enterprise Features (Phase 3.4) ─────────────────────────────────

/// SSO provider configuration for enterprise orgs.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SsoConfig {
    pub provider: SsoProvider,
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SsoProvider {
    Saml,
    Oidc,
}

/// Audit log entry for compliance tracking.
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RbacRole {
    pub name: String,
    pub permissions: Vec<String>,
}

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
            permissions: vec![
                "project:read".to_string(),
                "assets:read".to_string(),
            ],
        }
    }

    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| {
            p == perm || p.ends_with(":*") && perm.starts_with(&p[..p.len()-1])
        })
    }
}

// ── Machine Seat Licensing (Phase 3.9) ──────────────────────────────

/// Hardware fingerprint for machine seat enforcement.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineFingerprint {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub fingerprint_hash: String,
}

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
