use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for an asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub String);

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of an asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Image,
    Video,
    Audio,
    Font,
    Shader,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Image => write!(f, "image"),
            AssetType::Video => write!(f, "video"),
            AssetType::Audio => write!(f, "audio"),
            AssetType::Font => write!(f, "font"),
            AssetType::Shader => write!(f, "shader"),
        }
    }
}

/// A registered asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique asset identifier.
    pub id: AssetId,
    /// Type of the asset.
    pub asset_type: AssetType,
    /// Path to the asset file (relative to project root).
    pub path: PathBuf,
    /// Optional human-readable name.
    pub name: Option<String>,
}

impl Asset {
    pub fn new(id: AssetId, asset_type: AssetType, path: impl Into<PathBuf>) -> Self {
        Self {
            id,
            asset_type,
            path: path.into(),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Registry of all assets in a project.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetRegistry {
    assets: HashMap<AssetId, Asset>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    /// Register an asset. Returns the AssetId.
    pub fn register(&mut self, asset: Asset) -> AssetId {
        let id = asset.id.clone();
        self.assets.insert(id.clone(), asset);
        id
    }

    /// Get an asset by ID.
    pub fn get(&self, id: &AssetId) -> Option<&Asset> {
        self.assets.get(id)
    }

    /// Remove an asset by ID.
    pub fn remove(&mut self, id: &AssetId) -> Option<Asset> {
        self.assets.remove(id)
    }

    /// List all assets.
    pub fn all(&self) -> impl Iterator<Item = &Asset> {
        self.assets.values()
    }

    /// Number of registered assets.
    pub fn count(&self) -> usize {
        self.assets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_registry() {
        let mut registry = AssetRegistry::new();
        let asset = Asset::new(
            AssetId::new("hero-image"),
            AssetType::Image,
            "./assets/hero.jpg",
        );
        let id = registry.register(asset);
        assert_eq!(registry.count(), 1);
        assert!(registry.get(&id).is_some());
        assert_eq!(registry.get(&id).unwrap().asset_type, AssetType::Image);
    }

    #[test]
    fn test_asset_registry_remove() {
        let mut registry = AssetRegistry::new();
        let id = registry.register(Asset::new(
            AssetId::new("font1"),
            AssetType::Font,
            "/fonts/Inter.ttf",
        ));
        assert_eq!(registry.count(), 1);
        registry.remove(&id);
        assert_eq!(registry.count(), 0);
    }
}
