//! Platform-independent foundation types for FireEffects.
//!
//! This crate must stay free of UI, windowing, codec, and filesystem code so that
//! editing logic can be tested without a display or media files. The desktop
//! application depends on this crate; it never depends on the application.

mod error;
mod frame_rate;
mod input;
mod workspace;

pub use error::{Error, Result};
pub use frame_rate::FrameRate;
pub use input::{Action, BINDINGS, Binding, Key, Modifiers, binding_for, resolve};
pub use workspace::{Panel, Workspace};
