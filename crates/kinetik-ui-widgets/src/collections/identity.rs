use std::cmp::Ordering;

use kinetik_ui_core::Rect;

/// Stable collection item identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(u64);

impl ItemId {
    /// Creates an item ID from raw bits.
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

/// Rectangle assigned to an item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemRect {
    /// Item index.
    pub index: usize,
    /// Item rectangle.
    pub rect: Rect,
}

/// One source item resolved into a projected collection order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionProjectedItem {
    /// Stable item identity.
    pub id: ItemId,
    /// Index of the item in the application-owned source collection.
    pub source_index: usize,
}

impl CollectionProjectedItem {
    /// Creates a projected item entry.
    #[must_use]
    pub const fn new(id: ItemId, source_index: usize) -> Self {
        Self { id, source_index }
    }
}

/// Data-only filtered and sorted projection of a source collection.
///
/// Projection stores stable item IDs and source indices only; applications
/// retain ownership of the source data and decide filtering and sorting keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionProjection {
    items: Vec<CollectionProjectedItem>,
}

impl CollectionProjection {
    /// Creates an empty projection.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Creates an identity projection from source item IDs.
    #[must_use]
    pub fn from_source_ids(source_ids: &[ItemId]) -> Self {
        Self {
            items: source_ids
                .iter()
                .copied()
                .enumerate()
                .map(|(source_index, id)| CollectionProjectedItem::new(id, source_index))
                .collect(),
        }
    }

    /// Creates a projection from pre-resolved projected items.
    #[must_use]
    pub fn from_items(items: impl IntoIterator<Item = CollectionProjectedItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
        }
    }

    /// Returns projected items in visible order.
    #[must_use]
    pub fn items(&self) -> &[CollectionProjectedItem] {
        &self.items
    }

    /// Returns the projected item count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true when no items are visible through this projection.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns projected IDs in visible order.
    #[must_use]
    pub fn visible_ids(&self) -> Vec<ItemId> {
        self.items.iter().map(|item| item.id).collect()
    }

    /// Returns source indices in visible order.
    #[must_use]
    pub fn source_indices(&self) -> Vec<usize> {
        self.items.iter().map(|item| item.source_index).collect()
    }

    /// Returns projected IDs within a virtualized projected range.
    #[must_use]
    pub fn ids_in_range(&self, range: std::ops::Range<usize>) -> Vec<ItemId> {
        self.items
            .get(range.start.min(self.items.len())..range.end.min(self.items.len()))
            .unwrap_or_default()
            .iter()
            .map(|item| item.id)
            .collect()
    }

    /// Returns a projected item by projected index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<CollectionProjectedItem> {
        self.items.get(index).copied()
    }

    /// Returns the first source index for an item ID.
    #[must_use]
    pub fn source_index(&self, id: ItemId) -> Option<usize> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.source_index)
    }

    /// Returns the first projected index for an item ID.
    #[must_use]
    pub fn projected_index(&self, id: ItemId) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    /// Returns a filtered copy of this projection.
    #[must_use]
    pub fn filtered_by(&self, mut include: impl FnMut(CollectionProjectedItem) -> bool) -> Self {
        Self {
            items: self
                .items
                .iter()
                .copied()
                .filter(|item| include(*item))
                .collect(),
        }
    }

    /// Returns a sorted copy of this projection.
    ///
    /// Sorting is stable. Equal comparisons keep the incoming projection order.
    #[must_use]
    pub fn sorted_by(
        &self,
        mut compare: impl FnMut(&CollectionProjectedItem, &CollectionProjectedItem) -> Ordering,
    ) -> Self {
        let mut items = self.items.clone();
        items.sort_by(|lhs, rhs| compare(lhs, rhs));
        Self { items }
    }
}
