# 0329 — Repair stops at full HP or an empty bank; gliders aren't repaired; expansions regenerate instead of being repaired

- **Date**: 2026-11-26T18:45:00-05:00
- **Answers**: Q0124 (points 1–4; 5–6 in 0328)
- **Docs**:
  - `factions/cults/units/recruit.md > Repair`; `factions/syndicate/units/agent.md > Repair`; `factions/gdo/units/field_engineer.md > Abilities`;
  - `factions/syndicate/structures/README.md > Tunnel Expansions` (Not repaired; they regenerate).

## What changed and why
User:
1. **Repairers stop once the target is at full HP.**
2. **When the player runs out of Space Crystals, the repair order ends.** The repairers stop.
3. **Gliders can't be repaired**, because they can't stop.
4. **Agents can't repair Tunnel expansions.** Instead, a completed expansion **regenerates HP gradually while it is part of a Tunnel system**, that is, while its Tunnel is alive.

Placeholder (designer): the regeneration rate is 1% of MaxHP per second, always on, including during combat. Whether it pauses in combat is being confirmed with the user.

## Implementation impact
- Repair orders end on full HP or when the bank is empty.
- No repair targeting of Gliders.
- Expansion HP regeneration is new.
- Agents' Repair rejects expansions.

## Verification
- A repair stops when the building is full.
- With the bank at 0, the repairers stop.
- A damaged Hideout climbs back to full in about 100 s.
