# 0080 — Exit doorway: units spawn half inside, production pauses only when the doorway is occupied

- **Date**: 2026-11-24T02:15:00-05:00
- **Answers**: Q0023 (points 2–3; Q0023 is now fully answered)
- **Docs**:
  - `core/entities.md > Structure Type > Exit Doorway` (new)
  - `factions/gdo/structures.md > Barracks` (pointer)
  - `factions/cults/structures.md > Armory` (pointer)

## What changed and why
The user's own design replaces both "nearest free spot" (the Director's proposal) and "always wait".
1. A produced or released unit spawns **half inside the building** at the middle of the exit side. Nothing else may overlap a building, so this spot can't be blocked from outside.
2. The unit walks out toward its rally point, or straight out if there is none. If the way is blocked, it stops in the doorway. There it is a normal unit: selectable, attackable, and able to attack, so a lone enemy blocker gets shot.
3. The next unit only appears once the doorway is free. Until then the finished unit waits inside and the queue waits behind it.
4. Own idle units outside step aside for the unit walking out, under the existing yielding rule.
5. **Buildings and terrain may block exits** for now. Placement doesn't keep exits clear. A building right against the exit side stalls the producer, and that's the player's own mistake.

The user's reasoning: one enemy unit should never be able to stop production completely.

The Designer reconciled the rule with buildings that have **several exit sides** (the Colonists portals). The building picks the side by its own rules, and a side whose doorway is occupied counts as blocked. The building pauses only when all of its eligible doorways are occupied.

## Implementation impact
- Replaces the nearest-free-spot spawn and the "never stalls" fallback (Q0023 proposals 2–3).
- Needs a collision exception: the releasing building ignores its own unit while that unit overlaps the footprint. Once the unit is fully out, it can't re-enter that space.
- Pause state: the finished unit is held. Its production is complete, the queue doesn't advance, and it shows as waiting.
- Applies to the Cults Armory's exit, including Eject All, which now releases one Recruit per free doorway.
- It does not cover aircraft, or units released underground (Syndicate Tunnels).

## Verification
1. Queue 5 Peacekeepers at a Barracks with open ground in front of side B. Each one appears half inside the right end and walks out.
2. Park one enemy Peacekeeper right in front of side B. The next produced unit still appears in the doorway and attacks it.
3. Place a building flush against side B. The first unit appears in the doorway and stays there. The next unit waits, and the queue doesn't advance. Destroy the blocking building, and the unit walks out and production resumes.
4. Stand an own idle unit in front of side B. It steps aside when the new unit comes out.
