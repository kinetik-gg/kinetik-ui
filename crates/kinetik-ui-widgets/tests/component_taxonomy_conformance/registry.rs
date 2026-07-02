use super::{
    BTreeSet, COMPONENT_CONFORMANCE_MATRIX, COMPONENT_EVIDENCE, COMPONENT_METADATA,
    component_evidence, metadata_by_slug,
};

#[test]
fn registry_contains_unique_component_names() {
    let mut names = BTreeSet::new();

    for metadata in COMPONENT_METADATA {
        assert!(names.insert(metadata.name), "duplicate {}", metadata.name);
    }
}

#[test]
fn registry_contains_unique_component_slugs() {
    let mut slugs = BTreeSet::new();

    for metadata in COMPONENT_METADATA {
        assert!(slugs.insert(metadata.slug), "duplicate {}", metadata.slug);
    }
}

#[test]
fn registry_contains_unique_component_evidence_ids() {
    let mut ids = BTreeSet::new();

    for evidence in COMPONENT_EVIDENCE {
        assert!(ids.insert(evidence.id), "duplicate {}", evidence.id);
        assert!(!evidence.id.is_empty(), "{evidence:?}");
        assert!(!evidence.category.as_str().is_empty(), "{evidence:?}");
        assert!(!evidence.summary.is_empty(), "{evidence:?}");
        assert!(
            evidence.id.chars().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || character == '-'
                    || character == '.'
            }),
            "{evidence:?}"
        );
    }
}

#[test]
fn registry_contains_unique_conformance_matrix_slugs() {
    let mut slugs = BTreeSet::new();

    for row in COMPONENT_CONFORMANCE_MATRIX {
        assert!(slugs.insert(row.slug), "duplicate {}", row.slug);
        assert!(!row.capability.is_empty(), "{row:?}");
        assert!(!row.slug.is_empty(), "{row:?}");
        assert!(
            row.slug
                .chars()
                .all(|character| character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || character == '-'),
            "{row:?}"
        );
        assert!(!row.public_contracts.is_empty(), "{row:?}");
        assert!(!row.deterministic_tests.is_empty(), "{row:?}");
        assert!(!row.evidence_ids.is_empty(), "{row:?}");
        for evidence_id in row.evidence_ids {
            assert!(
                component_evidence(evidence_id).is_some(),
                "{row:?} references missing evidence {evidence_id}"
            );
        }
        if let Some(component_slug) = row.component_slug {
            assert!(
                metadata_by_slug(component_slug).is_some(),
                "{row:?} references missing component slug {component_slug}"
            );
        }
    }
}

#[test]
fn every_metadata_entry_has_stable_non_empty_fields() {
    for metadata in COMPONENT_METADATA {
        assert!(!metadata.name.is_empty(), "{metadata:?}");
        assert!(!metadata.slug.is_empty(), "{metadata:?}");
        assert!(!metadata.category.as_str().is_empty(), "{metadata:?}");
        assert!(!metadata.status.as_str().is_empty(), "{metadata:?}");
        assert!(
            metadata
                .slug
                .chars()
                .all(|character| character.is_ascii_lowercase() || character == '-'),
            "{metadata:?}"
        );
        assert!(!metadata.slug.starts_with('-'), "{metadata:?}");
        assert!(!metadata.slug.ends_with('-'), "{metadata:?}");
        assert!(!metadata.evidence_ids.is_empty(), "{metadata:?}");
        for evidence_id in metadata.evidence_ids {
            assert!(
                component_evidence(evidence_id).is_some(),
                "{metadata:?} references missing evidence {evidence_id}"
            );
        }
    }
}
