use geozero::error::GeozeroError;
use h3o::error::OutlinerError;
use std::{error::Error, fmt};

/// Errors occurring while rendering a set of cell indices to MVT.
#[derive(Debug)]
#[non_exhaustive]
pub enum RenderingError {
    /// Invalid input.
    InvalidInput(OutlinerError),
    /// MVT encoding failed.
    Encoding(GeozeroError),
}

impl fmt::Display for RenderingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidInput(ref source) => {
                write!(f, "invalid input: {source}")
            }
            Self::Encoding(ref source) => {
                write!(f, "MVT encoding failed: {source}")
            }
        }
    }
}

impl Error for RenderingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::InvalidInput(ref source) => Some(source),
            Self::Encoding(ref source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // All error must have a non-empty display.
    #[test]
    fn display() {
        assert!(!RenderingError::InvalidInput(OutlinerError::DuplicateInput)
            .to_string()
            .is_empty());
        assert!(!RenderingError::Encoding(GeozeroError::GeometryFormat)
            .to_string()
            .is_empty());
    }

    #[test]
    fn source() {
        assert!(RenderingError::InvalidInput(
            OutlinerError::HeterogeneousResolution
        )
        .source()
        .is_some());
        assert!(RenderingError::Encoding(GeozeroError::GeometryFormat)
            .source()
            .is_some());
    }
}
