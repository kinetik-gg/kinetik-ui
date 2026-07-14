use std::hash::Hash;

use stern_core::{
    Insets, LayoutItem, Measurement, Rect, Size, SizeRule, column_layout, grid_layout, pad_rect,
    row_layout, stack_layout,
};

use super::{ScrollAreaOutput, Ui};

impl Ui<'_> {
    /// Allocates a measured row and invokes `add` once per child rectangle.
    ///
    /// Each child runs in a stable index scope below `key`, so repeated local
    /// widget keys remain distinct across siblings.
    pub fn row<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        items: &[LayoutItem],
        spacing: f32,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> Vec<T> {
        let allocations = row_layout(rect, items, spacing);
        self.scope(("layout_row", key), |ui| {
            ui.layout_children("row_child", allocations, add)
        })
    }

    /// Allocates a measured column and invokes `add` once per child rectangle.
    ///
    /// Each child runs in a stable index scope below `key`, so repeated local
    /// widget keys remain distinct across siblings.
    pub fn column<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        items: &[LayoutItem],
        spacing: f32,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> Vec<T> {
        let allocations = column_layout(rect, items, spacing);
        self.scope(("layout_column", key), |ui| {
            ui.layout_children("column_child", allocations, add)
        })
    }

    /// Allocates a measured row-major grid and invokes `add` for each child.
    #[allow(clippy::too_many_arguments)]
    pub fn grid<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        columns: &[SizeRule],
        rows: &[SizeRule],
        measurements: &[Measurement],
        column_spacing: f32,
        row_spacing: f32,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> Vec<T> {
        let allocations = grid_layout(
            rect,
            columns,
            rows,
            measurements,
            column_spacing,
            row_spacing,
        );
        self.scope(("layout_grid", key), |ui| {
            ui.layout_children("grid_child", allocations, add)
        })
    }

    /// Applies padding and invokes `add` with the resulting inner rectangle.
    pub fn padding<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        insets: Insets,
        add: impl FnOnce(&mut Self, Rect) -> T,
    ) -> T {
        let inner = pad_rect(rect, insets);
        self.scope(("layout_padding", key), |ui| add(ui, inner))
    }

    /// Allocates `count` layers over the same rectangle.
    pub fn stack<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        count: usize,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> Vec<T> {
        let allocations = stack_layout(rect, count);
        self.scope(("layout_stack", key), |ui| {
            ui.layout_children("stack_child", allocations, add)
        })
    }

    /// Allocates a measured row inside a horizontally scrollable viewport.
    ///
    /// Non-fill overflow determines the content width. Child rectangles stay
    /// in content coordinates; [`Self::scroll_area`] owns the sole clip and
    /// translation.
    pub fn scroll_row<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        items: &[LayoutItem],
        spacing: f32,
        disabled: bool,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> ScrollAreaOutput<Vec<T>> {
        let rect = sanitize_rect(rect);
        let allocations = row_layout(rect, items, spacing);
        let content_size = Size::new(
            content_extent(rect.x, rect.width, allocations.last(), true),
            rect.height,
        );

        self.scroll_area(key, rect, content_size, disabled, move |ui, _offset| {
            ui.layout_children("scroll_row_child", allocations, add)
        })
    }

    /// Allocates a measured column inside a vertically scrollable viewport.
    ///
    /// Non-fill overflow determines the content height. Child rectangles stay
    /// in content coordinates; [`Self::scroll_area`] owns the sole clip and
    /// translation.
    pub fn scroll_column<T>(
        &mut self,
        key: impl Hash,
        rect: Rect,
        items: &[LayoutItem],
        spacing: f32,
        disabled: bool,
        add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> ScrollAreaOutput<Vec<T>> {
        let rect = sanitize_rect(rect);
        let allocations = column_layout(rect, items, spacing);
        let content_size = Size::new(
            rect.width,
            content_extent(rect.y, rect.height, allocations.last(), false),
        );

        self.scroll_area(key, rect, content_size, disabled, move |ui, _offset| {
            ui.layout_children("scroll_column_child", allocations, add)
        })
    }

    fn layout_children<T>(
        &mut self,
        scope: &'static str,
        allocations: Vec<Rect>,
        mut add: impl FnMut(&mut Self, usize, Rect) -> T,
    ) -> Vec<T> {
        allocations
            .into_iter()
            .enumerate()
            .map(|(index, rect)| self.scope((scope, index), |ui| add(ui, index, rect)))
            .collect()
    }
}

fn sanitize_rect(rect: Rect) -> Rect {
    Rect::new(
        sanitize_origin(rect.x),
        sanitize_origin(rect.y),
        sanitize_size(rect.width),
        sanitize_size(rect.height),
    )
}

fn sanitize_origin(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn sanitize_size(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn content_extent(origin: f32, viewport_extent: f32, last: Option<&Rect>, horizontal: bool) -> f32 {
    let child_extent = last.map_or(0.0, |rect| {
        let (child_origin, child_size) = if horizontal {
            (rect.x, rect.width)
        } else {
            (rect.y, rect.height)
        };
        let extent = child_origin - origin + child_size;
        if extent.is_finite() {
            extent.max(0.0)
        } else {
            f32::MAX
        }
    });

    viewport_extent.max(child_extent)
}
