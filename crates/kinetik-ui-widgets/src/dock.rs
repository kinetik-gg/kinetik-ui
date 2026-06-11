//! `DockArea`, `Frame`, and `Panel` models for editor layouts.

use std::collections::BTreeSet;

use kinetik_ui_core::{Axis, Rect};

/// Stable panel identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PanelId(u64);

impl PanelId {
    /// Creates a panel ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns raw ID bits.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Stable frame identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameId(u64);

impl FrameId {
    /// Creates a frame ID from raw bits.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns raw ID bits.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Passive panel metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panel {
    /// Panel identity.
    pub id: PanelId,
    /// Display title used by frame tabs.
    pub title: String,
}

impl Panel {
    /// Creates a panel.
    #[must_use]
    pub fn new(id: PanelId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
        }
    }
}

/// Docked frame containing tabbed panels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Frame identity.
    pub id: FrameId,
    /// Panels in tab order.
    pub panels: Vec<Panel>,
    /// Active panel index.
    pub active: usize,
    /// Panels whose frame tabs expose close/dismiss affordances.
    dismissible_panels: BTreeSet<PanelId>,
}

impl Frame {
    /// Creates a frame with panels.
    #[must_use]
    pub fn new(id: FrameId, panels: Vec<Panel>) -> Self {
        let dismissible_panels = panels.iter().map(|panel| panel.id).collect();
        Self {
            id,
            panels,
            active: 0,
            dismissible_panels,
        }
    }

    /// Returns the active panel.
    #[must_use]
    pub fn active_panel(&self) -> Option<&Panel> {
        self.panels.get(self.active)
    }

    /// Selects a panel by ID.
    pub fn select_panel(&mut self, panel: PanelId) -> bool {
        let Some(index) = self.panels.iter().position(|item| item.id == panel) else {
            return false;
        };
        self.active = index;
        true
    }

    /// Removes a panel by ID.
    pub fn remove_panel(&mut self, panel: PanelId) -> Option<Panel> {
        let (removed, _) = self.remove_panel_with_policy(panel)?;
        Some(removed)
    }

    fn remove_panel_with_policy(&mut self, panel: PanelId) -> Option<(Panel, bool)> {
        let index = self.panels.iter().position(|item| item.id == panel)?;
        let dismissible = self.dismissible_panels.remove(&panel);
        let removed = self.panels.remove(index);
        self.active = self.active.min(self.panels.len().saturating_sub(1));
        Some((removed, dismissible))
    }

    /// Adds a panel at the end.
    pub fn push_panel(&mut self, panel: Panel) {
        self.push_panel_with_policy(panel, true);
    }

    fn push_panel_with_policy(&mut self, panel: Panel, dismissible: bool) {
        let id = panel.id;
        self.panels.push(panel);
        self.set_panel_dismissible(id, dismissible);
    }

    /// Sets whether a frame tab can expose close/dismiss affordances.
    ///
    /// Returns `false` when the panel is not in this frame.
    pub fn set_panel_dismissible(&mut self, panel: PanelId, dismissible: bool) -> bool {
        if !self.panels.iter().any(|item| item.id == panel) {
            return false;
        }
        if dismissible {
            self.dismissible_panels.insert(panel);
        } else {
            self.dismissible_panels.remove(&panel);
        }
        true
    }

    /// Returns true when a frame tab can expose close/dismiss affordances.
    #[must_use]
    pub fn panel_dismissible(&self, panel: PanelId) -> bool {
        self.dismissible_panels.contains(&panel)
    }
}

/// Dock tree node.
#[derive(Debug, Clone, PartialEq)]
pub enum DockNode {
    /// Leaf frame.
    Frame(Frame),
    /// Split between two nodes.
    Split {
        /// Split axis.
        axis: Axis,
        /// First child ratio.
        ratio: f32,
        /// Minimum first child size.
        min_first: f32,
        /// Minimum second child size.
        min_second: f32,
        /// First child.
        first: Box<DockNode>,
        /// Second child.
        second: Box<DockNode>,
    },
}

