use std::ops::Range;

use kinetik_ui_core::{Rect, Size};

use super::ItemRect;
use super::math::{finite_non_negative, finite_positive};

/// Grid column behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridColumns {
    /// Fixed column count.
    Fixed(usize),
    /// Adaptive columns based on minimum item width.
    Adaptive {
        /// Minimum item width used to derive column count.
        min_width: f32,
    },
}

/// Grid layout model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridLayout {
    /// Column behavior.
    pub columns: GridColumns,
    /// Item size.
    pub item_size: Size,
    /// Gap between items.
    pub gap: f32,
}

impl GridLayout {
    /// Returns sanitized item size, or `None` when grid items cannot be laid out.
    #[must_use]
    pub fn effective_item_size(self) -> Option<Size> {
        let width = finite_positive(self.item_size.width)?;
        let height = finite_positive(self.item_size.height)?;
        Some(Size::new(width, height))
    }

    /// Returns the sanitized gap between grid items.
    #[must_use]
    pub fn effective_gap(self) -> f32 {
        finite_non_negative(self.gap)
    }

    /// Resolves the number of columns for bounds.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn column_count(self, bounds: Rect) -> usize {
        match self.columns {
            GridColumns::Fixed(count) => count.max(1),
            GridColumns::Adaptive { min_width } => {
                let gap = self.effective_gap();
                let available = finite_non_negative(bounds.width);
                let item_width = self
                    .effective_item_size()
                    .map_or(1.0, |size| size.width)
                    .max(1.0);
                let min_width = finite_positive(min_width).unwrap_or(item_width);
                ((available + gap) / (min_width + gap)).floor().max(1.0) as usize
            }
        }
    }

    /// Computes grid item rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn item_rects(self, bounds: Rect, count: usize, visible: Range<usize>) -> Vec<ItemRect> {
        let Some(item_size) = self.effective_item_size() else {
            return Vec::new();
        };
        let gap = self.effective_gap();
        let columns = self.column_count(bounds);
        visible
            .take_while(|index| *index < count)
            .map(|index| {
                let column = index % columns;
                let row = index / columns;
                ItemRect {
                    index,
                    rect: Rect::new(
                        bounds.x + column as f32 * (item_size.width + gap),
                        bounds.y + row as f32 * (item_size.height + gap),
                        item_size.width,
                        item_size.height,
                    ),
                }
            })
            .collect()
    }
}
