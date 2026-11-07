use std::error::Error as StdError;
use std::fmt;

/// Errors reported by the core crate.
///
/// Variants are `#[non_exhaustive]` so new failure cases can be added without
/// breaking callers outside this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A frame rate was constructed with a numerator of zero.
    ZeroFrameRateNumerator,
    /// A frame rate was constructed with a denominator of zero.
    ZeroFrameRateDenominator,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroFrameRateNumerator => f.write_str("frame rate numerator must not be zero"),
            Self::ZeroFrameRateDenominator => f.write_str("frame rate denominator must not be zero"),
        }
    }
}

impl StdError for Error {}

/// Result alias used by the core crate.
pub type Result<T> = std::result::Result<T, Error>;
