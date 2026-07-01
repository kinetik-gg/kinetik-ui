use std::collections::BTreeSet;

use super::{CollectionProjection, ItemId};

/// Shared multi-selection state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    selected: BTreeSet<ItemId>,
    /// Active item.
    pub active: Option<ItemId>,
    anchor: Option<ItemId>,
}

/// Selection projection behavior.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SelectionProjectionPolicy {
    /// Keep hidden selected IDs in the app-owned selection while exposing
    /// visible selected IDs and repaired visible active/anchor IDs.
    #[default]
    PreserveHidden,
}

/// Non-mutating view of selection state through a collection projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionProjection {
    /// All selected IDs in selection's deterministic order.
    pub selected: Vec<ItemId>,
    /// Selected IDs currently present in projected visible order.
    pub visible_selected: Vec<ItemId>,
    /// Selected IDs hidden by the projection, in selection's deterministic order.
    pub hidden_selected: Vec<ItemId>,
    /// App-owned active ID before projection.
    pub active: Option<ItemId>,
    /// App-owned anchor ID before projection.
    pub anchor: Option<ItemId>,
    /// Active ID only when it is visible through the projection.
    pub visible_active: Option<ItemId>,
    /// Anchor ID only when it is visible through the projection.
    pub visible_anchor: Option<ItemId>,
    /// Visible active ID repaired deterministically for the projected view.
    pub repaired_active: Option<ItemId>,
    /// Visible anchor ID repaired deterministically for the projected view.
    pub repaired_anchor: Option<ItemId>,
}

impl SelectionProjection {
    /// Returns true when at least one selected item is hidden by the projection.
    #[must_use]
    pub fn has_hidden_selection(&self) -> bool {
        !self.hidden_selected.is_empty()
    }
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

    /// Returns the range anchor, when one is set.
    #[must_use]
    pub const fn anchor(&self) -> Option<ItemId> {
        self.anchor
    }

    /// Returns true when the current range anchor is present in the visible items.
    #[must_use]
    pub fn anchor_visible(&self, visible_items: &[ItemId]) -> bool {
        self.anchor
            .is_some_and(|anchor| visible_items.contains(&anchor))
    }

    /// Retains selected, active, and anchored items present in the visible item set.
    ///
    /// Returns true when selection state changed.
    pub fn retain_visible(&mut self, visible_items: &[ItemId]) -> bool {
        let visible_items = visible_items.iter().copied().collect::<BTreeSet<_>>();
        let selected_len = self.selected.len();
        self.selected.retain(|item| visible_items.contains(item));

        let mut changed = self.selected.len() != selected_len;
        if self
            .active
            .is_some_and(|active| !visible_items.contains(&active))
        {
            self.active = None;
            changed = true;
        }
        if self
            .anchor
            .is_some_and(|anchor| !visible_items.contains(&anchor))
        {
            self.anchor = None;
            changed = true;
        }

        changed
    }

    /// Projects selection through visible item IDs without mutating selection.
    #[must_use]
    pub fn project_visible(
        &self,
        visible_items: &[ItemId],
        policy: SelectionProjectionPolicy,
    ) -> SelectionProjection {
        let projection = CollectionProjection::from_source_ids(visible_items);
        self.project(&projection, policy)
    }

    /// Projects selection through a collection projection without mutating selection.
    #[must_use]
    pub fn project(
        &self,
        projection: &CollectionProjection,
        policy: SelectionProjectionPolicy,
    ) -> SelectionProjection {
        match policy {
            SelectionProjectionPolicy::PreserveHidden => {
                let selected = self.selected();
                let selected_set = selected.iter().copied().collect::<BTreeSet<_>>();
                let visible_set = projection
                    .visible_ids()
                    .into_iter()
                    .collect::<BTreeSet<_>>();
                let visible_selected = projection
                    .items()
                    .iter()
                    .map(|item| item.id)
                    .filter(|id| selected_set.contains(id))
                    .collect::<Vec<_>>();
                let hidden_selected = selected
                    .iter()
                    .copied()
                    .filter(|id| !visible_set.contains(id))
                    .collect::<Vec<_>>();
                let visible_active = self.active.filter(|active| visible_set.contains(active));
                let visible_anchor = self.anchor.filter(|anchor| visible_set.contains(anchor));
                let repaired_active = visible_active.or_else(|| visible_selected.first().copied());
                let repaired_anchor = visible_anchor.or(repaired_active);

                SelectionProjection {
                    selected,
                    visible_selected,
                    hidden_selected,
                    active: self.active,
                    anchor: self.anchor,
                    visible_active,
                    visible_anchor,
                    repaired_active,
                    repaired_anchor,
                }
            }
        }
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
