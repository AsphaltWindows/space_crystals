# 0335 — Garrison rules and one door; production/research buildings have an entrance and an exit; Temple levels; prerequisites never cancel; Shrine rules and Ungroupable

- **Date**: 2026-11-26T20:15:00-05:00
- **Answers**: Q0115, Q0116, Q0117, Q0118
- **Docs**:
  - `core/entities.md > Entrance and exit (production and research buildings)` (new);
  - `factions/cults/structures/garrison.md`;
  - `factions/cults/structures/{temple,gunworks,machine_shop}.md` (sides; Temple Levels);
  - `core/tech_and_research.md > Research Prerequisites` (new);
  - `factions/cults/structures/shrine.md`.

## What changed and why
User:
- **Garrison (Q0115):**
  - Each unit inside picks its own target by the normal targeting rules.
  - Range is measured from the Garrison's edge.
  - No spells from inside.
  - Augmentations don't change the borrowed Militant attack.
  - It has **one door**, on side B (the side is a placeholder; SymmetryType AAAB).
  - On Eject All, everyone comes out **at once** through the door, ignoring collision with each other until clear. Evacuation on destruction is unchanged.
- **Entrances (Q0116):** "All production and research buildings should have an entrance and an exit, and units should come out of them one at a time (a little like from a Tunnel)."
  - The **Temple, Gunworks and Machine Shop** become ABAC, entrance B and exit C, like the Armory.
  - Staff leave one at a time through the Exit Doorway.
  - The Armory, Altar, Cathedral, Chop Shop and Airstrip already comply.
- **Temple (Q0117):**
  - Level 2 can't be queued until Level 1 completes.
  - Level 2's Cathedral requirement is checked only at queue. **General rule: losing a prerequisite never cancels or disables anything already started or finished**, for structures, units and research.
- **Shrine (Q0118):**
  - A unit is in the circle by its centre.
  - Recasting restarts the duration, and the effects don't stack.
  - **Shrines are Ungroupable.**
  - Whether the Shrine (not a production or research building) gets an entrance side is being asked. Keep the nearest side until then.

## Implementation impact
- Side changes for the Temple, Gunworks and Machine Shop: placement, entrance and exit pathing.
- The Garrison's door and simultaneous eject.
- Shrine and Garrison selection rules.

## Verification
- Four Militants in a Garrison: Eject All releases all four together from side B.
- Recruits staffing a Temple walk in at B, and Eject All lets them out of C one by one.
