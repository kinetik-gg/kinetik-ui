use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use super::{ItemId, TreeRow};

/// One item in a tree model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeItem {
    /// Stable item identity.
    pub id: ItemId,
    /// Parent item, or `None` for a root item.
    pub parent: Option<ItemId>,
    /// Whether the item should expose an expansion affordance even before children are loaded.
    pub has_children: bool,
}

/// Structural tree model error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeModelError {
    /// More than one item uses the same ID.
    DuplicateItemId {
        /// Duplicated item identity.
        id: ItemId,
    },
    /// An item points at itself as parent.
    SelfParent {
        /// Invalid item identity.
        id: ItemId,
    },
    /// An item points at a parent that is not present in the model.
    UnknownParent {
        /// Item carrying the invalid parent reference.
        id: ItemId,
        /// Missing parent identity.
        parent: ItemId,
    },
    /// Parent links contain a cycle.
    Cycle {
        /// First repeated item detected while walking parent links.
        id: ItemId,
    },
}

/// Flat tree model with parent links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeModel {
    items: Vec<TreeItem>,
}

impl TreeModel {
    /// Creates a tree model from items in deterministic presentation order.
    #[must_use]
    pub fn new(items: impl Into<Vec<TreeItem>>) -> Self {
        Self {
            items: items.into(),
        }
    }

    /// Returns all items in source order.
    #[must_use]
    pub fn items(&self) -> &[TreeItem] {
        &self.items
    }

    /// Returns the number of items in the model.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true when the model has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Validates tree identity and parent-link invariants.
    ///
    /// # Errors
    ///
    /// Returns [`TreeModelError`] for duplicate IDs, unknown parents, self
    /// parents, or cyclic parent links.
    pub fn validate(&self) -> Result<(), TreeModelError> {
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if !ids.insert(item.id) {
                return Err(TreeModelError::DuplicateItemId { id: item.id });
            }
        }

        for item in &self.items {
            if item.parent == Some(item.id) {
                return Err(TreeModelError::SelfParent { id: item.id });
            }
            if let Some(parent) = item.parent
                && !ids.contains(&parent)
            {
                return Err(TreeModelError::UnknownParent {
                    id: item.id,
                    parent,
                });
            }
        }

        let index_by_id = self.index_by_id();
        for item in &self.items {
            let mut visited = BTreeSet::new();
            let mut current = Some(item.id);
            while let Some(id) = current {
                if !visited.insert(id) {
                    return Err(TreeModelError::Cycle { id });
                }
                current = self.items[index_by_id[&id]].parent;
            }
        }

