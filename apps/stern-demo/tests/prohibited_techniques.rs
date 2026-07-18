//! Structural guard against private crates and substitute control painting.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn rust_sources(path: &Path, sources: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).expect("source directory") {
        let path = entry.expect("source entry").path();
        if path.is_dir() {
            rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

#[test]
fn prohibited_techniques_are_absent_from_public_consumer() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut paths = Vec::new();
    rust_sources(&root.join("src"), &mut paths);
    paths.sort();
    assert!(!paths.is_empty(), "demo source tree must not be empty");
    let source = paths
        .iter()
        .map(|path| fs::read_to_string(path).expect("source"))
        .collect::<String>();

    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(&root)
        .output()
        .expect("cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    let metadata = String::from_utf8(output.stdout).expect("metadata utf-8");
    let package_name = "\"name\":\"stern-demo\"";
    let name = metadata.find(package_name).expect("stern-demo metadata");
    let package = &metadata[metadata[..name].rfind("{\"name\":").expect("package start")..];
    let dependencies = package
        .split_once("\"dependencies\":[")
        .and_then(|(_, tail)| tail.split_once("],\"targets\""))
        .map(|(dependencies, _)| dependencies)
        .expect("dependency metadata");
    let normal_dependencies = dependencies
        .split("},{")
        .filter(|dependency| dependency.contains("\"kind\":null"))
        .map(|dependency| {
            dependency
                .split_once("\"name\":\"")
                .and_then(|(_, tail)| tail.split_once('"'))
                .map(|(name, _)| name)
                .expect("dependency name")
        })
        .collect::<Vec<_>>();
    assert_eq!(normal_dependencies, ["stern"]);

    for private_dependency in [
        "stern_core",
        "stern_render",
        "stern_text",
        "stern_vello",
        "stern_widgets",
        "stern_winit",
    ] {
        assert!(!source.contains(private_dependency), "{private_dependency}");
    }
    for substitute in [
        "RectPrimitive",
        "TextPrimitive",
        "LinePrimitive",
        "PathPrimitive",
        "SemanticNode::",
        "push_primitive",
        "fixtures_paint",
        "fn paint_",
        "struct DemoWidget",
        "struct DemoTheme",
        "struct DemoFramework",
        "unsafe ",
        "extern crate",
        "#[path",
        "include!",
        "include_str!",
    ] {
        assert!(!source.contains(substitute), "{substitute}");
    }
    for line in source.lines() {
        let import = line
            .trim_start()
            .strip_prefix("use ")
            .or_else(|| line.trim_start().strip_prefix("pub use "));
        if let Some(import) = import {
            let root = import.split([':', '{', ';']).next().expect("import root");
            assert!(["std", "stern", "stern_demo"].contains(&root), "{line}");
        }
    }
}
