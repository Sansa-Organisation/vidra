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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAiProviderConfig {
    /// API base URL (supports OpenAI-compatible providers).
    /// Examples:
    /// - https://api.openai.com
    /// - https://api.groq.com/openai
    #[serde(default)]
    pub base_url: String,
    /// Environment variable that contains the API key.
    /// Default: OPENAI_API_KEY
    #[serde(default)]
    pub api_key_env: String,
    /// Text-to-speech model name.
    #[serde(default)]
    pub tts_model: String,
    /// Audio format requested from the provider (mp3|wav).
    #[serde(default)]
    pub tts_format: String,
    /// Audio transcription model (Whisper-compatible).
    #[serde(default)]
    pub transcribe_model: String,
}

impl Default for OpenAiProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.openai.com".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
            tts_model: "gpt-4o-mini-tts".to_string(),
            tts_format: "mp3".to_string(),
            transcribe_model: "whisper-1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElevenLabsProviderConfig {
    #[serde(default)]
    pub base_url: String,
    /// Environment variable that contains the API key.
    /// Default: ELEVENLABS_API_KEY
    #[serde(default)]
    pub api_key_env: String,
}

impl Default for ElevenLabsProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.elevenlabs.io".to_string(),
            api_key_env: "ELEVENLABS_API_KEY".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoveBgProviderConfig {
    #[serde(default)]
    pub base_url: String,
    /// Environment variable that contains the API key.
    /// Default: REMOVEBG_API_KEY
    #[serde(default)]
    pub api_key_env: String,
}

impl Default for RemoveBgProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.remove.bg".to_string(),
            api_key_env: "REMOVEBG_API_KEY".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeminiProviderConfig {
    #[serde(default)]
    pub base_url: String,
    /// Environment variable that contains the API key.
    /// Default: GEMINI_API_KEY
    #[serde(default)]
    pub api_key_env: String,
    /// Gemini model name (without the `models/` prefix).
    #[serde(default)]
    pub model: String,
    /// Optional: post-process caption segments (punctuation/casing) after transcription.
    ///
    /// When enabled, Vidra will call Gemini during `vidra render` and cache the refined
    /// output under the AI cache directory.
    #[serde(default)]
    pub caption_refine: bool,
}

impl Default for GeminiProviderConfig {
    fn default() -> Self {
        Self {
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            api_key_env: "GEMINI_API_KEY".to_string(),
            model: "gemini-1.5-flash".to_string(),
            caption_refine: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiConfig {
    /// Master enable switch for AI materialization (TTS, captions, background removal).
    pub enabled: bool,
    /// Optional override for where AI outputs are cached.
    /// If unset, Vidra uses `resources.cache_dir`.
    pub cache_dir: Option<String>,
    #[serde(default)]
    pub openai: OpenAiProviderConfig,
    #[serde(default)]
    pub elevenlabs: ElevenLabsProviderConfig,
    #[serde(default)]
    pub removebg: RemoveBgProviderConfig,
    #[serde(default)]
    pub gemini: GeminiProviderConfig,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_dir: None,
            openai: OpenAiProviderConfig::default(),
            elevenlabs: ElevenLabsProviderConfig::default(),
            removebg: RemoveBgProviderConfig::default(),
            gemini: GeminiProviderConfig::default(),
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
    #[serde(default)]
    pub ai: AiConfig,
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
