use std::fmt;

/// Errors that can occur during merge operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeError {
    /// No images provided to merge.
    NoImages,

    /// Failed to decode an image.
    DecodeError {
        /// Zero-based index of the failing image.
        index: usize,
        /// Original filename if available.
        file_name: Option<String>,
        /// Error message from the decoder.
        message: String,
    },

    /// Internal encoding error.
    EncodeError { message: String },
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeError::NoImages => {
                write!(f, "No images provided")
            }
            MergeError::DecodeError {
                index,
                file_name,
                message,
            } => {
                write!(f, "Failed to decode image at index {}: {}", index, message)?;
                if let Some(name) = file_name {
                    write!(f, " (file: {})", name)?;
                }
                Ok(())
            }
            MergeError::EncodeError { message } => {
                write!(f, "Failed to encode output: {}", message)
            }
        }
    }
}

impl std::error::Error for MergeError {}

/// Error code strings for the worker protocol.
impl MergeError {
    pub fn code(&self) -> &'static str {
        match self {
            MergeError::NoImages => "NO_IMAGES",
            MergeError::DecodeError { .. } => "DECODE_FAILED",
            MergeError::EncodeError { .. } => "INTERNAL_ERROR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_no_images() {
        let err = MergeError::NoImages;
        assert_eq!(err.to_string(), "No images provided");
        assert_eq!(err.code(), "NO_IMAGES");
    }

    #[test]
    fn test_error_display_decode() {
        let err = MergeError::DecodeError {
            index: 2,
            file_name: Some("photo.jpg".to_string()),
            message: "invalid PNG header".to_string(),
        };
        assert!(err.to_string().contains("index 2"));
        assert!(err.to_string().contains("invalid PNG header"));
        assert!(err.to_string().contains("photo.jpg"));
        assert_eq!(err.code(), "DECODE_FAILED");
    }

    #[test]
    fn test_error_display_decode_no_filename() {
        let err = MergeError::DecodeError {
            index: 2,
            file_name: None,
            message: "invalid PNG header".to_string(),
        };
        assert!(err.to_string().contains("index 2"));
        assert!(!err.to_string().contains("file:"));
        assert_eq!(err.code(), "DECODE_FAILED");
    }

    #[test]
    fn test_error_display_encode() {
        let err = MergeError::EncodeError {
            message: "PNG write failed".to_string(),
        };
        assert!(err.to_string().contains("PNG write failed"));
        assert_eq!(err.code(), "INTERNAL_ERROR");
    }
}
