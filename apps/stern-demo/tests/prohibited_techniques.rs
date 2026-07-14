//! Structural guard against private crates and substitute control painting.

use std::fs;
use std::path::PathBuf;

#[test]
fn prohibited_techniques_are_absent_from_public_consumer() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("manifest");
    let source = ["src/lib.rs", "src/main.rs"]
        .into_iter()
        .map(|path| fs::read_to_string(root.join(path)).expect("source"))
        .collect::<String>();

    assert!(manifest.contains("stern = {"));
    for private_dependency in [
        "stern-core =",
        "stern-render =",
        "stern-text =",
        "stern-vello =",
        "stern-widgets =",
        "stern-winit =",
    ] {
        assert!(
            !manifest.contains(private_dependency),
            "{private_dependency}"
        );
    }
    for substitute in [
        "RectPrimitive",
        "TextPrimitive",
        "LinePrimitive",
        "push_primitive",
        "fixtures_paint",
        "fn paint_",
    ] {
        assert!(!source.contains(substitute), "{substitute}");
    }
    assert!(source.contains("use stern::"));
}
