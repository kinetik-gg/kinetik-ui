use std::collections::{BTreeMap, BTreeSet};

use stern_core::{
    PointerOrder, PointerTarget, PointerTargetPlan, Rect, Response, ScrollResponse, Size,
    Transform, Vec2, WidgetId, clamp_scroll_offset,
};

use super::{
    CollectionProjectedItem, CollectionProjection, ItemId, TableCellRect, TableColumn,
    TableColumnConstraints, TableHeaderRect, TableLayout, TableSort, VirtualWindow,
};

const RESIZE_HANDLE_WIDTH: f32 = 6.0;
const MIN_RESIZABLE_COLUMN_WIDTH: f32 = 1.0;

/// Configuration for one prepared fixed-height virtual table.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualTableConfig {
    /// Visible table viewport in logical coordinates.
    pub bounds: Rect,
    /// Header, column, row-height, and current sort state.
    pub layout: TableLayout,
    /// Stable-column width constraints applied to header and body geometry.
    pub column_constraints: BTreeMap<ItemId, TableColumnConstraints>,
    /// Extra body rows materialized before and after the strict visible range.
    pub overscan: usize,
    /// Accessible table name.
    pub label: String,
    /// Whether scroll, sort, selection, focus, and resize interaction are disabled.
    pub disabled: bool,
    /// Whether rows or individual cells own selection and keyboard focus.
    pub selection_mode: VirtualTableSelectionMode,
    /// Whether stable column-edge drag handles emit resize requests.
    ///
    /// Resizable columns must be at least one logical pixel wide and emitted
    /// deltas preserve that floor.
    pub resizable: bool,
}

impl VirtualTableConfig {
    /// Creates an enabled table with one overscan row.
    #[must_use]
    pub fn new(bounds: Rect, layout: TableLayout) -> Self {
        Self {
            bounds,
            layout,
            column_constraints: BTreeMap::new(),
            overscan: 1,
            label: "Table".to_owned(),
            disabled: false,
            selection_mode: VirtualTableSelectionMode::Row,
            resizable: true,
        }
    }

    /// Sets the accessible table name.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets stable-column width constraints.
    #[must_use]
    pub fn column_constraints(
        mut self,
        constraints: impl IntoIterator<Item = (ItemId, TableColumnConstraints)>,
    ) -> Self {
        self.column_constraints = constraints.into_iter().collect();
        self
    }

    /// Sets the number of body rows materialized around the strict window.
    #[must_use]
    pub const fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Sets whether table interaction is disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets whether rows or individual cells own selection and keyboard focus.
    #[must_use]
    pub const fn selection_mode(mut self, selection_mode: VirtualTableSelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }

    /// Sets whether column-edge drag handles emit resize requests.
    ///
    /// Enabling resizing requires every effective column width to be at least
    /// one logical pixel; emitted deltas never shrink below that floor.
    #[must_use]
    pub const fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

/// Single-selection behavior for a public virtual table.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VirtualTableSelectionMode {
    /// Rows own pointer selection, keyboard focus, and vertical traversal.
    #[default]
    Row,
    /// Cells own pointer selection, keyboard focus, and two-dimensional traversal.
    Cell,
}

/// One stable row or cell selected by a virtual table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualTableTarget {
    /// A whole row identified independently of projection order.
    Row(ItemId),
    /// One cell identified independently of row and column order.
    Cell {
        /// Stable row identity.
        row: ItemId,
        /// Stable column identity.
        column: ItemId,
    },
}

impl VirtualTableTarget {
    pub(crate) const fn row(self) -> ItemId {
        match self {
            Self::Row(row) | Self::Cell { row, .. } => row,
        }
    }
}

/// Resolved stable table cursor target in the current projection and layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualTableCursorTarget {
    /// Stable selected row or cell.
    pub target: VirtualTableTarget,
    /// Row index in the current collection projection.
    pub projected_row: usize,
    /// Column index for cell selection, or `None` for row selection.
    pub column: Option<usize>,
}

