//! Core runtime types for Kinetik UI.
//!
//! This crate owns platform-independent UI concepts. It must not depend on
//! windowing, renderer, or operating-system APIs.

pub mod geometry;
pub mod units;

pub use geometry::{Point, Rect, Size, Vec2};
pub use units::{PhysicalPoint, PhysicalSize, ScaleFactor};

/// Returns the crate name.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-ui-core"
}
