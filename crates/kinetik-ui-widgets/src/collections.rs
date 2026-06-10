//! Collection models for lists, grids, tables, virtualization, and selection.

use std::collections::BTreeSet;
use std::ops::Range;

use kinetik_ui_core::{Rect, Size};

/// Stable collection item identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(u64);

impl ItemId {
    /// Creates an item ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}

/// Rectangle assigned to an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemRect {
    /// Item index.
    pub index: usize,
    /// Item rectangle.
    pub rect: Rect,
}

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

    /// Computes row rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn row_rects(self, bounds: Rect, rows: usize, visible: Range<usize>) -> Vec<ItemRect> {
        visible
            .take_while(|index| *index < rows)
            .map(|index| ItemRect {
                index,
                rect: Rect::new(
                    bounds.x,
                    bounds.y + index as f32 * self.row_height,
                    bounds.width,
                    self.row_height,
                ),
            })
            .collect()
    }
}

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
    /// Resolves the number of columns for bounds.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn column_count(self, bounds: Rect) -> usize {
        match self.columns {
            GridColumns::Fixed(count) => count.max(1),
            GridColumns::Adaptive { min_width } => ((bounds.width + self.gap)
                / (min_width + self.gap))
                .floor()
                .max(1.0) as usize,
        }
    }

    /// Computes grid item rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn item_rects(self, bounds: Rect, count: usize, visible: Range<usize>) -> Vec<ItemRect> {
        let columns = self.column_count(bounds);
        visible
            .take_while(|index| *index < count)
            .map(|index| {
                let column = index % columns;
                let row = index / columns;
                ItemRect {
                    index,
                    rect: Rect::new(
                        bounds.x + column as f32 * (self.item_size.width + self.gap),
                        bounds.y + row as f32 * (self.item_size.height + self.gap),
                        self.item_size.width,
                        self.item_size.height,
                    ),
                }
            })
            .collect()
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
    /// Computes header cell rectangles.
    #[must_use]
    pub fn header_rects(&self, bounds: Rect) -> Vec<ItemRect> {
        let mut x = bounds.x;
        self.columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let rect = Rect::new(x, bounds.y, column.width, self.header_height);
                x += column.width;
                ItemRect { index, rect }
            })
            .collect()
    }

    /// Computes visible table cell rectangles.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn cell_rects(&self, bounds: Rect, rows: usize, visible: Range<usize>) -> Vec<ItemRect> {
        let mut rects = Vec::new();
        for row in visible.take_while(|row| *row < rows) {
            let mut x = bounds.x;
            for (column, model) in self.columns.iter().enumerate() {
                rects.push(ItemRect {
                    index: row * self.columns.len() + column,
                    rect: Rect::new(
                        x,
                        bounds.y + self.header_height + row as f32 * self.row_height,
                        model.width,
                        self.row_height,
                    ),
                });
                x += model.width;
            }
        }
        rects
    }
}

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

/// Computes a visible item range.
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn virtual_range(request: VirtualRangeRequest) -> Range<usize> {
    if request.item_count == 0 || request.item_extent <= 0.0 || request.viewport_extent <= 0.0 {
        return 0..0;
    }
    let first = (request.scroll_offset.max(0.0) / request.item_extent).floor() as usize;
    let visible = (request.viewport_extent / request.item_extent).ceil() as usize + 1;
    let start = first.saturating_sub(request.overscan);
    let end = first
        .saturating_add(visible)
        .saturating_add(request.overscan)
        .min(request.item_count);
    start..end
}

/// Shared multi-selection state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    selected: BTreeSet<ItemId>,
    /// Active item.
    pub active: Option<ItemId>,
    anchor: Option<ItemId>,
}

impl Selection {
    /// Creates an empty selection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when an item is selected.
    #[must_use]
    pub fn contains(&self, item: ItemId) -> bool {
        self.selected.contains(&item)
    }

    /// Returns selected items in sorted order.
    #[must_use]
    pub fn selected(&self) -> Vec<ItemId> {
        self.selected.iter().copied().collect()
    }