/// Root dock area.
#[derive(Debug, Clone, PartialEq)]
pub struct DockArea {
    /// Root dock node.
    pub root: DockNode,
}

impl DockArea {
    /// Creates a dock area.
    #[must_use]
    pub const fn new(root: DockNode) -> Self {
        Self { root }
    }

    /// Visits all frames in deterministic tree order.
    #[must_use]
    pub fn frames(&self) -> Vec<&Frame> {
        let mut frames = Vec::new();
        collect_frames(&self.root, &mut frames);
        frames
    }

    /// Finds an immutable frame.
    #[must_use]
    pub fn frame(&self, frame: FrameId) -> Option<&Frame> {
        find_frame(&self.root, frame)
    }

    /// Finds a mutable frame.
    pub fn frame_mut(&mut self, frame: FrameId) -> Option<&mut Frame> {
        find_frame_mut(&mut self.root, frame)
    }

    /// Selects a panel in a frame.
    pub fn select_panel(&mut self, frame: FrameId, panel: PanelId) -> bool {
        self.frame_mut(frame)
            .is_some_and(|frame| frame.select_panel(panel))
    }

    /// Moves a panel between frames.
    pub fn move_panel(&mut self, from: FrameId, to: FrameId, panel: PanelId) -> bool {
        if from == to || self.frame(to).is_none() {
            return false;
        }
        let Some((panel, dismissible)) = self
            .frame_mut(from)
            .and_then(|frame| frame.remove_panel_with_policy(panel))
        else {
            return false;
        };
        let Some(target) = self.frame_mut(to) else {
            return false;
        };
        target.push_panel_with_policy(panel, dismissible);
        target.active = target.panels.len().saturating_sub(1);
        prune_empty_frames(&mut self.root);
        true
    }

    /// Merges all source frame panels into target frame.
    pub fn merge_frames(&mut self, source: FrameId, target: FrameId) -> bool {
        if source == target || self.frame(source).is_none() || self.frame(target).is_none() {
            return false;
        }
        let Some((source_panels, dismissible_panels)) = self.frame_mut(source).map(|frame| {
            frame.active = 0;
            (
                core::mem::take(&mut frame.panels),
                core::mem::take(&mut frame.dismissible_panels),
            )
        }) else {
            return false;
        };
        let Some(target_frame) = self.frame_mut(target) else {
            return false;
        };
        target_frame.panels.extend(source_panels);
        target_frame.dismissible_panels.extend(dismissible_panels);
        target_frame.active = target_frame.panels.len().saturating_sub(1);
        prune_empty_frames(&mut self.root);
        true
    }

    /// Creates a snapshot for persistence.
    #[must_use]
    pub fn snapshot(&self) -> DockSnapshot {
        DockSnapshot {
            root: snapshot_node(&self.root),
        }
    }

    /// Restores a snapshot after validation.
    ///
    /// # Errors
    ///
    /// Returns [`DockRestoreError`] when persisted dock data is structurally
    /// invalid, contains duplicate identities, or stores invalid split values.
    pub fn restore(snapshot: DockSnapshot) -> Result<Self, DockRestoreError> {
        let mut validation = DockSnapshotValidation::default();
        validate_snapshot_node(&snapshot.root, &mut validation)?;
        Ok(Self {
            root: restore_node(snapshot.root),
        })
    }
}

fn collect_frames<'a>(node: &'a DockNode, frames: &mut Vec<&'a Frame>) {
    match node {
        DockNode::Frame(frame) => frames.push(frame),
        DockNode::Split { first, second, .. } => {
            collect_frames(first, frames);
            collect_frames(second, frames);
        }
    }
}

fn find_frame_mut(node: &mut DockNode, id: FrameId) -> Option<&mut Frame> {
    match node {
        DockNode::Frame(frame) if frame.id == id => Some(frame),
        DockNode::Frame(_) => None,
        DockNode::Split { first, second, .. } => {
            find_frame_mut(first, id).or_else(|| find_frame_mut(second, id))
        }
    }
}

