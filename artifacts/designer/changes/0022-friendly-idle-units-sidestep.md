# 0022 — Friendly idle units sidestep for blocked movers; no head-on swapping

- **Date**: 2026-11-23T18:26:59-05:00
- **Answers**: Q0012; Q0007 (parts 2 and 3; part 1 was 0021)
- **Docs**: `systems/units.md > UnitCollision > Ground Collision` ("Friendly yielding" replaces "Idle units do not move aside")

## What changed and why
The Director's playtest data showed the strict "nothing ever makes way" rule clots ordinary crowds, not just corridors. The user chose **Q0012 option B**:
- When a moving unit is blocked by an **idle** unit of the same owner (no active command, including units that have arrived), the idle unit takes **one short sidestep**.
- **Enemies never yield.**
- **Moving units never yield or swap**, friendly or not (Q0007 part 2 = A). Head-on blocks in a one-unit corridor still end in re-route or give-up.

## Implementation impact
- **Replaces** the "idle units never make way" rule (Q0007 part 3 is answered "no": own idle units now make way).
- Collision stays hard. Yielding is a short move by the idle unit, not overlap.
- Placeholders for you to choose until decided: sidestep distance; whether Hold Position units and stationary-but-busy units (attacking, deployed Artillery Rig, Riot Shield raised, mining, constructing) yield. Suggested default until then: **only units with no active command yield**. Hold Position and busy units don't.
- Allies: no teams yet, so "own units" only.

## Verification
- Re-run the 18-vs-18 head-on and 200-unit stress tests: the "gave up" count should drop sharply, with remaining failures mostly mover-vs-mover head-ons.
- Walk a unit through your own idle group: they step aside and it passes. Do the same through an enemy group: it must path around.
- Two own units meeting in a one-tile corridor with no detour: both give up (no swap).
