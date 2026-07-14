use std::collections::BTreeMap;
use std::ops::Range;

use stern_core::{Rect, Size};

use super::math::{
    finite_coordinate, finite_index_extent, finite_non_negative, finite_positive, finite_sum,
};
use super::{
    ItemId, ItemRect, VirtualWindow, VirtualWindowRequest, clamp_virtual_scroll_offset,
    virtual_content_extent, virtual_max_scroll_offset, virtual_window,
};

/// Table column width constraints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumnConstraints {
    /// Minimum column width.
    pub min_width: f32,
    /// Maximum column width.
    pub max_width: f32,
}

impl TableColumnConstraints {
    /// Creates column width constraints.
    #[must_use]
    pub const fn new(min_width: f32, max_width: f32) -> Self {
        Self {
            min_width,
            max_width,
        }
    }

    /// Creates unconstrained finite non-negative width bounds.
    #[must_use]
    pub const fn unconstrained() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::MAX,
        }
    }

    /// Returns deterministic finite non-negative constraints.
    #[must_use]
    pub fn sanitized(self) -> Self {
        let min_width = finite_non_negative(self.min_width);
        let max_width = if self.max_width.is_finite() {
            finite_non_negative(self.max_width)
        } else {
            f32::MAX
        }
        .max(min_width);

        Self {
            min_width,
            max_width,
        }
    }

    /// Clamps a width into these deterministic constraints.
    #[must_use]
    pub fn clamp_width(self, width: f32) -> f32 {
        let constraints = self.sanitized();
        finite_non_negative(width).clamp(constraints.min_width, constraints.max_width)
    }
}

impl Default for TableColumnConstraints {
    fn default() -> Self {
        Self::unconstrained()
    }
}

/// Table column.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// Column ID.
    pub id: ItemId,
    /// Header label.
    pub header: String,
    /// Column width.
    pub width: f32,
}

impl TableColumn {
    /// Creates a table column.
    #[must_use]
    pub fn new(id: ItemId, header: impl Into<String>, width: f32) -> Self {
        Self {
            id,
            header: header.into(),
            width,
        }
    }

    /// Returns the finite non-negative column width without additional constraints.
    #[must_use]
    pub fn effective_width(&self) -> f32 {
        self.clamped_width(TableColumnConstraints::default())
    }

    /// Returns the column width clamped by the supplied constraints.
    #[must_use]
    pub fn clamped_width(&self, constraints: TableColumnConstraints) -> f32 {
        constraints.clamp_width(self.width)
    }
}

/// Sort direction requested by table headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending sort.
    Ascending,
    /// Descending sort.
    Descending,
}

/// Table sort intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableSort {
    /// Column to sort by.
    pub column: ItemId,
    /// Direction.
    pub direction: SortDirection,
}

/// Rectangle assigned to a table header cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableHeaderRect {
    /// Column index.
    pub column: usize,
    /// Column identity.
    pub column_id: ItemId,
    /// Header rectangle.
    pub rect: Rect,
}

/// Rectangle assigned to a table body cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableCellRect {
    /// Row index.
    pub row: usize,
    /// Column index.
    pub column: usize,
    /// Column identity.
    pub column_id: ItemId,
    /// Flat cell index in row-major order.
    pub index: usize,
    /// Cell rectangle.
    pub rect: Rect,
}

/// Table layout model.
#[derive(Debug, Clone, PartialEq)]
pub struct TableLayout {
    /// Columns.
    pub columns: Vec<TableColumn>,
    /// Header height.
    pub header_height: f32,
    /// Row height.
    pub row_height: f32,
    /// Requested sort.
    pub sort: Option<TableSort>,
}

impl TableLayout {
    fn column_width_with_optional_constraints(
        column: &TableColumn,
        constraints: Option<&BTreeMap<ItemId, TableColumnConstraints>>,
    ) -> f32 {
        column.clamped_width(
            constraints
                .and_then(|constraints| constraints.get(&column.id).copied())
                .unwrap_or_default(),
        )
    }

    /// Returns the sanitized header height.
    #[must_use]
    pub fn effective_header_height(&self) -> f32 {
        finite_non_negative(self.header_height)
    }

    /// Returns the sanitized row height, or `None` when rows cannot be laid out.
    #[must_use]
    pub fn effective_row_height(&self) -> Option<f32> {
        finite_positive(self.row_height)
    }

    /// Returns the sanitized total width of all columns.
    #[must_use]
    pub fn total_width(&self) -> f32 {
        self.columns
            .iter()
            .map(|column| Self::column_width_with_optional_constraints(column, None))
            .sum()
    }

