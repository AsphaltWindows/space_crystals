# 0428 — Airfield: landing and take-off are air targets; new aircraft landed; destroyed with the Airfield; Dragonfly lands vertically

- **Date**: 2026-11-26T19:00:00-05:00
- **Answers**: Q0162
- **Docs**: `factions/gdo/structures/airfield.md > Assignment`, `> Returning and rearming`; `factions/gdo/units/dragonfly.md > UnitBase`; `factions/cults/units/sparrow.md > Placeholder stats` (Flight rules > Takeoff)

## What changed and why
User decisions:
1. **(a)** During the 16-frame landing and take-off the aircraft is an **air target**; only fully landed is it a ground target.
2. **(a)** New aircraft appear **landed on a free slot**, take off for the rally point, and stay landed with none. Also: the **Sparrow appears on the runway** (B end, facing C) and **takes off automatically** along it; it's an air target from the moment it appears.
3. Aircraft **landed** on a destroyed Airfield are **destroyed with it**; they don't take off by themselves. Designer's reading: one still landing counts as landed; one already taking off escapes.
4. **(a)** An order during rearming: it takes off at once with the ammo rearmed so far.
5. **(a)** The Dragonfly lands and takes off **vertically**, like the Specter at an air door.

## Implementation impact
- 3 replaces the placeholder (a) "stays landed, unassigned".
- 2 replaces GH1's "appear over their Airfield": new aircraft appear landed.
- Sparrow: appears on the runway rather than "over" it; otherwise as built.

## Verification
- A Hornet landing on its Airfield: an Air-only weapon can hit it until it touches down, not after.
- Build a Hornet with no rally point: it appears parked on a slot and stays.
- Destroy an Airfield with two Hornets parked and one taking off: the two die, the third flies on, unassigned.
- Order a Hornet off 5 s into rearming: it leaves with 2 missiles.
- A Dragonfly returns: it slows to IdleSpeed over its slot, descends, rearms, lifts straight up.
- A Sparrow finishes at the Airstrip: it appears on the runway's B end and rolls into the air toward C.
