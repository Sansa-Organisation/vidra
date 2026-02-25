use crate::gpu::GpuContext;
use std::sync::Arc;
use vidra_core::frame::{FrameBuffer, PixelFormat};
use vidra_core::types::LayerEffect;

use std::collections::HashMap;
use std::sync::Mutex;

pub struct GpuEffects {
    gpu: Arc<GpuContext>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    lut_cache: Mutex<HashMap<String, Lut3D>>,
}

#[derive(Clone)]
struct Lut3D {
    size: usize,
    // RGB triples in [0,1], length = size^3, order: b changes fastest, then g, then r.
    data: Vec<[f32; 3]>,
}

impl Lut3D {
    fn sample(&self, r: f32, g: f32, b: f32) -> [f32; 3] {
        let n = self.size as f32;
        let r = r.clamp(0.0, 1.0) * (n - 1.0);
        let g = g.clamp(0.0, 1.0) * (n - 1.0);
        let b = b.clamp(0.0, 1.0) * (n - 1.0);

        let r0 = r.floor() as usize;
        let g0 = g.floor() as usize;
        let b0 = b.floor() as usize;
        let r1 = (r0 + 1).min(self.size - 1);
        let g1 = (g0 + 1).min(self.size - 1);
        let b1 = (b0 + 1).min(self.size - 1);

        let tr = r - r0 as f32;
        let tg = g - g0 as f32;
        let tb = b - b0 as f32;

        let c000 = self.at(r0, g0, b0);
        let c100 = self.at(r1, g0, b0);
        let c010 = self.at(r0, g1, b0);
        let c110 = self.at(r1, g1, b0);
        let c001 = self.at(r0, g0, b1);
        let c101 = self.at(r1, g0, b1);
        let c011 = self.at(r0, g1, b1);
        let c111 = self.at(r1, g1, b1);

        let c00 = lerp3(c000, c100, tr);
        let c10 = lerp3(c010, c110, tr);
        let c01 = lerp3(c001, c101, tr);
        let c11 = lerp3(c011, c111, tr);

        let c0 = lerp3(c00, c10, tg);
        let c1 = lerp3(c01, c11, tg);
        lerp3(c0, c1, tb)
    }

    fn at(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        let idx = (r * self.size * self.size) + (g * self.size) + b;
        self.data.get(idx).copied().unwrap_or([0.0, 0.0, 0.0])
    }
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct EffectParams {
    effect_type: u32,
    intensity: f32,
    radius: f32,
    pad: f32,
}

impl GpuEffects {
    pub fn new(gpu: Arc<GpuContext>) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("effects.wgsl"));

