use std::ops::Range;

use kinetik_ui_core::Rect;

use super::math::{
    finite_coordinate, finite_index_extent, finite_non_negative, finite_positive, finite_sum,
};
use super::{
    ItemRect, VirtualWindow, VirtualWindowRequest, clamp_virtual_scroll_offset,
    virtual_content_extent, virtual_max_scroll_offset, virtual_window,
};

/// List layout model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListLayout {
    /// Row height in logical units.
    pub row_height: f32,
}

impl ListLayout {
    /// Creates a list layout.
    #[must_use]
    pub const fn new(row_height: f32) -> Self {
        Self { row_height }
    }

    /// Returns the sanitized row height, or `None` when rows cannot be laid out.
    #[must_use]
    pub fn effective_row_height(self) -> Option<f32> {
        finite_positive(self.row_height)
    }

    /// Computes total content height for the row count.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn content_height(self, rows: usize) -> f32 {
        self.effective_row_height()
            .map_or(0.0, |row_height| virtual_content_extent(rows, row_height))
    }

    /// Computes the maximum vertical scroll offset for this list.
    #[must_use]
    pub fn max_scroll_offset(self, rows: usize, viewport_height: f32) -> f32 {
        self.effective_row_height().map_or(0.0, |row_height| {
            virtual_max_scroll_offset(rows, row_height, viewport_height)
        })
    }

    /// Clamps a scroll offset to the valid list range.
    #[must_use]
    pub fn clamp_scroll_offset(self, rows: usize, viewport_height: f32, scroll_offset: f32) -> f32 {
        self.effective_row_height().map_or(0.0, |row_height| {
            clamp_virtual_scroll_offset(scroll_offset, rows, row_height, viewport_height)
        })
    }

    /// Computes the virtual window for a viewport.
    #[must_use]
    pub fn virtual_window(
        self,
        rows: usize,
        scroll_offset: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> VirtualWindow {
        virtual_window(VirtualWindowRequest {
            item_count: rows,
            scroll_offset,
            viewport_extent: viewport_height,
            item_extent: self.row_height,
            overscan,
        })
    }

    /// Computes the virtualized row range for a viewport.
    #[must_use]
    pub fn visible_range(
        self,
        rows: usize,
        scroll_offset: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> Range<usize> {
        self.virtual_window(rows, scroll_offset, viewport_height, overscan)
            .materialized_range
    }

    /// Computes one row rectangle in content coordinates.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn row_rect(self, bounds: Rect, index: usize) -> Option<Rect> {
        let row_height = self.effective_row_height()?;
        Some(Rect::new(
            finite_coordinate(bounds.x),
            finite_sum(
                finite_coordinate(bounds.y),
                finite_index_extent(index, row_height),
            ),
            finite_non_negative(bounds.width),
            row_height,
        ))
    }

    /// Computes row rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn row_rects(self, bounds: Rect, rows: usize, visible: Range<usize>) -> Vec<ItemRect> {
        visible
            .take_while(|index| *index < rows)
            .filter_map(|index| {
                self.row_rect(bounds, index)
                    .map(|rect| ItemRect { index, rect })
            })
            .collect()
    }

    /// Computes visible row rectangles in viewport coordinates.
    #[must_use]
    pub fn visible_row_rects(
        self,
        bounds: Rect,
        rows: usize,
        scroll_offset: f32,
        overscan: usize,
    ) -> Vec<ItemRect> {
        let Some(row_height) = self.effective_row_height() else {
            return Vec::new();
        };
        let clamped_scroll =
            clamp_virtual_scroll_offset(scroll_offset, rows, row_height, bounds.height);
        self.row_rects(
            Rect::new(
                finite_coordinate(bounds.x),
                finite_sum(finite_coordinate(bounds.y), -clamped_scroll),
                finite_non_negative(bounds.width),
                finite_non_negative(bounds.height),
            ),
            rows,
            self.visible_range(rows, clamped_scroll, bounds.height, overscan),
        )
    }
}