/// Retained single row-or-cell selection for a virtual table.
///
/// Stable identities survive projection and column reorder. When an active row
/// or column disappears, reconciliation repairs to its prior slot and then the
/// preceding tail.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VirtualTableSelection {
    target: Option<VirtualTableTarget>,
    last_projected_row: Option<usize>,
    last_column: Option<usize>,
}

impl VirtualTableSelection {
    /// Creates an empty retained table selection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            target: None,
            last_projected_row: None,
            last_column: None,
        }
    }

    /// Returns the active stable row or cell.
    #[must_use]
    pub const fn target(&self) -> Option<VirtualTableTarget> {
        self.target
    }

    pub(crate) const fn last_projected_row(&self) -> Option<usize> {
        self.last_projected_row
    }

    pub(crate) const fn last_column(&self) -> Option<usize> {
        self.last_column
    }

    /// Clears the retained selection and repair positions.
    pub fn clear(&mut self) {
        self.target = None;
        self.last_projected_row = None;
        self.last_column = None;
    }

    pub(crate) fn activate(
        &mut self,
        projection: &CollectionProjection,
        columns: &[TableColumn],
        target: VirtualTableTarget,
        mode: VirtualTableSelectionMode,
    ) -> Option<VirtualTableCursorTarget> {
        let projected_row = projection.projected_index(target.row())?;
        let column = match (mode, target) {
            (VirtualTableSelectionMode::Row, _) => None,
            (VirtualTableSelectionMode::Cell, VirtualTableTarget::Cell { column, .. }) => Some(
                columns
                    .iter()
                    .position(|candidate| candidate.id == column)?,
            ),
            (VirtualTableSelectionMode::Cell, VirtualTableTarget::Row(_)) => return None,
        };
        self.set_indices(projection, columns, projected_row, column, mode)
    }

    pub(crate) fn reconcile(
        &mut self,
        projection: &CollectionProjection,
        columns: &[TableColumn],
        mode: VirtualTableSelectionMode,
    ) -> Option<VirtualTableCursorTarget> {
        if projection.is_empty() || columns.is_empty() {
            self.clear();
            return None;
        }
        let active = self.target?;
        let projected_row = projection.projected_index(active.row()).unwrap_or_else(|| {
            self.last_projected_row
                .unwrap_or_default()
                .min(projection.len() - 1)
        });
        let column = match mode {
            VirtualTableSelectionMode::Row => None,
            VirtualTableSelectionMode::Cell => Some(match active {
                VirtualTableTarget::Cell { column, .. } => columns
                    .iter()
                    .position(|candidate| candidate.id == column)
                    .unwrap_or_else(|| self.last_column.unwrap_or_default().min(columns.len() - 1)),
                VirtualTableTarget::Row(_) => {
                    self.last_column.unwrap_or_default().min(columns.len() - 1)
                }
            }),
        };
        self.set_indices(projection, columns, projected_row, column, mode)
    }

    pub(crate) fn navigate(
        &mut self,
        projection: &CollectionProjection,
        columns: &[TableColumn],
        mode: VirtualTableSelectionMode,
        movement: VirtualTableCursorMove,
    ) -> Option<VirtualTableCursorTarget> {
        let current = self.reconcile(projection, columns, mode)?;
        let last_row = projection.len() - 1;
        let projected_row = match movement {
            VirtualTableCursorMove::FirstRow => 0,
            VirtualTableCursorMove::PreviousRow => current.projected_row.saturating_sub(1),
            VirtualTableCursorMove::NextRow => {
                current.projected_row.saturating_add(1).min(last_row)
            }
            VirtualTableCursorMove::LastRow => last_row,
            VirtualTableCursorMove::PagePrevious { rows } => {
                current.projected_row.saturating_sub(rows)
            }
            VirtualTableCursorMove::PageNext { rows } => {
                current.projected_row.saturating_add(rows).min(last_row)
            }
            VirtualTableCursorMove::FirstColumn
            | VirtualTableCursorMove::PreviousColumn
            | VirtualTableCursorMove::NextColumn
            | VirtualTableCursorMove::LastColumn => current.projected_row,
        };
        let column = current.column.map(|column| match movement {
            VirtualTableCursorMove::FirstColumn => 0,
            VirtualTableCursorMove::PreviousColumn => column.saturating_sub(1),
            VirtualTableCursorMove::NextColumn => column.saturating_add(1).min(columns.len() - 1),
            VirtualTableCursorMove::LastColumn => columns.len() - 1,
            _ => column,
        });
        self.set_indices(projection, columns, projected_row, column, mode)
    }

    fn set_indices(
        &mut self,
        projection: &CollectionProjection,
        columns: &[TableColumn],
        projected_row: usize,
        column: Option<usize>,
        mode: VirtualTableSelectionMode,
    ) -> Option<VirtualTableCursorTarget> {
        let row = projection.get(projected_row)?.id;
        let (target, column) = match mode {
            VirtualTableSelectionMode::Row => (VirtualTableTarget::Row(row), None),
            VirtualTableSelectionMode::Cell => {
                let column = column?.min(columns.len().checked_sub(1)?);
                (
                    VirtualTableTarget::Cell {
                        row,
                        column: columns.get(column)?.id,
                    },
                    Some(column),
                )
            }
        };
        self.target = Some(target);
        self.last_projected_row = Some(projected_row);
        self.last_column = column;
        Some(VirtualTableCursorTarget {
            target,
            projected_row,
            column,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VirtualTableCursorMove {
    FirstRow,
    PreviousRow,
    NextRow,
    LastRow,
    PagePrevious { rows: usize },
    PageNext { rows: usize },
    FirstColumn,
    PreviousColumn,
    NextColumn,
    LastColumn,
}

/// Presentation returned by the callback for one materialized table row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualTableRow {
    /// Cell labels in table-column order. Missing labels render as empty cells.
    pub cells: Vec<String>,
}

impl VirtualTableRow {
    /// Creates one materialized row presentation.
    #[must_use]
    pub fn new(cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
        }
    }
}

/// Frozen two-axis table window.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualTableWindow {
    /// Retained offset used by current-frame pointer, paint, and semantics.
    pub offset: Vec2,
    /// Strict and materialized vertical body-row ranges.
    pub body: VirtualWindow,
}

