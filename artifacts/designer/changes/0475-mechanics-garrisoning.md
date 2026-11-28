# 0475 — Mechanics doc: Garrisoning

- **Date**: 2026-11-28T12:00:00-05:00
- **Answers**: Q0193 (migration, part 4)
- **Docs**:
  - `systems/mechanics/garrisoning.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/cults/structures/garrison.md > Garrisoning` (now parameters only)
  - `factions/gdo/structures/bunker.md > Garrison` (now parameters only)

## What changed and why
The fourth mechanics doc. It holds the rules shared by every garrison:
- Enter;
- each occupant picks its own target;
- range is measured from the garrison's edge;
- a substitute weapon is used unchanged;
- no abilities from inside;
- protection;
- seen as a threat;
- capacity counts;
- Eject All lets everyone out at once;
- occupants survive the garrison's destruction.

**Parameters:** capacity, who can enter, weapons inside (own, substitute, or none), range bonus, sight, door.

**Archetypes:**
- **Garrison structure:** the Cults Garrison and the GDO Bunker.
- **Garrison vehicle:** a stub for the Cults heavy vehicle being designed.

**No rule changed for the Garrison.** For the Bunker, its doc said it "works like the Cults Garrison" except as noted. The shared rules it inherits, such as seen as a threat (Q0139), own target picking and range from the edge, are therefore unchanged in intent; they are now stated once.

**Inconsistency flagged for the user** (not changed):
- The Garrison's occupants come out **inside its former footprint** (Cults Evacuation on Destruction).
- The Bunker's doc says **around its footprint**.

Both are kept as written until the user decides.

## Implementation impact
One garrisoning module; the two structures are parameter sets. No behaviour changes.

## Verification
No gameplay change.