fn find_frame(node: &DockNode, id: FrameId) -> Option<&Frame> {
    match node {
        DockNode::Frame(frame) if frame.id == id => Some(frame),
        DockNode::Frame(_) => None,
        DockNode::Split { first, second, .. } => {
            find_frame(first, id).or_else(|| find_frame(second, id))
        }
    }
}

fn prune_empty_frames(node: &mut DockNode) -> bool {
    match node {
        DockNode::Frame(frame) => !frame.panels.is_empty(),
        DockNode::Split { first, second, .. } => {
            let first_has_panels = prune_empty_frames(first);
            let second_has_panels = prune_empty_frames(second);
            match (first_has_panels, second_has_panels) {
                (true, true) => true,
                (true, false) => {
                    *node = (**first).clone();
                    true
                }
                (false, true) => {
                    *node = (**second).clone();
                    true
                }
                (false, false) => false,
            }
        }
    }
}

/// Resolved frame rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameLayout {
    /// Frame identity.
    pub frame: FrameId,
    /// Frame rectangle.
    pub rect: Rect,
}

/// Resolves a dock tree into frame rectangles.
#[must_use]
pub fn solve_dock_layout(area: &DockArea, bounds: Rect) -> Vec<FrameLayout> {
    let mut frames = Vec::new();
    solve_node(&area.root, bounds, &mut frames);
    frames
}

fn solve_node(node: &DockNode, bounds: Rect, frames: &mut Vec<FrameLayout>) {
    match node {
        DockNode::Frame(frame) => frames.push(FrameLayout {
            frame: frame.id,
            rect: bounds,
        }),
        DockNode::Split {
            axis,
            ratio,
            min_first,
            min_second,
            first,
            second,
        } => {
            let total = match axis {
                Axis::Horizontal => bounds.width,
                Axis::Vertical => bounds.height,
            };
            let total = finite_non_negative(total);
            let min_first = finite_non_negative(*min_first);
            let min_second = finite_non_negative(*min_second);
            let desired = total * finite_ratio(*ratio);
            let first_size = if total >= min_first + min_second {
                desired.clamp(min_first, total - min_second)
            } else {
                desired.max(min_first.min(total)).min(total)
            };
            let second_size = (total - first_size).max(0.0);
            let (first_rect, second_rect) = match axis {
                Axis::Horizontal => (
                    Rect::new(bounds.x, bounds.y, first_size, bounds.height),
                    Rect::new(bounds.x + first_size, bounds.y, second_size, bounds.height),
                ),
                Axis::Vertical => (
                    Rect::new(bounds.x, bounds.y, bounds.width, first_size),
                    Rect::new(bounds.x, bounds.y + first_size, bounds.width, second_size),
                ),
            };
            solve_node(first, first_rect, frames);
            solve_node(second, second_rect, frames);
        }
    }
}

fn finite_ratio(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

/// Tab presentation data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameTab {
    /// Panel identity.
    pub panel: PanelId,
    /// Tab title.
    pub title: String,
    /// Whether this tab is active.
    pub active: bool,
    /// Whether this tab can be closed.
    pub close_visible: bool,
    /// Whether this tab can begin a drag operation.
    pub draggable: bool,
}

/// Produces frame tab presentation records.
#[must_use]
pub fn frame_tabs(frame: &Frame) -> Vec<FrameTab> {
    frame
        .panels
        .iter()
        .enumerate()
        .map(|(index, panel)| FrameTab {
            panel: panel.id,
            title: panel.title.clone(),
            active: index == frame.active,
            close_visible: frame.panel_dismissible(panel.id),
            draggable: true,
        })
        .collect()
}

/// Persistable dock snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DockSnapshot {
    /// Root snapshot node.
    pub root: DockSnapshotNode,
}

