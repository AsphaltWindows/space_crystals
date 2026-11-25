# Weapon Conversion

Sabre Legionnaire and Rifle Legionnaire are two forms of the same underlying unit, sharing an identical physical profile (Silhouette, MaxHP, MaxShields, PointArmor, FullArmor, SightRange, UnitBase, UnitBaseAttributes, BeaconCapacityCost) and differing only in AttackAttributes. Either form may convert into the other at any time via the Convert command.

## Convert Command
- Select Sabre Legionnaire(s) and/or Rifle Legionnaire(s). The two forms are separate SelectionGroups.
- Issue the Convert command (panel button R). **It goes only to the ActiveGroup** (user, 2026-11-25): with Sabres active, only the Sabres convert, to Rifles; the Rifles in the selection are untouched.
- The unit converts in place over 4 frames

## A toggle sets a target form
(user, 2026-11-25; Q0049) Pressing R **flips the unit's target form**; the unit then works to get itself into that form. So however fast R is pressed, every unit that received the presses ends up in the same form; quick presses simply skip the in-between conversions.
- **The unit's type, and so its SelectionGroup, switches at the press** to the target form. The pressed units join the group of that form (merging with any units of that form already selected), and **that merged group becomes the ActiveGroup**. Example: 4 Sabres (active) and 6 Rifles selected; R → 10 Rifles in one active group; R again at once → all 10 target Sabre, including the original 6.
- Then each unit works toward its target:
  - **waiting** (deferred during Firing/Cooldown, Eligibility below): it just takes the new target; if that is its current weapon again, it doesn't convert at all;
  - **mid-conversion**: it finishes the 4 frames (Completion), then converts back if the target has changed meanwhile;
  - **already in the other form**: it converts back.

## Cost
No resource cost.

## HP/Shields
No interpolation — since both forms share identical MaxHP/MaxShields, current HP/Shields values carry over unchanged.

## Vulnerability
The unit remains fully attackable throughout the 4-frame conversion — no invulnerability window.

## Eligibility
The Convert command follows the same interruption rules as any other command relative to an in-progress attack sequence (see systems/combat.md > AttackPhase): it may be issued freely while the unit is idle, moving, or in the Aiming or Reloading phases (both Interruptible = true), interrupting that phase immediately. It cannot interrupt the Firing or Cooldown phases (both Interruptible = false). **Pressed during those phases, it is deferred, not rejected** (user, 2026-11-24): the unit finishes Firing and Cooldown, then converts immediately (instead of starting its Reloading phase). So one press of R converts a whole group in a fight, the mid-swing units a few frames later than the rest. (Rejected: leaving them out, since pressing R again would flip the already-converted units back.)

**Pressing R again while a conversion is still deferred** (user, 2026-11-25; Q0049): the waiting unit's target flips back to its current weapon, so the waiting conversion is dropped (A toggle sets a target form, above). The shot already under way still finishes.

## Completion
Once the 4-frame conversion has actually started, it runs to completion and cannot be interrupted. If the target form changed meanwhile, the unit converts back right after (A toggle sets a target form, above). A deferred conversion hasn't started, so it is simply dropped when its target flips back.

## Orders are kept
(user, 2026-11-24) Convert is a **mode switch, not an order**: it doesn't replace the unit's orders. The unit holds still for the 4 frames, then **carries on** with its current orders using the other weapon (e.g. an attack-move continues as Rifles). Same principle as the Peacekeeper's Riot Shield (factions/gdo/units/peacekeeper.md > Riot Shield > Orders are kept).

**New weapon starts ready** (user, 2026-11-24): after converting, the new weapon is ready to begin an attack; a reload in progress on the old weapon is not carried over. Safe with current numbers: a round-trip conversion costs 8 frames, longer than either form's ReloadDuration (4–5 frames), so flipping back and forth never skips a reload. **Revisit if a Legionnaire's ReloadDuration ever exceeds ~8 frames.**

**Not queueable** (user, 2026-11-24): Convert always happens immediately; holding Shift changes nothing (it is never placed in the order queue).
