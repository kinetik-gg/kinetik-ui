//! Deterministic conformance evidence for shortcut presentation policy.

use stern_core::{Key, Modifiers, Shortcut, ShortcutPlatform};

const PLATFORMS: [ShortcutPlatform; 3] = [
    ShortcutPlatform::Windows,
    ShortcutPlatform::MacOs,
    ShortcutPlatform::Linux,
];

fn logical(key: Key) -> Shortcut {
    Shortcut::new(Modifiers::default(), key)
}

fn assert_labels(shortcut: &Shortcut, expected: [Option<&str>; 3]) {
    for (platform, expected) in PLATFORMS.into_iter().zip(expected) {
        assert_eq!(
            shortcut.english_label(platform),
            expected.map(str::to_owned),
            "unexpected {platform:?} label for {shortcut:?}"
        );
    }
}

#[test]
fn modifiers_use_exact_stable_platform_order_and_names() {
    let shortcut = Shortcut::new(
        Modifiers::new(true, true, true, true),
        Key::Character("k".into()),
    );

    assert_eq!(
        shortcut.english_label(ShortcutPlatform::Windows),
        Some("Ctrl+Alt+Shift+Win+K".into())
    );
    assert_eq!(
        shortcut.english_label(ShortcutPlatform::MacOs),
        Some("Control+Option+Shift+Command+K".into())
    );
    assert_eq!(
        shortcut.english_label(ShortcutPlatform::Linux),
        Some("Ctrl+Alt+Shift+Super+K".into())
    );
}

#[test]
fn presentation_includes_only_active_modifiers_once() {
    let shortcut = Shortcut::new(
        Modifiers::new(true, false, true, false),
        Key::Character("z".into()),
    );

    assert_eq!(
        shortcut.english_label(ShortcutPlatform::Windows),
        Some("Alt+Shift+Z".into())
    );
}

#[test]
fn logical_named_keys_have_exact_platform_labels() {
    let cases = [
        (Key::Enter, [Some("Enter"), Some("Return"), Some("Enter")]),
        (Key::Escape, [Some("Esc"); 3]),
        (Key::Tab, [Some("Tab"); 3]),
        (
            Key::Backspace,
            [Some("Backspace"), Some("Delete"), Some("Backspace")],
        ),
        (
            Key::Delete,
            [Some("Delete"), Some("Forward Delete"), Some("Delete")],
        ),
        (Key::Insert, [Some("Insert"); 3]),
        (Key::Home, [Some("Home"); 3]),
        (Key::End, [Some("End"); 3]),
        (Key::PageUp, [Some("Page Up"); 3]),
        (Key::PageDown, [Some("Page Down"); 3]),
        (Key::ArrowLeft, [Some("Left"); 3]),
        (Key::ArrowRight, [Some("Right"); 3]),
        (Key::ArrowUp, [Some("Up"); 3]),
        (Key::ArrowDown, [Some("Down"); 3]),
        (Key::Space, [Some("Space"); 3]),
        (Key::Function(1), [Some("F1"); 3]),
        (Key::Function(u8::MAX), [Some("F255"); 3]),
        (Key::Function(0), [None; 3]),
        (Key::Unidentified, [None; 3]),
    ];

    for (key, expected) in cases {
        assert_labels(&logical(key), expected);
    }
}

#[test]
fn logical_character_labels_normalize_only_one_ascii_letter() {
    let cases = [
        ("a", Some("A")),
        ("Z", Some("Z")),
        ("é", Some("é")),
        ("ßx", Some("ßx")),
        ("++", Some("++")),
        ("", None),
        (" \t", None),
    ];

    for (source, expected) in cases {
        assert_labels(&logical(Key::Character(source.into())), [expected; 3]);
    }
}
