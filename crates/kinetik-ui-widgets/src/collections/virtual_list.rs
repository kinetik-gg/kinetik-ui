use kinetik_ui_core::{
    PointerOrder, PointerTarget, PointerTargetPlan, Rect, Response, ScrollResponse, Size,
    Transform, Vec2, WidgetId,
};

use super::{
    CollectionCursorTarget, CollectionProjectedItem, CollectionProjection, ItemId, ListLayout,
    VirtualWindow,
};

/// Selection behavior used by a public virtual list.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VirtualListSelectionMode {
    /// Plain selection only; modifier keys do not retain other rows.
    #[default]
    Single,
    /// Control/Super toggles and Shift extends from the retained anchor.
    Multiple,
}

/// Configuration for one fixed-height virtual list.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualListConfig {
    /// Visible list viewport in logical coordinates.
    pub bounds: Rect,
    /// Fixed-height row layout.
    pub layout: ListLayout,
    /// Extra rows materialized before and after the strict visible range.
    pub overscan: usize,
    /// Accessible list name.
    pub label: String,
    /// Whether scrolling and row interaction are disabled.
    pub disabled: bool,
    /// Selection behavior for pointer and keyboard movement.
    pub selection_mode: VirtualListSelectionMode,
}

impl VirtualListConfig {
    /// Creates an enabled single-selection list with one overscan row.
    #[must_use]
    pub fn new(bounds: Rect, row_height: f32) -> Self {
        Self {
            bounds,
            layout: ListLayout::new(row_height),
            overscan: 1,
            label: "List".to_owned(),
            disabled: false,
            selection_mode: VirtualListSelectionMode::Single,
        }
    }

    /// Sets the accessible list name.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the number of rows materialized around the strict visible range.
    #[must_use]
    pub const fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    /// Sets whether list interaction is disabled.
    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets pointer and keyboard selection behavior.
    #[must_use]
    pub const fn selection_mode(mut self, selection_mode: VirtualListSelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }
}

/// Presentation returned by the callback for one materialized list row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualListRow {
    /// Visible and accessible row label.
    pub label: String,
}

impl VirtualListRow {
    /// Creates a materialized row presentation.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

/// Interaction response for one materialized virtual-list item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtualListItemResponse {
    /// Stable item identity.
    pub id: ItemId,
    /// Shared interaction response for the row.
    pub response: Response,
}

/// Output from one [`crate::Ui::virtual_list`] evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct VirtualListOutput {
    /// Scroll behavior result. Geometry for this frame remains frozen to
    /// [`Self::window`]; an applied wheel delta affects the next frame.
    pub scroll: ScrollResponse,
    /// Prepared virtual window shared by input, painting, and semantics.
    pub window: VirtualWindow,
    /// Item activated by double-click, Enter, or Space.
    pub activated: Option<ItemId>,
    /// Whether caller-owned selection changed this frame.
    pub selection_changed: bool,
    /// Last cursor target produced by reconciliation, navigation, or clicking.
    pub cursor_target: Option<CollectionCursorTarget>,
    /// Responses for materialized rows in projected order.
    pub responses: Vec<VirtualListItemResponse>,
}

/// Prepared fixed-height virtual-list frame.
///
/// Prepare this snapshot before resolving the frame pointer plan, then use the
/// same snapshot for pointer declaration and [`crate::Ui::virtual_list`].
#[derive(Debug)]
pub struct VirtualList<'a> {
    root: WidgetId,
    config: VirtualListConfig,
    projection: &'a CollectionProjection,
    window: VirtualWindow,
    rows: Vec<VirtualListProjectedRow>,
    content_height: f32,
}

impl<'a> VirtualList<'a> {
    pub(crate) fn prepare(
        root: WidgetId,
        config: VirtualListConfig,
        projection: &'a CollectionProjection,
        retained_scroll_offset: f32,
    ) -> Option<Self> {
        valid_viewport(config.bounds)?;
        let row_height = config.layout.effective_row_height()?;
        let window = config.layout.virtual_window(
            projection.len(),
            retained_scroll_offset,
            config.bounds.height,
            config.overscan,
        );
        let content_height = config.layout.content_height(projection.len());
        let content_bounds = Rect::new(
            config.bounds.x,
            config.bounds.y,
            config.bounds.width,
            content_height,
        );
        let rows = window
            .materialized_range
            .clone()
            .filter_map(|projected_index| {
                let item = projection.get(projected_index)?;
                let rect = config.layout.row_rect(content_bounds, projected_index)?;
                Some(VirtualListProjectedRow {
                    id: row_widget_id(root, item),
                    item,
                    rect: Rect::new(rect.x, rect.y, rect.width, row_height),
                })
            })
            .collect();

        Some(Self {
            root,
            config,
            projection,
            window,
            rows,
            content_height,
        })
    }

    /// Returns the stable widget ID for the list surface and scroll owner.
    #[must_use]
    pub const fn widget_id(&self) -> WidgetId {
        self.root
    }

    /// Returns the stable widget ID for one projected item.
    #[must_use]
    pub fn row_widget_id(&self, id: super::ItemId) -> WidgetId {
        self.root.child(("virtual-list-row", id.raw()))
    }

    /// Returns the fixed virtual window used by pointer, paint, and semantics.
    #[must_use]
    pub const fn window(&self) -> &VirtualWindow {
        &self.window
    }

    /// Adds the viewport blocker, wheel owner, and clipped materialized rows to
    /// one caller-owned frame pointer plan.
    ///
    /// The returned order is the first unused ordinal after this list.
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
        plan.with_clip(self.config.bounds, |plan| {
            plan.with_transform(
                Transform::translation(Vec2::new(0.0, -self.window.clamped_scroll_offset)),
                |plan| {
                    if !self.config.disabled {
                        for row in &self.rows {
                            plan.target(PointerTarget::new(
                                row.id,
                                row.rect,
                                take_order(&mut ordinal),
                            ));
                        }
                    }
                },
            );
        });
        PointerOrder::new(ordinal)
    }

    pub(crate) const fn config(&self) -> &VirtualListConfig {
        &self.config
    }

    pub(crate) const fn projection(&self) -> &'a CollectionProjection {
        self.projection
    }

    pub(crate) fn rows(&self) -> &[VirtualListProjectedRow] {
        &self.rows
    }

    pub(crate) fn contains_materialized(&self, id: super::ItemId) -> bool {
        self.rows.iter().any(|row| row.item.id == id)
    }

    pub(crate) fn row_is_visible(&self, row: &VirtualListProjectedRow) -> bool {
        let screen_rect = Rect::new(
            row.rect.x,
            row.rect.y - self.window.clamped_scroll_offset,
            row.rect.width,
            row.rect.height,
        );
        screen_rect.intersection(self.config.bounds).is_some()
    }

    pub(crate) fn content_size(&self) -> Size {
        Size::new(
            self.config.bounds.width,
            self.content_height.max(self.config.bounds.height),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VirtualListProjectedRow {
    pub(crate) id: WidgetId,
    pub(crate) item: CollectionProjectedItem,
    pub(crate) rect: Rect,
}

fn row_widget_id(root: WidgetId, item: CollectionProjectedItem) -> WidgetId {
    root.child(("virtual-list-row", item.id.raw()))
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

fn take_order(ordinal: &mut u64) -> PointerOrder {
    let order = PointerOrder::new(*ordinal);
    *ordinal = ordinal.saturating_add(1);
    order
}
