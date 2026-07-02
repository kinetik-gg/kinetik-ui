use crate::collections::ItemId;

use super::status::{PropertyGridRowStatus, PropertyGridStatusSeverity};

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

/// Reset-to-default affordance metadata for a property-grid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PropertyGridResetAffordance {
    /// True when a reset control should be presented.
    pub available: bool,
    /// True when the current value already matches the application-owned default.
    pub at_default: bool,
}

impl PropertyGridResetAffordance {
    /// Creates reset affordance metadata.
    #[must_use]
    pub const fn new(available: bool, at_default: bool) -> Self {
        Self {
            available,
            at_default,
        }
    }
}

/// Keyframe affordance metadata for a property-grid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PropertyGridKeyframeAffordance {
    /// True when a keyframe control should be presented.
    pub available: bool,
    /// True when the current property is keyed at the current application time.
    pub keyed: bool,
}

impl PropertyGridKeyframeAffordance {
    /// Creates keyframe affordance metadata.
    #[must_use]
    pub const fn new(available: bool, keyed: bool) -> Self {
        Self { available, keyed }
    }
}

/// App-owned property affordance metadata attached to a property-grid row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PropertyGridRowAffordances {
    /// Reset-to-default control metadata.
    pub reset: PropertyGridResetAffordance,
    /// Keyframe toggle control metadata.
    pub keyframe: PropertyGridKeyframeAffordance,
}

impl PropertyGridRowAffordances {
    /// Creates neutral affordance metadata.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            reset: PropertyGridResetAffordance::new(false, false),
            keyframe: PropertyGridKeyframeAffordance::new(false, false),
        }
    }

    /// Returns this metadata with reset-to-default state set.
    #[must_use]
    pub const fn with_reset(mut self, available: bool, at_default: bool) -> Self {
        self.reset = PropertyGridResetAffordance::new(available, at_default);
        self
    }

    /// Returns this metadata with keyframe state set.
    #[must_use]
    pub const fn with_keyframe(mut self, available: bool, keyed: bool) -> Self {
        self.keyframe = PropertyGridKeyframeAffordance::new(available, keyed);
        self
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
    /// Optional reset/keyframe affordance metadata owned by the application.
    pub affordances: PropertyGridRowAffordances,
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
            affordances: PropertyGridRowAffordances::neutral(),
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

    /// Returns this metadata with reset-to-default affordance state set.
    #[must_use]
    pub const fn with_resettable(mut self, available: bool, at_default: bool) -> Self {
        self.affordances = self.affordances.with_reset(available, at_default);
        self
    }

    /// Returns this metadata with keyframe affordance state set.
    #[must_use]
    pub const fn with_keyframeable(mut self, available: bool, keyed: bool) -> Self {
        self.affordances = self.affordances.with_keyframe(available, keyed);
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

    /// Returns this row with reset-to-default affordance state set.
    #[must_use]
    pub fn with_resettable(mut self, available: bool, at_default: bool) -> Self {
        self.state = self.state.with_resettable(available, at_default);
        self
    }

    /// Returns this row with keyframe affordance state set.
    #[must_use]
    pub fn with_keyframeable(mut self, available: bool, keyed: bool) -> Self {
        self.state = self.state.with_keyframeable(available, keyed);
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

    /// Returns true when this row can emit a reset-to-default request.
    #[must_use]
    pub fn can_request_reset(&self) -> bool {
        matches!(self.kind, PropertyGridRowKind::Property { .. })
            && !self.state.disabled
            && !self.state.read_only
            && self.state.affordances.reset.available
            && !self.state.affordances.reset.at_default
    }

    /// Returns true when this row can emit a keyframe toggle request.
    #[must_use]
    pub fn can_request_keyframe_toggle(&self) -> bool {
        matches!(self.kind, PropertyGridRowKind::Property { .. })
            && !self.state.disabled
            && !self.state.read_only
            && self.state.affordances.keyframe.available
    }
}