        Ok(())
    }

    /// Returns direct child IDs for a parent in source order.
    #[must_use]
    pub fn child_ids(&self, parent: Option<ItemId>) -> Vec<ItemId> {
        self.items
            .iter()
            .filter(|item| item.parent == parent)
            .map(|item| item.id)
            .collect()
    }

    /// Returns all descendant IDs for an item in source order.
    #[must_use]
    pub fn descendant_ids(&self, item: ItemId) -> Vec<ItemId> {
        let children_by_parent = self.children_by_parent();
        let mut descendants = Vec::new();
        let mut visited = BTreeSet::new();
        collect_descendant_ids(
            item,
            &children_by_parent,
            &self.items,
            &mut visited,
            &mut descendants,
        );
        descendants
    }

    /// Computes visible tree rows from the current expansion state.
    ///
    /// Invalid models return no visible rows; call [`Self::validate`] to
    /// distinguish an empty tree from a malformed one.
    #[must_use]
    pub fn visible_rows(&self, expansion: &TreeExpansion) -> Vec<TreeRow> {
        if self.validate().is_err() {
            return Vec::new();
        }

        let children_by_parent = self.children_by_parent();
        let mut rows = Vec::new();
        let mut visited = BTreeSet::new();
        self.push_visible_children(
            None,
            0,
            expansion,
            &children_by_parent,
            &mut visited,
            &mut rows,
        );
        rows
    }

    /// Counts visible tree rows from the current expansion state without
    /// materializing row metadata.
    ///
    /// Invalid models return zero visible rows for the same reason as
    /// [`Self::visible_rows`].
    #[must_use]
    pub fn visible_row_count(&self, expansion: &TreeExpansion) -> usize {
        if self.validate().is_err() {
            return 0;
        }

        let children_by_parent = self.children_by_parent();
        let mut visited = BTreeSet::new();
        self.count_visible_children(None, expansion, &children_by_parent, &mut visited)
    }

    /// Collects visible tree rows inside a global visible row range.
    ///
    /// Returned rows preserve their global visible row indices, so a request
    /// for `10..20` returns rows whose [`TreeRow::row`] values remain in that
    /// same global range instead of being rebased to zero.
    ///
    /// Invalid models return no visible rows; call [`Self::validate`] when the
    /// caller needs diagnostics.
    #[must_use]
    pub fn visible_rows_in_range(
        &self,
        expansion: &TreeExpansion,
        range: Range<usize>,
    ) -> Vec<TreeRow> {
        if range.start >= range.end || self.validate().is_err() {
            return Vec::new();
        }

        let children_by_parent = self.children_by_parent();
        let mut rows = Vec::with_capacity(range.end.saturating_sub(range.start).min(self.len()));
        let mut visited = BTreeSet::new();
        let mut next_row = 0;
        self.push_visible_children_in_range(
            None,
            0,
            expansion,
            &children_by_parent,
            &mut visited,
            &range,
            &mut next_row,
            &mut rows,
        );
        rows
    }

    /// Computes visible item IDs from the current expansion state.
    ///
    /// Invalid models return no visible item IDs for the same reason as
    /// [`Self::visible_rows`].
    #[must_use]
    pub fn visible_item_ids(&self, expansion: &TreeExpansion) -> Vec<ItemId> {
        self.visible_rows(expansion)
            .into_iter()
            .map(|row| row.id)
            .collect()
    }

    /// Computes visible tree rows after applying an app-owned filter.
    ///
    /// Matching descendants keep their ancestors visible, but expansion state
    /// is read-only and preserved by the caller. Collapsed ancestors therefore
    /// still hide their descendants until the app expands them.
    #[must_use]
    pub fn filtered_visible_rows(
        &self,
        expansion: &TreeExpansion,
        mut include: impl FnMut(&TreeItem) -> bool,
    ) -> Vec<TreeRow> {
        if self.validate().is_err() {
            return Vec::new();
        }

        let index_by_id = self.index_by_id();
        let mut included = BTreeSet::new();
        for item in &self.items {
            if include(item) {
                let mut current = Some(item.id);
                while let Some(id) = current {
                    if !included.insert(id) {
                        break;
                    }
                    current = self.items[index_by_id[&id]].parent;
                }
            }
        }

        self.visible_rows(expansion)
            .into_iter()
            .filter(|row| included.contains(&row.id))
            .enumerate()
            .map(|(row_index, mut row)| {
                row.row = row_index;
                row
            })
            .collect()
    }

    fn visible_row_for_item(
        &self,
        item_index: usize,
        row: usize,
        depth: usize,
        expansion: &TreeExpansion,
        children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
    ) -> TreeRow {
        let item = self.items[item_index];
        let (has_children, expanded) = visible_item_state(item, expansion, children_by_parent);
        TreeRow {
            row,
            item_index,
            id: item.id,
            parent: item.parent,
            depth,
            has_children,
            expanded,
        }
    }

    fn index_by_id(&self) -> BTreeMap<ItemId, usize> {
        self.items
            .iter()
            .enumerate()
            .map(|(index, item)| (item.id, index))
            .collect()
    }

    fn children_by_parent(&self) -> BTreeMap<Option<ItemId>, Vec<usize>> {
        let mut children = BTreeMap::<Option<ItemId>, Vec<usize>>::new();
        for (index, item) in self.items.iter().enumerate() {
            children.entry(item.parent).or_default().push(index);
        }
        children
    }

    fn push_visible_children(
        &self,
        parent: Option<ItemId>,
        depth: usize,
        expansion: &TreeExpansion,
        children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
        visited: &mut BTreeSet<ItemId>,
        rows: &mut Vec<TreeRow>,
    ) {
        let Some(children) = children_by_parent.get(&parent) else {
            return;
        };

        for index in children {
            let item = self.items[*index];
            if !visited.insert(item.id) {
                continue;
            }

            let row =
                self.visible_row_for_item(*index, rows.len(), depth, expansion, children_by_parent);
            rows.push(row);

            if row.expanded {
                self.push_visible_children(
                    Some(item.id),
                    depth.saturating_add(1),
                    expansion,
                    children_by_parent,
                    visited,
                    rows,
                );
            }
        }
    }

    fn count_visible_children(
        &self,
        parent: Option<ItemId>,
        expansion: &TreeExpansion,
        children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
        visited: &mut BTreeSet<ItemId>,
    ) -> usize {
        let Some(children) = children_by_parent.get(&parent) else {
            return 0;
        };

        let mut count = 0usize;
        for index in children {
            let item = self.items[*index];
            if !visited.insert(item.id) {
                continue;
            }

            let (_, expanded) = visible_item_state(item, expansion, children_by_parent);
            count = count.saturating_add(1);
            if expanded {
                count = count.saturating_add(self.count_visible_children(
                    Some(item.id),
                    expansion,
                    children_by_parent,
                    visited,
                ));
            }
        }
        count
    }

    #[allow(clippy::too_many_arguments)]
    fn push_visible_children_in_range(
        &self,
        parent: Option<ItemId>,
        depth: usize,
        expansion: &TreeExpansion,
        children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
        visited: &mut BTreeSet<ItemId>,
        range: &Range<usize>,
        next_row: &mut usize,
        rows: &mut Vec<TreeRow>,
    ) {
        let Some(children) = children_by_parent.get(&parent) else {
            return;
        };

        for index in children {
            if *next_row >= range.end {
                return;
            }

            let item = self.items[*index];
            if !visited.insert(item.id) {
                continue;
            }

            let row =
                self.visible_row_for_item(*index, *next_row, depth, expansion, children_by_parent);
            if range.contains(&row.row) {
                rows.push(row);
            }
            *next_row = next_row.saturating_add(1);

            if row.expanded {
                self.push_visible_children_in_range(
                    Some(item.id),
                    depth.saturating_add(1),
                    expansion,
                    children_by_parent,
                    visited,
                    range,
                    next_row,
                    rows,
                );
            }
        }
    }
}

