//! Deterministic width-based projection for compact editor chrome.

/// One caller-owned chrome item considered for overflow projection.
#[derive(Debug, Clone, PartialEq)]
pub struct ChromeOverflowItem<K> {
    /// Stable caller key preserved in projection output.
    pub key: K,
    /// Desired horizontal extent in logical units.
    pub desired_width: f32,
    /// Whether the item participates in the projection.
    pub visible: bool,
}

impl<K> ChromeOverflowItem<K> {
    /// Creates a visible item with a stable caller key and desired width.
    #[must_use]
    pub const fn new(key: K, desired_width: f32) -> Self {
        Self {
            key,
            desired_width,
            visible: true,
        }
    }

    /// Sets whether the item participates in the projection.
    #[must_use]
    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

/// Local horizontal placement for one visible chrome item.
#[derive(Debug, Clone, PartialEq)]
pub struct ChromeOverflowPlacement<K> {
    /// Stable caller key copied from the source item.
    pub key: K,
    /// Local horizontal origin in logical units.
    pub x: f32,
    /// Sanitized horizontal extent in logical units.
    pub width: f32,
}

/// Local horizontal placement reserved for the overflow trigger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChromeOverflowTrigger {
    /// Local horizontal origin in logical units.
    pub x: f32,
    /// Sanitized horizontal extent in logical units.
    pub width: f32,
}

/// Deterministic visible-prefix and overflow-tail projection.
#[derive(Debug, Clone, PartialEq)]
pub struct ChromeOverflowProjection<K> {
    visible: Vec<ChromeOverflowPlacement<K>>,
    overflowed: Vec<K>,
    trigger: Option<ChromeOverflowTrigger>,
}

impl<K> ChromeOverflowProjection<K> {
    /// Returns visible item placements in source order.
    #[must_use]
    pub fn visible(&self) -> &[ChromeOverflowPlacement<K>] {
        &self.visible
    }

    /// Returns overflowed stable keys in source order.
    #[must_use]
    pub fn overflowed(&self) -> &[K] {
        &self.overflowed
    }

    /// Returns the overflow-trigger placement when an overflow tail exists.
    #[must_use]
    pub const fn trigger(&self) -> Option<ChromeOverflowTrigger> {
        self.trigger
    }

    /// Returns true when at least one visible source item is overflowed.
    #[must_use]
    pub fn has_overflow(&self) -> bool {
        !self.overflowed.is_empty()
    }
}

/// Projects ordered chrome items into a visible prefix and overflowed tail.
///
/// Hidden items are excluded before width resolution. Invalid or negative
/// widths resolve to zero. When all participating items fit, no trigger is
/// returned. Otherwise, the trigger width is reserved before fitting the
/// longest source-order prefix; after the first item does not fit, the entire
/// remaining tail overflows.
#[must_use]
pub fn project_chrome_overflow<K>(
    items: impl IntoIterator<Item = ChromeOverflowItem<K>>,
    available_width: f32,
    trigger_width: f32,
) -> ChromeOverflowProjection<K> {
    let available_width = finite_non_negative(available_width);
    let items = items
        .into_iter()
        .filter(|item| item.visible)
        .map(|mut item| {
            item.desired_width = finite_non_negative(item.desired_width);
            item
        })
        .collect::<Vec<_>>();

    if prefix_extent(&items, available_width).is_some() {
        return ChromeOverflowProjection {
            visible: place_prefix(items),
            overflowed: Vec::new(),
            trigger: None,
        };
    }

    let trigger_width = finite_non_negative(trigger_width).min(available_width);
    let mut item_budget = available_width - trigger_width;
    while checked_end(item_budget, trigger_width, available_width).is_none() {
        item_budget = previous_non_negative(item_budget);
    }
    let mut x = 0.0;
    let mut fitting = true;
    let mut visible = Vec::new();
    let mut overflowed = Vec::new();

    for item in items {
        let end = fitting
            .then(|| checked_end(x, item.desired_width, item_budget))
            .flatten();
        if let Some(end) = end {
            visible.push(ChromeOverflowPlacement {
                key: item.key,
                x,
                width: item.desired_width,
            });
            x = end;
        } else {
            fitting = false;
            overflowed.push(item.key);
        }
    }

    ChromeOverflowProjection {
        visible,
        overflowed,
        trigger: Some(ChromeOverflowTrigger {
            x,
            width: trigger_width,
        }),
    }
}

fn prefix_extent<K>(items: &[ChromeOverflowItem<K>], limit: f32) -> Option<f32> {
    items
        .iter()
        .try_fold(0.0, |x, item| checked_end(x, item.desired_width, limit))
}

fn place_prefix<K>(items: Vec<ChromeOverflowItem<K>>) -> Vec<ChromeOverflowPlacement<K>> {
    let mut x = 0.0;
    items
        .into_iter()
        .map(|item| {
            let placement = ChromeOverflowPlacement {
                key: item.key,
                x,
                width: item.desired_width,
            };
            x = checked_end(x, item.desired_width, f32::MAX)
                .expect("a previously verified full prefix must remain finite");
            placement
        })
        .collect()
}

fn checked_end(x: f32, width: f32, limit: f32) -> Option<f32> {
    let end = x + width;
    end.is_finite().then_some(end).filter(|end| *end <= limit)
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn previous_non_negative(value: f32) -> f32 {
    if value > 0.0 {
        f32::from_bits(value.to_bits() - 1)
    } else {
        0.0
    }
}
