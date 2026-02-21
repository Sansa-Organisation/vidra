use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ProjectConfig {
    pub name: String,
    pub resolution: String,
    pub fps: u32,
    pub default_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct BrandConfig {
    pub kit: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub auto_sync: bool,
    pub sync_source: bool,
    pub sync_assets: String, // "none" | "on-demand" | "all"
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_sync: true,
            sync_source: false,
            sync_assets: "on-demand".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderConfig {
    pub target: String, // "local" | "cloud"
    pub cloud_fallback: bool,
    pub targets: Vec<String>,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            target: "local".to_string(),
            cloud_fallback: false,
            targets: vec!["default".to_string()],
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TelemetryConfig {
    pub level: String, // "anonymous" | "identified" | "diagnostics" | "off"
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            level: "identified".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourcesConfig {
    pub registries: Vec<String>,
    pub cache_dir: String,
    pub cache_max_gb: u32,
}

impl Default for ResourcesConfig {
    fn default() -> Self {
        Self {
            registries: vec!["vidra-commons".to_string()],
            cache_dir: "~/.vidra/cache".to_string(),
            cache_max_gb: 10,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub vlt_path: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            vlt_path: "~/.vidra/vlt.token".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct VidraConfig {
    pub project: ProjectConfig,
    #[serde(default)]
    pub brand: BrandConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default)]
    pub render: RenderConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub resources: ResourcesConfig,
    #[serde(default)]
    pub auth: AuthConfig,
}

impl VidraConfig {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let config: VidraConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
