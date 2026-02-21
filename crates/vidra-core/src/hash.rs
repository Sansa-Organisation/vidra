//! Content hashing for deterministic rendering verification.
//!
//! Produces a SHA-256 hash of frame buffer data, enabling bit-exact
//! output verification across platforms and runs.

use sha2::{Digest, Sha256};

use crate::frame::FrameBuffer;

/// A content hash digest (SHA-256, 32 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentHash {
    bytes: [u8; 32],
}

impl ContentHash {
    /// Create from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Get the hash as a hex string.
    pub fn to_hex(&self) -> String {
        self.bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Compute the content hash of a single frame buffer.
pub fn hash_frame(frame: &FrameBuffer) -> ContentHash {
    let mut hasher = Sha256::new();
    // Include dimensions and format in the hash so different-sized
    // buffers with identical pixel data produce different hashes.
    hasher.update(frame.width.to_le_bytes());
    hasher.update(frame.height.to_le_bytes());
    hasher.update(&[frame.format as u8]);
    hasher.update(&frame.data);
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    ContentHash::from_bytes(bytes)
}

/// Compute the content hash of a sequence of frames (entire render output).
pub fn hash_frames(frames: &[FrameBuffer]) -> ContentHash {
    let mut hasher = Sha256::new();
    // Include frame count
    hasher.update((frames.len() as u64).to_le_bytes());
    for frame in frames {
        hasher.update(frame.width.to_le_bytes());
        hasher.update(frame.height.to_le_bytes());
        hasher.update(&[frame.format as u8]);
        hasher.update(&frame.data);
    }
    let result = hasher.finalize();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    ContentHash::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn test_hash_deterministic() {
        let frame1 = FrameBuffer::solid(10, 10, &Color::RED);
        let frame2 = FrameBuffer::solid(10, 10, &Color::RED);
        assert_eq!(hash_frame(&frame1), hash_frame(&frame2));
    }

    #[test]
    fn test_hash_different_content() {
        let frame1 = FrameBuffer::solid(10, 10, &Color::RED);
        let frame2 = FrameBuffer::solid(10, 10, &Color::BLUE);
        assert_ne!(hash_frame(&frame1), hash_frame(&frame2));
    }

    #[test]
    fn test_hash_different_size() {
        let frame1 = FrameBuffer::solid(10, 10, &Color::RED);
        let frame2 = FrameBuffer::solid(20, 20, &Color::RED);
        assert_ne!(hash_frame(&frame1), hash_frame(&frame2));
    }

    #[test]
    fn test_hash_sequence_deterministic() {
        let frames = vec![
            FrameBuffer::solid(4, 4, &Color::RED),
            FrameBuffer::solid(4, 4, &Color::GREEN),
            FrameBuffer::solid(4, 4, &Color::BLUE),
        ];
        let hash1 = hash_frames(&frames);
        let hash2 = hash_frames(&frames);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_hex_format() {
        let frame = FrameBuffer::solid(2, 2, &Color::BLACK);
        let hash = hash_frame(&frame);
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64); // SHA-256 = 64 hex chars
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_display() {
        let frame = FrameBuffer::solid(2, 2, &Color::BLACK);
        let hash = hash_frame(&frame);
        assert_eq!(format!("{}", hash), hash.to_hex());
    }
}
