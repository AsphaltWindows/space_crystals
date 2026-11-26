# Q0118 — The Shrine's abilities: who is inside, recasting, and several Shrines

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a) on all three.
- **Docs**: `factions/cults/structures/shrine.md > Abilities`, `> Placeholder stats`; `systems/controls/selection_and_hud.md > SelectionGroup`

## Question
1. **Inside the area.** Is a unit inside when its centre is in the circle, or when any part of it overlaps?
2. **Recasting** on units that still have the buff. Does the duration restart, or do the effects add up (two health regenerations giving +4 HP/s)?
3. **Several Shrines selected.** Shrines are Groupable. Does one press cast once, or once per selected Shrine?

## Context
- "No cooldowns." With effects that add up, two Shrines double a heal.
- The caster rule: "one cast should spend one unit's energy, not the whole group's".

## Options
1. **(a)** the centre; **(b)** any overlap.
2. **(a)** the duration restarts, and the rates don't add; **(b)** the rates add.
3. **(a)** one cast, from the selected Shrine with the most energy; **(b)** every selected Shrine casts at the point.

Proposed: (a) on all three.
