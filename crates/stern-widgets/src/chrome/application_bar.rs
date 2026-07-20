use super::{MenuBar, MenuBarMenuId};
use stern_core::{PointerOrder, PointerTarget, PointerTargetPlan, Rect, Response, WidgetId};
/// Stable identity for an application workspace tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceTabId(u64);
impl WorkspaceTabId {
    /// Creates an identity from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }
}
/// Passive application-owned workspace presentation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTab {
    /// Stable workspace identity.
    pub id: WorkspaceTabId,
    /// Visible and accessible label.
    pub label: String,
    /// Whether the application currently considers this workspace active.
    pub active: bool,
    /// Whether the workspace can receive focus and activation.
    pub enabled: bool,
}
impl WorkspaceTab {
    /// Creates an enabled workspace tab.
    #[must_use]
    pub fn new(id: WorkspaceTabId, label: impl Into<String>, active: bool) -> Self {
        Self {
            id,
            label: label.into(),
            active,
            enabled: true,
        }
    }
}
/// Stable workspace activation target with current presentation index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkspaceTabTarget {
    /// Stable workspace identity.
    pub id: WorkspaceTabId,
    /// Current source-order index.
    pub index: usize,
}
/// Caller-owned geometry for one application bar.
#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationBarConfig {
    /// Stable root identity.
    pub root: WidgetId,
    /// Application-bar bounds. Its height should use `Theme::sizes.workspace_bar`.
    pub bounds: Rect,
    /// Width assigned to each visible application-menu heading.
    pub menu_width: f32,
    /// Width assigned to each workspace tab.
    pub workspace_width: f32,
}
impl ApplicationBarConfig {
    /// Creates compact default geometry over caller-owned bounds.
    #[must_use]
    pub const fn new(root: WidgetId, bounds: Rect) -> Self {
        Self {
            root,
            bounds,
            menu_width: 64.0,
            workspace_width: 96.0,
        }
    }
}
/// Retained application-bar composition over the existing menu model.
#[derive(Debug, Clone, PartialEq)]
pub struct ApplicationBar {
    /// Caller-owned geometry and stable root.
    pub config: ApplicationBarConfig,
    /// Existing action-backed application-menu model.
    pub menu_bar: MenuBar,
    /// Passive application-owned workspace presentation data.
    pub workspaces: Vec<WorkspaceTab>,
    pub(crate) workspace_focus: Option<WorkspaceTabId>,
}
impl ApplicationBar {
    /// Creates a composition from the existing action-backed menu model.
    #[must_use]
    pub fn new(
        config: ApplicationBarConfig,
        menu_bar: MenuBar,
        workspaces: impl IntoIterator<Item = WorkspaceTab>,
    ) -> Self {
        Self {
            config,
            menu_bar,
            workspaces: workspaces.into_iter().collect(),
            workspace_focus: None,
        }
    }
    /// Returns the stable widget identity for a menu heading.
    #[must_use]
    pub fn menu_widget_id(&self, id: MenuBarMenuId) -> WidgetId {
        self.menu_composite_id().child(("menu", id.raw()))
    }
    /// Returns the stable widget identity for a workspace tab.
    #[must_use]
    pub fn workspace_widget_id(&self, id: WorkspaceTabId) -> WidgetId {
        self.workspace_composite_id().child(("workspace", id.0))
    }
    /// Declares the root blocker and valid enabled child targets.
    pub fn declare_pointer_targets(
        &self,
        plan: &mut PointerTargetPlan,
        first_order: PointerOrder,
    ) -> PointerOrder {
        let Some(layout) = self.layout() else {
            return first_order;
        };
        let mut ordinal = first_order.raw();
        plan.blocker(layout.bounds, take_order(&mut ordinal));
        plan.with_clip(layout.bounds, |plan| {
            for row in layout.menu_rows.iter().chain(&layout.workspace_rows) {
                if row.enabled {
                    plan.target(PointerTarget::new(
                        row.id,
                        row.rect,
                        take_order(&mut ordinal),
                    ));
                }
            }
        });
        PointerOrder::new(ordinal)
    }
    /// Returns finite non-interactive space between menus and workspaces.
    #[must_use]
    pub fn drag_safe_regions(&self) -> Vec<Rect> {
        self.layout()
            .and_then(|layout| layout.drag_safe)
            .into_iter()
            .collect()
    }
    pub(crate) fn layout(&self) -> Option<ApplicationBarLayout> {
        let bounds = valid_rect(self.config.bounds)?;
        let menu_ids = self
            .menu_bar
            .menus()
            .iter()
            .filter(|menu| menu.has_visible_items())
            .map(|menu| menu.id)
            .collect::<Vec<_>>();
        let menu_rows = self.menu_rows(bounds, &menu_ids);
        let workspace_rows = self.workspace_rows(bounds);
        let geometry_valid = menu_rows.is_some() && workspace_rows.is_some();
        let (mut menu_rows, mut workspace_rows) = (
            menu_rows.unwrap_or_default(),
            workspace_rows.unwrap_or_default(),
        );
        let overlap = geometry_valid
            && menu_rows.iter().any(|left| {
                workspace_rows
                    .iter()
                    .any(|right| rects_overlap(left.rect, right.rect))
            });
        if overlap {
            menu_rows.clear();
            workspace_rows.clear();
        }
        let drag_safe = (geometry_valid && !overlap)
            .then(|| {
                let start = menu_rows.last().map_or(bounds.x, |row| row.rect.max_x());
                let end = workspace_rows
                    .first()
                    .map_or(bounds.max_x(), |row| row.rect.x);
                valid_rect(Rect::new(start, bounds.y, end - start, bounds.height))
            })
            .flatten();
        Some(ApplicationBarLayout {
            bounds,
            menu_composite: self.menu_composite_id(),
            workspace_composite: self.workspace_composite_id(),
            menu_rows,
            workspace_rows,
            drag_safe,
        })
    }
    fn menu_rows(&self, bounds: Rect, ids: &[MenuBarMenuId]) -> Option<Vec<ApplicationBarRow>> {
        let width = valid_width(self.config.menu_width)?;
        unique(ids.iter().map(|id| id.raw()))?;
        ids.iter()
            .enumerate()
            .map(|(index, id)| {
                let x = bounds.x + width * f32::from(u16::try_from(index).ok()?);
                let rect = valid_child(Rect::new(x, bounds.y, width, bounds.height), bounds)?;
                let menu = self.menu_bar.menus().iter().find(|menu| menu.id == *id)?;
                Some(ApplicationBarRow {
                    id: self.menu_widget_id(*id),
                    rect,
                    label: menu.title.clone(),
                    enabled: true,
                    kind: ApplicationBarRowKind::Menu(*id),
                })
            })
            .collect()
    }
    fn workspace_rows(&self, bounds: Rect) -> Option<Vec<ApplicationBarRow>> {
        let width = valid_width(self.config.workspace_width)?;
        unique(self.workspaces.iter().map(|tab| tab.id.0))?;
        let count = f32::from(u16::try_from(self.workspaces.len()).ok()?);
        let start = bounds.max_x() - width * count;
        self.workspaces
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                let x = start + width * f32::from(u16::try_from(index).ok()?);
                Some(ApplicationBarRow {
                    id: self.workspace_widget_id(tab.id),
                    rect: valid_child(Rect::new(x, bounds.y, width, bounds.height), bounds)?,
                    label: tab.label.clone(),
                    enabled: tab.enabled,
                    kind: ApplicationBarRowKind::Workspace(WorkspaceTabTarget {
                        id: tab.id,
                        index,
                    }),
                })
            })
            .collect()
    }
    fn menu_composite_id(&self) -> WidgetId {
        self.config.root.child("application-menu")
    }
    fn workspace_composite_id(&self) -> WidgetId {
        self.config.root.child("application-workspaces")
    }
}
/// Application-owned intent emitted by one bar evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationBarIntent {
    /// A menu heading requested its existing menu chain.
    OpenMenu {
        /// Stable menu identity.
        menu: MenuBarMenuId,
        /// Heading bounds suitable for overlay anchoring.
        anchor: Rect,
    },
    /// Escape or trigger-toggle dismissed the active menu.
    DismissMenu {
        /// Stable dismissed menu identity.
        menu: MenuBarMenuId,
    },
    /// A passive workspace requested application-owned activation.
    ActivateWorkspace(WorkspaceTabTarget),
}
/// Frame-local application-bar result.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplicationBarOutput {
    /// Child responses in menu-then-workspace source order.
    pub responses: Vec<Response>,
    /// Application-owned intents in event order.
    pub intents: Vec<ApplicationBarIntent>,
    /// Non-interactive middle geometry suitable for titlebar drag policy.
    pub drag_safe_regions: Vec<Rect>,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ApplicationBarLayout {
    pub(crate) bounds: Rect,
    pub(crate) menu_composite: WidgetId,
    pub(crate) workspace_composite: WidgetId,
    pub(crate) menu_rows: Vec<ApplicationBarRow>,
    pub(crate) workspace_rows: Vec<ApplicationBarRow>,
    pub(crate) drag_safe: Option<Rect>,
}
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ApplicationBarRow {
    pub(crate) id: WidgetId,
    pub(crate) rect: Rect,
    pub(crate) label: String,
    pub(crate) enabled: bool,
    pub(crate) kind: ApplicationBarRowKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ApplicationBarRowKind {
    Menu(MenuBarMenuId),
    Workspace(WorkspaceTabTarget),
}
fn unique(values: impl Iterator<Item = u64>) -> Option<()> {
    let values = values.collect::<Vec<_>>();
    values
        .iter()
        .enumerate()
        .all(|(index, value)| !values[..index].contains(value))
        .then_some(())
}
fn valid_width(width: f32) -> Option<f32> {
    (width.is_finite() && width > 0.0).then_some(width)
}
fn valid_child(rect: Rect, bounds: Rect) -> Option<Rect> {
    valid_rect(rect).filter(|rect| bounds.contains_rect(*rect))
}
fn valid_rect(rect: Rect) -> Option<Rect> {
    (!rect.is_empty()
        && rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.max_x().is_finite()
        && rect.max_y().is_finite())
    .then_some(rect)
}
fn rects_overlap(left: Rect, right: Rect) -> bool {
    left.x < right.max_x()
        && right.x < left.max_x()
        && left.y < right.max_y()
        && right.y < left.max_y()
}
fn take_order(ordinal: &mut u64) -> PointerOrder {
    let order = PointerOrder::new(*ordinal);
    *ordinal = ordinal.saturating_add(1);
    order
}