/// Header interaction response for one stable column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualTableHeaderResponse {
    /// Stable column identity.
    pub column: ItemId,
    /// Shared header interaction response.
    pub response: Response,
    /// Column-edge drag response when resizing is enabled.
    pub resize_response: Option<Response>,
}

/// One application-owned constrained column resize request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumnResizeRequest {
    /// Stable column identity.
    pub column: ItemId,
    /// Width delta already clamped against the prepared column constraints.
    pub delta: f32,
}

/// Interaction response for one materialized selectable row or cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualTableSelectionResponse {
    /// Stable row or cell target.
    pub target: VirtualTableTarget,
    /// Shared captured-selection response.
    pub response: Response,
}

/// Metadata for one materialized body row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualTableMaterializedRow {
    /// Stable row identity.
    pub id: ItemId,
    /// Row index in the current collection projection.
    pub projected_index: usize,
}

/// Output from one [`crate::Ui::virtual_table`] evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualTableOutput {
    /// Two-axis scroll behavior result. Applied deltas affect the next frame.
    pub scroll: ScrollResponse,
    /// Frozen current-frame offset and vertical row window.
    pub window: VirtualTableWindow,
    /// Application-owned sort state requested by a header click.
    pub sort_requested: Option<TableSort>,
    /// Application-owned constrained column resize request.
    pub resize_requested: Option<TableColumnResizeRequest>,
    /// Whether caller-owned retained selection changed this frame.
    pub selection_changed: bool,
    /// Last cursor target produced by reconciliation, navigation, or clicking.
    pub cursor_target: Option<VirtualTableCursorTarget>,
    /// Header responses in column order.
    pub headers: Vec<VirtualTableHeaderResponse>,
    /// Selectable materialized row or cell responses in projection order.
    pub selection_responses: Vec<VirtualTableSelectionResponse>,
    /// Materialized body rows in projected order.
    pub rows: Vec<VirtualTableMaterializedRow>,
}

