struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct ProjectiveParams {
    // Column-major representation of a 3x3 inverse homography matrix.
    col0: vec4<f32>,
    col1: vec4<f32>,
    col2: vec4<f32>,
    // src.xy = (width, height)
    src: vec4<f32>,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var<uniform> params: ProjectiveParams;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let inv_h = mat3x3<f32>(
        vec3<f32>(params.col0.xyz),
        vec3<f32>(params.col1.xyz),
        vec3<f32>(params.col2.xyz),
    );

    // Fragment position is in pixel coordinates.
    let p = in.position.xy + vec2<f32>(0.5, 0.5);
    let s = inv_h * vec3<f32>(p, 1.0);
    if (abs(s.z) < 1e-6) {
        return vec4<f32>(0.0);
    }

    let sx = s.x / s.z;
    let sy = s.y / s.z;
    let src_w = max(params.src.x, 1.0);
    let src_h = max(params.src.y, 1.0);

    let uv = vec2<f32>(sx / src_w, sy / src_h);
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0);
    }

    return textureSample(t_diffuse, s_diffuse, uv);
}
