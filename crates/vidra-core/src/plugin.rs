use std::collections::HashMap;
use std::path::Path;
use std::fmt;

use crate::frame::FrameBuffer;
use crate::types::LayerEffect;
use crate::VidraError;

// ──────────────────────────────────────────────────────────────────────────────
// Core Plugin Trait
// ──────────────────────────────────────────────────────────────────────────────

/// Metadata describing a Vidra plugin.
#[derive(Debug, Clone)]
pub struct PluginManifest {
    /// Unique plugin identifier (e.g. "vidra-plugin-particles")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Semantic version
    pub version: String,
    /// Author
    pub author: String,
    /// Short description
    pub description: String,
}

impl fmt::Display for PluginManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{} by {}", self.name, self.version, self.author)
    }
}

/// Base trait that all Vidra plugins must implement.
pub trait VidraPlugin: Send + Sync {
    /// Returns the plugin's manifest with metadata.
    fn manifest(&self) -> PluginManifest;

    /// Called when the plugin is loaded. Perform initialization here.
    fn on_load(&mut self) -> Result<(), VidraError> {
        Ok(())
    }

    /// Called when the plugin is unloaded. Clean up resources here.
    fn on_unload(&mut self) -> Result<(), VidraError> {
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Effect Plugin
// ──────────────────────────────────────────────────────────────────────────────

/// Parameters passed to an effect plugin during processing.
#[derive(Debug, Clone)]
pub struct EffectContext {
    /// Width of the frame.
    pub width: u32,
    /// Height of the frame.
    pub height: u32,
    /// Current time in seconds.
    pub time: f64,
    /// FPS of the project.
    pub fps: f64,
    /// Named parameters from VidraScript (e.g. `effect(myPlugin, intensity: 0.5)`)
    pub params: HashMap<String, f64>,
}

/// A plugin that provides custom visual effects.
pub trait EffectPlugin: VidraPlugin {
    /// The effect name as used in VidraScript (e.g. `effect("myBlur")`).
    fn effect_name(&self) -> &str;

    /// Apply the effect to a frame buffer in-place.
    fn apply(&self, frame: &mut FrameBuffer, ctx: &EffectContext) -> Result<(), VidraError>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Layer Plugin
// ──────────────────────────────────────────────────────────────────────────────

/// Context provided to layer plugins for rendering.
#[derive(Debug, Clone)]
pub struct LayerContext {
    /// Width of the canvas.
    pub width: u32,
    /// Height of the canvas.
    pub height: u32,
    /// Current frame number.
    pub frame: u32,
    /// Current time in seconds.
    pub time: f64,
    /// FPS of the project.
    pub fps: f64,
    /// Named parameters from VidraScript.
    pub params: HashMap<String, String>,
}

/// A plugin that provides a custom layer type.
pub trait LayerPlugin: VidraPlugin {
    /// The layer type name as used in VidraScript.
    fn layer_type(&self) -> &str;

    /// Render the custom layer content, returning a frame buffer.
    fn render(&self, ctx: &LayerContext) -> Result<FrameBuffer, VidraError>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Transition Plugin
// ──────────────────────────────────────────────────────────────────────────────

/// Context for scene transitions.
#[derive(Debug, Clone)]
pub struct TransitionContext {
    /// Width of the frame.
    pub width: u32,
    /// Height of the frame.
    pub height: u32,
    /// Transition progress (0.0 = start, 1.0 = end).
    pub progress: f64,
    /// Named parameters.
    pub params: HashMap<String, f64>,
}

/// A plugin that provides a custom scene transition.
pub trait TransitionPlugin: VidraPlugin {
    /// The transition name as used in VidraScript (e.g. `transition(myWipe)`).
    fn transition_name(&self) -> &str;

    /// Apply the transition to two frames (outgoing and incoming), returning the blended result.
    fn apply(
        &self,
        outgoing: &FrameBuffer,
        incoming: &FrameBuffer,
        ctx: &TransitionContext,
    ) -> Result<FrameBuffer, VidraError>;
}

// ──────────────────────────────────────────────────────────────────────────────
// Plugin Registry
// ──────────────────────────────────────────────────────────────────────────────

/// Central registry that manages loaded plugins.
pub struct PluginRegistry {
    effects: HashMap<String, Box<dyn EffectPlugin>>,
    layers: HashMap<String, Box<dyn LayerPlugin>>,
    transitions: HashMap<String, Box<dyn TransitionPlugin>>,
}

impl PluginRegistry {
    /// Create a new empty plugin registry.
    pub fn new() -> Self {
        Self {
            effects: HashMap::new(),
            layers: HashMap::new(),
            transitions: HashMap::new(),
        }
    }

    /// Register an effect plugin.
    pub fn register_effect(&mut self, mut plugin: Box<dyn EffectPlugin>) -> Result<(), VidraError> {
        let name = plugin.effect_name().to_string();
        plugin.on_load()?;
        // tracing not available in core crate
        self.effects.insert(name, plugin);
        Ok(())
    }

    /// Register a layer plugin.
    pub fn register_layer(&mut self, mut plugin: Box<dyn LayerPlugin>) -> Result<(), VidraError> {
        let name = plugin.layer_type().to_string();
        plugin.on_load()?;
        // tracing not available in core crate
        self.layers.insert(name, plugin);
        Ok(())
    }

    /// Register a transition plugin.
    pub fn register_transition(&mut self, mut plugin: Box<dyn TransitionPlugin>) -> Result<(), VidraError> {
        let name = plugin.transition_name().to_string();
        plugin.on_load()?;
        // tracing not available in core crate
        self.transitions.insert(name, plugin);
        Ok(())
    }

    /// Look up a registered effect plugin by name.
    pub fn get_effect(&self, name: &str) -> Option<&dyn EffectPlugin> {
        self.effects.get(name).map(|p| p.as_ref())
    }

    /// Look up a registered layer plugin by name.
    pub fn get_layer(&self, name: &str) -> Option<&dyn LayerPlugin> {
        self.layers.get(name).map(|p| p.as_ref())
    }

    /// Look up a registered transition plugin by name.
    pub fn get_transition(&self, name: &str) -> Option<&dyn TransitionPlugin> {
        self.transitions.get(name).map(|p| p.as_ref())
    }

    /// List all registered plugin manifests.
    pub fn list(&self) -> Vec<PluginManifest> {
        let mut result = Vec::new();
        for p in self.effects.values() { result.push(p.manifest()); }
        for p in self.layers.values() { result.push(p.manifest()); }
        for p in self.transitions.values() { result.push(p.manifest()); }
        result
    }

    /// Load a plugin from a dynamic library (.dylib/.so/.dll) at the given path.
    ///
    /// The library must export a `vidra_plugin_create` function with the signature:
    /// `extern "C" fn() -> *mut dyn VidraPlugin`
    ///
    /// For safety, this is stubbed in Phase 3. Full dylib loading will come later.
    pub fn load_from_path(&mut self, _path: &Path) -> Result<(), VidraError> {
        // Dynamic loading not yet implemented
        Err(VidraError::Encode(
            "Dynamic plugin loading not yet available. Register plugins programmatically.".into(),
        ))
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test effect plugin
    struct InvertEffect;

    impl VidraPlugin for InvertEffect {
        fn manifest(&self) -> PluginManifest {
            PluginManifest {
                id: "test-invert".to_string(),
                name: "Test Invert".to_string(),
                version: "0.1.0".to_string(),
                author: "Vidra Tests".to_string(),
                description: "A test invert effect".to_string(),
            }
        }
    }

    impl EffectPlugin for InvertEffect {
        fn effect_name(&self) -> &str { "testInvert" }

        fn apply(&self, frame: &mut FrameBuffer, _ctx: &EffectContext) -> Result<(), VidraError> {
            for y in 0..frame.height {
                for x in 0..frame.width {
                    let [r, g, b, a] = frame.get_pixel(x, y).unwrap_or([0, 0, 0, 0]);
                    frame.set_pixel(x, y, [255 - r, 255 - g, 255 - b, a]);
                }
            }
            Ok(())
        }
    }

    // A test layer plugin
    struct GradientLayer;

    impl VidraPlugin for GradientLayer {
        fn manifest(&self) -> PluginManifest {
            PluginManifest {
                id: "test-gradient".to_string(),
                name: "Test Gradient".to_string(),
                version: "0.1.0".to_string(),
                author: "Vidra Tests".to_string(),
                description: "A gradient layer".to_string(),
            }
        }
    }

    impl LayerPlugin for GradientLayer {
        fn layer_type(&self) -> &str { "gradient" }

        fn render(&self, ctx: &LayerContext) -> Result<FrameBuffer, VidraError> {
            let mut fb = FrameBuffer::new(ctx.width, ctx.height, crate::PixelFormat::Rgba8);
            for y in 0..ctx.height {
                let t = y as f64 / ctx.height as f64;
                let g = (t * 255.0) as u8;
                for x in 0..ctx.width {
                    fb.set_pixel(x, y, [g, 0, 255 - g, 255]);
                }
            }
            Ok(fb)
        }
    }

    #[test]
    fn test_register_effect() {
        let mut registry = PluginRegistry::new();
        registry.register_effect(Box::new(InvertEffect)).unwrap();
        
        assert!(registry.get_effect("testInvert").is_some());
        assert!(registry.get_effect("nonExistent").is_none());
    }

    #[test]
    fn test_apply_effect_plugin() {
        let plugin = InvertEffect;
        let mut frame = FrameBuffer::new(2, 2, crate::PixelFormat::Rgba8);
        frame.set_pixel(0, 0, [100, 150, 200, 255]);
        
        let ctx = EffectContext {
            width: 2, height: 2, time: 0.0, fps: 30.0,
            params: HashMap::new(),
        };
        
        plugin.apply(&mut frame, &ctx).unwrap();
        assert_eq!(frame.get_pixel(0, 0).unwrap(), [155, 105, 55, 255]);
    }

    #[test]
    fn test_register_layer() {
        let mut registry = PluginRegistry::new();
        registry.register_layer(Box::new(GradientLayer)).unwrap();
        
        assert!(registry.get_layer("gradient").is_some());
    }

    #[test]
    fn test_render_layer_plugin() {
        let plugin = GradientLayer;
        let ctx = LayerContext {
            width: 4, height: 4, frame: 0, time: 0.0, fps: 30.0,
            params: HashMap::new(),
        };
        let frame = plugin.render(&ctx).unwrap();
        assert_eq!(frame.width, 4);
        assert_eq!(frame.height, 4);
        let pixel = frame.get_pixel(0, 3).unwrap();
        // Bottom row (y=3, height=4), t=0.75, g should be around 191
        assert!(pixel[0] > 0 || pixel[1] > 0, "gradient should produce non-zero pixels");
    }

    #[test]
    fn test_registry_list() {
        let mut registry = PluginRegistry::new();
        registry.register_effect(Box::new(InvertEffect)).unwrap();
        registry.register_layer(Box::new(GradientLayer)).unwrap();
        
        let manifests = registry.list();
        assert_eq!(manifests.len(), 2);
    }
}
