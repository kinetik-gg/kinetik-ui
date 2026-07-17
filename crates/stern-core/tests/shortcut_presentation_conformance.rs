use stern_core::{Key, Modifiers, Shortcut, ShortcutPlatform};

fn logical(key: Key) -> Shortcut {
    Shortcut::new(Modifiers::default(), key)
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