/// Prepared fixed-height virtual-table frame.
///
/// Prepare this snapshot before resolving the frame pointer plan, then share
/// it with pointer declaration and [`crate::Ui::virtual_table`].
#[derive(Debug)]
pub struct VirtualTable<'a> {
    root: WidgetId,
    config: VirtualTableConfig,
    projection: &'a CollectionProjection,
    window: VirtualTableWindow,
    headers: Vec<VirtualTableProjectedHeader>,
    resize_handles: Vec<VirtualTableProjectedResizeHandle>,
    rows: Vec<VirtualTableProjectedRow>,
    header_clip: Rect,
    body_clip: Rect,
    content_size: Size,
    total_width: f32,
}

impl<'a> VirtualTable<'a> {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn prepare(
        root: WidgetId,
        config: VirtualTableConfig,
        projection: &'a CollectionProjection,
        retained_scroll_offset: Vec2,
    ) -> Option<Self> {
        valid_viewport(config.bounds)?;
        let header_height = valid_header_height(&config)?;
        let row_height = config.layout.effective_row_height()?;
        validate_columns(&config)?;
        let total_width = config
            .layout
            .total_width_with_constraints(&config.column_constraints);
        if !total_width.is_finite() || total_width <= 0.0 {
            return None;
        }

        let content_size = Size::new(
            total_width.max(config.bounds.width),
            (header_height + config.layout.body_height(projection.len())).max(config.bounds.height),
        );
        let offset = clamp_scroll_offset(
            retained_scroll_offset,
            Size::new(config.bounds.width, config.bounds.height),
            content_size,
        );
        let body = config.layout.body_virtual_window(
            projection.len(),
            offset.y,
            config.bounds.height,
            config.overscan,
        );
        let window = VirtualTableWindow { offset, body };
        let header_clip = Rect::new(
            config.bounds.x,
            config.bounds.y,
            config.bounds.width,
            header_height,
        );
        let body_clip = Rect::new(
            config.bounds.x,
            config.bounds.y + header_height,
            config.bounds.width,
            config.bounds.height - header_height,
        );
        let headers = config
            .layout
            .header_cells_with_constraints(config.bounds, &config.column_constraints)
            .into_iter()
            .map(|cell| VirtualTableProjectedHeader {
                id: header_widget_id(root, cell.column_id),
                cell,
            })
            .collect::<Vec<_>>();
        let resize_handles = headers
            .iter()
            .map(|header| {
                let width = RESIZE_HANDLE_WIDTH.min(header.cell.rect.width);
                VirtualTableProjectedResizeHandle {
                    id: resize_widget_id(root, header.cell.column_id),
                    column: header.cell.column_id,
                    rect: Rect::new(
                        header.cell.rect.max_x() - width * 0.5,
                        header.cell.rect.y,
                        width,
                        header.cell.rect.height,
                    ),
                }
            })
            .collect();
        let cells = config.layout.body_cells_with_constraints(
            config.bounds,
            projection.len(),
            window.body.materialized_range.clone(),
            &config.column_constraints,
        );
        let rows = window
            .body
            .materialized_range
            .clone()
            .filter_map(|projected_index| {
                let item = projection.get(projected_index)?;
                let row_cells = cells
                    .iter()
                    .copied()
                    .filter(|cell| cell.row == projected_index)
                    .map(|cell| VirtualTableProjectedCell {
                        id: cell_widget_id(root, item.id, cell.column_id),
                        cell,
                    })
                    .collect::<Vec<_>>();
                let row_y = row_cells.first()?.cell.rect.y;
                let rect = Rect::new(config.bounds.x, row_y, total_width, row_height);
                Some(VirtualTableProjectedRow {
                    id: row_widget_id(root, item.id),
                    item,
                    projected_index,
                    rect,
                    cells: row_cells,
                })
            })
            .collect();

        Some(Self {
            root,
            config,
            projection,
            window,
            headers,
            resize_handles,
            rows,
            header_clip,
            body_clip,
            content_size,
            total_width,
        })
    }

