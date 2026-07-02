use super::types::{ComponentEvidence, ComponentEvidenceCategory};

use ComponentEvidenceCategory::{Conformance, Showcase, Stage, Status};

pub(super) const IMPLEMENTED_TAXONOMY_EVIDENCE: &[&str] = &[
    "status.implemented-public-api",
    "conformance.component-taxonomy",
    "showcase.metadata-only",
];
pub(super) const PARTIAL_TAXONOMY_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "conformance.component-taxonomy",
    "showcase.metadata-only",
];
pub(super) const STAGE_10_PARTIAL_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.component-taxonomy",
    "showcase.metadata-only",
];
pub(super) const STAGE_11_PARTIAL_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.component-taxonomy",
    "showcase.metadata-only",
];
pub(super) const S10_OUTLINER_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.outliner-contracts",
    "showcase.metadata-only",
];
pub(super) const S10_ASSET_BROWSER_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.asset-browser-contracts",
    "showcase.metadata-only",
];
pub(super) const S10_INLINE_EDIT_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.inline-edit-contracts",
    "showcase.metadata-only",
];
pub(super) const S10_COLLECTION_DRAG_CONTEXT_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.collection-drag-context-contracts",
    "showcase.metadata-only",
];
pub(super) const S10_COLLECTION_PROJECTION_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.10-outliner-asset-browser",
    "conformance.collection-projection-contracts",
    "showcase.metadata-only",
];
pub(super) const S11_TIMELINE_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.timeline-contracts",
    "showcase.metadata-only",
];
pub(super) const S11_RULER_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.timeline-ruler-contracts",
    "showcase.metadata-only",
];
pub(super) const S11_TRANSPORT_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.timeline-transport-contracts",
    "showcase.metadata-only",
];
pub(super) const S11_SNAPPING_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.timeline-snapping-contracts",
    "showcase.metadata-only",
];
pub(super) const S11_PRESERVATION_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.11-timeline",
    "conformance.timeline-preservation-contracts",
    "showcase.metadata-only",
];
pub(super) const S12_VIEWPORT_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.12-viewport-tools",
    "conformance.viewport-surface-contracts",
    "showcase.metadata-only",
];
pub(super) const S12_VIEWPORT_TOOLS_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.12-viewport-tools",
    "conformance.viewport-tool-contracts",
    "showcase.metadata-only",
];
pub(super) const S12_VIEWPORT_ACTION_ROUTING_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.12-viewport-tools",
    "conformance.viewport-action-routing-contracts",
    "showcase.metadata-only",
];
pub(super) const S12_VIEWPORT_OVERLAY_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.12-viewport-tools",
    "conformance.viewport-overlay-contracts",
    "showcase.metadata-only",
];
pub(super) const STAGE_13_PARTIAL_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.13-jobs-diagnostics",
    "conformance.component-taxonomy",
    "showcase.metadata-only",
];
pub(super) const S13_PROGRESS_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.13-jobs-diagnostics",
    "conformance.progress-indicator-contracts",
    "showcase.metadata-only",
];
pub(super) const S13_JOB_LIST_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.13-jobs-diagnostics",
    "conformance.job-list-contracts",
    "showcase.metadata-only",
];
pub(super) const S13_DIAGNOSTIC_STRIP_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.13-jobs-diagnostics",
    "conformance.diagnostic-strip-contracts",
    "showcase.metadata-only",
];
pub(super) const S13_FEEDBACK_STACK_EVIDENCE: &[&str] = &[
    "status.partial-public-contract",
    "stage.13-jobs-diagnostics",
    "conformance.feedback-stack-contracts",
    "showcase.metadata-only",
];

