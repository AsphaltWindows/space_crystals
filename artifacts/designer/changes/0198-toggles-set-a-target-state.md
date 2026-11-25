# 0198 — Mode toggles set a target state; the press goes to the ActiveGroup and moves units to the other group at once

- **Date**: 2026-11-25T02:20:00-05:00
- **Answers**: Q0049 (replaces 0197), Q0053 points 2 and 3
- **Docs**: `factions/colonists/units.md > Weapon Conversion > Convert Command`, `> A toggle sets a target form` (new), `> Eligibility`, `> Completion`; `factions/gdo/units.md > Peacekeeper > Riot Shield` (Two unit types, During the toggle)

## What changed and why
User: pressing a mode toggle (Legionnaire Convert, Peacekeeper Riot Shield) **flips the unit's target state**, and the unit works to get itself into that state. However fast the button is pressed, every unit that received the presses ends up in the same state; quick presses just skip the in-between conversions.
- **Only the ActiveGroup** receives the press. With Sabres and Rifles selected and Sabres active, only the Sabres convert. This replaces "each converts to its respective other form" for mixed selections.
- **The type, and so the SelectionGroup, switches at the press**, including for units whose change is deferred behind a shot. The pressed units merge into the other type's group, and that group becomes the ActiveGroup. A second press therefore reaches everyone, original members included.
- Per unit:
  - **deferred** (waiting for Firing/Cooldown to end): takes the new target; if it is back to its current state, nothing happens;
  - **mid-change**: finishes the change, then changes back if the target differs (a Legionnaire conversion can't be interrupted; the Riot Shield already reverses from the current point, 0171);
  - **already changed**: changes back.

**Replaces 0197**: there is no longer a split group after a double-tap.

Q0053 pt 2: Lower pressed while a raise is still deferred drops the raise (as M3-S does now).

Q0053 pt 3: the Shielded group becomes active as soon as R is pressed (option B; this changes M3-S). It also replaces 0173's "type switches when the toggle starts": the switch now happens at the press.

## Implementation impact
- Convert on a mixed selection affects only the ActiveGroup.
- Type and group switch at the press, even when deferred; the ActiveGroup follows the pressed units into the merged group.
- A Legionnaire whose target changes mid-conversion converts back right after finishing.
- The shot already under way when the press is deferred still finishes normally.
- Q0053 point 1 (orders while lowering) is still open.

## Verification
- 4 Sabres (active) and 6 Rifles: press R. Only the Sabres convert, and all 10 show as one active Rifle group. Press R again: all 10 become Sabres.
- Spam R on a fighting Legionnaire group. When you stop, all units share one form, the one matching the parity of the presses.
- Peacekeepers mid-shot: press R. The panel shows Lower Shield at once. Press R again before the shot ends: none raise.
