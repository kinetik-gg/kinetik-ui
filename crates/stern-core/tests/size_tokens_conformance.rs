//! Exact size-token foundation and legacy-isolation conformance.

#![allow(clippy::float_cmp)]

use std::{fs, path::Path};

use stern_core::{
    Color, ControlMetrics, ControlSizeScale, CornerRadius, HandleSizeScale, IconSizeScale,
    RadiusScale, RowSizeScale, SizeScale, SizeToken, SpacingScale, StrokeScale, default_dark_theme,
};

const TOKENS: [SizeToken; 14] = [
    SizeToken::ControlXs,
    SizeToken::ControlSm,
    SizeToken::ControlMd,
    SizeToken::ControlLg,
    SizeToken::RowCompact,
    SizeToken::RowStandard,
    SizeToken::Tab,
    SizeToken::PanelHeader,
    SizeToken::WorkspaceBar,
    SizeToken::IconSm,
    SizeToken::IconMd,
    SizeToken::IconLg,
    SizeToken::HandleVisual,
    SizeToken::HandleHit,
];

const DEFAULT_VALUES: [f32; 14] = [
    20.0, 24.0, 28.0, 32.0, 24.0, 28.0, 28.0, 30.0, 40.0, 12.0, 16.0, 20.0, 1.0, 7.0,
];

const SENTINEL_VALUES: [f32; 14] = [
    101.0, 103.0, 107.0, 109.0, 113.0, 127.0, 131.0, 137.0, 139.0, 149.0, 151.0, 157.0, 163.0,
    167.0,
];

fn sentinel_sizes() -> SizeScale {
    SizeScale::new(
        ControlSizeScale::new(
            SENTINEL_VALUES[0],
            SENTINEL_VALUES[1],
            SENTINEL_VALUES[2],
            SENTINEL_VALUES[3],
        ),
        RowSizeScale::new(SENTINEL_VALUES[4], SENTINEL_VALUES[5]),
        SENTINEL_VALUES[6],
        SENTINEL_VALUES[7],
        SENTINEL_VALUES[8],
        IconSizeScale::new(SENTINEL_VALUES[9], SENTINEL_VALUES[10], SENTINEL_VALUES[11]),
        HandleSizeScale::new(SENTINEL_VALUES[12], SENTINEL_VALUES[13]),
    )
}

#[test]
fn default_size_scale_matches_all_fourteen_normative_tokens() {
    let sizes = default_dark_theme().sizes;

    assert_eq!(SizeToken::ALL, TOKENS);
    assert_eq!(sizes.control.xs, 20.0);
    assert_eq!(sizes.control.sm, 24.0);
    assert_eq!(sizes.control.md, 28.0);
    assert_eq!(sizes.control.lg, 32.0);
    assert_eq!(sizes.row.compact, 24.0);
    assert_eq!(sizes.row.standard, 28.0);
    assert_eq!(sizes.tab, 28.0);
    assert_eq!(sizes.panel_header, 30.0);
    assert_eq!(sizes.workspace_bar, 40.0);
    assert_eq!(sizes.icon.sm, 12.0);
    assert_eq!(sizes.icon.md, 16.0);
    assert_eq!(sizes.icon.lg, 20.0);
    assert_eq!(sizes.handle.visual, 1.0);
    assert_eq!(sizes.handle.hit, 7.0);
    assert_ne!(sizes.handle.visual, sizes.handle.hit);

    for ((token, expected), inventory_token) in TOKENS
        .into_iter()
        .zip(DEFAULT_VALUES)
        .zip(SizeToken::ALL.iter().copied())
    {
        assert_eq!(token, inventory_token);
        assert_eq!(sizes.get(token), expected, "wrong value for {token:?}");
    }
}

