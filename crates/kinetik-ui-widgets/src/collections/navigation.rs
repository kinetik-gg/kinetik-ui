use super::{CollectionProjection, ItemId};

/// Bounded vertical movement over a projected collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionCursorMove {
    /// Move to the first projected item.
    First,
    /// Move to the preceding projected item without wrapping.
    Previous,
    /// Move to the next projected item without wrapping.
    Next,
    /// Move to the last projected item.
    Last,
    /// Move toward the start by a caller-defined visible row count.
    PagePrevious {
        /// Number of projected rows in one page.
        rows: usize,
    },
    /// Move toward the end by a caller-defined visible row count.
    PageNext {
        /// Number of projected rows in one page.
        rows: usize,
    },
}

/// One resolved collection cursor target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionCursorTarget {
    /// Stable item identity.
    pub id: ItemId,
    /// Item index in the current collection projection.
    pub projected_index: usize,
}

/// Retained vertical cursor for filtered, sorted, or virtualized collections.
///
/// The cursor retains stable item identity plus its last projected index. When
/// an active item disappears, reconciliation chooses the item now occupying
/// that index, falling back to the preceding tail when the projection shrinks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionCursor {
    active: Option<ItemId>,
    last_projected_index: Option<usize>,
}

impl CollectionCursor {
    /// Creates an empty collection cursor.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: None,
            last_projected_index: None,
        }
    }

    /// Returns the active stable item identity.
    #[must_use]
    pub const fn active(&self) -> Option<ItemId> {
        self.active
    }

    /// Returns the active item's last resolved projected index.
    #[must_use]
    pub const fn last_projected_index(&self) -> Option<usize> {
        self.last_projected_index
    }

    /// Clears active identity and projected position.
    pub fn clear(&mut self) {
        self.active = None;
        self.last_projected_index = None;
    }

    /// Activates an item when it is present in the current projection.
    ///
    /// An absent ID leaves the current cursor unchanged.
    pub fn activate(
        &mut self,
        projection: &CollectionProjection,
        id: ItemId,
    ) -> Option<CollectionCursorTarget> {
        let projected_index = projection.projected_index(id)?;
        self.set_index(projection, projected_index)
    }

    /// Reconciles retained identity with the current projection.
    ///
    /// Reordering preserves active identity. Removal or filtering repairs to
    /// the old projected slot, then to the preceding tail. An empty projection
    /// clears the cursor. An already-empty cursor remains empty.
    pub fn reconcile(
        &mut self,
        projection: &CollectionProjection,
    ) -> Option<CollectionCursorTarget> {
        if projection.is_empty() {
            self.clear();
            return None;
        }

        let active = self.active?;
        let projected_index = projection.projected_index(active).unwrap_or_else(|| {
            self.last_projected_index
                .unwrap_or_default()
                .min(projection.len() - 1)
        });
        self.set_index(projection, projected_index)
    }

    /// Applies bounded movement over the current projection.
    ///
    /// If the cursor is empty, forward movement starts at the first item and
    /// backward movement starts at the last item. A retained but hidden item is
    /// reconciled before movement is applied.
    pub fn navigate(
        &mut self,
        projection: &CollectionProjection,
        movement: CollectionCursorMove,
    ) -> Option<CollectionCursorTarget> {
        if projection.is_empty() {
            self.clear();
            return None;
        }

        let last_index = projection.len() - 1;
        let current = self
            .reconcile(projection)
            .map(|target| target.projected_index);
        let projected_index = match (current, movement) {
            (_, CollectionCursorMove::First)
            | (None, CollectionCursorMove::Next | CollectionCursorMove::PageNext { .. }) => 0,
            (_, CollectionCursorMove::Last)
            | (None, CollectionCursorMove::Previous | CollectionCursorMove::PagePrevious { .. }) => {
                last_index
            }
            (Some(index), CollectionCursorMove::Previous) => index.saturating_sub(1),
            (Some(index), CollectionCursorMove::Next) => index.saturating_add(1).min(last_index),
            (Some(index), CollectionCursorMove::PagePrevious { rows }) => {
                index.saturating_sub(rows)
            }
            (Some(index), CollectionCursorMove::PageNext { rows }) => {
                index.saturating_add(rows).min(last_index)
            }
        };

        self.set_index(projection, projected_index)
    }

    fn set_index(
        &mut self,
        projection: &CollectionProjection,
        projected_index: usize,
    ) -> Option<CollectionCursorTarget> {
        let item = projection.get(projected_index)?;
        self.active = Some(item.id);
        self.last_projected_index = Some(projected_index);
        Some(CollectionCursorTarget {
            id: item.id,
            projected_index,
        })
    }
}