/// Snapshot node.
#[derive(Debug, Clone, PartialEq)]
pub enum DockSnapshotNode {
    /// Frame snapshot.
    Frame {
        /// Frame identity.
        id: FrameId,
        /// Panels.
        panels: Vec<Panel>,
        /// Active panel index.
        active: usize,
        /// Panels whose frame tabs expose close/dismiss affordances.
        dismissible_panels: Vec<PanelId>,
    },
    /// Split snapshot.
    Split {
        /// Split axis.
        axis: Axis,
        /// First child ratio.
        ratio: f32,
        /// Minimum first size.
        min_first: f32,
        /// Minimum second size.
        min_second: f32,
        /// First child.
        first: Box<DockSnapshotNode>,
        /// Second child.
        second: Box<DockSnapshotNode>,
    },
}

/// Snapshot restore error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockRestoreError {
    /// Frame contains no panels.
    EmptyFrame,
    /// Active tab index is outside the panel list.
    InvalidActiveIndex,
    /// Two frames use the same stable frame identity.
    DuplicateFrameId,
    /// Two panels use the same stable panel identity.
    DuplicatePanelId,
    /// Dismissible panel policy references a panel missing from the frame.
    InvalidDismissiblePanel,
    /// Dismissible panel policy contains the same panel more than once.
    DuplicateDismissiblePanel,
    /// Split ratio is not finite or is outside the inclusive 0.0..=1.0 range.
    InvalidSplitRatio,
    /// Split minimum is not finite or is negative.
    InvalidSplitMinimum,
}

fn snapshot_node(node: &DockNode) -> DockSnapshotNode {
    match node {
        DockNode::Frame(frame) => DockSnapshotNode::Frame {
            id: frame.id,
            panels: frame.panels.clone(),
            active: frame.active,
            dismissible_panels: frame.dismissible_panels.iter().copied().collect(),
        },
        DockNode::Split {
            axis,
            ratio,
            min_first,
            min_second,
            first,
            second,
        } => DockSnapshotNode::Split {
            axis: *axis,
            ratio: *ratio,
            min_first: *min_first,
            min_second: *min_second,
            first: Box::new(snapshot_node(first)),
            second: Box::new(snapshot_node(second)),
        },
    }
}

fn restore_node(snapshot: DockSnapshotNode) -> DockNode {
    match snapshot {
        DockSnapshotNode::Frame {
            id,
            panels,
            active,
            dismissible_panels,
        } => DockNode::Frame(Frame {
            id,
            panels,
            active,
            dismissible_panels: dismissible_panels.into_iter().collect(),
        }),
        DockSnapshotNode::Split {
            axis,
            ratio,
            min_first,
            min_second,
            first,
            second,
        } => DockNode::Split {
            axis,
            ratio,
            min_first,
            min_second,
            first: Box::new(restore_node(*first)),
            second: Box::new(restore_node(*second)),
        },
    }
}

#[derive(Default)]
struct DockSnapshotValidation {
    frame_ids: BTreeSet<FrameId>,
    panel_ids: BTreeSet<PanelId>,
}

