/// Kinetik-owned component category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentCategory {
    /// Static display and decoration components.
    Display,
    /// Clickable, selectable, or adjustable controls.
    Control,
    /// Non-text input controls.
    Input,
    /// Text editing and text-query controls.
    TextEditing,
    /// Collection, virtualization, and structured data components.
    Collection,
    /// Docking, frame, and panel workspace components.
    Docking,
    /// Menus, popovers, command palettes, and other overlay surfaces.
    Overlay,
    /// Media, image, video, and editor viewport surfaces.
    Viewport,
    /// Property editing and inspector patterns.
    Inspector,
    /// System-level editor chrome and status patterns.
    System,
}

impl ComponentCategory {
    /// Returns a stable display name for the category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::Control => "Control",
            Self::Input => "Input",
            Self::TextEditing => "TextEditing",
            Self::Collection => "Collection",
            Self::Docking => "Docking",
            Self::Overlay => "Overlay",
            Self::Viewport => "Viewport",
            Self::Inspector => "Inspector",
            Self::System => "System",
        }
    }
}

/// Honest implementation status for a component or editor pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentConformanceStatus {
    /// Public widget behavior exists for common usage.
    Implemented,
    /// Public models, helpers, or partial behavior exist, but the full component is incomplete.
    Partial,
    /// The component is part of the Kinetik vocabulary but is not implemented in this crate yet.
    Planned,
}

impl ComponentConformanceStatus {
    /// Returns a stable display name for the status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "Implemented",
            Self::Partial => "Partial",
            Self::Planned => "Planned",
        }
    }
}

/// Category for evidence attached to a component taxonomy entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentEvidenceCategory {
    /// Evidence explaining the honest implementation status.
    Status,
    /// Evidence tying the entry to a restarted editor-toolkit stage.
    Stage,
    /// Evidence from deterministic conformance tests or contracts.
    Conformance,
    /// Evidence describing showcase/catalogue coverage without implying runtime behavior.
    Showcase,
}

impl ComponentEvidenceCategory {
    /// Returns a stable display name for the evidence category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "Status",
            Self::Stage => "Stage",
            Self::Conformance => "Conformance",
            Self::Showcase => "Showcase",
        }
    }
}

/// Stable evidence descriptor referenced by component taxonomy entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentEvidence {
    /// Stable lower-kebab or dotted identifier.
    pub id: &'static str,
    /// Evidence category.
    pub category: ComponentEvidenceCategory,
    /// Short human-readable evidence summary.
    pub summary: &'static str,
}

impl ComponentEvidence {
    /// Creates a component taxonomy evidence descriptor.
    #[must_use]
    pub const fn new(
        id: &'static str,
        category: ComponentEvidenceCategory,
        summary: &'static str,
    ) -> Self {
        Self {
            id,
            category,
            summary,
        }
    }
}

/// Public component taxonomy entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentMetadata {
    /// Public component or pattern name.
    pub name: &'static str,
    /// Stable lower-kebab identifier.
    pub slug: &'static str,
    /// Kinetik-owned category.
    pub category: ComponentCategory,
    /// Honest implementation status.
    pub status: ComponentConformanceStatus,
    /// Restarted editor-toolkit stage that currently owns the catalogue entry.
    pub stage: Option<u8>,
    /// Stable evidence identifiers backing the status and coverage metadata.
    pub evidence_ids: &'static [&'static str],
}

impl ComponentMetadata {
    /// Creates a component taxonomy entry.
    #[must_use]
    pub const fn new(
        name: &'static str,
        slug: &'static str,
        category: ComponentCategory,
        status: ComponentConformanceStatus,
    ) -> Self {
        Self {
            name,
            slug,
            category,
            status,
            stage: None,
            evidence_ids: &[],
        }
    }

    /// Sets the restarted editor-toolkit stage for this taxonomy entry.
    #[must_use]
    pub const fn with_stage(mut self, stage: u8) -> Self {
        self.stage = Some(stage);
        self
    }

    /// Sets the stable evidence identifiers for this taxonomy entry.
    #[must_use]
    pub const fn with_evidence(mut self, evidence_ids: &'static [&'static str]) -> Self {
        self.evidence_ids = evidence_ids;
        self
    }
}

/// Data-only conformance matrix row for a spec-stage capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentConformanceMatrixRow {
    /// Public capability or pattern name.
    pub capability: &'static str,
    /// Stable lower-kebab capability identifier.
    pub slug: &'static str,
    /// Optional component slug this capability supports.
    pub component_slug: Option<&'static str>,
    /// Kinetik-owned category.
    pub category: ComponentCategory,
    /// Honest implementation status for the capability.
    pub status: ComponentConformanceStatus,
    /// Restarted editor-toolkit stage that owns this matrix row.
    pub stage: u8,
    /// Public data-only contracts that provide the capability surface.
    pub public_contracts: &'static [&'static str],
    /// Deterministic tests that prove the evidence claim.
    pub deterministic_tests: &'static [&'static str],
    /// Stable evidence identifiers backing the row.
    pub evidence_ids: &'static [&'static str],
}

impl ComponentConformanceMatrixRow {
    /// Creates a partial component conformance matrix row.
    #[must_use]
    pub const fn partial(
        capability: &'static str,
        slug: &'static str,
        category: ComponentCategory,
        stage: u8,
        public_contracts: &'static [&'static str],
        deterministic_tests: &'static [&'static str],
        evidence_ids: &'static [&'static str],
    ) -> Self {
        Self {
            capability,
            slug,
            component_slug: None,
            category,
            status: ComponentConformanceStatus::Partial,
            stage,
            public_contracts,
            deterministic_tests,
            evidence_ids,
        }
    }

    /// Associates this matrix row with a component metadata slug.
    #[must_use]
    pub const fn with_component_slug(mut self, component_slug: &'static str) -> Self {
        self.component_slug = Some(component_slug);
        self
    }
}
