# Stage 3: Ordered Input And Shell

[Back to the alpha-readiness index](../alpha-readiness.md)

## Execution Contract

| Field | Decision |
| --- | --- |
| Status | Authorized / Queued |
| Scope | Sequence-preserving input, platform request execution, and pointer normalization |
| Impact / confidence | Critical / High (`IN-03` is High / High) |
| Campaign prerequisite | Stage 2 gate; campaign authorization recorded |
| Token checkpoint | Medium-large; remain serial through input-contract freeze |

## Packets

| ID | Goal | Dependency | Impact / confidence | Ownership |
| --- | --- | --- | --- | --- |
| `IN-01` | Preserve one ordered key/text/IME/pointer/focus/wheel stream and wire ordinary `KeyEvent.text` typing | Stage 2 gate | Critical / High | Root-owned contract |
| `IN-02` | Execute clipboard, URL, cursor, IME, repaint, and async shell results with one-frame request ownership | `IN-01` | Critical / High | Root integration |
| `IN-03` | Normalize line/pixel wheel, click counts, drag threshold, and drag-release click suppression | `IN-01`, `RT-02` | High / High | Root-owned while input contract is active |

## Ownership And Overlap

`IN-01` and `IN-03` own Z2 and remain serial with text-input consumption. `IN-01` must replace the separate key/text collections with an ordered stream or equivalent sequence-preserving contract. `IN-02` owns Z3 and may not overlap `REND-03` or live `SHOW-01/02` changes.

## Acceptance Gate And Verification Expectations

Go only when hardware-style typing and the IME lifecycle work in the supported live shell; mixed key/text order is preserved; copy/cut/paste, URLs, cursor, IME rectangles, repaint, and async requests execute with one-frame ownership; and mouse/touchpad scroll, double-click, drag threshold, and click suppression are deterministic.

Packet tasks must include contract, core, adapter, and supported-shell checks appropriate to their owned paths. Event reordering, stale requests, Z2/Z3 overlap, and shell behavior with no recorded owner are stop conditions; otherwise, record the gate and advance to the already Authorized / Queued Stage 4 without new approval.

## Deferrals

Desktop/Unicode editing, presenter extraction, and showcase workflow integration remain later-stage work.
