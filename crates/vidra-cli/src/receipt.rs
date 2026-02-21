use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderReceiptPayload {
    pub vlt_id: String,
    pub ir_hash: String,
    pub output_hash: Option<String>,
    pub hardware_info: String,
    pub render_duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RenderReceipt {
    #[serde(flatten)]
    pub payload: RenderReceiptPayload,
    pub signature: String, // base64 encoded signature
}

impl RenderReceipt {
    pub fn new(
        vlt_id: String,
        ir_hash: String,
        output_hash: Option<String>,
        hardware_info: String,
        render_duration_ms: u64,
        signing_key: &SigningKey,
    ) -> Result<Self> {
        let payload = RenderReceiptPayload {
            vlt_id,
            ir_hash,
            output_hash,
            hardware_info,
            render_duration_ms,
            timestamp: chrono::Utc::now(),
        };

        let msg = serde_json::to_string(&payload)?;
        let signature_bytes = signing_key.sign(msg.as_bytes());
        
        use base64::Engine;
        let signature = base64::prelude::BASE64_STANDARD.encode(signature_bytes.to_bytes());

        Ok(Self { payload, signature })
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<bool> {
        use base64::Engine;
        let sig_bytes = base64::prelude::BASE64_STANDARD.decode(&self.signature)
            .context("Invalid signature base64")?;
        
        let signature = Signature::from_slice(&sig_bytes)
            .context("Invalid signature length")?;

        let msg = serde_json::to_string(&self.payload)?;

        Ok(verifying_key.verify(msg.as_bytes(), &signature).is_ok())
    }

    pub fn save_to_dir(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let filename = format!("receipt_{}.json", self.payload.timestamp.timestamp());
        let path = dir.join(filename);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