#[test]
fn distinct_sentinels_prove_complete_independent_lookup_routing() {
    let sizes = sentinel_sizes();

    for (index, token) in TOKENS.into_iter().enumerate() {
        assert_eq!(sizes.get(token), SENTINEL_VALUES[index]);
        for other in &SENTINEL_VALUES[index + 1..] {
            assert_ne!(SENTINEL_VALUES[index], *other);
        }
    }

    assert_eq!(sizes.control.xs, SENTINEL_VALUES[0]);
    assert_eq!(sizes.control.sm, SENTINEL_VALUES[1]);
    assert_eq!(sizes.control.md, SENTINEL_VALUES[2]);
    assert_eq!(sizes.control.lg, SENTINEL_VALUES[3]);
    assert_eq!(sizes.row.compact, SENTINEL_VALUES[4]);
    assert_eq!(sizes.row.standard, SENTINEL_VALUES[5]);
    assert_eq!(sizes.tab, SENTINEL_VALUES[6]);
    assert_eq!(sizes.panel_header, SENTINEL_VALUES[7]);
    assert_eq!(sizes.workspace_bar, SENTINEL_VALUES[8]);
    assert_eq!(sizes.icon.sm, SENTINEL_VALUES[9]);
    assert_eq!(sizes.icon.md, SENTINEL_VALUES[10]);
    assert_eq!(sizes.icon.lg, SENTINEL_VALUES[11]);
    assert_eq!(sizes.handle.visual, SENTINEL_VALUES[12]);
    assert_eq!(sizes.handle.hit, SENTINEL_VALUES[13]);
    assert_ne!(sizes.handle.visual, sizes.handle.hit);
}

#[test]
fn with_sizes_changes_only_the_size_foundation() {
    let mut baseline = default_dark_theme();
    baseline.colors.surface.workspace = Color::rgb8(1, 3, 5);
    baseline.spacing = SpacingScale::new(
        173.0, 179.0, 181.0, 191.0, 193.0, 197.0, 199.0, 211.0, 223.0,
    );
    baseline.radii = RadiusScale::from_values(227.0, 229.0, 233.0, 239.0);
    baseline.strokes = StrokeScale::from_values(241.0, 251.0, 257.0, 263.0, 269.0);
    baseline.typography.label.size = 271.0;
    baseline.opacity.pressed = 277.0;
    baseline.elevation.medium = 281.0;
    baseline.duration.fast = 283.0;
    baseline.controls = ControlMetrics {
        control_height: 293.0,
        compact_control_height: 307.0,
        check_size: 313.0,
        padding_x: 317.0,
        padding_y: 331.0,
    };
    baseline.radius = CornerRadius::all(337.0);
    baseline.border_width = 347.0;
    baseline.text_size = 349.0;

    let customized = baseline.with_sizes(sentinel_sizes());

    assert_eq!(customized.sizes, sentinel_sizes());
    assert_eq!(customized.colors, baseline.colors);
    assert_eq!(customized.spacing, baseline.spacing);
    assert_eq!(customized.radii, baseline.radii);
    assert_eq!(customized.strokes, baseline.strokes);
    assert_eq!(customized.typography, baseline.typography);
    assert_eq!(customized.opacity, baseline.opacity);
    assert_eq!(customized.elevation, baseline.elevation);
    assert_eq!(customized.duration, baseline.duration);
    assert_eq!(customized.controls, baseline.controls);
    assert_eq!(customized.radius, baseline.radius);
    assert_eq!(customized.border_width, baseline.border_width);
    assert_eq!(customized.text_size, baseline.text_size);
}

#[test]
fn spacing_and_remaining_control_customization_do_not_mirror_size_tokens() {
    let controls = ControlMetrics {
        control_height: 353.0,
        compact_control_height: 359.0,
        check_size: 373.0,
        padding_x: 379.0,
        padding_y: 383.0,
    };
    let spacing = SpacingScale::new(
        389.0, 397.0, 401.0, 409.0, 419.0, 421.0, 431.0, 433.0, 439.0,
    );
    let customized = default_dark_theme()
        .with_sizes(sentinel_sizes())
        .with_controls(controls)
        .with_spacing(spacing);

    assert_eq!(customized.sizes, sentinel_sizes());
    assert_eq!(customized.spacing, spacing);
    assert_eq!(customized.controls, controls);
    assert_ne!(
        customized.controls.control_height,
        customized.sizes.control.md
    );

    assert_eq!(default_dark_theme().controls.control_height, 28.0);
    assert_eq!(default_dark_theme().controls.compact_control_height, 22.0);
    assert_eq!(default_dark_theme().controls.check_size, 14.0);
    assert_eq!(default_dark_theme().controls.padding_x, 8.0);
    assert_eq!(default_dark_theme().controls.padding_y, 4.0);
}

