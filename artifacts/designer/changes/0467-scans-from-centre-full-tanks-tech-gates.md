# 0467 — Target scans measure from the centre; a new Shadebringer starts full; Tech Gates

- **Date**: 2026-11-28T09:35:00-05:00
- **Answers**: Q0190
- **Docs**:
  - `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`
  - `core/vision.md` (intro)
  - `factions/syndicate/units/shadebringer.md > Charges and recharging`
  - `core/tech_and_research.md > Tech Gates` (new)

## What changed and why
1. **Option (b)** (user). Every target scan measures SightRange from the unit's **centre**, as vision does: idle, HoldPosition, AttackMove and Patrol. A unit's scan and its own eyesight then always agree, so a sight buff never changes when a unit engages. That follows the user's principle that buffs shouldn't change behaviour (0463). Weapon range stays edge to edge.
2. **Option (a)** (user, as built). A new Shadebringer starts with full tanks.
3. **New shared sub-mechanic: Tech Gates** (user). Any doc can mark an ability, effect or starting state with:
   - **"Tech gate: <tech>"**: unavailable until the player has that tech. A gated ability shows greyed with "Requires <tech>".
   - **"Tech gate: possible"**: may be gated by a tech that isn't designed yet. Build it ungated now, but as a switchable gate.

   The user said one of these points "may require a tech in the future". The designer's reading is the Shadebringer's full tanks, marked "Tech gate: possible". This reading is to be confirmed with the user.

## Implementation impact
- **Changed:** scans measure from the centre. Every unit now engages about half its width later than before. Goldens and computer players' games will shift.
- **Unchanged:** the tanks.
- **New:** a generic gate check that features can reference, plus a list of "possible" gates for the techs pass.

## Verification
1. A lone Guard engages at the same distance buffed and unbuffed.
2. The distance at which a lone unit picks a target equals its SightRange from its centre to the target's edge or centre, as the Director chooses consistently.
3. A new Shadebringer from production has 100/100 in both tanks.
