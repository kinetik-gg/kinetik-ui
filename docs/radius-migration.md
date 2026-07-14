# Radius migration

Stern now exposes exactly five radius roles: `none`, `sm`, `md`, `lg`, and
`full`. Their default scalar values are `0`, `3`, `6`, `12`, and `9999`.

## Fields and intent

- Remove `xs` uses. Ordinary buttons, checkboxes, fields, compact controls,
  and compact selections use `sm`; flush structural surfaces use `none`.
- Rename `pill` to `full` only for deliberate pills, radio circles, dots, and
  circular handles.
- Menus, dropdowns, nodes, popovers, and pickers continue to use `md`.
- Use `lg` for dialogs or prominent floating surfaces only after reviewing the
  component's intent. This migration does not change existing modal surfaces.

Ordinary buttons, tabs, and fields must not default to `full`.

## Stern consumer mappings

The built-in direct consumers migrate as follows:

- checkbox recipes: `xs` to `sm`;
- radio and slider recipes: `pill` to `full`;
- outliner visibility glyph: `pill` to `full`;
- outliner lock glyph: `xs` to `sm`;
- square viewport transform handles: `xs` to `sm`;
- overlays and inspector pickers: remain `md`, including Modal surfaces.

The square viewport handles intentionally do not use `full`; their geometry is
not circular. Modal and picker radii remain unchanged in this slice.

## Constructor

The old five-argument constructor accepted `xs`, `sm`, `md`, `lg`, and `pill`.
The replacement accepts the four configurable nonzero levels and always fixes
`none` at zero:

```rust
use stern_core::{CornerRadius, RadiusScale, default_dark_theme};

let radii = RadiusScale::from_values(3.0, 6.0, 12.0, 9999.0);
assert_eq!(radii.none, CornerRadius::all(0.0));

let theme = default_dark_theme().with_radii(radii);
assert_eq!(theme.radius, theme.radii.sm);
```

There are no `xs` or `pill` compatibility aliases.