    /// Returns the constrained total width of all columns.
    #[must_use]
    pub fn total_width_with_constraints(
        &self,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> f32 {
        self.columns
            .iter()
            .map(|column| Self::column_width_with_optional_constraints(column, Some(constraints)))
            .sum()
    }

    /// Returns a column width by stable column ID.
    #[must_use]
    pub fn column_width(&self, column_id: ItemId) -> Option<f32> {
        self.columns
            .iter()
            .find(|column| column.id == column_id)
            .map(|column| Self::column_width_with_optional_constraints(column, None))
    }

    /// Returns a constrained column width by stable column ID.
    #[must_use]
    pub fn column_width_with_constraints(
        &self,
        column_id: ItemId,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> Option<f32> {
        self.columns
            .iter()
            .find(|column| column.id == column_id)
            .map(|column| Self::column_width_with_optional_constraints(column, Some(constraints)))
    }

    /// Resizes a column by stable column ID without additional constraints.
    pub fn resize_column(&mut self, column_id: ItemId, delta: f32) -> bool {
        self.resize_column_with_optional_constraints(column_id, delta, None)
    }

    /// Resizes a column by stable column ID and clamps the result to keyed constraints.
    pub fn resize_column_with_constraints(
        &mut self,
        column_id: ItemId,
        delta: f32,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> bool {
        self.resize_column_with_optional_constraints(column_id, delta, Some(constraints))
    }

    fn resize_column_with_optional_constraints(
        &mut self,
        column_id: ItemId,
        delta: f32,
        constraints: Option<&BTreeMap<ItemId, TableColumnConstraints>>,
    ) -> bool {
        let Some(column) = self
            .columns
            .iter_mut()
            .find(|column| column.id == column_id)
        else {
            return false;
        };

        let constraints = constraints
            .and_then(|constraints| constraints.get(&column.id).copied())
            .unwrap_or_default();
        let current_width = column.clamped_width(constraints);
        let resized_width =
            constraints.clamp_width(finite_sum(current_width, finite_coordinate(delta)));

        if current_width.to_bits() == resized_width.to_bits() {
            return false;
        }

        column.width = resized_width;
        true
    }

    /// Computes the total body height for a row count.
    #[must_use]
    pub fn body_height(&self, rows: usize) -> f32 {
        self.effective_row_height()
            .map_or(0.0, |row_height| virtual_content_extent(rows, row_height))
    }

    /// Computes the total table content size for a row count.
    #[must_use]
    pub fn content_size(&self, rows: usize) -> Size {
        Size::new(
            self.total_width(),
            self.effective_header_height() + self.body_height(rows),
        )
    }

    /// Computes the maximum vertical body scroll offset for this table.
    #[must_use]
    pub fn max_scroll_offset(&self, rows: usize, viewport_height: f32) -> f32 {
        self.effective_row_height().map_or(0.0, |row_height| {
            let body_viewport =
                finite_non_negative(viewport_height) - self.effective_header_height();
            virtual_max_scroll_offset(rows, row_height, body_viewport)
        })
    }

    /// Clamps a vertical body scroll offset to the valid table range.
    #[must_use]
    pub fn clamp_scroll_offset(
        &self,
        rows: usize,
        viewport_height: f32,
        scroll_offset: f32,
    ) -> f32 {
        self.effective_row_height().map_or(0.0, |row_height| {
            let body_viewport =
                finite_non_negative(viewport_height) - self.effective_header_height();
            clamp_virtual_scroll_offset(scroll_offset, rows, row_height, body_viewport)
        })
    }

    /// Computes the virtual window for body rows, excluding the header height from the viewport.
    #[must_use]
    pub fn body_virtual_window(
        &self,
        rows: usize,
        scroll_offset: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> VirtualWindow {
        virtual_window(VirtualWindowRequest {
            item_count: rows,
            scroll_offset,
            viewport_extent: finite_non_negative(viewport_height) - self.effective_header_height(),
            item_extent: self.row_height,
            overscan,
        })
    }

    /// Computes the virtualized body row range for a viewport.
    #[must_use]
    pub fn visible_row_range(
        &self,
        rows: usize,
        scroll_offset: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> Range<usize> {
        self.body_virtual_window(rows, scroll_offset, viewport_height, overscan)
            .materialized_range
    }

    /// Computes header cell rectangles.
    #[must_use]
    pub fn header_rects(&self, bounds: Rect) -> Vec<ItemRect> {
        self.header_cells(bounds)
            .into_iter()
            .map(|cell| ItemRect {
                index: cell.column,
                rect: cell.rect,
            })
            .collect()
    }

    /// Computes header cell rectangles with table-specific metadata.
    #[must_use]
    pub fn header_cells(&self, bounds: Rect) -> Vec<TableHeaderRect> {
        self.header_cells_with_optional_constraints(bounds, None)
    }

    /// Computes constrained header cell rectangles with table-specific metadata.
    #[must_use]
    pub fn header_cells_with_constraints(
        &self,
        bounds: Rect,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> Vec<TableHeaderRect> {
        self.header_cells_with_optional_constraints(bounds, Some(constraints))
    }

    fn header_cells_with_optional_constraints(
        &self,
        bounds: Rect,
        constraints: Option<&BTreeMap<ItemId, TableColumnConstraints>>,
    ) -> Vec<TableHeaderRect> {
        let mut x = finite_coordinate(bounds.x);
        let y = finite_coordinate(bounds.y);
        self.columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let width = Self::column_width_with_optional_constraints(column, constraints);
                let rect = Rect::new(x, y, width, self.effective_header_height());
                x = finite_sum(x, width);
                TableHeaderRect {
                    column: index,
                    column_id: column.id,
                    rect,
                }
            })
            .collect()
    }

    /// Computes visible table cell rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn cell_rects(&self, bounds: Rect, rows: usize, visible: Range<usize>) -> Vec<ItemRect> {
        self.body_cells(bounds, rows, visible)
            .into_iter()
            .map(|cell| ItemRect {
                index: cell.index,
                rect: cell.rect,
            })
            .collect()
    }

    /// Computes visible table cell rectangles with row and column metadata.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn body_cells(
        &self,
        bounds: Rect,
        rows: usize,
        visible: Range<usize>,
    ) -> Vec<TableCellRect> {
        self.body_cells_with_optional_constraints(bounds, rows, visible, None)
    }

    /// Computes constrained visible table cell rectangles with row and column metadata.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn body_cells_with_constraints(
        &self,
        bounds: Rect,
        rows: usize,
        visible: Range<usize>,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> Vec<TableCellRect> {
        self.body_cells_with_optional_constraints(bounds, rows, visible, Some(constraints))
    }

    #[allow(clippy::cast_precision_loss)]
    fn body_cells_with_optional_constraints(
        &self,
        bounds: Rect,
        rows: usize,
        visible: Range<usize>,
        constraints: Option<&BTreeMap<ItemId, TableColumnConstraints>>,
    ) -> Vec<TableCellRect> {
        let Some(row_height) = self.effective_row_height() else {
            return Vec::new();
        };
        let mut rects = Vec::new();
        for row in visible.take_while(|row| *row < rows) {
            let mut x = finite_coordinate(bounds.x);
            for (column, model) in self.columns.iter().enumerate() {
                let width = Self::column_width_with_optional_constraints(model, constraints);
                rects.push(TableCellRect {
                    row,
                    column,
                    column_id: model.id,
                    index: row
                        .saturating_mul(self.columns.len())
                        .saturating_add(column),
                    rect: Rect::new(
                        x,
                        finite_sum(
                            finite_sum(finite_coordinate(bounds.y), self.effective_header_height()),
                            finite_index_extent(row, row_height),
                        ),
                        width,
                        row_height,
                    ),
                });
                x = finite_sum(x, width);
            }
        }
        rects
    }

    /// Computes visible table body cells in viewport coordinates.
    #[must_use]
    pub fn visible_body_cells(
        &self,
        bounds: Rect,
        rows: usize,
        scroll_offset: f32,
        overscan: usize,
    ) -> Vec<TableCellRect> {
        self.visible_body_cells_with_optional_constraints(
            bounds,
            rows,
            scroll_offset,
            overscan,
            None,
        )
    }

    /// Computes constrained visible table body cells in viewport coordinates.
    #[must_use]
    pub fn visible_body_cells_with_constraints(
        &self,
        bounds: Rect,
        rows: usize,
        scroll_offset: f32,
        overscan: usize,
        constraints: &BTreeMap<ItemId, TableColumnConstraints>,
    ) -> Vec<TableCellRect> {
        self.visible_body_cells_with_optional_constraints(
            bounds,
            rows,
            scroll_offset,
            overscan,
            Some(constraints),
        )
    }

    fn visible_body_cells_with_optional_constraints(
        &self,
        bounds: Rect,
        rows: usize,
        scroll_offset: f32,
        overscan: usize,
        constraints: Option<&BTreeMap<ItemId, TableColumnConstraints>>,
    ) -> Vec<TableCellRect> {
        let clamped_scroll =
            self.clamp_scroll_offset(rows, finite_non_negative(bounds.height), scroll_offset);
        self.body_cells_with_optional_constraints(
            Rect::new(
                finite_coordinate(bounds.x),
                finite_sum(finite_coordinate(bounds.y), -clamped_scroll),
                finite_non_negative(bounds.width),
                finite_non_negative(bounds.height),
            ),
            rows,
            self.visible_row_range(
                rows,
                clamped_scroll,
                finite_non_negative(bounds.height),
                overscan,
            ),
            constraints,
        )
    }
}
