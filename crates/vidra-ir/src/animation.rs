use serde::{Deserialize, Serialize};
use vidra_core::types::Easing;

/// Identifies the property being animated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimatableProperty {
    PositionX,
    PositionY,
    ScaleX,
    ScaleY,
    Rotation,
    Opacity,
}

impl std::fmt::Display for AnimatableProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimatableProperty::PositionX => write!(f, "position.x"),
            AnimatableProperty::PositionY => write!(f, "position.y"),
            AnimatableProperty::ScaleX => write!(f, "scale.x"),
            AnimatableProperty::ScaleY => write!(f, "scale.y"),
            AnimatableProperty::Rotation => write!(f, "rotation"),
            AnimatableProperty::Opacity => write!(f, "opacity"),
        }
    }
}

/// A keyframe: a value at a specific time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time offset from the start of the animation.
    pub time: vidra_core::Duration,
    /// Target value at this keyframe.
    pub value: f64,
    /// Easing function to use when interpolating TO this keyframe.
    pub easing: Easing,
}

impl Keyframe {
    pub fn new(time: vidra_core::Duration, value: f64) -> Self {
        Self {
            time,
            value,
            easing: Easing::Linear,
        }
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

/// An animation definition: a property + keyframes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// The property being animated.
    pub property: AnimatableProperty,
    /// Ordered keyframes (must be sorted by time).
    pub keyframes: Vec<Keyframe>,
    /// Delay before the animation starts (relative to scene start).
    pub delay: vidra_core::Duration,
}

impl Animation {
    /// Create a new animation for the given property.
    pub fn new(property: AnimatableProperty) -> Self {
        Self {
            property,
            keyframes: Vec::new(),
            delay: vidra_core::Duration::zero(),
        }
    }

    /// Create a simple "from → to" animation.
    pub fn from_to(
        property: AnimatableProperty,
        from: f64,
        to: f64,
        duration: vidra_core::Duration,
        easing: Easing,
    ) -> Self {
        Self {
            property,
            keyframes: vec![
                Keyframe::new(vidra_core::Duration::zero(), from),
                Keyframe::new(duration, to).with_easing(easing),
            ],
            delay: vidra_core::Duration::zero(),
        }
    }

    /// Set the delay.
    pub fn with_delay(mut self, delay: vidra_core::Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Add a keyframe. Keyframes are kept sorted by time.
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        self.keyframes.push(keyframe);
        self.keyframes.sort_by(|a, b| {
            a.time
                .as_seconds()
                .partial_cmp(&b.time.as_seconds())
                .unwrap()
        });
    }

    /// Evaluate the animation at a given time (relative to the animation start, after delay).
    /// Returns None if time is before the animation starts.
    pub fn evaluate(&self, time: vidra_core::Duration) -> Option<f64> {
        if self.keyframes.is_empty() {
            return None;
        }

        // Subtract delay
        let effective_secs = time.as_seconds() - self.delay.as_seconds();
        if effective_secs < 0.0 {
            return None;
        }

        // Before first keyframe
        if effective_secs <= self.keyframes[0].time.as_seconds() {
            return Some(self.keyframes[0].value);
        }

        // After last keyframe
        let last = &self.keyframes[self.keyframes.len() - 1];
        if effective_secs >= last.time.as_seconds() {
            return Some(last.value);
        }

        // Find the two surrounding keyframes
        for i in 0..self.keyframes.len() - 1 {
            let kf_a = &self.keyframes[i];
            let kf_b = &self.keyframes[i + 1];
            let t_a = kf_a.time.as_seconds();
            let t_b = kf_b.time.as_seconds();

            if effective_secs >= t_a && effective_secs <= t_b {
                let segment_duration = t_b - t_a;
                if segment_duration == 0.0 {
                    return Some(kf_b.value);
                }
                let local_t = (effective_secs - t_a) / segment_duration;
                let eased_t = kf_b.easing.apply(local_t);
                return Some(kf_a.value + (kf_b.value - kf_a.value) * eased_t);
            }
        }

        Some(last.value)
    }

    /// Get the total duration of the animation (from first to last keyframe).
    pub fn duration(&self) -> vidra_core::Duration {
        if self.keyframes.len() < 2 {
            return vidra_core::Duration::zero();
        }
        let first = self.keyframes[0].time.as_seconds();
        let last = self.keyframes[self.keyframes.len() - 1].time.as_seconds();
        vidra_core::Duration::from_seconds(last - first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vidra_core::types::Easing;

    #[test]
    fn test_animation_from_to() {
        let anim = Animation::from_to(
            AnimatableProperty::Opacity,
            1.0,
            0.0,
            vidra_core::Duration::from_seconds(2.0),
            Easing::Linear,
        );
        assert_eq!(anim.keyframes.len(), 2);
        assert!((anim.duration().as_seconds() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_animation_evaluate_linear() {
        let anim = Animation::from_to(
            AnimatableProperty::Opacity,
            0.0,
            1.0,
            vidra_core::Duration::from_seconds(1.0),
            Easing::Linear,
        );

        // At t=0
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(0.0))
            .unwrap();
        assert!((v - 0.0).abs() < 0.01);

        // At t=0.5
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(0.5))
            .unwrap();
        assert!((v - 0.5).abs() < 0.01);

        // At t=1.0
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(1.0))
            .unwrap();
        assert!((v - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_evaluate_with_delay() {
        let anim = Animation::from_to(
            AnimatableProperty::PositionX,
            0.0,
            100.0,
            vidra_core::Duration::from_seconds(1.0),
            Easing::Linear,
        )
        .with_delay(vidra_core::Duration::from_seconds(0.5));

        // Before delay — None
        assert!(anim
            .evaluate(vidra_core::Duration::from_seconds(0.3))
            .is_none());

        // At delay start
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(0.5))
            .unwrap();
        assert!((v - 0.0).abs() < 0.01);

        // Midpoint (0.5 delay + 0.5 into animation = 1.0 total)
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(1.0))
            .unwrap();
        assert!((v - 50.0).abs() < 0.01);

        // After animation
        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(2.0))
            .unwrap();
        assert!((v - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_animation_evaluate_ease_in() {
        let anim = Animation::from_to(
            AnimatableProperty::Opacity,
            0.0,
            1.0,
            vidra_core::Duration::from_seconds(1.0),
            Easing::EaseIn,
        );

        let v = anim
            .evaluate(vidra_core::Duration::from_seconds(0.5))
            .unwrap();
        // EaseIn at 0.5 = 0.25 (quadratic), so value should be 0.25
        assert!(v < 0.5, "EaseIn at midpoint should be < 0.5, got {}", v);
    }

    #[test]
    fn test_animation_add_keyframe_sorts() {
        let mut anim = Animation::new(AnimatableProperty::ScaleX);
        anim.add_keyframe(Keyframe::new(vidra_core::Duration::from_seconds(2.0), 1.5));
        anim.add_keyframe(Keyframe::new(vidra_core::Duration::from_seconds(0.0), 1.0));
        anim.add_keyframe(Keyframe::new(vidra_core::Duration::from_seconds(1.0), 1.2));

        assert!((anim.keyframes[0].time.as_seconds()).abs() < 0.001);
        assert!((anim.keyframes[1].time.as_seconds() - 1.0).abs() < 0.001);
        assert!((anim.keyframes[2].time.as_seconds() - 2.0).abs() < 0.001);
    }
}
