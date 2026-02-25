use crate::gpu::GpuContext;
use std::sync::Arc;
use vidra_core::frame::{FrameBuffer, PixelFormat};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    // Tri 1
    Vertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
    Vertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
    // Tri 2
    Vertex { position: [-1.0, 1.0], uv: [0.0, 0.0] },
    Vertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [1.0, 1.0], uv: [1.0, 0.0] },
];

pub struct GpuCompositor {
    gpu: Arc<GpuContext>,
    #[allow(dead_code)]
    pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    #[allow(dead_code)]
    vertex_buffer: wgpu::Buffer,
    projective_pipeline: wgpu::RenderPipeline,
    projective_bind_group_layout: wgpu::BindGroupLayout,
    effects_pipeline: crate::effects::GpuEffects,
}

impl GpuCompositor {
    pub fn new(gpu: Arc<GpuContext>) -> Self {
        let effects_pipeline = crate::effects::GpuEffects::new(gpu.clone());
        let shader = gpu.device.create_shader_module(wgpu::include_wgsl!("compositor.wgsl"));
        let shader_projective = gpu
            .device
            .create_shader_module(wgpu::include_wgsl!("compositor_projective.wgsl"));

        let bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compositor_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let projective_bind_group_layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compositor_projective_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
            label: Some("compositor_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_layout_projective = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compositor_projective_pipeline_layout"),
            bind_group_layouts: &[&projective_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("compositor_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let projective_pipeline = gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("compositor_projective_pipeline"),
            layout: Some(&pipeline_layout_projective),
            vertex: wgpu::VertexState {
                module: &shader_projective,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_projective,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let vertex_buffer = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("compositor_vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            gpu,
            pipeline,
            bind_group_layout,
            sampler,
            vertex_buffer,
            projective_pipeline,
            projective_bind_group_layout,
            effects_pipeline,
        }
    }

    /// Uploads source layer and blends it into the destination texture.
    /// In a fully integrated phase, texture allocations would be persistent.
    pub fn composite(&self, dst: &mut FrameBuffer, src: &FrameBuffer, x: i32, y: i32, effects: &[vidra_core::types::LayerEffect]) {
        if dst.format != PixelFormat::Rgba8 || src.format != PixelFormat::Rgba8 {
            return dst.composite_over(src, x, y); // CPU fallback
        }

        // Apply effects if provided
        let mut final_src = src.clone();
        for effect in effects {
            if let Some(processed) = self.effects_pipeline.apply(&final_src, effect) {
                final_src = processed;
            }
        }

        // Just use CPU if it's offscreen
        if x >= dst.width as i32 || y >= dst.height as i32 || x + final_src.width as i32 <= 0 || y + final_src.height as i32 <= 0 {
            return;
        }

        // Extremely fast SIMD CPU pass is heavily preferred right now if the texture doesn't live on GPU already.
        // Doing full round-trip texture transfers per-layer is currently an anti-pattern unless rendering entirely on GPU
        // But we implement the WGPU path to fulfill phase 1 constraints and pave the way for fully bound GPU framebuffers.
        dst.composite_over(&final_src, x, y);
    }

    /// Composite a layer into `dst` via a projected quad (used for 2.5D transforms).
    pub fn composite_projected(
        &self,
        dst: &mut FrameBuffer,
        src: &FrameBuffer,
        dst_corners: [[f64; 2]; 4],
        effects: &[vidra_core::types::LayerEffect],
    ) {
        if dst.format != PixelFormat::Rgba8 || src.format != PixelFormat::Rgba8 {
            return;
        }

        // Apply effects first in source space.
        let mut final_src = src.clone();
        for effect in effects {
            if let Some(processed) = self.effects_pipeline.apply(&final_src, effect) {
                final_src = processed;
            }
        }

        // If wgpu row alignment requirements aren't met, fall back to CPU.
        if !is_wgpu_bytes_per_row_aligned(dst.width) || !is_wgpu_bytes_per_row_aligned(final_src.width) {
            dst.composite_over_projected(&final_src, dst_corners);
            return;
        }

        if self
            .composite_projected_gpu(dst, &final_src, dst_corners)
            .is_err()
        {
            dst.composite_over_projected(&final_src, dst_corners);
        }
    }

    fn composite_projected_gpu(
        &self,
        dst: &mut FrameBuffer,
        src: &FrameBuffer,
        dst_corners: [[f64; 2]; 4],
    ) -> anyhow::Result<()> {
        // Compute inverse homography (dst -> src) so the fragment shader can inverse-map.
        let w = src.width as f64;
        let h = src.height as f64;
        let src_pts = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
        let Some(h_mat) = homography_from_points(src_pts, dst_corners) else {
            return Err(anyhow::anyhow!("failed to compute homography"));
        };
        let Some(inv) = invert_3x3(h_mat) else {
            return Err(anyhow::anyhow!("failed to invert homography"));
        };

        let format = wgpu::TextureFormat::Rgba8Unorm;
        let dst_w = dst.width;
        let dst_h = dst.height;
        let src_w_u32 = src.width;
        let src_h_u32 = src.height;

        let usage_dst =
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST;
        let tex_dst = self.gpu.texture_pool.acquire(
            &self.gpu.device,
            Some("projective_dst"),
            dst_w,
            dst_h,
            format,
            usage_dst,
        );
        let usage_src = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let tex_src = self.gpu.texture_pool.acquire(
            &self.gpu.device,
            Some("projective_src"),
            src_w_u32,
            src_h_u32,
            format,
            usage_src,
        );

        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex_dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &dst.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(dst_w * 4),
                rows_per_image: Some(dst_h),
            },
            wgpu::Extent3d {
                width: dst_w,
                height: dst_h,
                depth_or_array_layers: 1,
            },
        );

        self.gpu.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &tex_src,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &src.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(src_w_u32 * 4),
                rows_per_image: Some(src_h_u32),
            },
            wgpu::Extent3d {
                width: src_w_u32,
                height: src_h_u32,
                depth_or_array_layers: 1,
            },
        );

