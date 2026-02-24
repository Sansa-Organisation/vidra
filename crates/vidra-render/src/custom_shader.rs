use std::sync::Arc;
use wgpu::util::DeviceExt;
use vidra_core::frame::FrameBuffer;
use crate::gpu::GpuContext;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShaderUniforms {
    resolution: [f32; 2],
    time: f32,
    _pad: f32,
}

pub struct CustomShaderRenderer {
    gpu: Arc<GpuContext>,
}

impl CustomShaderRenderer {
    pub fn new(gpu: Arc<GpuContext>) -> Self {
        Self { gpu }
    }

    pub fn render(
        &self,
        shader_source: &str,
        width: u32,
        height: u32,
        time_sec: f32,
    ) -> Result<FrameBuffer, vidra_core::VidraError> {
        // Inject uniforms into the provided source.
        // We will assume the user provides a compute shader with entry point `main`.
        // We inject the `out_tex` binding and `uniforms` struct.
        let injected_source = format!(
            r#"
struct Uniforms {{
    resolution: vec2<f32>,
    time: f32,
}};

@group(0) @binding(0) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

// User provided:
{}
"#,
            shader_source
        );

        let module = self.gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("custom_shader_injected"),
            source: wgpu::ShaderSource::Wgsl(injected_source.into()),
        });

        let bind_group_layout = self.gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom_shader_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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

        let pipeline_layout = self.gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("custom_shader_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = self.gpu.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("custom_shader_pipeline"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        // 2) Allocate output texture
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("custom_shader_out"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture_out = self.gpu.device.create_texture(&texture_desc);
        let view_out = texture_out.create_view(&wgpu::TextureViewDescriptor::default());

        // 3) Setup Uniforms Data
        let uniforms = ShaderUniforms {
            resolution: [width as f32, height as f32],
            time: time_sec,
            _pad: 0.0,
        };

        let params_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("custom_shader_params_buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("custom_shader_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_out),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        // 4) Execute Compute Pass
        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups((width + 15) / 16, (height + 15) / 16, 1);
        }

        // 5) Readback into CPU buffer
        let padded_bytes_per_row = (width * 4 + 255) & !255;
        let texture_out_buf = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("custom_shader_readback"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

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
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );

        let submission_idx = self.gpu.queue.submit(Some(encoder.finish()));
        
        // Wait and map
        let slice = texture_out_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());

        self.gpu.device.poll(wgpu::Maintain::Wait);

        if rx.recv().unwrap().is_ok() {
            let data = slice.get_mapped_range();
            let mut result_fb = FrameBuffer::new(width, height, vidra_core::PixelFormat::Rgba8);
            
            for y in 0..height {
                let row_start = (y * padded_bytes_per_row) as usize;
                let src_row = &data[row_start..row_start + (width * 4) as usize];
                for x in 0..width {
                    let off_src = (x * 4) as usize;
                    let r = src_row[off_src];
                    let g = src_row[off_src + 1];
                    let b = src_row[off_src + 2];
                    let a = src_row[off_src + 3];
                    result_fb.set_pixel(x, y, [r, g, b, a]);
                }
            }
            drop(data);
            let _ = slice;
            texture_out_buf.unmap();
            
            Ok(result_fb)
        } else {
            Err(vidra_core::VidraError::Render("Failed to map shader output buffer".into()))
        }
    }
}
