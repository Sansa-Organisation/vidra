use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Mul, Sub};

/// Time duration with sub-millisecond precision (stored as fractional seconds).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Duration {
    /// Duration in seconds.
    seconds: f64,
}

impl Duration {
    /// Create a duration from seconds.
    pub fn from_seconds(s: f64) -> Self {
        Self {
            seconds: s.max(0.0),
        }
    }

    /// Create a duration from milliseconds.
    pub fn from_millis(ms: f64) -> Self {
        Self::from_seconds(ms / 1000.0)
    }

    /// Create a zero duration.
    pub fn zero() -> Self {
        Self { seconds: 0.0 }
    }

    /// Get duration as seconds.
    pub fn as_seconds(&self) -> f64 {
        self.seconds
    }

    /// Get duration as milliseconds.
    pub fn as_millis(&self) -> f64 {
        self.seconds * 1000.0
    }

    /// Compute number of frames for a given FPS.
    pub fn frame_count(&self, fps: f64) -> u64 {
        (self.seconds * fps).ceil() as u64
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::zero()
    }
}

impl Add for Duration {
    type Output = Duration;
    fn add(self, rhs: Duration) -> Duration {
        Duration::from_seconds(self.seconds + rhs.seconds)
    }
}

impl Sub for Duration {
    type Output = Duration;
    fn sub(self, rhs: Duration) -> Duration {
        Duration::from_seconds((self.seconds - rhs.seconds).max(0.0))
    }
}

impl Mul<f64> for Duration {
    type Output = Duration;
    fn mul(self, rhs: f64) -> Duration {
        Duration::from_seconds(self.seconds * rhs)
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.seconds < 1.0 {
            write!(f, "{:.0}ms", self.seconds * 1000.0)
        } else {
            write!(f, "{:.2}s", self.seconds)
        }
    }
}

/// A point in time within a video.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Timestamp {
    /// Time in seconds from the start of the video.
    seconds: f64,
}

impl Timestamp {
    /// Create a timestamp from seconds.
    pub fn from_seconds(s: f64) -> Self {
        Self {
            seconds: s.max(0.0),
        }
    }

    /// Create a timestamp at the start (0.0).
    pub fn zero() -> Self {
        Self { seconds: 0.0 }
    }

    /// Get the time in seconds.
    pub fn as_seconds(&self) -> f64 {
        self.seconds
    }

    /// Convert to a frame index for a given FPS.
    pub fn to_frame(&self, fps: f64) -> u64 {
        (self.seconds * fps).floor() as u64
    }

    /// Compute the duration between two timestamps.
    pub fn duration_to(&self, other: &Timestamp) -> Duration {
        Duration::from_seconds((other.seconds - self.seconds).abs())
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Timestamp::zero()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Timestamp {
        Timestamp::from_seconds(self.seconds + rhs.as_seconds())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_ms = (self.seconds * 1000.0) as u64;
        let hours = total_ms / 3_600_000;
        let minutes = (total_ms % 3_600_000) / 60_000;
        let secs = (total_ms % 60_000) / 1_000;
        let ms = total_ms % 1_000;
        write!(f, "{:02}:{:02}:{:02}.{:03}", hours, minutes, secs, ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_from_seconds() {
        let d = Duration::from_seconds(2.5);
        assert!((d.as_seconds() - 2.5).abs() < 0.001);
        assert!((d.as_millis() - 2500.0).abs() < 0.1);
    }

    #[test]
    fn test_duration_frame_count() {
        let d = Duration::from_seconds(1.0);
        assert_eq!(d.frame_count(30.0), 30);
    }

    #[test]
    fn test_duration_arithmetic() {
        let a = Duration::from_seconds(1.0);
        let b = Duration::from_seconds(0.5);
        assert!((a + b).as_seconds() - 1.5 < 0.001);
        assert!((a - b).as_seconds() - 0.5 < 0.001);
        assert!((a * 3.0).as_seconds() - 3.0 < 0.001);
    }

    #[test]
    fn test_duration_display() {
        assert_eq!(format!("{}", Duration::from_seconds(2.5)), "2.50s");
        assert_eq!(format!("{}", Duration::from_millis(500.0)), "500ms");
    }

    #[test]
    fn test_timestamp_to_frame() {
        let ts = Timestamp::from_seconds(1.0);
        assert_eq!(ts.to_frame(30.0), 30);
    }

    #[test]
    fn test_timestamp_display() {
        let ts = Timestamp::from_seconds(3661.5);
        assert_eq!(format!("{}", ts), "01:01:01.500");
    }

    #[test]
    fn test_timestamp_add_duration() {
        let ts = Timestamp::from_seconds(1.0);
        let d = Duration::from_seconds(0.5);
        let result = ts + d;
        assert!((result.as_seconds() - 1.5).abs() < 0.001);
    }
}
