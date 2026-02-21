/// Core error types for the Vidra engine.
use std::path::PathBuf;

/// A specialized Result type for Vidra operations.
pub type VidraResult<T> = Result<T, VidraError>;

/// Top-level error type encompassing all Vidra subsystems.
#[derive(Debug, thiserror::Error)]
pub enum VidraError {
    #[error("parse error: {message} at {file}:{line}:{column}")]
    Parse {
        message: String,
        file: String,
        line: usize,
        column: usize,
    },

    #[error("type error: {message} at {file}:{line}:{column}")]
    Type {
        message: String,
        file: String,
        line: usize,
        column: usize,
    },

    #[error("compile error: {0}")]
    Compile(String),

    #[error("render error: {0}")]
    Render(String),

    #[error("encode error: {0}")]
    Encode(String),

    #[error("asset error: {message} ({path:?})")]
    Asset { message: String, path: PathBuf },

    #[error("IR validation error: {0}")]
    IrValidation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("unsupported feature: {0}")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}

impl VidraError {
    /// Create a parse error with source location.
    pub fn parse(
        message: impl Into<String>,
        file: impl Into<String>,
        line: usize,
        column: usize,
    ) -> Self {
        VidraError::Parse {
            message: message.into(),
            file: file.into(),
            line,
            column,
        }
    }

    /// Create an asset error.
    pub fn asset(message: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        VidraError::Asset {
            message: message.into(),
            path: path.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = VidraError::parse("unexpected token", "main.vidra", 10, 5);
        assert_eq!(
            err.to_string(),
            "parse error: unexpected token at main.vidra:10:5"
        );
    }

    #[test]
    fn test_asset_error_display() {
        let err = VidraError::asset("file not found", "/assets/hero.jpg");
        assert!(err.to_string().contains("file not found"));
    }
}
