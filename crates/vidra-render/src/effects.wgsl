@group(0) @binding(0) var t_in: texture_2d<f32>;
@group(0) @binding(1) var t_out: texture_storage_2d<rgba8unorm, write>;

struct EffectParams {
    effect_type: u32, // 0 = none, 1 = blur, 2 = grayscale, 3 = invert
    intensity: f32,
    radius: f32,
    pad: f32,
};

@group(0) @binding(2) var<uniform> params: EffectParams;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(t_in);
    let coords = vec2<i32>(global_id.xy);

    if (coords.x >= i32(size.x) || coords.y >= i32(size.y)) {
        return;
    }

    var color: vec4<f32> = textureLoad(t_in, coords, 0);

    if (params.effect_type == 2u) {
        // Grayscale
        let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        let blended = mix(color.rgb, vec3<f32>(gray), params.intensity);
        color = vec4<f32>(blended, color.a);
    } else if (params.effect_type == 3u) {
        // Invert (keeping alpha intact)
        let inverted = vec3<f32>(1.0) - color.rgb;
        let blended = mix(color.rgb, inverted, params.intensity);
        color = vec4<f32>(blended, color.a);
    } else if (params.effect_type == 1u) {
        // Blur (very basic box blur for now)
        var sum = vec4<f32>(0.0);
        let r = i32(params.radius);
        var count = 0.0;
        
        for (var y = -r; y <= r; y = y + 1) {
            for (var x = -r; x <= r; x = x + 1) {
                let sample_coords = coords + vec2<i32>(x, y);
                // Clamp to edge
                let clamped_coords = vec2<i32>(
                    max(0, min(sample_coords.x, i32(size.x) - 1)),
                    max(0, min(sample_coords.y, i32(size.y) - 1))
                );
                sum = sum + textureLoad(t_in, clamped_coords, 0);
                count = count + 1.0;
            }
        }
        color = sum / count;
    }

    textureStore(t_out, coords, color);
}
