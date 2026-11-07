//! Platform-independent foundation types for FireEffects.
//!
//! This crate must stay free of UI, windowing, codec, and filesystem code so that
//! editing logic can be tested without a display or media files. The desktop
//! application depends on this crate; it never depends on the application.

mod error;
mod frame_rate;

pub use error::{Error, Result};
pub use frame_rate::FrameRate;