#[test]
fn control_metric_field_audit_is_declaration_scoped() {
    let unrelated_icon_size = r"
        pub struct IconSizeScale {
            pub icon_size: f32,
        }

        pub struct ControlMetrics {
            pub control_height: f32,
        }
    ";
    assert!(!control_metrics_declares_icon_size(unrelated_icon_size));

    let mutated_control_metrics = r"
        pub struct ControlMetrics {
            pub control_height: f32,
            pub icon_size : core::primitive::f32,
        }
    ";
    assert!(control_metrics_declares_icon_size(mutated_control_metrics));
}

#[test]
fn production_sources_have_no_removed_icon_size_authority() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root
        .parent()
        .and_then(Path::parent)
        .expect("stern-core must live under the workspace crates directory");
    let mut sources = Vec::new();
    for production_root in [workspace_root.join("crates"), workspace_root.join("apps")] {
        collect_production_rust_sources(&production_root, &mut sources);
    }

    assert!(
        sources
            .iter()
            .any(|path| path.starts_with(workspace_root.join("crates"))),
        "workspace crate production sources must be audited"
    );
    assert!(
        sources
            .iter()
            .any(|path| path.starts_with(workspace_root.join("apps"))),
        "workspace application production sources must be audited"
    );
    assert!(
        sources.iter().any(|path| {
            fs::read_to_string(path).is_ok_and(|source| source.starts_with("// @generated"))
        }),
        "checked-in generated Rust sources must be audited"
    );

    for path in &sources {
        let source = fs::read_to_string(path).expect("production Rust source must be readable");
        assert!(
            !source.contains(".controls.icon_size"),
            "removed icon-size consumer remains in {}",
            path.display()
        );
    }

    let tokens_path = crate_root.join("src/theme/tokens.rs");
    let tokens_source = fs::read_to_string(&tokens_path).expect("theme tokens must be readable");
    assert!(
        !control_metrics_declares_icon_size(&tokens_source),
        "ControlMetrics still declares the removed icon_size field"
    );
    let compact_body: String = control_metrics_body(&tokens_source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    for remaining in [
        "control_height",
        "compact_control_height",
        "check_size",
        "padding_x",
        "padding_y",
    ] {
        assert!(
            compact_body.contains(&format!("pub{remaining}:f32")),
            "remaining ControlMetrics field {remaining} must stay intact"
        );
    }
}

fn control_metrics_declares_icon_size(source: &str) -> bool {
    control_metrics_body(source)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .contains("pubicon_size:")
}

fn control_metrics_body(source: &str) -> &str {
    const DECLARATION: &str = "pub struct ControlMetrics";
    let declaration_start = source
        .find(DECLARATION)
        .expect("ControlMetrics declaration must exist");
    let opening_brace = source[declaration_start..]
        .find('{')
        .map(|offset| declaration_start + offset)
        .expect("ControlMetrics declaration must have a body");
    let body_start = opening_brace + 1;
    let mut depth = 1_usize;
    for (offset, character) in source[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[body_start..body_start + offset];
                }
            }
            _ => {}
        }
    }
    panic!("ControlMetrics declaration body must close");
}

fn collect_production_rust_sources(directory: &Path, output: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).expect("production source directory must be readable") {
        let path = entry.expect("source entry must be readable").path();
        if path.is_dir() {
            if !is_nonproduction_directory(&path) {
                collect_production_rust_sources(&path, output);
            }
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
            && !is_nonproduction_rust_file(&path)
        {
            output.push(path);
        }
    }
}

fn is_nonproduction_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(
            "tests"
                | "test"
                | "testdata"
                | "test-data"
                | "test_data"
                | "benches"
                | "benchmarks"
                | "examples"
                | "fixtures"
                | "snapshots"
                | "goldens"
                | "target"
                | ".runway"
        )
    )
}

fn is_nonproduction_rust_file(path: &Path) -> bool {
    matches!(
        path.file_stem().and_then(|name| name.to_str()),
        Some("test" | "tests")
    )
}
