//! Logical and physical unit helpers.

use crate::geometry::{Point, Size};

/// A point in physical framebuffer pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalPoint {
    /// Horizontal physical pixel coordinate.
    pub x: i32,
    /// Vertical physical pixel coordinate.
    pub y: i32,
}

impl PhysicalPoint {
    /// The physical origin.
    pub const ZERO: Self = Self::new(0, 0);

    /// Creates a physical point from pixel coordinates.
    #[must_use]
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A size in physical framebuffer pixels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalSize {
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
}

impl PhysicalSize {
    /// A zero physical size.
    pub const ZERO: Self = Self::new(0, 0);

    /// Creates a physical size from pixel dimensions.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// A scale factor between logical UI units and physical framebuffer pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleFactor {
    value: f64,
}

impl ScaleFactor {
    /// A scale factor of `1.0`.
    pub const ONE: Self = Self::new(1.0);

    /// Creates a scale factor.
    ///
    /// Values less than or equal to zero are accepted by this constructor so
    /// callers can validate external input explicitly with [`Self::is_valid`].
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self { value }
    }

    /// Returns the raw scale factor value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Returns true when this scale factor can be used for conversion.
    #[must_use]
    pub fn is_valid(self) -> bool {
        self.value.is_finite() && self.value > 0.0
    }

    /// Converts a logical point to physical pixels using nearest rounding.
    #[must_use]
    pub fn logical_point_to_physical(self, point: Point) -> PhysicalPoint {
        PhysicalPoint::new(
            round_f32_to_i32(f64::from(point.x) * self.value),
            round_f32_to_i32(f64::from(point.y) * self.value),
        )
    }

    /// Converts a physical point to logical units.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn physical_point_to_logical(self, point: PhysicalPoint) -> Point {
        debug_assert!(self.is_valid(), "invalid scale factor");
        Point::new(
            point.x as f32 / self.value as f32,
            point.y as f32 / self.value as f32,
        )
    }

    /// Converts a logical size to physical pixels using ceil rounding.
    ///
    /// Ceil rounding avoids allocating a physical render target smaller than
    /// the logical area requires at fractional scale factors.
    #[must_use]
    pub fn logical_size_to_physical(self, size: Size) -> PhysicalSize {
        PhysicalSize::new(
            ceil_f32_to_u32(f64::from(size.width) * self.value),
            ceil_f32_to_u32(f64::from(size.height) * self.value),
        )
    }

    /// Converts a physical size to logical units.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn physical_size_to_logical(self, size: PhysicalSize) -> Size {
        debug_assert!(self.is_valid(), "invalid scale factor");
        Size::new(
            size.width as f32 / self.value as f32,
            size.height as f32 / self.value as f32,
        )
    }
}

#[allow(clippy::cast_possible_truncation)]
fn round_f32_to_i32(value: f64) -> i32 {
    if !value.is_finite() {
        return 0;
    }

    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ceil_f32_to_u32(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    value.ceil().min(f64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::{PhysicalPoint, PhysicalSize, ScaleFactor};
    use crate::geometry::{Point, Size};

    #[test]
    fn constructs_physical_units() {
        assert_eq!(PhysicalPoint::new(1, 2).x, 1);
        assert_eq!(PhysicalSize::new(3, 4).height, 4);
    }

    #[test]
    fn validates_scale_factors() {
        assert!(ScaleFactor::new(1.0).is_valid());
        assert!(ScaleFactor::new(1.5).is_valid());
        assert!(!ScaleFactor::new(0.0).is_valid());
        assert!(!ScaleFactor::new(-1.0).is_valid());
        assert!(!ScaleFactor::new(f64::NAN).is_valid());
    }

    #[test]
    fn converts_logical_points_to_physical_with_nearest_rounding() {
        let scale = ScaleFactor::new(1.5);

        assert_eq!(
            scale.logical_point_to_physical(Point::new(10.0, 11.0)),
            PhysicalPoint::new(15, 17)
        );
    }

    #[test]
    fn converts_logical_sizes_to_physical_with_ceil_rounding() {
        let scale = ScaleFactor::new(1.25);

        assert_eq!(
            scale.logical_size_to_physical(Size::new(10.0, 11.0)),
            PhysicalSize::new(13, 14)
        );
    }

    #[test]
    fn round_trips_common_scale_factors() {
        for value in [1.0, 1.25, 1.5, 2.0] {
            let scale = ScaleFactor::new(value);
            let logical_size = Size::new(1920.0, 1080.0);
            let physical_size = scale.logical_size_to_physical(logical_size);
            let round_trip = scale.physical_size_to_logical(physical_size);

            assert!(round_trip.width >= logical_size.width);
            assert!(round_trip.height >= logical_size.height);
            assert!(round_trip.width - logical_size.width <= 1.0);
            assert!(round_trip.height - logical_size.height <= 1.0);
        }
    }

    #[test]
    fn converts_physical_points_to_logical() {
        let scale = ScaleFactor::new(2.0);

        assert_eq!(
            scale.physical_point_to_logical(PhysicalPoint::new(100, 50)),
            Point::new(50.0, 25.0)
        );
    }

    #[test]
    fn non_finite_or_negative_physical_sizes_clamp_to_zero() {
        assert_eq!(
            ScaleFactor::new(f64::NAN).logical_size_to_physical(Size::new(10.0, 10.0)),
            PhysicalSize::ZERO
        );
        assert_eq!(
            ScaleFactor::new(1.0).logical_size_to_physical(Size::new(-10.0, 10.0)),
            PhysicalSize::new(0, 10)
        );
    }
}
