# add_colonists_legionnaire

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the Colonists' first combat units to `artifacts/designer/design/colonists_objects.md`, closing the long-standing "Colonists have zero combat units" gap. Introduces two unit forms — Sabre Legionnaire (melee) and Rifle Legionnaire (ranged) — plus a new mechanic, Weapon Conversion, that lets a Legionnaire switch between the two forms in the field.

Changes to `colonists_objects.md`:

1. **Bioport `Produces:` list filled in** (was previously TBD): produces Sabre Legionnaire only, 100 Space Crystals / 160 frames (10 seconds). Rifle Legionnaire is explicitly NOT directly producible — it's reached only by converting an existing Sabre Legionnaire.

2. **New `## Sabre Legionnaire` unit**: UnitBase HeavyInfantry, 36x36 silhouette, 70 MaxHP, 10 MaxShields, 0/0 armor, SightRange 5, Groupable true, MaxSpeed 5, TurnRate 180 deg/frame, infinite accel/decel, BeaconCapacityCost 2 (the first unit to actually use BeaconCapacityCost — previously only defined abstractly in factions.md with no unit consuming it). AttackAttributes: FullyConnected (Melee), TargetDomain Ground, SingleTarget, Damage 15, Range melee/adjacent, AimDuration 3f, FiringDuration 3f, CooldownDuration 1f, ReloadDuration 5f. ObjectInterfaceState: BasicCombatUnitInterfaceState plus a new **R: Convert to Rifle Legionnaire** button (CommandIssuingTransition).

3. **New `## Rifle Legionnaire` unit**: identical physical profile to Sabre Legionnaire (same silhouette/HP/Shields/armor/SightRange/UnitBase/UnitBaseAttributes/BeaconCapacityCost — only the AttackAttributes differ). AttackAttributes: FullyConnected (Ranged), TargetDomain Ground, SingleTarget, Damage 10, Range 4 grid units, MinRange 0, AimDuration 3f, FiringDuration 2f, CooldownDuration 7f, ReloadDuration 4f. ObjectInterfaceState: BasicCombatUnitInterfaceState plus **R: Convert to Sabre Legionnaire**.

4. **New `## Weapon Conversion` mechanic** (new precedent — first unit-level in-place form-swap in the design, distinct from the existing Beacon Deployment / Warp In structure-transformation mechanics): Sabre <-> Rifle conversion takes 4 frames, costs no resources, and requires no HP/Shields interpolation (both forms share identical MaxHP/MaxShields, so current HP/Shields values just carry over unchanged — this is simpler than Warp In/Beacon Deployment's interpolation, which only exists because those transformations connect entities with different max stats). The unit remains fully attackable throughout (no invulnerability window). Eligibility to start a conversion follows the EXISTING AttackPhase.Interruptible rules already defined in combat.md — no new interruption logic was introduced: conversion may start while idle/moving/Aiming/Reloading (Interruptible = true phases) but is rejected during Firing/Cooldown (Interruptible = false phases). Once started, conversion is uninterruptible/uncancellable and always completes.

## QA Instructions

1. Build a Bioport (via Warp In from an established Colonists Beacon) and confirm its build queue now offers "Sabre Legionnaire" for 100 Space Crystals / 160 frames (10 seconds). Confirm "Rifle Legionnaire" is NOT directly available as a production option in the Bioport's queue.
2. Produce a Sabre Legionnaire and confirm its stats: 70 MaxHP, 10 MaxShields, 0/0 armor, SightRange 5, MaxSpeed 5. Confirm it consumes 2 BeaconCapacity out of the player's total BeaconCapacity pool (max 200, per factions.md).
3. Confirm Sabre Legionnaire's attack is melee (adjacent-range only, no ranged targeting), dealing 15 damage per hit, with the stated Aim/Firing/Cooldown/Reload frame timings.
4. Select the Sabre Legionnaire and use the Convert command (panel button R). Confirm it becomes a Rifle Legionnaire after 4 frames, with current HP/Shields preserved exactly (no interpolation/scaling), and that it now has a ranged attack (4 grid unit range, 10 damage, MinRange 0) instead of melee.
5. Confirm the reverse conversion works identically (Rifle -> Sabre Legionnaire, R button, 4 frames, HP/Shields preserved, attack switches back to melee).
6. Confirm Convert can be issued while the unit is idle, moving, or in the middle of the Aiming or Reloading phase of an attack (interrupting that phase immediately to begin conversion). Confirm Convert is REJECTED (does not start) if issued while the unit is in the Firing or Cooldown phase of an attack.
7. Confirm converting has no Space Crystals or other resource cost, and that once a conversion begins it cannot be cancelled before completing.
