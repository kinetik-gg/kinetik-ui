//! Inspector and property-grid layout primitives.

use std::collections::BTreeSet;
use std::ops::Range;

use kinetik_ui_core::Rect;

use crate::collections::ItemId;

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_positive(value: f32) -> Option<f32> {
    value
        .is_finite()
        .then_some(value)
        .filter(|value| *value > 0.0)
}

/// Property-grid row kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyGridRowKind {
    /// Section heading row.
    Section,
    /// Editable property row.
    Property {
        /// Nesting depth for grouped properties.
        depth: usize,
    },
}

/// Validation or help status severity for a property-grid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyGridStatusSeverity {
    /// No status is attached to the row.
    #[default]
    None,
    /// Informational row status.
    Info,
    /// Non-blocking warning row status.
    Warning,
    /// Blocking error row status.
    Error,
}

/// Data-only validation or help status for a property-grid row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyGridRowStatus {
    /// Status severity.
    pub severity: PropertyGridStatusSeverity,
    /// Optional status message owned by the application.
    pub message: Option<String>,
}

impl PropertyGridRowStatus {
    /// Creates a row status with the given severity and no message.
    #[must_use]
    pub const fn severity(severity: PropertyGridStatusSeverity) -> Self {
        Self {
            severity,
            message: None,
        }
    }

    /// Creates an informational row status.
    #[must_use]
    pub fn info(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Info).with_message(message)
    }

    /// Creates a warning row status.
    #[must_use]
    pub fn warning(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Warning).with_message(message)
    }

    /// Creates an error row status.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::severity(PropertyGridStatusSeverity::Error).with_message(message)
    }

    /// Returns this status with an attached message.
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Returns true when this status represents a blocking error.
    #[must_use]
    pub const fn is_blocking_error(&self) -> bool {
        matches!(self.severity, PropertyGridStatusSeverity::Error)
    }
}

/// Data-only form state metadata for a property-grid row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PropertyGridRowState {
    /// True when the row should not accept interaction.
    pub disabled: bool,
    /// True when the row value should be presented as non-editable.
    pub read_only: bool,
    /// True when the row represents a required property.
    pub required: bool,
    /// Optional help text owned by the application.
    pub help_text: Option<String>,
    /// Optional validation or help status owned by the application.
    pub status: PropertyGridRowStatus,
}

impl PropertyGridRowState {
    /// Creates neutral row state metadata.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            disabled: false,
            read_only: false,
            required: false,
            help_text: None,
            status: PropertyGridRowStatus::severity(PropertyGridStatusSeverity::None),
        }
    }

    /// Returns this metadata with disabled state set.
    #[must_use]
    pub const fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns this metadata with read-only state set.
    #[must_use]
    pub const fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Returns this metadata with required state set.
    #[must_use]
    pub const fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Returns this metadata with help text attached.
    #[must_use]
    pub fn with_help_text(mut self, help_text: impl Into<String>) -> Self {
        self.help_text = Some(help_text.into());
        self
    }

    /// Returns this metadata with status attached.
    #[must_use]
    pub fn with_status(mut self, status: PropertyGridRowStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns true when this metadata carries a blocking error status.
    #[must_use]
    pub const fn has_blocking_error(&self) -> bool {
        self.status.is_blocking_error()
    }
}

/// One property-grid row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyGridRow {
    /// Stable row identity.
    pub id: ItemId,
    /// User-visible row label.
    pub label: String,
    /// Row kind.
    pub kind: PropertyGridRowKind,
    /// Data-only row state metadata.
    pub state: PropertyGridRowState,
}

