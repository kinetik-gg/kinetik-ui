//! Geometry types expressed in logical UI units.

/// A 2D point in logical UI coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Point {
    /// The origin point.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a point from logical coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns this point translated by a vector.
    #[must_use]
    pub const fn translate(self, offset: Vec2) -> Self {
        Self::new(self.x + offset.x, self.y + offset.y)
    }
}

/// A 2D vector in logical UI units.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Vec2 {
    /// The zero vector.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a vector from logical components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A 2D size in logical UI units.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Size {
    /// Horizontal extent.
    pub width: f32,
    /// Vertical extent.
    pub height: f32,
}

impl Size {
    /// A zero size.
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Creates a size from logical dimensions.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Returns true when either dimension is zero or negative.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Clamps negative dimensions to zero.
    #[must_use]
    pub fn max_zero(self) -> Self {
        Self::new(self.width.max(0.0), self.height.max(0.0))
    }
}

/// An axis-aligned rectangle in logical UI coordinates.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    /// Minimum x coordinate.
    pub x: f32,
    /// Minimum y coordinate.
    pub y: f32,
    /// Rectangle width.
    pub width: f32,
    /// Rectangle height.
    pub height: f32,
}

impl Rect {
    /// An empty rectangle at the origin.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// Creates a rectangle from origin and size components.
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a rectangle from an origin point and size.
    #[must_use]
    pub const fn from_origin_size(origin: Point, size: Size) -> Self {
        Self::new(origin.x, origin.y, size.width, size.height)
    }

    /// Creates a rectangle from minimum and maximum points.
    #[must_use]
    pub fn from_min_max(min: Point, max: Point) -> Self {
        Self::new(min.x, min.y, max.x - min.x, max.y - min.y)
    }

    /// Returns the origin point.
    #[must_use]
    pub const fn origin(self) -> Point {
        Point::new(self.x, self.y)
    }

    /// Returns the rectangle size.
    #[must_use]
    pub const fn size(self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Returns the minimum x coordinate.
    #[must_use]
    pub const fn min_x(self) -> f32 {
        self.x
    }

    /// Returns the minimum y coordinate.
    #[must_use]
    pub const fn min_y(self) -> f32 {
        self.y
    }

    /// Returns the maximum x coordinate.
    #[must_use]
    pub const fn max_x(self) -> f32 {
        self.x + self.width
    }

    /// Returns the maximum y coordinate.
    #[must_use]
    pub const fn max_y(self) -> f32 {
        self.y + self.height
    }

    /// Returns the minimum point.
    #[must_use]
    pub const fn min(self) -> Point {
        Point::new(self.min_x(), self.min_y())
    }

    /// Returns the maximum point.
    #[must_use]
    pub const fn max(self) -> Point {
        Point::new(self.max_x(), self.max_y())
    }

    /// Returns the center point.
    #[must_use]
    pub const fn center(self) -> Point {
        Point::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// Returns true when either dimension is zero or negative.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Returns true when the point is inside the rectangle.
    ///
    /// The minimum edge is inclusive and the maximum edge is exclusive.
    #[must_use]
    pub const fn contains_point(self, point: Point) -> bool {
        point.x >= self.min_x()
            && point.y >= self.min_y()
            && point.x < self.max_x()
            && point.y < self.max_y()
    }

    /// Returns true when `other` is fully contained by this rectangle.
    #[must_use]
    pub const fn contains_rect(self, other: Self) -> bool {
        other.min_x() >= self.min_x()
            && other.min_y() >= self.min_y()
            && other.max_x() <= self.max_x()
            && other.max_y() <= self.max_y()
    }

    /// Returns the intersection between two rectangles.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let min_x = self.min_x().max(other.min_x());
        let min_y = self.min_y().max(other.min_y());
        let max_x = self.max_x().min(other.max_x());
        let max_y = self.max_y().min(other.max_y());

        if max_x <= min_x || max_y <= min_y {
            None
        } else {
            Some(Self::from_min_max(
                Point::new(min_x, min_y),
                Point::new(max_x, max_y),
            ))
        }
    }

    /// Returns the smallest rectangle containing both rectangles.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }

        if other.is_empty() {
            return self;
        }

        Self::from_min_max(
            Point::new(
                self.min_x().min(other.min_x()),
                self.min_y().min(other.min_y()),
            ),
            Point::new(
                self.max_x().max(other.max_x()),
                self.max_y().max(other.max_y()),
            ),
        )
    }