    /// Returns the stable table surface and scroll-owner ID.
    #[must_use]
    pub const fn widget_id(&self) -> WidgetId {
        self.root
    }

    /// Returns the stable semantic header-row ID.
    #[must_use]
    pub fn header_row_widget_id(&self) -> WidgetId {
        self.root.child("virtual-table-header-row")
    }

    /// Returns a stable header ID derived from column identity.
    #[must_use]
    pub fn header_widget_id(&self, column: ItemId) -> WidgetId {
        header_widget_id(self.root, column)
    }

    /// Returns a stable body-row ID derived from row identity.
    #[must_use]
    pub fn row_widget_id(&self, row: ItemId) -> WidgetId {
        row_widget_id(self.root, row)
    }

    /// Returns a stable cell ID derived from row and column identities.
    #[must_use]
    pub fn cell_widget_id(&self, row: ItemId, column: ItemId) -> WidgetId {
        cell_widget_id(self.root, row, column)
    }

    /// Returns a stable column-resize handle ID derived from column identity.
    #[must_use]
    pub fn resize_widget_id(&self, column: ItemId) -> WidgetId {
        resize_widget_id(self.root, column)
    }

    /// Returns the frozen current-frame two-axis window.
    #[must_use]
    pub const fn window(&self) -> &VirtualTableWindow {
        &self.window
    }

