//! Public exact typography-foundation token conformance.

#![allow(clippy::float_cmp)]

use stern_core::{
    FontFeatureScale, FontFeatureToken, FontLineHeightScale, FontLineHeightToken, FontSizeScale,
    FontSizeToken, FontWeightScale, FontWeightToken, default_dark_theme,
};

const EXPECTED_SIZE_TOKENS: [FontSizeToken; 6] = [
    FontSizeToken::Ui,
    FontSizeToken::Dense,
    FontSizeToken::Metadata,
    FontSizeToken::Section,
    FontSizeToken::Dialog,
    FontSizeToken::Heading,
];

const EXPECTED_LINE_HEIGHT_TOKENS: [FontLineHeightToken; 3] = [
    FontLineHeightToken::Ui,
    FontLineHeightToken::Dense,
    FontLineHeightToken::Metadata,
];

const EXPECTED_WEIGHT_TOKENS: [FontWeightToken; 4] = [
    FontWeightToken::Regular,
    FontWeightToken::Medium,
    FontWeightToken::Semibold,
    FontWeightToken::Bold,
];

const EXPECTED_FEATURE_TOKENS: [FontFeatureToken; 1] = [FontFeatureToken::Numeric];

#[test]
fn token_inventories_have_exact_normative_order() {
    assert_eq!(FontSizeToken::ALL, EXPECTED_SIZE_TOKENS.as_slice());
    assert_eq!(
        FontLineHeightToken::ALL,
        EXPECTED_LINE_HEIGHT_TOKENS.as_slice()
    );
    assert_eq!(FontWeightToken::ALL, EXPECTED_WEIGHT_TOKENS.as_slice());
    assert_eq!(FontFeatureToken::ALL, EXPECTED_FEATURE_TOKENS.as_slice());
}

#[test]
fn default_foundation_values_and_storage_types_are_exact() {
    let typography = default_dark_theme().typography;

    let _: f32 = typography.sizes.ui;
    let _: f32 = typography.line_heights.ui;
    let _: u16 = typography.weights.regular;
    let _: &'static str = typography.features.numeric;

    assert_eq!(typography.sizes.get(FontSizeToken::Ui), 12.0);
    assert_eq!(typography.sizes.get(FontSizeToken::Dense), 11.0);
    assert_eq!(typography.sizes.get(FontSizeToken::Metadata), 10.0);
    assert_eq!(typography.sizes.get(FontSizeToken::Section), 14.0);
    assert_eq!(typography.sizes.get(FontSizeToken::Dialog), 16.0);
    assert_eq!(typography.sizes.get(FontSizeToken::Heading), 20.0);

    assert_eq!(
        typography.line_heights.get(FontLineHeightToken::Ui),
        16.0
    );
    assert_eq!(
        typography.line_heights.get(FontLineHeightToken::Dense),
        15.0
    );
    assert_eq!(
        typography
            .line_heights
            .get(FontLineHeightToken::Metadata),
        14.0
    );

    assert_eq!(typography.weights.get(FontWeightToken::Regular), 400);
    assert_eq!(typography.weights.get(FontWeightToken::Medium), 500);
    assert_eq!(typography.weights.get(FontWeightToken::Semibold), 600);
    assert_eq!(typography.weights.get(FontWeightToken::Bold), 700);
    assert_eq!(
        typography.features.get(FontFeatureToken::Numeric),
        "tabular-nums"
    );
}

#[test]
fn typed_lookups_route_every_independent_sentinel() {
    let sizes = FontSizeScale::new(101.0, 103.0, 107.0, 109.0, 113.0, 127.0);
    assert_eq!(sizes.get(FontSizeToken::Ui), 101.0);
    assert_eq!(sizes.get(FontSizeToken::Dense), 103.0);
    assert_eq!(sizes.get(FontSizeToken::Metadata), 107.0);
    assert_eq!(sizes.get(FontSizeToken::Section), 109.0);
    assert_eq!(sizes.get(FontSizeToken::Dialog), 113.0);
    assert_eq!(sizes.get(FontSizeToken::Heading), 127.0);

    let line_heights = FontLineHeightScale::new(131.0, 137.0, 139.0);
    assert_eq!(line_heights.get(FontLineHeightToken::Ui), 131.0);
    assert_eq!(line_heights.get(FontLineHeightToken::Dense), 137.0);
    assert_eq!(
        line_heights.get(FontLineHeightToken::Metadata),
        139.0
    );

    let weights = FontWeightScale::new(601, 607, 613, 617);
    assert_eq!(weights.get(FontWeightToken::Regular), 601);
    assert_eq!(weights.get(FontWeightToken::Medium), 607);
    assert_eq!(weights.get(FontWeightToken::Semibold), 613);
    assert_eq!(weights.get(FontWeightToken::Bold), 617);

    let features = FontFeatureScale::new("sentinel-tabular-numeric");
    assert_eq!(
        features.get(FontFeatureToken::Numeric),
        "sentinel-tabular-numeric"
    );
}