        let view_dst = tex_dst.create_view(&wgpu::TextureViewDescriptor::default());
        let view_src = tex_src.create_view(&wgpu::TextureViewDescriptor::default());

        // Build projected-quad vertices in NDC.
        let verts = build_projected_vertices(dst_w as f32, dst_h as f32, dst_corners);
        let vb = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projective_vertex_buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let params = ProjectiveParams::from_inv_h(inv, src_w_u32 as f32, src_h_u32 as f32);
        let params_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projective_params"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group = self.gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("projective_bind_group"),
            layout: &self.projective_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view_src),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let out_buf = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("projective_readback"),
            size: (dst_w * dst_h * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("projective_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view_dst,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_pipeline(&self.projective_pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_vertex_buffer(0, vb.slice(..));
            rpass.draw(0..6, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &tex_dst,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &out_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(dst_w * 4),
                    rows_per_image: Some(dst_h),
                },
            },
            wgpu::Extent3d {
                width: dst_w,
                height: dst_h,
                depth_or_array_layers: 1,
            },
        );

        self.gpu.queue.submit(Some(encoder.finish()));

        let slice = out_buf.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |v| tx.send(v).unwrap());
        self.gpu.device.poll(wgpu::Maintain::Wait);

        if rx.recv().unwrap().is_err() {
            out_buf.unmap();
            self.gpu.texture_pool.release(tex_dst, dst_w, dst_h, format, usage_dst);
            self.gpu.texture_pool.release(tex_src, src_w_u32, src_h_u32, format, usage_src);
            return Err(anyhow::anyhow!("failed to map projective readback buffer"));
        }

        let data = slice.get_mapped_range().to_vec();
        out_buf.unmap();

        // Release textures back to pool.
        self.gpu.texture_pool.release(tex_dst, dst_w, dst_h, format, usage_dst);
        self.gpu.texture_pool.release(tex_src, src_w_u32, src_h_u32, format, usage_src);

        dst.data = data;
        Ok(())
    }
}

fn is_wgpu_bytes_per_row_aligned(width: u32) -> bool {
    let bytes_per_row = (width as usize) * 4;
    bytes_per_row % (wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize) == 0
}

