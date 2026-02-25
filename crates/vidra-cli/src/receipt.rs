use anyhow::{Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use base64::Engine;
use rand_core::RngCore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareInfo {
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vram_gb: Option<u32>,
}

impl HardwareInfo {
    pub fn basic() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu: None,
            gpu: None,
            vram_gb: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderReceiptPayload {
    pub receipt_id: String,
    pub project_id: String,
    pub ir_hash: String,
    pub output_hash: String,
    pub output_format: String,
    pub render_duration_ms: u64,
    pub frame_count: u64,
    pub hardware: HardwareInfo,
    pub vlt_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderReceipt {
    #[serde(flatten)]
    pub payload: RenderReceiptPayload,
    /// Signature over canonical JSON of `payload`.
    /// Format: `ed25519:<base64>`
    pub signature: String,
}

impl RenderReceipt {
    pub fn new(
        project_id: String,
        ir_hash: String,
        output_hash: String,
        output_format: String,
        render_duration_ms: u64,
        frame_count: u64,
        hardware: HardwareInfo,
        vlt_id: String,
        signing_key: &SigningKey,
    ) -> Result<Self> {
        let timestamp = chrono::Utc::now();

        let receipt_id = {
            let mut hasher = Sha256::new();
            hasher.update(project_id.as_bytes());
            hasher.update(ir_hash.as_bytes());
            hasher.update(output_hash.as_bytes());
            hasher.update(timestamp.to_rfc3339().as_bytes());
            let digest = hasher.finalize();
            let hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
            format!("rr_{}", &hex[..8])
        };

        let payload = RenderReceiptPayload {
            receipt_id,
            project_id,
            ir_hash,
            output_hash,
            output_format,
            render_duration_ms,
            frame_count,
            hardware,
            vlt_id,
            timestamp,
        };

        let msg = serde_json::to_string(&payload)?;
        let signature_bytes = signing_key.sign(msg.as_bytes());

        let signature_b64 = base64::prelude::BASE64_STANDARD.encode(signature_bytes.to_bytes());
        let signature = format!("ed25519:{}", signature_b64);

        Ok(Self { payload, signature })
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<bool> {
        let sig_part = self
            .signature
            .strip_prefix("ed25519:")
            .context("Invalid signature format; must start with ed25519:")?;

        let sig_bytes = base64::prelude::BASE64_STANDARD
            .decode(sig_part)
            .context("Invalid signature base64")?;

        let signature = Signature::from_slice(&sig_bytes).context("Invalid signature length")?;

        let msg = serde_json::to_string(&self.payload)?;

        Ok(verifying_key.verify(msg.as_bytes(), &signature).is_ok())
    }

    pub fn save_to_dir(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let filename = format!("{}.json", self.payload.receipt_id);
        let path = dir.join(filename);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

pub fn receipts_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".vidra").join("receipts"))
}

/// Persistent device signing key used to sign receipts locally.
///
/// Stored at `~/.vidra/receipt_signing_key` as base64 of 32 raw bytes.
pub fn load_or_create_device_signing_key() -> Result<SigningKey> {
    let home = dirs::home_dir().context("failed to resolve home dir")?;
    let dir = home.join(".vidra");
    std::fs::create_dir_all(&dir).context("failed to create ~/.vidra")?;
    let path = dir.join("receipt_signing_key");

    if path.exists() {
        let raw = std::fs::read_to_string(&path).context("failed to read receipt_signing_key")?;
        let bytes = base64::prelude::BASE64_STANDARD
            .decode(raw.trim())
            .context("invalid base64 in receipt_signing_key")?;
        let key_bytes: [u8; 32] = bytes
            .as_slice()
            .try_into()
            .context("receipt_signing_key must decode to 32 bytes")?;
        return Ok(SigningKey::from_bytes(&key_bytes));
    }

    let mut seed = [0u8; 32];
    rand_core::OsRng.fill_bytes(&mut seed);
    let key = SigningKey::from_bytes(&seed);

    let b64 = base64::prelude::BASE64_STANDARD.encode(seed);
    std::fs::write(&path, b64).context("failed to write receipt_signing_key")?;
    Ok(key)
}

pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receipt_sign_and_verify_roundtrip() {
        let mut seed = [0u8; 32];
        seed[0] = 42;
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        let receipt = RenderReceipt::new(
            "proj_test".to_string(),
            "sha256:ir".to_string(),
            "sha256:out".to_string(),
            "mp4_1080p".to_string(),
            123,
            60,
            HardwareInfo::basic(),
            "vlt_test".to_string(),
            &signing_key,
        )
        .unwrap();

        assert!(receipt.verify(&verifying_key).unwrap());
    }

    #[test]
    fn receipt_save_to_dir_writes_file() {
        let mut seed = [0u8; 32];
        seed[0] = 7;
        let signing_key = SigningKey::from_bytes(&seed);

        let receipt = RenderReceipt::new(
            "proj_test".to_string(),
            "sha256:ir".to_string(),
            "sha256:out".to_string(),
            "mp4_1080p".to_string(),
            123,
            60,
            HardwareInfo::basic(),
            "vlt_test".to_string(),
            &signing_key,
        )
        .unwrap();

        let dir = std::env::temp_dir().join(format!(
            "vidra_receipt_test_{}_{}",
            std::process::id(),
            receipt.payload.receipt_id
        ));
        let _ = std::fs::remove_dir_all(&dir);
        receipt.save_to_dir(&dir).unwrap();

        let written = dir.join(format!("{}.json", receipt.payload.receipt_id));
        assert!(written.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
