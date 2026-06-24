use std::{
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SnapshotArtifactPaths {
    pub directory: PathBuf,
    pub expected: PathBuf,
    pub actual: PathBuf,
    pub diff: PathBuf,
}

pub fn artifact_paths(snapshot_name: &str) -> SnapshotArtifactPaths {
    let directory = artifact_root().join(sanitize_snapshot_name(snapshot_name));

    SnapshotArtifactPaths {
        expected: directory.join("expected.txt"),
        actual: directory.join("actual.txt"),
        diff: directory.join("diff.txt"),
        directory,
    }
}

pub fn emit_snapshot_artifacts(
    snapshot_name: &str,
    expected: &str,
    actual: &str,
) -> io::Result<SnapshotArtifactPaths> {
    let paths = artifact_paths(snapshot_name);

    fs::create_dir_all(&paths.directory)?;
    fs::write(&paths.expected, expected)?;
    fs::write(&paths.actual, actual)?;
    fs::write(&paths.diff, format_diff(expected, actual))?;

    Ok(paths)
}

#[track_caller]
pub fn assert_snapshot_text(snapshot_name: &str, expected: &str, actual: &str) {
    if expected == actual {
        return;
    }

    let paths = emit_snapshot_artifacts(snapshot_name, expected, actual).unwrap_or_else(|error| {
        let paths = artifact_paths(snapshot_name);
        panic!(
            "snapshot `{snapshot_name}` did not match, and artifact writing failed: {error}\nexpected artifact: {}\nactual artifact: {}\ndiff artifact: {}",
            paths.expected.display(),
            paths.actual.display(),
            paths.diff.display()
        );
    });

    panic!(
        "snapshot `{snapshot_name}` did not match\nexpected artifact: {}\nactual artifact: {}\ndiff artifact: {}",
        paths.expected.display(),
        paths.actual.display(),
        paths.diff.display()
    );
}

fn artifact_root() -> PathBuf {
    workspace_root()
        .join("target")
        .join("kinetik-ui-artifacts")
        .join("kinetik-ui-render")
        .join("resource-snapshots")
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("kinetik-ui-render manifest should live under crates/")
        .to_path_buf()
}

fn sanitize_snapshot_name(snapshot_name: &str) -> String {
    let mut sanitized = String::new();
    let mut last_was_separator = false;

    for character in snapshot_name.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() || character == '_' {
            sanitized.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            sanitized.push('-');
            last_was_separator = true;
        }
    }

    let trimmed = sanitized.trim_matches('-');

    if trimmed.is_empty() {
        "snapshot".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn format_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<_> = expected.lines().collect();
    let actual_lines: Vec<_> = actual.lines().collect();
    let mut diff = String::from("--- expected\n+++ actual\n");
    let mut found_line_difference = false;

    for line_index in 0..expected_lines.len().max(actual_lines.len()) {
        match (expected_lines.get(line_index), actual_lines.get(line_index)) {
            (Some(expected_line), Some(actual_line)) if expected_line == actual_line => {}
            (Some(expected_line), Some(actual_line)) => {
                found_line_difference = true;
                write!(
                    diff,
                    "@@ line {} @@\n- {}\n+ {}\n",
                    line_index + 1,
                    expected_line,
                    actual_line
                )
                .expect("writing to String should not fail");
            }
            (Some(expected_line), None) => {
                found_line_difference = true;
                write!(diff, "@@ line {} @@\n- {}\n", line_index + 1, expected_line)
                    .expect("writing to String should not fail");
            }
            (None, Some(actual_line)) => {
                found_line_difference = true;
                write!(diff, "@@ line {} @@\n+ {}\n", line_index + 1, actual_line)
                    .expect("writing to String should not fail");
            }
            (None, None) => {}
        }
    }

    if expected.ends_with('\n') != actual.ends_with('\n') {
        found_line_difference = true;
        diff.push_str("@@ trailing newline @@\n");
        diff.push_str(if expected.ends_with('\n') {
            "- expected ends with newline\n+ actual does not end with newline\n"
        } else {
            "- expected does not end with newline\n+ actual ends with newline\n"
        });
    }

    if !found_line_difference {
        diff.push_str("snapshots differ without line-level changes\n");
    }

    diff
}
