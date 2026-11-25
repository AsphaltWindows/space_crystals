# 0248 — Tunnel sides corrected: Supplies hatch on D, back wall on C; exit nearer the crystal hatch

- **Date**: 2026-11-25T11:15:00-05:00
- **Answers**: Q0063 point 2 (which half of Side A)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel > SymmetryType - ABCD` (sides; Which half of Side A); `factions/syndicate/units.md > Agent > Gathering` (drop-off sides, Drop Off Resources)

## What changed and why
User: both hatches sit **next to** the entrance side, and the back wall is **opposite** it. The docs had the Supplies hatch opposite the entrance. The sides, clockwise from the top in the default orientation, are now:
- **A**: entrance/exit;
- **B**: crystal hatch;
- **C**: back wall (no function);
- **D**: Supplies hatch.

**Which half of Side A**: the exit path is the half nearer the crystal hatch (B); the entry path is the half nearer the Supplies hatch (D). Both turn and flip with the Tunnel.

## Implementation impact
- **Supplies are delivered at Side D** (was Side C), and Side C has no function.
- Update the Tunnel's side data, the Agent's Supplies drop-off side, and any art/layout assumptions.
- The entry/exit spots on Side A are as above.

## Verification
- An Agent carrying Supplies walks to the side next to the entrance, not the one opposite it.
- Ejected units come out of the half of Side A nearer the crystal hatch.
