//! Data-only taxonomy metadata for Kinetik UI widget components.

mod evidence;
mod matrix;
mod metadata;
mod queries;
mod types;

pub use evidence::COMPONENT_EVIDENCE;
pub use matrix::COMPONENT_CONFORMANCE_MATRIX;
pub use metadata::COMPONENT_METADATA;
pub use queries::{
    component_conformance_matrix_by_stage, component_conformance_matrix_row, component_evidence,
    component_evidence_for, component_metadata, component_status_evidence, components_by_category,
    components_by_evidence_category,
};
pub use types::{
    ComponentCategory, ComponentConformanceMatrixRow, ComponentConformanceStatus,
    ComponentEvidence, ComponentEvidenceCategory, ComponentMetadata,
};
