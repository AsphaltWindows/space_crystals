# 0125 — Lingering build ghosts; Drones Groupable; group Deploy goes to one Drone

- **Date**: 2026-11-24T15:30:00-05:00
- **Answers**: Q0040 (point 9; with 0117–0124, Q0040 is fully answered)
- **Docs**: `core/entities.md > Placement Validation > Lingering build ghosts` (new); `systems/controls/selection_and_hud.md > SelectionGroup` (Groupable guidance; new "Groupable workers used up by a build"), `> AwaitingTarget > Shift while placing` (Drone exception); `factions/colonists/units.md > Drone` (Groupable true; multi-select paragraph)

## What changed and why
Started from Q0040 point 9 (Set Drop-off with several Drones). The user chose to make Drones **Groupable** so bulk commands (Move, Stop, Set Drop-off) reach every selected Drone, and added **lingering build ghosts**.

1. **Lingering build ghosts.** Every ordered worker build (Drone Deploy, Agent build, Recruits' Construct), including every Shift-queued one, leaves a ghost on its site until construction starts. Visible to the owner only (allies TBD). It reserves the site against the owner's further placements (overlapping ghosts are red). It does not block units. It disappears on construction start, cancel, replacement or failure.
2. **Drone Groupable: false → true.** Set Drop-off goes to every selected Drone (Q0040 pt 9: A, by way of groupability).
3. **A Deploy given to a group of Drones goes to one Drone**: the first in selection order (SelectionGroup portrait order) that can take it. A plain click goes to the first Drone (replacing its orders). A Shift-click goes to the first Drone with no Deploy queued yet. The other Drones are untouched. This is a deliberate exception to No smart-casting (that rule is about ability power; a deploy has none, and one Beacon takes one Drone).
4. **Shift-placement chains through the Drone group**: each Shift-click goes to the next Drone without a Deploy; placement ends once every Drone has one. A single selected Drone still ends placement after one Shift-click.

## Implementation impact
- **Replaces 0071 for Drones** ("Drones don't chain; no auto-advance to the next worker"). Agents keep 0071 (still Ungroupable, Tab to the next).
- Beacons stay Ungroupable (0122).
- Ghosts are a new per-order visual + reservation, owner-side only; they never affect other players' placement validity (allies TBD).
- With a Drone group, the Deploy placement ghost validity must account for the player's existing lingering ghosts.

## Verification
- Select 6 mining Drones, press A, click a site: one Drone (the first portrait) leaves and a ghost stays on the site; the other 5 keep mining.
- Select 4 Drones, A, hold Shift, click 4 sites: 4 ghosts appear, each assigned to a different Drone; placement ends after the 4th.
- Try to place a ghost overlapping an existing lingering ghost: red.
- Walk a unit over a ghost: not blocked. Cancel the Drone's order: the ghost disappears.
- Select 12 Drones, S, click a new Egress Node: all 12 now deliver there.
- An enemy never sees your ghosts.