fn validate_snapshot_node(
    snapshot: &DockSnapshotNode,
    validation: &mut DockSnapshotValidation,
) -> Result<(), DockRestoreError> {
    match snapshot {
        DockSnapshotNode::Frame {
            id,
            panels,
            active,
            dismissible_panels,
        } => {
            if !validation.frame_ids.insert(*id) {
                return Err(DockRestoreError::DuplicateFrameId);
            }
            if panels.is_empty() {
                return Err(DockRestoreError::EmptyFrame);
            }
            if *active >= panels.len() {
                return Err(DockRestoreError::InvalidActiveIndex);
            }

            let mut frame_panel_ids = BTreeSet::new();
            for panel in panels {
                if !frame_panel_ids.insert(panel.id) || !validation.panel_ids.insert(panel.id) {
                    return Err(DockRestoreError::DuplicatePanelId);
                }
            }

            let mut frame_dismissible_ids = BTreeSet::new();
            for id in dismissible_panels {
                if !frame_dismissible_ids.insert(*id) {
                    return Err(DockRestoreError::DuplicateDismissiblePanel);
                }
                if !frame_panel_ids.contains(id) {
                    return Err(DockRestoreError::InvalidDismissiblePanel);
                }
            }
            Ok(())
        }
        DockSnapshotNode::Split {
            ratio,
            min_first,
            min_second,
            first,
            second,
            ..
        } => {
            if !ratio.is_finite() || !(0.0..=1.0).contains(ratio) {
                return Err(DockRestoreError::InvalidSplitRatio);
            }
            if !min_first.is_finite()
                || !min_second.is_finite()
                || *min_first < 0.0
                || *min_second < 0.0
            {
                return Err(DockRestoreError::InvalidSplitMinimum);
            }
            validate_snapshot_node(first, validation)?;
            validate_snapshot_node(second, validation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DockArea, DockNode, DockRestoreError, DockSnapshot, DockSnapshotNode, Frame, FrameId,
        Panel, PanelId, frame_tabs, solve_dock_layout,
    };
    use kinetik_ui_core::{Axis, Rect};

    fn panel(id: u64, title: &str) -> Panel {
        Panel::new(PanelId::from_raw(id), title)
    }

    fn frame(id: u64, panels: Vec<Panel>) -> Frame {
        Frame::new(FrameId::from_raw(id), panels)
    }

    fn dock_area() -> DockArea {
        DockArea::new(DockNode::Split {
            axis: Axis::Horizontal,
            ratio: 0.25,
            min_first: 100.0,
            min_second: 100.0,
            first: Box::new(DockNode::Frame(frame(1, vec![panel(1, "Media")]))),
            second: Box::new(DockNode::Frame(frame(
                2,
                vec![panel(2, "Viewport"), panel(3, "Timeline")],
            ))),
        })
    }

    #[test]
    fn dock_tree_visits_frames_in_order() {
        let area = dock_area();
        let frames = area.frames();

        assert_eq!(frames[0].id, FrameId::from_raw(1));
        assert_eq!(frames[1].id, FrameId::from_raw(2));
    }

    #[test]
    fn selects_and_removes_frame_tabs() {
        let mut frame = frame(1, vec![panel(1, "A"), panel(2, "B")]);

        assert!(frame.select_panel(PanelId::from_raw(2)));
        assert_eq!(
            frame.active_panel().expect("active").id,
            PanelId::from_raw(2)
        );
        assert_eq!(
            frame
                .remove_panel(PanelId::from_raw(2))
                .expect("removed")
                .title,
            "B"
        );
        assert_eq!(frame.active, 0);
    }

    #[test]
    fn moves_panels_between_frames() {
        let mut area = dock_area();

        assert!(area.move_panel(
            FrameId::from_raw(2),
            FrameId::from_raw(1),
            PanelId::from_raw(3)
        ));

        assert_eq!(
            area.frame_mut(FrameId::from_raw(1))
                .expect("frame")
                .panels
                .len(),
            2
        );
    }

    #[test]
    fn moving_panels_preserves_frame_owned_dismissal_policy() {
        let mut area = dock_area();
        area.frame_mut(FrameId::from_raw(2))
            .expect("source")
            .set_panel_dismissible(PanelId::from_raw(3), false);

        assert!(area.move_panel(
            FrameId::from_raw(2),
            FrameId::from_raw(1),
            PanelId::from_raw(3)
        ));

        assert!(
            !area
                .frame(FrameId::from_raw(1))
                .expect("target")
                .panel_dismissible(PanelId::from_raw(3))
        );
    }

    #[test]
    fn moving_panel_to_missing_target_does_not_remove_it() {
        let mut area = dock_area();

        assert!(!area.move_panel(
            FrameId::from_raw(2),
            FrameId::from_raw(99),
            PanelId::from_raw(3)
        ));

        let source = area.frame(FrameId::from_raw(2)).expect("source");
        assert_eq!(source.panels.len(), 2);
        assert!(
            source
                .panels
                .iter()
                .any(|panel| panel.id == PanelId::from_raw(3))
        );
    }

    #[test]
    fn moving_last_panel_prunes_empty_source_frame() {
        let mut area = dock_area();

        assert!(area.move_panel(
            FrameId::from_raw(1),
            FrameId::from_raw(2),
            PanelId::from_raw(1)
        ));

        assert_eq!(area.frames().len(), 1);
        assert!(area.frame(FrameId::from_raw(1)).is_none());
        assert_eq!(
            area.frame(FrameId::from_raw(2))
                .expect("target")
                .panels
                .len(),
            3
        );
    }

    #[test]
    fn merges_frames_into_target() {
        let mut area = dock_area();

        assert!(area.merge_frames(FrameId::from_raw(1), FrameId::from_raw(2)));

        assert_eq!(
            area.frame_mut(FrameId::from_raw(2))
                .expect("target")
                .panels
                .len(),
            3
        );
        assert_eq!(area.frames().len(), 1);
        assert!(area.frame(FrameId::from_raw(1)).is_none());
    }

    #[test]
    fn merging_missing_target_does_not_remove_source_panels() {
        let mut area = dock_area();

        assert!(!area.merge_frames(FrameId::from_raw(1), FrameId::from_raw(99)));

        assert_eq!(
            area.frame(FrameId::from_raw(1))
                .expect("source")
                .panels
                .len(),
            1
        );
    }

    #[test]
    fn solves_horizontal_split_layout() {
        let area = dock_area();
        let layout = solve_dock_layout(&area, Rect::new(0.0, 0.0, 1000.0, 500.0));

        assert_eq!(layout.len(), 2);
        assert!((layout[0].rect.width - 250.0).abs() < f32::EPSILON);
        assert!((layout[1].rect.x - 250.0).abs() < f32::EPSILON);
    }

    #[test]
    fn split_layout_respects_minimums() {
        let area = DockArea::new(DockNode::Split {
            axis: Axis::Horizontal,
            ratio: 0.05,
            min_first: 100.0,
            min_second: 100.0,
            first: Box::new(DockNode::Frame(frame(1, vec![panel(1, "A")]))),
            second: Box::new(DockNode::Frame(frame(2, vec![panel(2, "B")]))),
        });
        let layout = solve_dock_layout(&area, Rect::new(0.0, 0.0, 500.0, 200.0));

        assert!((layout[0].rect.width - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn split_layout_never_emits_negative_sizes_when_minimums_exceed_bounds() {
        let area = DockArea::new(DockNode::Split {
            axis: Axis::Horizontal,
            ratio: 0.5,
            min_first: 100.0,
            min_second: 100.0,
            first: Box::new(DockNode::Frame(frame(1, vec![panel(1, "A")]))),
            second: Box::new(DockNode::Frame(frame(2, vec![panel(2, "B")]))),
        });
        let layout = solve_dock_layout(&area, Rect::new(0.0, 0.0, 120.0, 200.0));

        assert_eq!(layout.len(), 2);
        assert!(layout[0].rect.width >= 0.0);
        assert!(layout[1].rect.width >= 0.0);
        assert!((layout[0].rect.width + layout[1].rect.width - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn split_layout_sanitizes_direct_non_finite_values() {
        let area = DockArea::new(DockNode::Split {
            axis: Axis::Horizontal,
            ratio: f32::NAN,
            min_first: f32::INFINITY,
            min_second: -100.0,
            first: Box::new(DockNode::Frame(frame(1, vec![panel(1, "A")]))),
            second: Box::new(DockNode::Frame(frame(2, vec![panel(2, "B")]))),
        });
        let layout = solve_dock_layout(&area, Rect::new(0.0, 0.0, 120.0, 200.0));

        assert_eq!(layout.len(), 2);
        assert!(layout[0].rect.width.is_finite());
        assert!(layout[1].rect.width.is_finite());
        assert!((layout[0].rect.width + layout[1].rect.width - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn frame_tabs_expose_presentation_state() {
        let mut frame = frame(1, vec![panel(1, "A"), panel(2, "B")]);
        frame.select_panel(PanelId::from_raw(2));
        assert!(frame.set_panel_dismissible(PanelId::from_raw(1), false));

        let tabs = frame_tabs(&frame);

        assert!(!tabs[0].active);
        assert!(!tabs[0].close_visible);
        assert!(tabs[1].active);
        assert!(tabs[1].close_visible);
        assert!(tabs[1].draggable);
    }

    #[test]
    fn snapshots_round_trip() {
        let area = dock_area();
        let snapshot = area.snapshot();
        let restored = DockArea::restore(snapshot).expect("restore");

        assert_eq!(restored.frames().len(), 2);
    }

    #[test]
    fn invalid_snapshots_are_rejected() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Frame {
                id: FrameId::from_raw(1),
                panels: vec![],
                active: 0,
                dismissible_panels: vec![],
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::EmptyFrame
        );
    }

    #[test]
    fn invalid_snapshot_rejects_invalid_active_panel() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Frame {
                id: FrameId::from_raw(1),
                panels: vec![panel(1, "A")],
                active: 1,
                dismissible_panels: vec![PanelId::from_raw(1)],
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::InvalidActiveIndex
        );
    }

    #[test]
    fn invalid_snapshot_rejects_unknown_dismissible_panel() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Frame {
                id: FrameId::from_raw(1),
                panels: vec![panel(1, "A")],
                active: 0,
                dismissible_panels: vec![PanelId::from_raw(2)],
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::InvalidDismissiblePanel
        );
    }

    #[test]
    fn invalid_snapshot_rejects_duplicate_frame_ids() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Split {
                axis: Axis::Horizontal,
                ratio: 0.5,
                min_first: 0.0,
                min_second: 0.0,
                first: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(1),
                    panels: vec![panel(1, "A")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(1)],
                }),
                second: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(1),
                    panels: vec![panel(2, "B")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(2)],
                }),
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::DuplicateFrameId
        );
    }

    #[test]
    fn invalid_snapshot_rejects_duplicate_panel_ids() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Split {
                axis: Axis::Horizontal,
                ratio: 0.5,
                min_first: 0.0,
                min_second: 0.0,
                first: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(1),
                    panels: vec![panel(1, "A")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(1)],
                }),
                second: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(2),
                    panels: vec![panel(1, "B")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(1)],
                }),
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::DuplicatePanelId
        );
    }

    #[test]
    fn invalid_snapshot_rejects_duplicate_dismissible_policy_entries() {
        let snapshot = DockSnapshot {
            root: DockSnapshotNode::Frame {
                id: FrameId::from_raw(1),
                panels: vec![panel(1, "A")],
                active: 0,
                dismissible_panels: vec![PanelId::from_raw(1), PanelId::from_raw(1)],
            },
        };

        assert_eq!(
            DockArea::restore(snapshot).expect_err("error"),
            DockRestoreError::DuplicateDismissiblePanel
        );
    }

    #[test]
    fn invalid_snapshot_rejects_invalid_split_numbers() {
        let invalid_ratio = DockSnapshot {
            root: DockSnapshotNode::Split {
                axis: Axis::Horizontal,
                ratio: f32::NAN,
                min_first: 0.0,
                min_second: 0.0,
                first: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(1),
                    panels: vec![panel(1, "A")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(1)],
                }),
                second: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(2),
                    panels: vec![panel(2, "B")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(2)],
                }),
            },
        };
        assert_eq!(
            DockArea::restore(invalid_ratio).expect_err("error"),
            DockRestoreError::InvalidSplitRatio
        );

        let invalid_minimum = DockSnapshot {
            root: DockSnapshotNode::Split {
                axis: Axis::Horizontal,
                ratio: 0.5,
                min_first: -1.0,
                min_second: 0.0,
                first: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(1),
                    panels: vec![panel(1, "A")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(1)],
                }),
                second: Box::new(DockSnapshotNode::Frame {
                    id: FrameId::from_raw(2),
                    panels: vec![panel(2, "B")],
                    active: 0,
                    dismissible_panels: vec![PanelId::from_raw(2)],
                }),
            },
        };

        assert_eq!(
            DockArea::restore(invalid_minimum).expect_err("error"),
            DockRestoreError::InvalidSplitMinimum
        );
    }
}
