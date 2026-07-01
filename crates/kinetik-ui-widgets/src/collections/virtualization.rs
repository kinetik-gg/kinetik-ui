use std::ops::Range;

use super::math::{finite_non_negative, finite_positive, finite_sum};

/// Virtualization request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualRangeRequest {
    /// Total item count.
    pub item_count: usize,
    /// Scroll offset in logical units.
    pub scroll_offset: f32,
    /// Viewport extent in logical units.
    pub viewport_extent: f32,
    /// Item extent in logical units.
    pub item_extent: f32,
    /// Extra items before and after the visible range.
    pub overscan: usize,
}

/// Fixed-extent virtualization request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualWindowRequest {
    /// Total item count.
    pub item_count: usize,
    /// Scroll offset in logical units.
    pub scroll_offset: f32,
    /// Viewport extent in logical units.
    pub viewport_extent: f32,
    /// Item extent in logical units.
    pub item_extent: f32,
    /// Extra items before and after the visible range.
    pub overscan: usize,
}

impl From<VirtualRangeRequest> for VirtualWindowRequest {
    fn from(request: VirtualRangeRequest) -> Self {
        Self {
            item_count: request.item_count,
            scroll_offset: request.scroll_offset,
            viewport_extent: request.viewport_extent,
            item_extent: request.item_extent,
            overscan: request.overscan,
        }
    }
}

/// Fixed-extent virtualization result.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualWindow {
    /// Total content extent in logical units.
    pub content_extent: f32,
    /// Maximum valid scroll offset in logical units.
    pub max_scroll_offset: f32,
    /// Scroll offset clamped to finite valid bounds.
    pub clamped_scroll_offset: f32,
    /// Strict visible item range before overscan.
    pub visible_range: Range<usize>,
    /// Overscanned range to materialize for layout and painting.
    pub materialized_range: Range<usize>,
}

impl VirtualWindow {
    fn empty() -> Self {
        Self {
            content_extent: 0.0,
            max_scroll_offset: 0.0,
            clamped_scroll_offset: 0.0,
            visible_range: 0..0,
            materialized_range: 0..0,
        }
    }
}

/// Computes a fixed-extent virtual window.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn virtual_window(request: VirtualWindowRequest) -> VirtualWindow {
    let Some(item_extent) = finite_positive(request.item_extent) else {
        return VirtualWindow::empty();
    };
    let Some(viewport_extent) = finite_positive(request.viewport_extent) else {
        return VirtualWindow::empty();
    };
    if request.item_count == 0 {
        return VirtualWindow::empty();
    }

    let content_extent = virtual_content_extent(request.item_count, item_extent);
    let max_scroll_offset =
        virtual_max_scroll_offset(request.item_count, item_extent, viewport_extent);
    let clamped_scroll_offset = finite_non_negative(request.scroll_offset).min(max_scroll_offset);

    let first = ((clamped_scroll_offset / item_extent).floor() as usize).min(request.item_count);
    let visible_start = first;
    let visible_end =
        (finite_sum(clamped_scroll_offset, viewport_extent) / item_extent).ceil() as usize;
    let visible_end = visible_end.min(request.item_count).max(visible_start);
    let visible_range = visible_start..visible_end;

    let materialized_visible = ((viewport_extent / item_extent).ceil() as usize)
        .saturating_add(1)
        .min(request.item_count);
    let start = first.saturating_sub(request.overscan);
    let end = first
        .saturating_add(materialized_visible)
        .saturating_add(request.overscan)
        .min(request.item_count);
    let materialized_range = start..end;

    VirtualWindow {
        content_extent,
        max_scroll_offset,
        clamped_scroll_offset,
        visible_range,
        materialized_range,
    }
}

/// Computes an overscanned item range for compatibility with existing callers.
#[must_use]
pub fn virtual_range(request: VirtualRangeRequest) -> Range<usize> {
    virtual_window(request.into()).materialized_range
}

/// Computes virtualized content extent for a fixed item extent.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn virtual_content_extent(item_count: usize, item_extent: f32) -> f32 {
    finite_positive(item_extent).map_or(0.0, |item_extent| {
        let extent = item_count as f32 * item_extent;
        if extent.is_finite() { extent } else { f32::MAX }
    })
}

/// Computes the maximum valid scroll offset for virtualized fixed-extent items.
#[must_use]
pub fn virtual_max_scroll_offset(item_count: usize, item_extent: f32, viewport_extent: f32) -> f32 {
    let content_extent = virtual_content_extent(item_count, item_extent);
    (content_extent - finite_non_negative(viewport_extent)).max(0.0)
}

/// Clamps a virtualized scroll offset to finite, valid bounds.
#[must_use]
pub fn clamp_virtual_scroll_offset(
    scroll_offset: f32,
    item_count: usize,
    item_extent: f32,
    viewport_extent: f32,
) -> f32 {
    let scroll_offset = finite_non_negative(scroll_offset);
    scroll_offset.min(virtual_max_scroll_offset(
        item_count,
        item_extent,
        viewport_extent,
    ))
}
