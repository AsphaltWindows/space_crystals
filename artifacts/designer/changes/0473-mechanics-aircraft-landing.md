# 0473 — Mechanics doc: Aircraft Landing and Rearming

- **Date**: 2026-11-28T11:30:00-05:00
- **Answers**: Q0193 (migration, part 3)
- **Docs**:
  - `systems/mechanics/aircraft_landing.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/gdo/structures/airfield.md > Aircraft Support` (now the Airfield's parameters only; Assignment, Production limit, Landing slots, Returning and rearming folded in)
  - `factions/syndicate/units/specter.md` (Uses line)
  - `factions/cults/units/pilgrim.md > On the runway`
  - `factions/cults/units/sparrow.md` (Takeoff)
  - `factions/cults/structures/airstrip.md > Production`
  - `factions/gdo/units/supply_chopper.md > Unit Transport`
  - `factions/syndicate/units/phantom.md`
  - `factions/gdo/units/hornet.md` and `dragonfly.md` (references)

## What changed and why
The third mechanics doc, defined independently of units.

**Rules for every landed aircraft:**
- an air target while landing and taking off, a ground target while landed;
- holds fire while landed;
- outside the air separation while landed;
- destroyed with the structure it is landed on (one still landing counts as landed; one already taking off escapes).

**Landing archetypes:**
- **Vertical**: slots, air doors or ground. Includes the glider vertical approach and circling when no spot is free.
- **Runway**: the take-off run always finishes; it lands lined up along the runway; occupancy is a parameter of the runway.
- **Field landing**: walkable free ground, or the nearest free spot.

**Rearming archetypes:**
- **Assigned base**: the GDO Airfield's assignment, reassigning, automatic return, Return to Base, queued orders on landing, partial rearm, and new aircraft appearing landed.
- **Network base**: the Specter's rules, which land in the nearest own structure of the network and rearm inside via a service; orders are wiped on entering.

**No rule changed.** The Airfield's doc keeps only its parameters: 4 assignments and slots, the 2x2 layout, F, 16 frames.

## Implementation impact
One landing and rearming module with these archetypes. No behaviour changes.
- The generic "destroyed with the structure it is landed on" rule was written for the Airfield. It now sits in the shared rules, with "a structure's doc may say otherwise".
- This applies it to the **Pilgrim on an Airstrip** and to **aircraft at a Tunnel's air doors**. Neither doc said what happens to them before.
  - Pilgrim on a destroyed Airstrip: destroyed with it. Its passengers die, as every carrier's do.
  - Specter landing through a destroyed Tunnel's door: likewise.

  **Flag for the user**: this is the designer's reading, written in the shared rules. Until the user confirms, treat it as a placeholder.

## Verification
No gameplay change, except the flagged rule (verify only once confirmed).
