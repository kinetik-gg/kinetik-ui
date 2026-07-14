use super::evidence::COMPONENT_EVIDENCE;
use super::matrix::COMPONENT_CONFORMANCE_MATRIX;
use super::metadata::COMPONENT_METADATA;
use super::types::{
    ComponentCategory, ComponentConformanceMatrixRow, ComponentEvidence, ComponentEvidenceCategory,
    ComponentMetadata,
};

/// Looks up evidence metadata by exact stable evidence identifier.
#[must_use]
pub fn component_evidence(id: &str) -> Option<&'static ComponentEvidence> {
    COMPONENT_EVIDENCE.iter().find(|evidence| evidence.id == id)
}

/// Returns resolved evidence descriptors for a component taxonomy entry.
pub fn component_evidence_for(
    metadata: &ComponentMetadata,
) -> impl Iterator<Item = &'static ComponentEvidence> + '_ {
    metadata
        .evidence_ids
        .iter()
        .filter_map(|id| component_evidence(id))
}

/// Returns status evidence descriptors for a component taxonomy entry.
pub fn component_status_evidence(
    metadata: &ComponentMetadata,
) -> impl Iterator<Item = &'static ComponentEvidence> + '_ {
    component_evidence_for(metadata)
        .filter(|evidence| evidence.category == ComponentEvidenceCategory::Status)
}

/// Looks up component metadata by exact public name.
#[must_use]
pub fn component_metadata(name: &str) -> Option<&'static ComponentMetadata> {
    COMPONENT_METADATA
        .iter()
        .find(|metadata| metadata.name == name)
}

/// Returns all component metadata entries for a category.
pub fn components_by_category(
    category: ComponentCategory,
) -> impl Iterator<Item = &'static ComponentMetadata> {
    COMPONENT_METADATA
        .iter()
        .filter(move |metadata| metadata.category == category)
}

/// Returns all component metadata entries carrying evidence in a category.
pub fn components_by_evidence_category(
    category: ComponentEvidenceCategory,
) -> impl Iterator<Item = &'static ComponentMetadata> {
    COMPONENT_METADATA.iter().filter(move |metadata| {
        component_evidence_for(metadata).any(|evidence| evidence.category == category)
    })
}

/// Looks up a conformance matrix row by exact stable row slug.
#[must_use]
pub fn component_conformance_matrix_row(
    slug: &str,
) -> Option<&'static ComponentConformanceMatrixRow> {
    COMPONENT_CONFORMANCE_MATRIX
        .iter()
        .find(|row| row.slug == slug)
}

/// Returns all conformance matrix rows for a restarted editor-toolkit stage.
pub fn component_conformance_matrix_by_stage(
    stage: u8,
) -> impl Iterator<Item = &'static ComponentConformanceMatrixRow> {
    COMPONENT_CONFORMANCE_MATRIX
        .iter()
        .filter(move |row| row.stage == stage)
}
