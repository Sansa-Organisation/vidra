use crate::ast::*;
use vidra_core::VidraError;

pub struct Codegen {
    wgsl: String,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            wgsl: String::new(),
        }
    }

    pub fn generate(&mut self, ast: &EffectDef) -> Result<String, VidraError> {
        self.wgsl.push_str("
// VidraFX Generated Shader
@group(0) @binding(0) var t_in: texture_2d<f32>;
@group(0) @binding(1) var t_out: texture_storage_2d<rgba8unorm, write>;

struct Params {
    effect_type: u32,
    time: f32,
");
        for (_i, param) in ast.params.iter().enumerate() {
            self.wgsl.push_str(&format!("    p_{}: f32,\n", param.name));
        }

        self.wgsl.push_str("};
@group(0) @binding(2) var<uniform> params: Params;

// Built-in functions
fn mod289(x: vec2<f32>) -> vec2<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn mod289_3(x: vec3<f32>) -> vec3<f32> { return x - floor(x * (1.0 / 289.0)) * 289.0; }
fn permute3(x: vec3<f32>) -> vec3<f32> { return mod289_3(((x*34.0)+1.0)*x); }
fn snoise2(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
    var i  = floor(v + dot(v, C.yy));
    var x0 = v -   i + dot(i, C.xx);
    var i1 = select(vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), x0.x > x0.y);
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;
    i = mod289(i);
    var p = permute3(permute3(i.y + vec3<f32>(0.0, i1.y, 1.0)) + i.x + vec3<f32>(0.0, i1.x, 1.0));
    var m = max(0.5 - vec3<f32>(dot(x0,x0), dot(x12.xy,x12.xy), dot(x12.zw,x12.zw)), vec3<f32>(0.0));
    m = m*m; m = m*m;
    var x = 2.0 * fract(p * C.www) - 1.0;
    var h = abs(x) - 0.5;
    var ox = floor(x + 0.5);
    var a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0*a0 + h*h);
    var g = vec3<f32>(0.0);
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.y = a0.y * x12.x + h.y * x12.y;
    g.z = a0.z * x12.z + h.z * x12.w;
    return 130.0 * dot(m, g);
}
fn fbm(uv: vec2<f32>) -> f32 {
    var value = 0.0;
    var amplitude = 0.5;
    var frequency = 0.0;
    var st = uv;
    for (var i = 0; i < 5; i++) {
        value += amplitude * snoise2(st);
        st *= 2.0;
        amplitude *= 0.5;
    }
    return value * 0.5 + 0.5;
}

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let size = textureDimensions(t_in);
    let coords = vec2<i32>(global_id.xy);
    if (coords.x >= i32(size.x) || coords.y >= i32(size.y)) { return; }
    
    let uv = vec2<f32>(coords) / vec2<f32>(size);
");

        // Body
        let mut last_expr = String::new();
        for stmt in &ast.body {
            match stmt {
                Statement::Let { name, value, .. } => {
                    let val_str = self.gen_expr(value)?;
                    self.wgsl.push_str(&format!("    let {} = {};\n", name, val_str));
                }
                Statement::Expr(expr) => {
                    last_expr = self.gen_expr(expr)?;
                }
            }
        }
        
        self.wgsl.push_str(&format!("    let final_color = {};\n", last_expr));
        self.wgsl.push_str("    textureStore(t_out, coords, final_color);\n");
        self.wgsl.push_str("}\n");

        Ok(self.wgsl.clone())
    }

    fn gen_expr(&mut self, expr: &Expr) -> Result<String, VidraError> {
        match expr {
            Expr::Call { name, args, .. } => self.gen_call(name, args, None),
            Expr::Pipe { left, right, .. } => {
                let left_str = self.gen_expr(left)?;
                match &**right {
                    Expr::Call { name, args, .. } => self.gen_call(name, args, Some(left_str)),
                    _ => Err(VidraError::parse("Right side of pipe must be a function call", "", 0, 0)),
                }
            }
            Expr::Ident(name, _) => Ok(name.clone()),
            Expr::Number(val, _) => {
                let s = format!("{}", val);
                if s.contains('.') { Ok(s) } else { Ok(format!("{}.0", s)) }
            },
            Expr::ColorHex(hex, _) => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                let a = if hex.len() == 8 { u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) } else { 255 };
                Ok(format!("vec4<f32>({}, {}, {}, {})", r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0))
            }
            Expr::StringLit(s, _) => Ok(s.clone()), // Usually for enum-like params, not well-supported in WGSL without mapping to numbers
            Expr::BinOp { op, left, right, .. } => {
                let left_str = self.gen_expr(left)?;
                let right_str = self.gen_expr(right)?;
                let op_str = match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                };
                Ok(format!("({} {} {})", left_str, op_str, right_str))
            }
        }
    }

    fn gen_call(&mut self, name: &str, args: &[Arg], piped_arg: Option<String>) -> Result<String, VidraError> {
        let mut arg_strs = Vec::new();
        if let Some(pipe_val) = piped_arg {
            arg_strs.push(pipe_val);
        }
        for arg in args {
            arg_strs.push(self.gen_expr(&arg.value)?);
        }

        match name {
            "source" => Ok("textureLoad(t_in, coords, 0)".to_string()),
            "noise" => {
                let scale = arg_strs.get(0).unwrap_or(&"1.0".to_string()).clone();
                let speed = arg_strs.get(1).unwrap_or(&"1.0".to_string()).clone();
                Ok(format!("vec4<f32>(vec3<f32>(fbm(uv * {} + vec2<f32>(0.0, params.time * {}))), 1.0)", scale, speed))
            }
            "mask" => {
                let input = arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                let min = arg_strs.get(1).unwrap_or(&"0.0".to_string()).clone();
                let max = arg_strs.get(2).unwrap_or(&"1.0".to_string()).clone();
                Ok(format!("smoothstep(vec4<f32>({}), vec4<f32>({}), {})", min, max, input))
            }
            "brightness" => {
                let input = arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                let factor = arg_strs.get(1).unwrap_or(&"1.0".to_string()).clone();
                Ok(format!("({} * vec4<f32>({}, {}, {}, 1.0))", input, factor, factor, factor))
            }
            "grayscale" => {
                let input = arg_strs
                    .get(0)
                    .unwrap_or(&"vec4<f32>(0.0)".to_string())
                    .clone();
                let intensity = arg_strs.get(1).unwrap_or(&"1.0".to_string()).clone();
                Ok(format!(
                    "vec4<f32>(mix(({}).rgb, vec3<f32>(dot(({}).rgb, vec3<f32>(0.299, 0.587, 0.114))), {}), ({}).a)",
                    input, input, intensity, input
                ))
            }
            "invert" => {
                let input = arg_strs
                    .get(0)
                    .unwrap_or(&"vec4<f32>(0.0)".to_string())
                    .clone();
                let intensity = arg_strs.get(1).unwrap_or(&"1.0".to_string()).clone();
                Ok(format!(
                    "vec4<f32>(mix(({}).rgb, (vec3<f32>(1.0) - ({}).rgb), {}), ({}).a)",
                    input, input, intensity, input
                ))
            }
            "tint" => {
                let input = arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                let color = arg_strs.get(1).unwrap_or(&"vec4<f32>(1.0)".to_string()).clone();
                Ok(format!("({} * {})", input, color))
            }
            "blend" => {
                let a = arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                let b = arg_strs.get(1).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                let factor = arg_strs.get(2).unwrap_or(&"0.5".to_string()).clone();
                Ok(format!("mix({}, {}, {})", a, b, factor))
            }
            "blur" => {
                let input = arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0)".to_string()).clone();
                // Pass-through operation for WGSL compute shader chaining
                Ok(input)
            }
            "color" => {
                Ok(arg_strs.get(0).unwrap_or(&"vec4<f32>(0.0, 0.0, 0.0, 1.0)".to_string()).clone())
            }
            _ => {
                // Return generic call if not built-in
                Ok(format!("{}({})", name, arg_strs.join(", ")))
            }
        }
    }
}
