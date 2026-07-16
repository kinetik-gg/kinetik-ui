//! Public exact typography-foundation token conformance.

#![allow(clippy::float_cmp)]

use stern_core::{
    FontFeatureScale, FontFeatureToken, FontLineHeightScale, FontLineHeightToken, FontSizeScale,
    FontSizeToken, FontWeightScale, FontWeightToken, TextRole, TypographyScale, default_dark_theme,
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

const TEXT_ROLES: [TextRole; 5] = [
    TextRole::Body,
    TextRole::Label,
    TextRole::Caption,
    TextRole::Title,
    TextRole::Monospace,
];

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

#[test]
fn replacing_any_foundation_scale_preserves_theme_and_resolved_text_roles() {
    let base_theme = default_dark_theme();
    let base = base_theme.typography;
    let sizes = FontSizeScale::new(201.0, 203.0, 207.0, 209.0, 211.0, 223.0);
    let line_heights = FontLineHeightScale::new(227.0, 229.0, 233.0);
    let weights = FontWeightScale::new(701, 709, 719, 727);
    let features = FontFeatureScale::new("replacement-numeric");
    let variants = [
        TypographyScale { sizes, ..base },
        TypographyScale {
            line_heights,
            ..base
        },
        TypographyScale { weights, ..base },
        TypographyScale { features, ..base },
        TypographyScale {
            sizes,
            line_heights,
            weights,
            features,
            ..base
        },
    ];

    for typography in variants {
        let customized = base_theme.with_typography(typography);

        assert_ne!(typography, base);
        assert_eq!(typography.families, base.families);
        for role in TEXT_ROLES {
            assert_eq!(typography.metrics(role), base.metrics(role));
            assert_eq!(customized.font(role), base_theme.font(role));
        }

        assert_eq!(customized.text_size, base_theme.text_size);
        assert_eq!(customized.colors, base_theme.colors);
        assert_eq!(customized.spacing, base_theme.spacing);
        assert_eq!(customized.sizes, base_theme.sizes);
        assert_eq!(customized.radii, base_theme.radii);
        assert_eq!(customized.strokes, base_theme.strokes);
        assert_eq!(customized.opacity, base_theme.opacity);
        assert_eq!(customized.elevation, base_theme.elevation);
        assert_eq!(customized.duration, base_theme.duration);
        assert_eq!(customized.controls, base_theme.controls);
        assert_eq!(customized.radius, base_theme.radius);
        assert_eq!(customized.border_width, base_theme.border_width);
    }
}