fn visible_item_state(
    item: TreeItem,
    expansion: &TreeExpansion,
    children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
) -> (bool, bool) {
    let has_loaded_children = children_by_parent.contains_key(&Some(item.id));
    let has_children = item.has_children || has_loaded_children;
    let expanded = has_children && expansion.is_expanded(item.id);
    (has_children, expanded)
}

fn collect_descendant_ids(
    parent: ItemId,
    children_by_parent: &BTreeMap<Option<ItemId>, Vec<usize>>,
    items: &[TreeItem],
    visited: &mut BTreeSet<ItemId>,
    descendants: &mut Vec<ItemId>,
) {
    let Some(children) = children_by_parent.get(&Some(parent)) else {
        return;
    };
    for index in children {
        let child = items[*index].id;
        if !visited.insert(child) {
            continue;
        }
        descendants.push(child);
        collect_descendant_ids(child, children_by_parent, items, visited, descendants);
    }
}

/// Retained tree expansion state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeExpansion {
    expanded: BTreeSet<ItemId>,
}

impl TreeExpansion {
    /// Creates empty expansion state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when an item is expanded.
    #[must_use]
    pub fn is_expanded(&self, item: ItemId) -> bool {
        self.expanded.contains(&item)
    }

    /// Returns expanded item IDs in sorted order.
    #[must_use]
    pub fn expanded(&self) -> Vec<ItemId> {
        self.expanded.iter().copied().collect()
    }

    /// Expands an item.
    pub fn expand(&mut self, item: ItemId) -> bool {
        self.expanded.insert(item)
    }

    /// Collapses an item.
    pub fn collapse(&mut self, item: ItemId) -> bool {
        self.expanded.remove(&item)
    }

    /// Toggles expansion and returns whether the item is now expanded.
    pub fn toggle(&mut self, item: ItemId) -> bool {
        if self.expanded.remove(&item) {
            false
        } else {
            self.expanded.insert(item);
            true
        }
    }

    /// Collapses an item and any currently expanded descendants.
    pub fn collapse_descendants(&mut self, model: &TreeModel, item: ItemId) -> bool {
        let mut changed = self.collapse(item);
        for descendant in model.descendant_ids(item) {
            changed |= self.collapse(descendant);
        }
        changed
    }

    /// Removes expanded IDs that are not present in a valid tree model.
    ///
    /// Invalid models clear expansion state. This keeps cleanup deterministic
    /// and avoids interpreting malformed parent links.
    pub fn retain_model(&mut self, model: &TreeModel) -> bool {
        if model.validate().is_err() {
            return self.clear_changed();
        }

        let ids = model
            .items
            .iter()
            .map(|item| item.id)
            .collect::<BTreeSet<_>>();
        self.retain_ids(&ids)
    }

    /// Removes expanded IDs that are not currently visible in the tree.
    ///
    /// This also removes stale IDs, because unknown IDs cannot appear in the
    /// model's visible row list. Invalid models clear expansion state.
    pub fn retain_visible(&mut self, model: &TreeModel) -> bool {
        if model.validate().is_err() {
            return self.clear_changed();
        }

        let visible = model
            .visible_item_ids(self)
            .into_iter()
            .collect::<BTreeSet<_>>();
        self.retain_ids(&visible)
    }

    /// Clears all expansion state.
    pub fn clear(&mut self) {
        self.expanded.clear();
    }

    fn retain_ids(&mut self, ids: &BTreeSet<ItemId>) -> bool {
        let before = self.expanded.len();
        self.expanded.retain(|item| ids.contains(item));
        self.expanded.len() != before
    }

    fn clear_changed(&mut self) -> bool {
        let changed = !self.expanded.is_empty();
        self.clear();
        changed
    }
}
