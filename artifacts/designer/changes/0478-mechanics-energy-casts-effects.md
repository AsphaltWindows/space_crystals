# 0478 — Mechanics doc: Energy, Casts and Timed Effects

- **Date**: 2026-11-28T12:45:00-05:00
- **Answers**: Q0193 (migration, part 6)
- **Docs**:
  - `systems/mechanics/energy_and_effects.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/cults/units/disciple.md > Energy`
  - `factions/cults/units/prayer_mobile.md` (Energy)
  - `factions/cults/structures/shrine.md > Abilities`
  - `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`

## What changed and why
The sixth mechanics doc.
- **Energy pools:**
  - Parameters: max, start, regeneration (flat or per Recruit inside).
  - Regeneration continues while channelling.
  - Pools can be boosted.
  - Shown to every player.
  - Casters aren't Groupable: one press, one caster.
- **Cast archetypes:**
  - **Instant cast**: around the caster, in an arc in front of it, or on itself.
  - **Targeted area cast.**
  - **Channel**: drain, minimum to start, and what ends it (death, an unqueued order or cancel, out of energy; not damage). It may linger after it ends.
  - Shift-queueing of instant casts where the doc says so.
- **Timed effects:**
  - applied at the cast and kept after leaving;
  - a recast restarts the duration and doesn't stack;
  - percentages combine per AttackSpeed;
  - buffs never change behaviour;
  - buff marks are shown to all;
  - copies, such as illusions, inherit remaining effects;
  - debuffs follow the same rules (Suppress);
  - services give the effect on leaving (Null Chamber).

**No rule changed.** These rules were stated in the Disciple's, Prayer-mobile's, Shrine's and Null Chamber's docs; those docs now say which parts they use. Their detailed rules and examples stay in place as parameters. Shroud's damage spreading is specific to Shroud and stays in the Disciple's doc.

## Implementation impact
One module for energy, casting and timed effects. No behaviour changes.

## Verification
No gameplay change.
