use geozero::error::GeozeroError;
use h3o::error::DissolutionError;
use std::{error::Error, fmt};

/// Error occurring while rendering a set of cell indices to MVT.
#[derive(Debug)]
#[non_exhaustive]
pub enum InvalidTileID {
    /// Invalid X coordinate.
    InvalidX(u32),
    /// Invalid Y coordinate.
    InvalidY(u32),
    /// Invalid Z coordinate.
    InvalidZ(u32),
}

impl fmt::Display for InvalidTileID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidX(value) => {
                write!(f, "invalid x coordinate: {value}")
            }
            Self::InvalidY(value) => {
                write!(f, "invalid y coordinate: {value}")
            }
            Self::InvalidZ(value) => {
                write!(f, "invalid z coordinate: {value}")
            }
        }
    }
}

impl Error for InvalidTileID {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            Self::InvalidX(_) | Self::InvalidY(_) | Self::InvalidZ(_) => None,
        }
    }
}

/// Errors occurring while rendering a set of cell indices to MVT.
#[derive(Debug)]
#[non_exhaustive]
pub enum RenderingError {
    /// Invalid input.
    InvalidInput(DissolutionError),
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
        assert!(!RenderingError::InvalidInput(
            DissolutionError::DuplicateInput
        )
        .to_string()
        .is_empty());
        assert!(!RenderingError::Encoding(GeozeroError::GeometryFormat)
            .to_string()
            .is_empty());
        assert!(!InvalidTileID::InvalidX(42).to_string().is_empty());
        assert!(!InvalidTileID::InvalidY(42).to_string().is_empty());
        assert!(!InvalidTileID::InvalidZ(42).to_string().is_empty());
    }

    #[test]
    fn source() {
        assert!(RenderingError::InvalidInput(
            DissolutionError::HeterogeneousResolution
        )
        .source()
        .is_some());
        assert!(RenderingError::Encoding(GeozeroError::GeometryFormat)
            .source()
            .is_some());
        assert!(InvalidTileID::InvalidX(42).source().is_none());
        assert!(InvalidTileID::InvalidY(42).source().is_none());
        assert!(InvalidTileID::InvalidZ(42).source().is_none());
    }
}