impl PropertyGridRow {
    /// Creates a section heading row.
    #[must_use]
    pub fn section(id: ItemId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            kind: PropertyGridRowKind::Section,
            state: PropertyGridRowState::neutral(),
        }
    }

    /// Creates an editable property row.
    #[must_use]
    pub fn property(id: ItemId, label: impl Into<String>, depth: usize) -> Self {
        Self {
            id,
            label: label.into(),
            kind: PropertyGridRowKind::Property { depth },
            state: PropertyGridRowState::neutral(),
        }
    }

    /// Returns this row with state metadata attached.
    #[must_use]
    pub fn with_state(mut self, state: PropertyGridRowState) -> Self {
        self.state = state;
        self
    }

    /// Returns this row with disabled state set.
    #[must_use]
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.state = self.state.with_disabled(disabled);
        self
    }

    /// Returns this row with read-only state set.
    #[must_use]
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.state = self.state.with_read_only(read_only);
        self
    }

    /// Returns this row with required state set.
    #[must_use]
    pub fn with_required(mut self, required: bool) -> Self {
        self.state = self.state.with_required(required);
        self
    }

    /// Returns this row with help text attached.
    #[must_use]
    pub fn with_help_text(mut self, help_text: impl Into<String>) -> Self {
        self.state = self.state.with_help_text(help_text);
        self
    }

    /// Returns this row with status attached.
    #[must_use]
    pub fn with_status(mut self, status: PropertyGridRowStatus) -> Self {
        self.state = self.state.with_status(status);
        self
    }

    /// Returns true when this row can accept interaction.
    #[must_use]
    pub fn is_interactable(&self) -> bool {
        matches!(self.kind, PropertyGridRowKind::Property { .. }) && !self.state.disabled
    }

    /// Returns true when this row represents an editable property value.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        self.is_interactable() && !self.state.read_only
    }

    /// Returns true when this row carries a blocking error status.
    #[must_use]
    pub fn has_blocking_error(&self) -> bool {
        self.state.has_blocking_error()
    }
}

/// Property-grid structural error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyGridError {
    /// More than one row uses the same ID.
    DuplicateRowId {
        /// Duplicated row identity.
        id: ItemId,
    },
}

/// Rectangle assigned to one property-grid row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyGridRowRect {
    /// Source row index.
    pub index: usize,
    /// Stable row identity.
    pub id: ItemId,
    /// Row kind.
    pub kind: PropertyGridRowKind,
    /// Full row rectangle.
    pub rect: Rect,
    /// Label or section-title rectangle.
    pub label_rect: Rect,
    /// Value/control rectangle.
    pub value_rect: Rect,
}

/// Inspector-style property-grid layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyGridLayout {
    /// Regular property row height.
    pub row_height: f32,
    /// Section heading row height.
    pub section_height: f32,
    /// Preferred label column width.
    pub label_width: f32,
    /// Gap between label and value columns.
    pub column_gap: f32,
    /// Per-depth indentation.
    pub indent_width: f32,
}

impl PropertyGridLayout {
    /// Creates a property-grid layout.
    #[must_use]
    pub const fn new(
        row_height: f32,
        section_height: f32,
        label_width: f32,
        column_gap: f32,
        indent_width: f32,
    ) -> Self {
        Self {
            row_height,
            section_height,
            label_width,
            column_gap,
            indent_width,
        }
    }

    /// Returns the sanitized property row height.
    #[must_use]
    pub fn effective_row_height(self) -> Option<f32> {
        finite_positive(self.row_height)
    }

    /// Returns the sanitized section heading height.
    #[must_use]
    pub fn effective_section_height(self) -> Option<f32> {
        finite_positive(self.section_height)
    }

    /// Returns the sanitized label column width.
    #[must_use]
    pub fn effective_label_width(self) -> f32 {
        finite_non_negative(self.label_width)
    }

    /// Returns the sanitized gap between label and value columns.
    #[must_use]
    pub fn effective_column_gap(self) -> f32 {
        finite_non_negative(self.column_gap)
    }

    /// Returns the sanitized per-depth indentation.
    #[must_use]
    pub fn effective_indent_width(self) -> f32 {
        finite_non_negative(self.indent_width)
    }

