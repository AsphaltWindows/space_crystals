# 0483 — Energy doc: Drain toggle and Ground fields added

- **Date**: 2026-11-28T14:05:00-05:00
- **Answers**: Q0193 (migration, part 11)
- **Docs**:
  - `systems/mechanics/energy_and_effects.md > Drain toggle`, `> Ground fields` (new)
  - `factions/syndicate/units/shadebringer.md` (Uses line)
  - `README.md`

## What changed and why
The audit added two shared parts to the energy doc.
- **Drain toggle**, from the Shadebringer:
  - an on/off ability that drains a charge pool, once per toggle that is on;
  - an empty pool switches its toggles off;
  - several toggles can be on at once;
  - the pool's refill is a parameter.
- **Ground fields**, from Shroud's cloud and the sludge and smoke trails:
  - an object is in a field when its centre is inside;
  - each field has a duration and may linger;
  - it is shown to every player over the tiles Visible to them;
  - who it affects is a parameter.

**No rule changed.** One generalisation is worth checking: "shown to every player over Visible tiles" was stated for Shroud's cloud (Q0113) and is now the default for all fields, sludge and smoke included. If the Shadebringer's trails were built otherwise, report it.

## Implementation impact
None expected; this only reorganises the docs.

## Verification
No gameplay change.