    /// Adds the table blocker, wheel owner, headers, resize handles, and body
    /// selection targets to one caller-owned pointer plan.
    pub fn declare_pointer_targets(
        &self,
        plan: &mut PointerTargetPlan,
        first_order: PointerOrder,
    ) -> PointerOrder {
        let mut ordinal = first_order.raw();
        plan.blocker(self.config.bounds, take_order(&mut ordinal));
        plan.target(PointerTarget::wheel_only(
            self.root,
            self.config.bounds,
            take_order(&mut ordinal),
        ));
        plan.with_clip(self.header_clip, |plan| {
            plan.with_transform(
                Transform::translation(Vec2::new(-self.window.offset.x, 0.0)),
                |plan| {
                    if !self.config.disabled {
                        for header in &self.headers {
                            plan.target(PointerTarget::new(
                                header.id,
                                header.cell.rect,
                                take_order(&mut ordinal),
                            ));
                        }
                        if self.config.resizable {
                            for handle in &self.resize_handles {
                                plan.target(
                                    PointerTarget::new(
                                        handle.id,
                                        handle.rect,
                                        take_order(&mut ordinal),
                                    )
                                    .domain_drag_source(),
                                );
                            }
                        }
                    }
                },
            );
        });
        plan.with_clip(self.body_clip, |plan| {
            plan.with_transform(
                Transform::translation(Vec2::new(-self.window.offset.x, -self.window.offset.y)),
                |plan| {
                    if !self.config.disabled {
                        for row in &self.rows {
                            match self.config.selection_mode {
                                VirtualTableSelectionMode::Row => plan.target(PointerTarget::new(
                                    row.id,
                                    row.rect,
                                    take_order(&mut ordinal),
                                )),
                                VirtualTableSelectionMode::Cell => {
                                    for cell in &row.cells {
                                        plan.target(PointerTarget::new(
                                            cell.id,
                                            cell.cell.rect,
                                            take_order(&mut ordinal),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                },
            );
        });
        PointerOrder::new(ordinal)
    }

    pub(crate) const fn config(&self) -> &VirtualTableConfig {
        &self.config
    }

    /// Returns the frozen row projection used by this table frame.
    #[must_use]
    pub const fn projection(&self) -> &'a CollectionProjection {
        self.projection
    }

    pub(crate) fn headers(&self) -> &[VirtualTableProjectedHeader] {
        &self.headers
    }

    pub(crate) fn resize_handles(&self) -> &[VirtualTableProjectedResizeHandle] {
        &self.resize_handles
    }

    pub(crate) fn rows(&self) -> &[VirtualTableProjectedRow] {
        &self.rows
    }

    pub(crate) const fn header_clip(&self) -> Rect {
        self.header_clip
    }

    pub(crate) const fn body_clip(&self) -> Rect {
        self.body_clip
    }

    pub(crate) const fn content_size(&self) -> Size {
        self.content_size
    }

    pub(crate) const fn total_width(&self) -> f32 {
        self.total_width
    }

    pub(crate) fn target_widget_id(&self, target: VirtualTableTarget) -> WidgetId {
        match target {
            VirtualTableTarget::Row(row) => self.row_widget_id(row),
            VirtualTableTarget::Cell { row, column } => self.cell_widget_id(row, column),
        }
    }

    pub(crate) fn contains_materialized_target(&self, target: VirtualTableTarget) -> bool {
        self.rows.iter().any(|row| match target {
            VirtualTableTarget::Row(id) => row.item.id == id,
            VirtualTableTarget::Cell {
                row: row_id,
                column,
            } => {
                row.item.id == row_id && row.cells.iter().any(|cell| cell.cell.column_id == column)
            }
        })
    }

    pub(crate) fn revealed_offset(&self, target: VirtualTableCursorTarget) -> Vec2 {
        let Some(rect) = self.target_rect(target) else {
            return self.window.offset;
        };
        let mut desired = self.window.offset;
        if matches!(target.target, VirtualTableTarget::Cell { .. }) {
            desired.x = if rect.x < self.config.bounds.x + desired.x {
                rect.x - self.config.bounds.x
            } else if rect.max_x() > self.config.bounds.max_x() + desired.x {
                rect.max_x() - self.config.bounds.max_x()
            } else {
                desired.x
            };
        }
        desired.y = if rect.y < self.body_clip.y + desired.y {
            rect.y - self.body_clip.y
        } else if rect.max_y() > self.body_clip.max_y() + desired.y {
            rect.max_y() - self.body_clip.max_y()
        } else {
            desired.y
        };
        clamp_scroll_offset(
            desired,
            Size::new(self.config.bounds.width, self.config.bounds.height),
            self.content_size,
        )
    }

    pub(crate) fn constrained_resize_delta(&self, column: ItemId, delta: f32) -> Option<f32> {
        if !delta.is_finite() {
            return None;
        }
        let current = self
            .config
            .layout
            .column_width_with_constraints(column, &self.config.column_constraints)?;
        let candidate = current + delta;
        if !candidate.is_finite() {
            return None;
        }
        let constraints = self
            .config
            .column_constraints
            .get(&column)
            .copied()
            .unwrap_or_default()
            .sanitized();
        let resized = constraints
            .clamp_width(candidate)
            .max(constraints.min_width.max(MIN_RESIZABLE_COLUMN_WIDTH));
        let applied = resized - current;
        (applied.is_finite() && applied != 0.0).then_some(applied)
    }

    fn target_rect(&self, target: VirtualTableCursorTarget) -> Option<Rect> {
        let cells = self.config.layout.body_cells_with_constraints(
            self.config.bounds,
            self.projection.len(),
            target.projected_row..target.projected_row.saturating_add(1),
            &self.config.column_constraints,
        );
        match target.target {
            VirtualTableTarget::Row(_) => {
                let first = cells.first()?;
                Some(Rect::new(
                    self.config.bounds.x,
                    first.rect.y,
                    self.total_width,
                    first.rect.height,
                ))
            }
            VirtualTableTarget::Cell { column, .. } => cells
                .into_iter()
                .find(|cell| cell.column_id == column)
                .map(|cell| cell.rect),
        }
    }

    pub(crate) fn header_is_visible(&self, header: &VirtualTableProjectedHeader) -> bool {
        translated_rect(header.cell.rect, -self.window.offset.x, 0.0)
            .intersection(self.header_clip)
            .is_some()
    }

    pub(crate) fn row_is_visible(&self, row: &VirtualTableProjectedRow) -> bool {
        translated_rect(row.rect, -self.window.offset.x, -self.window.offset.y)
            .intersection(self.body_clip)
            .is_some()
    }

    pub(crate) fn cell_is_visible(&self, cell: &VirtualTableProjectedCell) -> bool {
        translated_rect(cell.cell.rect, -self.window.offset.x, -self.window.offset.y)
            .intersection(self.body_clip)
            .is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VirtualTableProjectedHeader {
    pub(crate) id: WidgetId,
    pub(crate) cell: TableHeaderRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VirtualTableProjectedResizeHandle {
    pub(crate) id: WidgetId,
    pub(crate) column: ItemId,
    pub(crate) rect: Rect,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VirtualTableProjectedRow {
    pub(crate) id: WidgetId,
    pub(crate) item: CollectionProjectedItem,
    pub(crate) projected_index: usize,
    pub(crate) rect: Rect,
    pub(crate) cells: Vec<VirtualTableProjectedCell>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VirtualTableProjectedCell {
    pub(crate) id: WidgetId,
    pub(crate) cell: TableCellRect,
}

fn valid_viewport(rect: Rect) -> Option<Rect> {
    (rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
        && rect.max_x().is_finite()
        && rect.max_y().is_finite())
    .then_some(rect)
}

fn valid_header_height(config: &VirtualTableConfig) -> Option<f32> {
    let header = config.layout.effective_header_height();
    (header.is_finite() && header > 0.0 && header < config.bounds.height).then_some(header)
}

fn validate_columns(config: &VirtualTableConfig) -> Option<()> {
    if config.layout.columns.is_empty() {
        return None;
    }
    let mut ids = BTreeSet::new();
    for column in &config.layout.columns {
        let width = effective_column_width(config, column);
        if !ids.insert(column.id)
            || width <= 0.0
            || (config.resizable && width < MIN_RESIZABLE_COLUMN_WIDTH)
        {
            return None;
        }
    }
    Some(())
}

fn effective_column_width(config: &VirtualTableConfig, column: &TableColumn) -> f32 {
    column.clamped_width(
        config
            .column_constraints
            .get(&column.id)
            .copied()
            .unwrap_or_default(),
    )
}

fn translated_rect(rect: Rect, x: f32, y: f32) -> Rect {
    Rect::new(rect.x + x, rect.y + y, rect.width, rect.height)
}

fn header_widget_id(root: WidgetId, column: ItemId) -> WidgetId {
    root.child(("virtual-table-header", column.raw()))
}

fn row_widget_id(root: WidgetId, row: ItemId) -> WidgetId {
    root.child(("virtual-table-row", row.raw()))
}

fn cell_widget_id(root: WidgetId, row: ItemId, column: ItemId) -> WidgetId {
    root.child(("virtual-table-cell", row.raw(), column.raw()))
}

fn resize_widget_id(root: WidgetId, column: ItemId) -> WidgetId {
    root.child(("virtual-table-resize", column.raw()))
}

fn take_order(ordinal: &mut u64) -> PointerOrder {
    let order = PointerOrder::new(*ordinal);
    *ordinal = ordinal.saturating_add(1);
    order
}
