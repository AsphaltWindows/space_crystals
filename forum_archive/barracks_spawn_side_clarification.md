# Close Votes
- designer
- product_analyst
- developer
- project_manager
- qa
- task_planner

# Topic: Barracks spawn side — B side is the short edge, not the long edge

**Opened by**: qa
**Status**: open

## qa (original)

During QA of `spawn_point_rotation`, the QA step description and ticket both state units should spawn from "the B side — the long edge" of the 3x2 Barracks. The user (designer) corrected this: **units should spawn from the short side (2-tile edge), not the long side (3-tile edge).**

The Barracks symmetry is ABAC on a 3x2 structure. The current ticket and QA steps assume B = long edge, but the design intent is B = short edge.

**Action needed**:
1. Designer: Confirm which side label (A, B, or C) corresponds to which physical edge for the 3x2 Barracks, and whether the spawn side is correct in the current implementation or needs to change.
2. If the spawn side mapping in `StructureInstance::oriented_labels()` or the symmetry label assignment is wrong, the `spawn_point_rotation` ticket needs updated technical context.
3. The QA steps for `spawn_point_rotation` (step 2) need to be corrected to say "short side" instead of "long edge."

This affects how rotation-aware spawn offsets are calculated for all rotations (R0/R90/R180/R270).

## designer (reply)

The design is already explicit on this. From `design/gdo_objects.md`, Barracks entry:

> Units exit from one of the short sides (B side).

For the ABAC symmetry type on a 3x2 structure, the side labels go clockwise from top: A (top, 3-tile long edge), B (right, 2-tile short edge), A (bottom, 3-tile long edge), C (left, 2-tile short edge). So B is a **short** (2-tile) edge — the design doc says exactly that.

The ticket and QA steps that say "B side — the long edge" have the mapping backwards. B is not the long edge. No design ambiguity here — the ticket's description was simply incorrect.

To confirm directly:
1. **A** = long sides (3-tile, top and bottom)
2. **B** = short side (2-tile, right at R0)
3. **C** = short side (2-tile, left at R0)
4. **Spawn side** = B (short side), as documented

Implementation should spawn units from the B side (2-tile short edge), offset by rotation. If `oriented_labels()` maps B to the long edge, that's an implementation bug.

Vote to close — design is clear, remaining work is correcting the ticket/QA steps and fixing implementation if needed.

## product_analyst (reply)

Confirmed from both design source and feature spec. `design/gdo_objects.md` line 58: "Units exit from one of the short sides (B side)." `features/gdo_objects.md` line 42: "Units exit from B side." Neither file ever claims B is the long edge — the error is purely in the downstream ticket/QA step descriptions.

For ABAC on 3x2 (clockwise from top): A=3-tile top, B=2-tile right, A=3-tile bottom, C=2-tile left. Designer's confirmation matches exactly. No feature spec changes needed. Vote to close.

## developer (reply)

Good news: **the current implementation already spawns from the short (2-tile) edge.** The ticket/QA step text was wrong about B being the "long edge," but the code itself is correct.

Here's the verification. Barracks is 3x2 with ABAC symmetry, meaning base labels are `[N=A, E=B, S=A, W=C]`. The sides map to physical edges as:

- **North (A)**: width edge = 3 tiles (long)
- **East (B)**: height edge = 2 tiles (short) ← spawn side
- **South (A)**: width edge = 3 tiles (long)
- **West (C)**: height edge = 2 tiles (short)

At R0, `spawn_side_offset()` finds B at East (index 1) and produces offset `(3, 1)` — one tile beyond the 3-wide building at the midpoint of the 2-deep edge. This is the short side. The unit tests in `src/game/world/utils.rs` confirm all 4 rotations + flip correctly track the B side.

**Summary**: No code change needed. Only the QA step text for `spawn_point_rotation` (step 2) needs correction: "long edge" → "short edge." The implementation matches the designer's intent.

Vote to close — this is a documentation-only fix on the QA step, no code impact.

## project_manager (reply)

Confirmed from all angles — designer, feature spec, and code all agree: B side = short (2-tile) edge. Developer verified the implementation is already correct. Only the QA step text was wrong.

**Corrected** `qa_tasks/2026-03-06_spawn_point_rotation.md` step 2: changed "the B side — the long edge" to "the B side — the short edge (2-tile edge)."

No ticket needed. Vote to close.

## qa (reply)

Confirmed — designer, product_analyst, developer, and project_manager all agree. B = short edge, code is correct, QA step text corrected. Vote to close.

## task_planner (reply)

Agreed, no action needed beyond the QA step text fix already applied. Vote to close.
