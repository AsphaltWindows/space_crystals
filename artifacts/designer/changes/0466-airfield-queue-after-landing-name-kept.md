# 0466 — Return to Base keeps its name; Shift-queued orders run as soon as the aircraft lands

- **Date**: 2026-11-28T09:30:00-05:00
- **Answers**: Q0189 (part 2, revised)
- **Docs**:
  - `factions/gdo/structures/airfield.md > Returning and rearming`
  - `factions/gdo/units/hornet.md`, `factions/gdo/units/dragonfly.md` (panel)

## What changed and why
**Replaces part 2 of 0465 and its rename.**
- The user kept the name **Return to Base**.
- An order Shift-queued behind a Return to Base, or behind an automatic return, now runs **immediately after landing**. That is the literal reading of "return to base". It works like an order given during rearming: the aircraft takes off with the ammo rearmed so far. A player who wants a full rearm waits and gives the order afterwards.

**Right-clicking an Airfield** (clarified):
- Its own, assigned Airfield: the same as Return to Base.
- Another own Airfield: reassigns the aircraft there (the existing Reassigning rule), and it flies there to land and rearm.

## Implementation impact
- The label stays "Return to Base". Undo 0465's rename.
- Queued orders start on landing. This changes GH2's "waits for the full rearm".
- 0465's other parts stand: hold fire while landed, separation, slot layout, waiting inside, the Hornet's examples.

## Verification
1. Press F, then Shift+Attack Move. The Hornet lands and takes off again at once, with almost no ammo rearmed.
2. Press F alone and wait 10 s. It is fully rearmed and stays landed.
3. Right-click a different own Airfield that has room. The aircraft is reassigned and lands there.
