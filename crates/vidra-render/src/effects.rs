use crate::gpu::GpuContext;
use std::sync::Arc;
use vidra_core::frame::{FrameBuffer, PixelFormat};
use vidra_core::types::LayerEffect;

pub struct GpuEffects {
    gpu: Arc<GpuContext>,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
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
        let shader = gpu.device.create_shader_module(wgpu::include_wgsl!("effects.wgsl"));

        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("effects_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
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
        }
    }

    pub fn apply(&self, src: &FrameBuffer, effect: &LayerEffect) -> Option<FrameBuffer> {
        if src.format != PixelFormat::Rgba8 {
            return None;
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
            width, height, format, usage_in
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
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
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
            width, height, format, usage_out
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
        }

        let params_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("effect_params_buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(&self.pipeline);
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
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
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

        // Important: Unmap to safely clean up buffer!
        drop(slice);
        texture_out_buf.unmap();

        // Release textures back to pool!
        self.gpu.texture_pool.release(texture_in, width, height, format, usage_in);
        self.gpu.texture_pool.release(texture_out, width, height, format, usage_out);

        result
    }
}
