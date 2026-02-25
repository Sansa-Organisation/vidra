use vidra_core::Duration;
use vidra_ir::animation::{AnimatableProperty, Animation, Keyframe};

pub fn compile_spring(
    property: AnimatableProperty,
    from: f64,
    to: f64,
    stiffness: f64,
    damping: f64,
    initial_velocity: f64,
) -> Animation {
    let mut anim = Animation::new(property);
    
    // Euler integration parameters
    let mass = 1.0;
    let dt = 1.0 / 60.0;
    
    let mut position = from;
    let mut velocity = initial_velocity;
    let mut t = 0.0;
    
    anim.add_keyframe(Keyframe::new(Duration::from_seconds(0.0), position));

    let epsilon = 0.001;
    let mut stable_frames = 0;
    let max_duration = 10.0; // Max 10 seconds timeout

    while t < max_duration && stable_frames < 5 {
        t += dt;
        
        // F = -kX - cv
        let displacement = position - to;
        let spring_force = -stiffness * displacement;
        let damping_force = -damping * velocity;
        
        let acceleration = (spring_force + damping_force) / mass;
        velocity += acceleration * dt;
        position += velocity * dt;
        
        anim.add_keyframe(Keyframe::new(Duration::from_seconds(t), position));

        if displacement.abs() < epsilon && velocity.abs() < epsilon {
            stable_frames += 1;
        } else {
            stable_frames = 0;
        }
    }

    if anim.keyframes.len() < 2 {
        anim.add_keyframe(Keyframe::new(Duration::from_seconds(dt), to));
    }
    
    // Hard set the final value to the exact `to` value to avoid drift
    if let Some(last) = anim.keyframes.last_mut() {
        last.value = to;
    }

    anim
}

use evalexpr::*;
pub fn compile_expression(
    property: AnimatableProperty,
    expr_str: &str,
    duration: f64,
    audio_amp_samples: Option<&[f64]>,
) -> Animation {
    let mut anim = Animation::new(property);
    
    let dt = 1.0 / 60.0;
    let mut t = 0.0;
    
    let compiled: Node<DefaultNumericTypes> = match build_operator_tree(expr_str) {
        Ok(c) => c,
        Err(_) => {
            // Fallback to constant
            anim.add_keyframe(Keyframe::new(Duration::from_seconds(0.0), 0.0));
            anim.add_keyframe(Keyframe::new(Duration::from_seconds(duration), 0.0));
            return anim;
        }
    };

    let mut context = HashMapContext::new();

    while t <= duration + 0.0001 {
        let _ = context.set_value("t".to_string(), evalexpr::Value::Float(t));
        // optionally provide p progression from 0 to 1
        let p = if duration > 0.0 { t / duration } else { 1.0 };
        let _ = context.set_value("p".to_string(), evalexpr::Value::Float(p));
        let _ = context.set_value("T".to_string(), evalexpr::Value::Float(duration));

        if let Some(samples) = audio_amp_samples {
            let idx = ((t / dt).round() as usize).min(samples.len().saturating_sub(1));
            let amp = samples.get(idx).copied().unwrap_or(0.0);
            let _ = context.set_value("audio_amp".to_string(), evalexpr::Value::Float(amp));
        } else {
            let _ = context.set_value("audio_amp".to_string(), evalexpr::Value::Float(0.0));
        }

        let value = match compiled.eval_with_context(&context) {
            Ok(v) => v.as_number().unwrap_or(0.0),
            _ => 0.0,
        };

        anim.add_keyframe(Keyframe::new(Duration::from_seconds(t), value));
        t += dt;
    }

    anim
}

pub fn compile_path_animations(
    _path_data: &str,
    _duration: f64,
) -> (Animation, Animation) {
    // Stubbed until needed
    let mut anim_x = Animation::new(AnimatableProperty::PositionX);
    let mut anim_y = Animation::new(AnimatableProperty::PositionY);
    anim_x.add_keyframe(Keyframe::new(Duration::from_seconds(0.0), 0.0));
    anim_y.add_keyframe(Keyframe::new(Duration::from_seconds(0.0), 0.0));
    (anim_x, anim_y)
}