    /// Returns this rectangle translated by a vector.
    #[must_use]
    pub const fn translate(self, offset: Vec2) -> Self {
        Self::new(
            self.x + offset.x,
            self.y + offset.y,
            self.width,
            self.height,
        )
    }

    /// Returns a rectangle inset on all sides.
    #[must_use]
    pub fn inset(self, amount: f32) -> Self {
        Self::new(
            self.x + amount,
            self.y + amount,
            self.width - amount * 2.0,
            self.height - amount * 2.0,
        )
    }

    /// Returns a rectangle outset on all sides.
    #[must_use]
    pub fn outset(self, amount: f32) -> Self {
        self.inset(-amount)
    }

    /// Returns a rectangle with negative dimensions clamped to zero.
    #[must_use]
    pub fn max_zero(self) -> Self {
        let size = self.size().max_zero();
        Self::from_origin_size(self.origin(), size)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::{Point, Rect, Size, Vec2};

    #[test]
    fn constructs_basic_geometry_types() {
        assert_eq!(Point::new(1.0, 2.0).x, 1.0);
        assert_eq!(Vec2::new(3.0, 4.0).y, 4.0);
        assert_eq!(Size::new(5.0, 6.0).width, 5.0);
        assert_eq!(Rect::new(1.0, 2.0, 3.0, 4.0).size(), Size::new(3.0, 4.0));
    }

    #[test]
    fn reports_empty_sizes_and_rectangles() {
        assert!(Size::new(0.0, 4.0).is_empty());
        assert!(Size::new(4.0, -1.0).is_empty());
        assert!(Rect::new(0.0, 0.0, 0.0, 10.0).is_empty());
        assert!(!Rect::new(0.0, 0.0, 10.0, 10.0).is_empty());
    }

    #[test]
    fn translates_points_and_rectangles() {
        let offset = Vec2::new(3.0, -2.0);
        assert_eq!(Point::new(1.0, 5.0).translate(offset), Point::new(4.0, 3.0));
        assert_eq!(
            Rect::new(1.0, 2.0, 3.0, 4.0).translate(offset),
            Rect::new(4.0, 0.0, 3.0, 4.0)
        );
    }

    #[test]
    fn contains_points_with_exclusive_max_edge() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert!(rect.contains_point(Point::new(10.0, 20.0)));
        assert!(rect.contains_point(Point::new(39.999, 59.999)));
        assert!(!rect.contains_point(Point::new(40.0, 30.0)));
        assert!(!rect.contains_point(Point::new(20.0, 60.0)));
    }

    #[test]
    fn contains_rectangles() {
        let outer = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(outer.contains_rect(Rect::new(10.0, 10.0, 20.0, 20.0)));
        assert!(outer.contains_rect(Rect::new(0.0, 0.0, 100.0, 100.0)));
        assert!(!outer.contains_rect(Rect::new(-1.0, 0.0, 10.0, 10.0)));
    }

    #[test]
    fn intersects_rectangles_symmetrically() {
        let a = Rect::new(0.0, 0.0, 20.0, 20.0);
        let b = Rect::new(10.0, 5.0, 20.0, 20.0);
        let expected = Some(Rect::new(10.0, 5.0, 10.0, 15.0));

        assert_eq!(a.intersection(b), expected);
        assert_eq!(b.intersection(a), expected);
    }

    #[test]
    fn returns_no_intersection_for_touching_edges() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 0.0, 10.0, 10.0);

        assert_eq!(a.intersection(b), None);
    }

    #[test]
    fn union_contains_both_rectangles() {
        let a = Rect::new(10.0, 20.0, 30.0, 40.0);
        let b = Rect::new(-5.0, 15.0, 10.0, 20.0);
        let union = a.union(b);

        assert!(union.contains_rect(a));
        assert!(union.contains_rect(b));
        assert_eq!(union, Rect::new(-5.0, 15.0, 45.0, 45.0));
    }

    #[test]
    fn inset_and_outset_adjust_all_edges() {
        let rect = Rect::new(10.0, 20.0, 30.0, 40.0);

        assert_eq!(rect.inset(5.0), Rect::new(15.0, 25.0, 20.0, 30.0));
        assert_eq!(rect.outset(5.0), Rect::new(5.0, 15.0, 40.0, 50.0));
    }

    #[test]
    fn clamps_negative_size_to_zero() {
        assert_eq!(Size::new(-1.0, 2.0).max_zero(), Size::new(0.0, 2.0));
        assert_eq!(
            Rect::new(1.0, 2.0, -3.0, 4.0).max_zero(),
            Rect::new(1.0, 2.0, 0.0, 4.0)
        );
    }
}