/// Data-only registry of stable evidence descriptors.
pub const COMPONENT_EVIDENCE: &[ComponentEvidence] = &[
    ComponentEvidence::new(
        "status.implemented-public-api",
        Status,
        "Public widget behavior exists for common usage.",
    ),
    ComponentEvidence::new(
        "status.partial-public-contract",
        Status,
        "Public models, helpers, or partial behavior exist, but the full component is incomplete.",
    ),
    ComponentEvidence::new(
        "stage.10-outliner-asset-browser",
        Stage,
        "Restarted editor-toolkit Stage 10 outliner and asset browser catalogue coverage.",
    ),
    ComponentEvidence::new(
        "stage.11-timeline",
        Stage,
        "Restarted editor-toolkit Stage 11 timeline catalogue coverage.",
    ),
    ComponentEvidence::new(
        "stage.12-viewport-tools",
        Stage,
        "Restarted editor-toolkit Stage 12 viewport tools catalogue coverage.",
    ),
    ComponentEvidence::new(
        "stage.13-jobs-diagnostics",
        Stage,
        "Restarted editor-toolkit Stage 13 jobs and diagnostics catalogue coverage.",
    ),
    ComponentEvidence::new(
        "conformance.component-taxonomy",
        Conformance,
        "Covered by the component taxonomy conformance test matrix.",
    ),
    ComponentEvidence::new(
        "conformance.outliner-contracts",
        Conformance,
        "Covered by public outliner data contracts and deterministic outliner conformance tests.",
    ),
    ComponentEvidence::new(
        "conformance.asset-browser-contracts",
        Conformance,
        "Covered by public asset browser data contracts and deterministic asset browser conformance tests.",
    ),
    ComponentEvidence::new(
        "conformance.inline-edit-contracts",
        Conformance,
        "Covered by public inline edit session contracts and deterministic rename conformance tests.",
    ),
    ComponentEvidence::new(
        "conformance.collection-drag-context-contracts",
        Conformance,
        "Covered by public collection drag/drop/context contracts and deterministic conformance tests.",
    ),
    ComponentEvidence::new(
        "conformance.collection-projection-contracts",
        Conformance,
        "Covered by public collection projection, filter, sort, and selection preservation contracts.",
    ),
    ComponentEvidence::new(
        "conformance.timeline-contracts",
        Conformance,
        "Covered by public timeline layout, coordinate, selection, and semantic contracts.",
    ),
    ComponentEvidence::new(
        "conformance.timeline-ruler-contracts",
        Conformance,
        "Covered by public timeline ruler tick and timecode contracts.",
    ),
    ComponentEvidence::new(
        "conformance.timeline-transport-contracts",
        Conformance,
        "Covered by public timeline transport action and semantic contracts.",
    ),
    ComponentEvidence::new(
        "conformance.timeline-snapping-contracts",
        Conformance,
        "Covered by public timeline snap candidate and snap resolution contracts.",
    ),
    ComponentEvidence::new(
        "conformance.timeline-preservation-contracts",
        Conformance,
        "Covered by public timeline viewport state and selection preservation contracts.",
    ),
    ComponentEvidence::new(
        "conformance.viewport-surface-contracts",
        Conformance,
        "Covered by public viewport surface, pan/zoom, overlay descriptor, and hit-test contracts.",
    ),
    ComponentEvidence::new(
        "conformance.viewport-tool-contracts",
        Conformance,
        "Covered by public viewport tool, transform handle, and drag request contracts.",
    ),
    ComponentEvidence::new(
        "conformance.viewport-action-routing-contracts",
        Conformance,
        "Covered by public viewport action request, action semantic, and cursor routing contracts.",
    ),
    ComponentEvidence::new(
        "conformance.viewport-overlay-contracts",
        Conformance,
        "Covered by public viewport guide, ruler, safe-area, and HUD descriptor contracts.",
    ),
    ComponentEvidence::new(
        "conformance.progress-indicator-contracts",
        Conformance,
        "Covered by public status and job progress metadata contracts.",
    ),
    ComponentEvidence::new(
        "conformance.job-list-contracts",
        Conformance,
        "Covered by public job row, progress, cancel, and summary metadata contracts.",
    ),
    ComponentEvidence::new(
        "conformance.diagnostic-strip-contracts",
        Conformance,
        "Covered by public diagnostic strip item, field, severity, source, and ordering contracts.",
    ),
    ComponentEvidence::new(
        "conformance.feedback-stack-contracts",
        Conformance,
        "Covered by public feedback lifetime, action, dismissal, ordering, and repaint contracts.",
    ),
    ComponentEvidence::new(
        "showcase.metadata-only",
        Showcase,
        "Tracked by taxonomy metadata only; no interactive showcase behavior is claimed.",
    ),
];