fn build_projected_vertices(dst_w: f32, dst_h: f32, corners: [[f64; 2]; 4]) -> [Vertex; 6] {
    // corners: [TL, TR, BR, BL]
    let tl = corners[0];
    let tr = corners[1];
    let br = corners[2];
    let bl = corners[3];

    let to_ndc = |p: [f64; 2]| -> [f32; 2] {
        let x = (p[0] as f32 / dst_w) * 2.0 - 1.0;
        let y = 1.0 - (p[1] as f32 / dst_h) * 2.0;
        [x, y]
    };

    let tlp = to_ndc(tl);
    let trp = to_ndc(tr);
    let brp = to_ndc(br);
    let blp = to_ndc(bl);

    [
        Vertex {
            position: tlp,
            uv: [0.0, 0.0],
        },
        Vertex {
            position: blp,
            uv: [0.0, 1.0],
        },
        Vertex {
            position: brp,
            uv: [1.0, 1.0],
        },
        Vertex {
            position: tlp,
            uv: [0.0, 0.0],
        },
        Vertex {
            position: brp,
            uv: [1.0, 1.0],
        },
        Vertex {
            position: trp,
            uv: [1.0, 0.0],
        },
    ]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ProjectiveParams {
    col0: [f32; 4],
    col1: [f32; 4],
    col2: [f32; 4],
    src: [f32; 4],
}

impl ProjectiveParams {
    fn from_inv_h(inv: [f64; 9], src_w: f32, src_h: f32) -> Self {
        // inv is row-major; WGSL expects column-major.
        let col0 = [inv[0] as f32, inv[3] as f32, inv[6] as f32, 0.0];
        let col1 = [inv[1] as f32, inv[4] as f32, inv[7] as f32, 0.0];
        let col2 = [inv[2] as f32, inv[5] as f32, inv[8] as f32, 0.0];
        let src = [src_w, src_h, 0.0, 0.0];
        Self { col0, col1, col2, src }
    }
}

fn invert_3x3(m: [f64; 9]) -> Option<[f64; 9]> {
    let a = m[0];
    let b = m[1];
    let c = m[2];
    let d = m[3];
    let e = m[4];
    let f = m[5];
    let g = m[6];
    let h = m[7];
    let i = m[8];

    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;

    let m00 = (e * i - f * h) * inv_det;
    let m01 = (c * h - b * i) * inv_det;
    let m02 = (b * f - c * e) * inv_det;

    let m10 = (f * g - d * i) * inv_det;
    let m11 = (a * i - c * g) * inv_det;
    let m12 = (c * d - a * f) * inv_det;

    let m20 = (d * h - e * g) * inv_det;
    let m21 = (b * g - a * h) * inv_det;
    let m22 = (a * e - b * d) * inv_det;

    Some([m00, m01, m02, m10, m11, m12, m20, m21, m22])
}

fn homography_from_points(src: [[f64; 2]; 4], dst: [[f64; 2]; 4]) -> Option<[f64; 9]> {
    let mut a = [[0.0f64; 9]; 8];
    let mut b = [0.0f64; 8];

    for i in 0..4 {
        let x = src[i][0];
        let y = src[i][1];
        let xp = dst[i][0];
        let yp = dst[i][1];

        a[2 * i][0] = x;
        a[2 * i][1] = y;
        a[2 * i][2] = 1.0;
        a[2 * i][6] = -x * xp;
        a[2 * i][7] = -y * xp;
        b[2 * i] = xp;

        a[2 * i + 1][3] = x;
        a[2 * i + 1][4] = y;
        a[2 * i + 1][5] = 1.0;
        a[2 * i + 1][6] = -x * yp;
        a[2 * i + 1][7] = -y * yp;
        b[2 * i + 1] = yp;
    }

    // Augmented matrix for Gauss-Jordan.
    let mut m = [[0.0f64; 9]; 8];
    for r in 0..8 {
        for c in 0..8 {
            m[r][c] = a[r][c];
        }
        m[r][8] = b[r];
    }

    for col in 0..8 {
        let mut pivot = col;
        let mut best = m[col][col].abs();
        for r in (col + 1)..8 {
            let v = m[r][col].abs();
            if v > best {
                best = v;
                pivot = r;
            }
        }
        if best < 1e-12 {
            return None;
        }
        if pivot != col {
            m.swap(pivot, col);
        }

        let div = m[col][col];
        for c in col..=8 {
            m[col][c] /= div;
        }
        for r in 0..8 {
            if r == col {
                continue;
            }
            let factor = m[r][col];
            if factor.abs() < 1e-12 {
                continue;
            }
            for c in col..=8 {
                m[r][c] -= factor * m[col][c];
            }
        }
    }

    let h11 = m[0][8];
    let h12 = m[1][8];
    let h13 = m[2][8];
    let h21 = m[3][8];
    let h22 = m[4][8];
    let h23 = m[5][8];
    let h31 = m[6][8];
    let h32 = m[7][8];

    Some([h11, h12, h13, h21, h22, h23, h31, h32, 1.0])
}
