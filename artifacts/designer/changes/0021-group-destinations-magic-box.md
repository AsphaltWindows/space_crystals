# 0021 — Group destinations: magic box (keep arrangement outside, converge inside); no formation movement

- **Date**: 2026-11-23T17:57:16-05:00
- **Answers**: Q0007 (part 1, group destination. Head-on blocking and "idle units never make way" are still open)
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)` (new), `README.md`

## What changed and why
The user chose StarCraft's **magic box**, not the Director's default A (compact grid). They found the grid "a little bit too random from a player's perspective". For a group given one location-targeted movement order (Move, Attack Move, Patrol):
- The **box** is the bounding rectangle of the ordered units' positions.
- **Click outside the box:** each unit goes to click point + its current offset from the group center, so the group keeps its shape.
- **Click inside the box:** all units target the exact clicked point and converge, stopping where the crowd blocks them.
- **No formation movement:** each unit moves at its own speed, and the group doesn't wait for its slowest member.

## Implementation impact
- **Replaces the Director's default A** (grid around the click point).
- Applying the box to Attack Move and Patrol as well as Move is how the doc is written. Treat it as the rule unless a later entry says otherwise.
- "Converge" relies on the stuck/give-up rule for units that can't reach the point (Q0008, still open; keep the current defaults).
- TBD: an offset destination that's unreachable (Q0008), and any size limit on the box.

## Verification
- Select units standing in a horizontal line and right-click far away (outside their box): they arrive as the same horizontal line, centered on the click.
- Right-click in the middle of the same group: they all converge on that point and bunch up.
- A mixed group of fast and slow units strings out on a long move; the fast ones arrive first.
