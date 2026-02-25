use anyhow::Result;
use std::collections::HashMap;
use std::sync::Mutex;
use wgpu::{
    Adapter, Device, Extent3d, Instance, Queue, Texture, TextureDescriptor, TextureFormat,
    TextureUsages,
};

#[derive(Hash, Eq, PartialEq, Clone)]
struct TextureDescKey {
    width: u32,
    height: u32,
    format: TextureFormat,
    usage: TextureUsages,
}

pub struct TexturePool {
    free_textures: Mutex<HashMap<TextureDescKey, Vec<Texture>>>,
}

impl TexturePool {
    pub fn new() -> Self {
        Self {
            free_textures: Mutex::new(HashMap::new()),
        }
    }

    pub fn acquire(
        &self,
        device: &Device,
        label: Option<&str>,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> Texture {
        let key = TextureDescKey {
            width,
            height,
            format,
            usage,
        };
        let mut pool = self.free_textures.lock().unwrap();

        if let Some(textures) = pool.get_mut(&key) {
            if let Some(texture) = textures.pop() {
                // If a texture is available, reuse it
                return texture;
            }
        }

        // Creating a new texture if pool is empty
        device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    }

    pub fn release(
        &self,
        texture: Texture,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsages,
    ) {
        let key = TextureDescKey {
            width,
            height,
            format,
            usage,
        };
        let mut pool = self.free_textures.lock().unwrap();
        pool.entry(key).or_insert_with(Vec::new).push(texture);
    }
}

/// A shared context for all GPU-accelerated operations.
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub texture_pool: TexturePool,
}

impl GpuContext {
    /// Initializes WGPU, selecting the best available backend (Metal, Vulkan, DX12, etc.)
    pub fn init() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Find the best adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None, // Headless rendering
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| anyhow::anyhow!("Failed to find suitable wgpu adapter"))?;

        // Request device
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Vidra Headless GPU Device"),
                required_features: wgpu::Features::empty(), // Add features if needed for advanced blending
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            texture_pool: TexturePool::new(),
        })
    }
}