        let bind_group_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("effects_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: wgpu::TextureFormat::Rgba8Unorm,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("effects_pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gpu
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("effects_compute_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            });

        Self {
            gpu,
            pipeline,
            bind_group_layout,
            lut_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn apply(&self, src: &FrameBuffer, effect: &LayerEffect) -> Option<FrameBuffer> {
        if src.format != PixelFormat::Rgba8 {
            return None;
        }

        if let LayerEffect::Lut { path, intensity } = effect {
            let intensity = (*intensity as f32).clamp(0.0, 1.0);
            if intensity <= 0.0 {
                return Some(src.clone());
            }

            let lut = {
                let mut cache = self.lut_cache.lock().ok()?;
                if !cache.contains_key(path) {
                    let parsed = parse_cube_lut(path).ok()?;
                    cache.insert(path.clone(), parsed);
                }
                cache.get(path).cloned()?
            };

            let mut out = src.clone();
            for px in out.data.chunks_exact_mut(4) {
                let r = px[0] as f32 / 255.0;
                let g = px[1] as f32 / 255.0;
                let b = px[2] as f32 / 255.0;
                let graded = lut.sample(r, g, b);

                let rr = r + (graded[0] - r) * intensity;
                let gg = g + (graded[1] - g) * intensity;
                let bb = b + (graded[2] - b) * intensity;

                px[0] = (rr.clamp(0.0, 1.0) * 255.0).round() as u8;
                px[1] = (gg.clamp(0.0, 1.0) * 255.0).round() as u8;
                px[2] = (bb.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            return Some(out);
        }

        let width = src.width;
        let height = src.height;

        // Ensure we don't attempt zero size
        if width == 0 || height == 0 {
            return Some(src.clone());
        }

        let format = wgpu::TextureFormat::Rgba8Unorm;
        let usage_in = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let texture_in = self.gpu.texture_pool.acquire(
            &self.gpu.device,
            Some("effect_texture_in"),
            width,
            height,
            format,
            usage_in,
        );

        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture_in,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &src.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_out_buf = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("effect_texture_out_buf"),
            size: (width * height * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let usage_out = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC;
        let texture_out = self.gpu.texture_pool.acquire(
            &self.gpu.device,
            Some("effect_texture_out"),
            width,
            height,
            format,
            usage_out,
        );

        let view_in = texture_in.create_view(&wgpu::TextureViewDescriptor::default());
        let view_out = texture_out.create_view(&wgpu::TextureViewDescriptor::default());

        use wgpu::util::DeviceExt;
        let mut params = EffectParams {
            effect_type: 0,
            intensity: 0.0,
            radius: 0.0,
            pad: 0.0,
        };

        let mut custom_pipeline = None;
        match effect {
            LayerEffect::Blur(radius) => {
                params.effect_type = 1;
                params.radius = *radius as f32;
            }
            LayerEffect::Grayscale(intensity) => {
                params.effect_type = 2;
                params.intensity = *intensity as f32;
            }
            LayerEffect::Invert(intensity) => {
                params.effect_type = 3;
                params.intensity = *intensity as f32;
            }
            LayerEffect::CustomShader { wgsl_source } => {
                params.effect_type = 4; // Or ignored because we use custom pipeline
                let module = self
                    .gpu
                    .device
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("custom_effect_shader"),
                        source: wgpu::ShaderSource::Wgsl(wgsl_source.as_str().into()),
                    });

                let pipeline_layout =
                    self.gpu
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("custom_effects_pipeline_layout"),
                            bind_group_layouts: &[&self.bind_group_layout],
                            push_constant_ranges: &[],
                        });

                custom_pipeline = Some(self.gpu.device.create_compute_pipeline(
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("custom_effect_compute_pipeline"),
                        layout: Some(&pipeline_layout),
                        module: &module,
                        entry_point: "main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                ));
            }
            LayerEffect::Brightness(amount) => {
                params.effect_type = 5;
                params.intensity = *amount as f32;
            }
            LayerEffect::Contrast(amount) => {
                params.effect_type = 6;
                params.intensity = *amount as f32;
            }
            LayerEffect::Saturation(amount) => {
                params.effect_type = 7;
                params.intensity = *amount as f32;
            }
            LayerEffect::HueRotate(degrees) => {
                params.effect_type = 8;
                params.intensity = *degrees as f32;
            }
            LayerEffect::Vignette(amount) => {
                params.effect_type = 9;
                params.intensity = *amount as f32;
            }
            LayerEffect::RemoveBackground => {
                // Background removal is expected to be materialized into an alpha image
                // in the CLI (or by web/mobile runtimes) before render.
                params.effect_type = 0;
            }
            LayerEffect::Lut { .. } => {
                // Handled in the CPU early-return path above.
                params.effect_type = 0;
            }
        }

        let params_buffer = self
            .gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("effect_params_buffer"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group = self
            .gpu
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("effect_bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view_in),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view_out),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(custom_pipeline.as_ref().unwrap_or(&self.pipeline));
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((width + 15) / 16, (height + 15) / 16, 1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &texture_out,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &texture_out_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.gpu.queue.submit(Some(encoder.finish()));

        let slice = texture_out_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        self.gpu.device.poll(wgpu::Maintain::Wait);

        let result = if rx.recv().unwrap().is_ok() {
            let data = slice.get_mapped_range().to_vec();
            Some(FrameBuffer {
                width,
                height,
                format: PixelFormat::Rgba8,
                data,
            })
        } else {
            None
        };

        texture_out_buf.unmap();

        // Release textures back to pool!
        self.gpu
            .texture_pool
            .release(texture_in, width, height, format, usage_in);
        self.gpu
            .texture_pool
            .release(texture_out, width, height, format, usage_out);

        result
    }
}

fn parse_cube_lut(path: &str) -> Result<Lut3D, std::io::Error> {
    let raw = std::fs::read_to_string(path)?;
    let mut size: Option<usize> = None;
    let mut data: Vec<[f32; 3]> = Vec::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("LUT_3D_SIZE") {
            let s = rest.trim().parse::<usize>().ok();
            size = s;
            continue;
        }
        if line.starts_with("DOMAIN_") {
            // Ignore for now.
            continue;
        }
        // Data row: 3 floats
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
            ) {
                data.push([r, g, b]);
            }
        }
    }

    let Some(size) = size else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing LUT_3D_SIZE",
        ));
    };

    let expected = size * size * size;
    if data.len() != expected {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "invalid LUT length: got {}, expected {}",
                data.len(),
                expected
            ),
        ));
    }

    Ok(Lut3D { size, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cube_lut_2x2x2() {
        // Minimal valid 2^3 LUT where output == input.
        // Order: b fastest, then g, then r.
        let cube = r#"
LUT_3D_SIZE 2
0.0 0.0 0.0
0.0 0.0 1.0
0.0 1.0 0.0
0.0 1.0 1.0
1.0 0.0 0.0
1.0 0.0 1.0
1.0 1.0 0.0
1.0 1.0 1.0
"#;

        let dir = std::env::temp_dir();
        let path = dir.join("vidra_test_identity_2.cube");
        std::fs::write(&path, cube).unwrap();

        let lut = parse_cube_lut(path.to_string_lossy().as_ref()).unwrap();
        assert_eq!(lut.size, 2);
        assert_eq!(lut.data.len(), 8);

        let s = lut.sample(0.25, 0.5, 0.75);
        assert!((s[0] - 0.25).abs() < 0.2);
        assert!((s[1] - 0.5).abs() < 0.2);
        assert!((s[2] - 0.75).abs() < 0.2);
    }
}
