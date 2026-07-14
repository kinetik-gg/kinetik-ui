# Docking Interactions

Docking interactions are deterministic model operations in `stern-widgets`.
`Dock` owns frame arrangement, split ratios, tab merges, and split insertion.
`Panel` remains passive content metadata.

## Splitter Resize

Use solved splitter hit rectangles with the neutral draggable primitive. Feed the
drag delta back into the dock model:

```rust
use stern_core::{Rect, Vec2};
use stern_widgets::{Dock, solve_dock_splitters};

fn drag_splitter(area: &mut Dock, bounds: Rect, drag_delta: Vec2) {
    let splitters = solve_dock_splitters(area, bounds, 6.0);
    if let Some(splitter) = splitters.first() {
        area.resize_split(&splitter.path, bounds, drag_delta);
    }
}
```

`resize_split` clamps ratios to the split minimums and updates the same
serializable dock tree used by `Dock::snapshot`.

## Tab Drag And Drop

Frame chrome starts tab drags; panels do not own drag behavior:

```rust
use stern_core::Point;
use stern_widgets::{
    Dock, FrameId, PanelId, resolve_dock_drop_target, solve_dock_layout,
};

fn drop_tab(area: &mut Dock, bounds: stern_core::Rect, pointer: Point) {
    let Some(drag) = area.begin_tab_drag(FrameId::from_raw(2), PanelId::from_raw(3)) else {
        return;
    };

    let frames = solve_dock_layout(area, bounds);
    let Some(target) = resolve_dock_drop_target(&frames, pointer, FrameId::from_raw(9)) else {
        return;
    };

    area.drop_tab(drag, target);
}
```

Dropping in the frame center merges the panel into the target tab group.
Dropping near an edge inserts the panel as a new frame split adjacent to the
target frame. The operation preserves panel dismissible policy and remains
round-trippable through snapshots.
