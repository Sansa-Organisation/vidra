@group(0) @binding(0) var t_in: texture_2d<f32>;
@group(0) @binding(1) var t_out: texture_storage_2d<rgba8unorm, write>;

struct EffectParams {
    effect_type: u32, // 0 = none, 1 = blur, 2 = grayscale, 3 = invert
    intensity: f32,
    radius: f32,
    pad: f32,
};

@group(0) @binding(2) var<uniform> params: EffectParams;

// Helper: RGB to YIQ
fn rgb2yiq(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(c, vec3<f32>(0.299, 0.587, 0.114)),
        dot(c, vec3<f32>(0.596, -0.274, -0.322)),
        dot(c, vec3<f32>(0.211, -0.523, 0.312))
    );
}

// Helper: YIQ to RGB
fn yiq2rgb(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(c, vec3<f32>(1.0, 0.956, 0.621)),
        dot(c, vec3<f32>(1.0, -0.272, -0.647)),
        dot(c, vec3<f32>(1.0, -1.106, 1.703))
    );
}

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
    } else if (params.effect_type == 5u) {
        // Brightness
        color = vec4<f32>(color.rgb * params.intensity, color.a);
    } else if (params.effect_type == 6u) {
        // Contrast
        color = vec4<f32>((color.rgb - 0.5) * params.intensity + 0.5, color.a);
    } else if (params.effect_type == 7u) {
        // Saturation
        let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, params.intensity), color.a);
    } else if (params.effect_type == 8u) {
        // Hue Rotate (using YIQ color space)
        let hue_rad = params.intensity * 3.14159265 / 180.0;
        let c = cos(hue_rad);
        let s = sin(hue_rad);
        var yiq = rgb2yiq(color.rgb);
        let rot_yiq = vec3<f32>(
            yiq.r,
            yiq.g * c - yiq.b * s,
            yiq.g * s + yiq.b * c
        );
        color = vec4<f32>(yiq2rgb(rot_yiq), color.a);
    } else if (params.effect_type == 9u) {
        // Vignette
        let uv = vec2<f32>(coords) / vec2<f32>(size);
        let coord = (uv - 0.5) * 2.0; // -1.0 to 1.0
        let dist = length(coord);
        // Intensity 0..1 controls how far the vignette reaches
        let v = smoothstep(1.5 - params.intensity, 0.5 - params.intensity * 0.5, dist);
        color = vec4<f32>(color.rgb * v, color.a);
    }

    textureStore(t_out, coords, color);
}