    /// Validates row identity invariants.
    ///
    /// # Errors
    ///
    /// Returns [`PropertyGridError`] when duplicate row IDs are present.
    pub fn validate_rows(rows: &[PropertyGridRow]) -> Result<(), PropertyGridError> {
        let mut ids = BTreeSet::new();
        for row in rows {
            if !ids.insert(row.id) {
                return Err(PropertyGridError::DuplicateRowId { id: row.id });
            }
        }
        Ok(())
    }

    /// Computes the height for one row kind.
    #[must_use]
    pub fn row_extent(self, kind: PropertyGridRowKind) -> f32 {
        match kind {
            PropertyGridRowKind::Section => self.effective_section_height(),
            PropertyGridRowKind::Property { .. } => self.effective_row_height(),
        }
        .unwrap_or(0.0)
    }

    /// Computes total content height.
    #[must_use]
    pub fn content_height(self, rows: &[PropertyGridRow]) -> f32 {
        rows.iter()
            .map(|row| self.row_extent(row.kind))
            .sum::<f32>()
    }

    /// Computes the maximum vertical scroll offset.
    #[must_use]
    pub fn max_scroll_offset(self, rows: &[PropertyGridRow], viewport_height: f32) -> f32 {
        (self.content_height(rows) - finite_non_negative(viewport_height)).max(0.0)
    }

    /// Clamps a vertical scroll offset to the valid range.
    #[must_use]
    pub fn clamp_scroll_offset(
        self,
        rows: &[PropertyGridRow],
        viewport_height: f32,
        scroll_offset: f32,
    ) -> f32 {
        finite_non_negative(scroll_offset).min(self.max_scroll_offset(rows, viewport_height))
    }

    /// Computes visible row indexes for a viewport.
    #[must_use]
    pub fn visible_range(
        self,
        rows: &[PropertyGridRow],
        scroll_offset: f32,
        viewport_height: f32,
        overscan: usize,
    ) -> Range<usize> {
        let Some(viewport_height) = finite_positive(viewport_height) else {
            return 0..0;
        };
        if rows.is_empty() {
            return 0..0;
        }
        if self.content_height(rows) <= 0.0 {
            return 0..0;
        }

        let scroll_offset = self.clamp_scroll_offset(rows, viewport_height, scroll_offset);
        let viewport_end = scroll_offset + viewport_height;
        let mut y = 0.0;
        let mut start = None;
        let mut end = rows.len();

        for (index, row) in rows.iter().enumerate() {
            let height = self.row_extent(row.kind);
            let row_end = y + height;
            if start.is_none() && row_end > scroll_offset {
                start = Some(index);
            }
            if y >= viewport_end {
                end = index;
                break;
            }
            y = row_end;
        }

        let start = start.unwrap_or(rows.len()).saturating_sub(overscan);
        let end = end.saturating_add(overscan).min(rows.len());
        start..end
    }

