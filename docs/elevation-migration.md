# Elevation migration

Stern's provisional elevation API now uses four typed semantic levels. The
default `ElevationScale` values are `none = 0`, `low = 1`, `medium = 2`, and
`high = 3`.

## Token fields

Migrate legacy fields as follows:

- `flat` becomes `none`.
- `raised` becomes `low` only when the surface is intentionally a small
  floating affordance. Review the surface's intent: docked panels and ordinary
  controls use `none` and remain shadowless.
- `overlay` has no single mechanical replacement. Choose `Low`, `Medium`, or
  `High` from the surface's real layering and input behavior.

Construct or customize the scale through `Theme::with_elevation`:

```rust
use stern_core::{ElevationScale, default_dark_theme};

let theme = default_dark_theme()
    .with_elevation(ElevationScale::new(0.0, 1.0, 2.0, 3.0));
```

## Shadow resolution

`Theme::elevation_shadow(f32, radius)` is now
`Theme::elevation_shadow(ElevationLevel, radius)`. Select levels by intent:

- `Low`: tooltips and small floating affordances.
- `Medium`: menus, context menus, dropdowns, popovers, and inspector pickers.
- `High`: dialogs, command palettes, and any surface with modal capture.
- `None`: no shadow.

```rust
use stern_core::{ElevationLevel, default_dark_theme};

let theme = default_dark_theme();
let menu_shadow = theme
    .elevation_shadow(ElevationLevel::Medium, theme.radii.md.top_left)
    .expect("medium elevation has a shadow");
assert!(theme
    .elevation_shadow(ElevationLevel::None, theme.radii.md.top_left)
    .is_none());
```

The old fields and float-based shadow API have no compatibility aliases.