    /// Clears selection.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.active = None;
        self.anchor = None;
    }

    /// Replaces selection with one item.
    pub fn replace(&mut self, item: ItemId) {
        self.selected.clear();
        self.selected.insert(item);
        self.active = Some(item);
        self.anchor = Some(item);
    }

    /// Toggles an item.
    pub fn toggle(&mut self, item: ItemId) {
        if !self.selected.remove(&item) {
            self.selected.insert(item);
        }
        self.active = Some(item);
        self.anchor = Some(item);
    }

    /// Selects a range using the current anchor or the provided end as anchor.
    pub fn select_range(&mut self, ordered_items: &[ItemId], end: ItemId) -> bool {
        let anchor = self.anchor.unwrap_or(end);
        let Some(anchor_index) = ordered_items.iter().position(|item| *item == anchor) else {
            return false;
        };
        let Some(end_index) = ordered_items.iter().position(|item| *item == end) else {
            return false;
        };
        let range = anchor_index.min(end_index)..=anchor_index.max(end_index);
        self.selected.clear();
        self.selected.extend(ordered_items[range].iter().copied());
        self.active = Some(end);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GridColumns, GridLayout, ItemId, ListLayout, Selection, SortDirection, TableColumn,
        TableLayout, TableSort, VirtualRangeRequest, virtual_range,
    };
    use kinetik_ui_core::{Rect, Size};

    #[test]
    fn list_layout_computes_row_rectangles() {
        let rows = ListLayout::new(20.0).row_rects(Rect::new(0.0, 0.0, 100.0, 200.0), 10, 2..5);

        assert_eq!(rows.len(), 3);
        assert!((rows[0].rect.y - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn grid_layout_supports_fixed_columns() {
        let grid = GridLayout {
            columns: GridColumns::Fixed(2),
            item_size: Size::new(10.0, 10.0),
            gap: 2.0,
        };
        let items = grid.item_rects(Rect::new(0.0, 0.0, 100.0, 100.0), 4, 0..4);

        assert!((items[2].rect.y - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn grid_layout_supports_adaptive_columns() {
        let grid = GridLayout {
            columns: GridColumns::Adaptive { min_width: 20.0 },
            item_size: Size::new(20.0, 20.0),
            gap: 5.0,
        };

        assert_eq!(grid.column_count(Rect::new(0.0, 0.0, 75.0, 100.0)), 3);
    }

    #[test]
    fn table_layout_computes_header_and_cell_rectangles() {
        let table = TableLayout {
            columns: vec![
                TableColumn {
                    id: ItemId::from_raw(1),
                    header: "Name".to_owned(),
                    width: 100.0,
                },
                TableColumn {
                    id: ItemId::from_raw(2),
                    header: "Kind".to_owned(),
                    width: 50.0,
                },
            ],
            header_height: 24.0,
            row_height: 18.0,
            sort: Some(TableSort {
                column: ItemId::from_raw(1),
                direction: SortDirection::Ascending,
            }),
        };

        assert_eq!(
            table.header_rects(Rect::new(0.0, 0.0, 200.0, 200.0)).len(),
            2
        );
        assert_eq!(
            table
                .cell_rects(Rect::new(0.0, 0.0, 200.0, 200.0), 2, 0..2)
                .len(),
            4
        );
    }

    #[test]
    fn virtual_range_applies_overscan_and_bounds() {
        let range = virtual_range(VirtualRangeRequest {
            item_count: 100,
            scroll_offset: 50.0,
            viewport_extent: 40.0,
            item_extent: 10.0,
            overscan: 2,
        });

        assert_eq!(range, 3..12);
    }

    #[test]
    fn virtual_range_handles_empty_inputs() {
        assert_eq!(
            virtual_range(VirtualRangeRequest {
                item_count: 0,
                scroll_offset: 0.0,
                viewport_extent: 100.0,
                item_extent: 20.0,
                overscan: 1,
            }),
            0..0
        );
    }

    #[test]
    fn selection_supports_replace_toggle_clear() {
        let mut selection = Selection::new();
        let one = ItemId::from_raw(1);

        selection.replace(one);
        assert!(selection.contains(one));
        selection.toggle(one);
        assert!(!selection.contains(one));
        selection.clear();
        assert!(selection.selected().is_empty());
    }

    #[test]
    fn selection_supports_ranges_from_anchor() {
        let items = [
            ItemId::from_raw(1),
            ItemId::from_raw(2),
            ItemId::from_raw(3),
            ItemId::from_raw(4),
        ];
        let mut selection = Selection::new();

        selection.replace(ItemId::from_raw(2));
        assert!(selection.select_range(&items, ItemId::from_raw(4)));

        assert_eq!(
            selection.selected(),
            vec![
                ItemId::from_raw(2),
                ItemId::from_raw(3),
                ItemId::from_raw(4)
            ]
        );
    }
}