    /// Computes row rectangles in viewport coordinates.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn visible_row_rects(
        self,
        bounds: Rect,
        rows: &[PropertyGridRow],
        scroll_offset: f32,
        overscan: usize,
    ) -> Vec<PropertyGridRowRect> {
        let scroll_offset = self.clamp_scroll_offset(rows, bounds.height, scroll_offset);
        let visible = self.visible_range(rows, scroll_offset, bounds.height, overscan);
        let mut y = bounds.y - scroll_offset;
        for row in rows.iter().take(visible.start) {
            y += self.row_extent(row.kind);
        }

        visible
            .map(|index| {
                let row = &rows[index];
                let height = self.row_extent(row.kind);
                let rect = Rect::new(
                    bounds.x,
                    y,
                    finite_non_negative(bounds.width),
                    finite_non_negative(height),
                );
                y += height;
                self.row_rect(index, row, rect)
            })
            .collect()
    }

    #[allow(clippy::cast_precision_loss)]
    fn row_rect(self, index: usize, row: &PropertyGridRow, rect: Rect) -> PropertyGridRowRect {
        match row.kind {
            PropertyGridRowKind::Section => PropertyGridRowRect {
                index,
                id: row.id,
                kind: row.kind,
                rect,
                label_rect: rect,
                value_rect: Rect::new(rect.max_x(), rect.y, 0.0, rect.height),
            },
            PropertyGridRowKind::Property { depth } => {
                let indent = depth as f32 * self.effective_indent_width();
                let x = rect.x + indent;
                let available = (rect.width - indent).max(0.0);
                let label_width = self.effective_label_width().min(available);
                let gap = if available > label_width {
                    self.effective_column_gap().min(available - label_width)
                } else {
                    0.0
                };
                let value_x = x + label_width + gap;
                let value_width = (rect.max_x() - value_x).max(0.0);
                PropertyGridRowRect {
                    index,
                    id: row.id,
                    kind: row.kind,
                    rect,
                    label_rect: Rect::new(x, rect.y, label_width, rect.height),
                    value_rect: Rect::new(value_x, rect.y, value_width, rect.height),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PropertyGridError, PropertyGridLayout, PropertyGridRow, PropertyGridRowState,
        PropertyGridRowStatus, PropertyGridStatusSeverity,
    };
    use crate::ItemId;
    use kinetik_ui_core::Rect;

    fn assert_approx(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {actual} to equal {expected}"
        );
    }

    fn rows() -> Vec<PropertyGridRow> {
        vec![
            PropertyGridRow::section(ItemId::from_raw(1), "Transform"),
            PropertyGridRow::property(ItemId::from_raw(2), "Position", 0),
            PropertyGridRow::property(ItemId::from_raw(3), "X", 1),
            PropertyGridRow::property(ItemId::from_raw(4), "Y", 1),
        ]
    }

    #[test]
    fn property_grid_validates_duplicate_row_ids() {
        let rows = vec![
            PropertyGridRow::property(ItemId::from_raw(1), "A", 0)
                .with_status(PropertyGridRowStatus::warning("Check value")),
            PropertyGridRow::property(ItemId::from_raw(1), "B", 0)
                .with_disabled(true)
                .with_required(true),
        ];

        assert_eq!(
            PropertyGridLayout::validate_rows(&rows),
            Err(PropertyGridError::DuplicateRowId {
                id: ItemId::from_raw(1)
            })
        );
    }

    #[test]
    fn property_grid_row_metadata_defaults_to_neutral_state() {
        let section = PropertyGridRow::section(ItemId::from_raw(1), "Transform");
        let property = PropertyGridRow::property(ItemId::from_raw(2), "Position", 0);

        assert_eq!(section.state, PropertyGridRowState::neutral());
        assert_eq!(property.state, PropertyGridRowState::neutral());
        assert!(!section.is_interactable());
        assert!(!section.is_editable());
        assert!(property.is_interactable());
        assert!(property.is_editable());
        assert!(!property.has_blocking_error());
    }

    #[test]
    fn property_grid_row_builder_attaches_state_metadata() {
        let row = PropertyGridRow::property(ItemId::from_raw(1), "Exposure", 0)
            .with_disabled(true)
            .with_read_only(true)
            .with_required(true)
            .with_help_text("Use scene-referred values")
            .with_status(PropertyGridRowStatus::warning(
                "Value is above preview range",
            ));

        assert!(row.state.disabled);
        assert!(row.state.read_only);
        assert!(row.state.required);
        assert_eq!(
            row.state.help_text.as_deref(),
            Some("Use scene-referred values")
        );
        assert_eq!(
            row.state.status.severity,
            PropertyGridStatusSeverity::Warning
        );
        assert_eq!(
            row.state.status.message.as_deref(),
            Some("Value is above preview range")
        );
        assert!(!row.is_interactable());
        assert!(!row.is_editable());
        assert!(!row.has_blocking_error());
    }

    #[test]
    fn property_grid_row_helpers_reflect_editability_and_error_state() {
        let read_only =
            PropertyGridRow::property(ItemId::from_raw(1), "Script", 0).with_read_only(true);
        let disabled =
            PropertyGridRow::property(ItemId::from_raw(2), "Collider", 0).with_disabled(true);
        let error = PropertyGridRow::property(ItemId::from_raw(3), "Mass", 0)
            .with_status(PropertyGridRowStatus::error("Mass must be positive"));
        let info = PropertyGridRow::property(ItemId::from_raw(4), "Material", 0)
            .with_status(PropertyGridRowStatus::info("Inherited from parent"));

        assert!(read_only.is_interactable());
        assert!(!read_only.is_editable());
        assert!(!read_only.has_blocking_error());
        assert!(!disabled.is_interactable());
        assert!(!disabled.is_editable());
        assert!(error.is_interactable());
        assert!(error.is_editable());
        assert!(error.has_blocking_error());
        assert!(!info.has_blocking_error());
    }

    #[test]
    fn property_grid_computes_content_and_scroll_extents() {
        let rows = rows();
        let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);

        assert_approx(layout.content_height(&rows), 84.0);
        assert_approx(layout.max_scroll_offset(&rows, 44.0), 40.0);
        assert_approx(layout.clamp_scroll_offset(&rows, 44.0, 500.0), 40.0);
        assert_eq!(layout.visible_range(&rows, 20.0, 44.0, 0), 0..3);
        assert_eq!(layout.visible_range(&rows, 44.0, 20.0, 0), 2..3);
    }

    #[test]
    fn property_grid_assigns_section_label_and_value_rects() {
        let rows = rows();
        let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
        let rects = layout.visible_row_rects(Rect::new(10.0, 100.0, 220.0, 84.0), &rows, 0.0, 0);

        assert_eq!(rects.len(), 4);
        assert_eq!(rects[0].id, ItemId::from_raw(1));
        assert_eq!(rects[0].label_rect, rects[0].rect);
        assert_approx(rects[1].label_rect.x, 10.0);
        assert_approx(rects[1].label_rect.width, 90.0);
        assert_approx(rects[1].value_rect.x, 108.0);
        assert_approx(rects[2].label_rect.x, 22.0);
        assert_approx(rects[2].value_rect.x, 120.0);
    }

    #[test]
    fn property_grid_metadata_does_not_change_row_rectangles() {
        let plain = rows();
        let annotated = vec![
            PropertyGridRow::section(ItemId::from_raw(1), "Transform")
                .with_help_text("Object transform"),
            PropertyGridRow::property(ItemId::from_raw(2), "Position", 0)
                .with_required(true)
                .with_status(PropertyGridRowStatus::severity(
                    PropertyGridStatusSeverity::Info,
                )),
            PropertyGridRow::property(ItemId::from_raw(3), "X", 1)
                .with_status(PropertyGridRowStatus::warning("Outside guide range")),
            PropertyGridRow::property(ItemId::from_raw(4), "Y", 1)
                .with_read_only(true)
                .with_status(PropertyGridRowStatus::error("Missing linked property")),
        ];
        let layout = PropertyGridLayout::new(20.0, 24.0, 90.0, 8.0, 12.0);
        let bounds = Rect::new(10.0, 100.0, 220.0, 84.0);

        assert_eq!(
            layout.visible_row_rects(bounds, &plain, 0.0, 0),
            layout.visible_row_rects(bounds, &annotated, 0.0, 0)
        );
    }

    #[test]
    fn property_grid_sanitizes_invalid_sizes() {
        let rows = rows();
        let layout = PropertyGridLayout::new(f32::NAN, -1.0, f32::NAN, f32::NAN, -12.0);

        assert_approx(layout.content_height(&rows), 0.0);
        assert_eq!(layout.visible_range(&rows, 0.0, 44.0, 0), 0..0);
        let rects = layout.visible_row_rects(Rect::new(10.0, 20.0, 100.0, 44.0), &rows, 0.0, 0);
        assert!(rects.is_empty());
    }
}
