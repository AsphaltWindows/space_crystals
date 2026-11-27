# 0460 — Assault Bike mortar MinRange 1 GU; Attack Ground repositions when it can't aim

- **Date**: 2026-11-27T14:45:00-05:00
- **Answers**: Q0184
- **Docs**:
  - `factions/syndicate/units/assault_bike.md > Placeholder stats` (Mortar MinRange)
  - `systems/controls/command_behaviors.md > AttackingLocation > Algorithm` (new bullet)

## What changed and why
1. **Case 1** (target inside its turning circle). Settled by 0454 (`AttackingObject > Can't aim from where it stands`), as the Director read it: the Bike repositions.
2. **Its own blast** (user's decision). The mortar gets **MinRange 1 GU** (placeholder; was 0), just beyond its 48 su blast. With the Q0179 back-off rule, a Bike ordered onto a close target backs off to 2 GU before firing, and it never shells itself. Auto-targeting skips enemies inside MinRange, as for every weapon.
3. **Attack Ground out of arc** (user's decision). "Can't aim from where it stands" now applies to **AttackingLocation** too. A unit in range of the spot that can't bring its weapon to bear repositions to a spot it can fire from; it no longer circles forever. Target inside MinRange also applies to a location: the unit backs off to MinRange + 1 GU.

## Implementation impact
- **Changed:** the Bike's MinRange goes from 0 to 1 GU.
- **New:** repositioning in AttackingLocation, the same mechanism as 0454.
- **Unchanged:** the friendly-fire rule. A shooter is still hit by its own blast if it is ever inside it, but the MinRange now prevents this for the Bike.

## Verification
1. Order a Bike to attack a target 48 su to its side. It drives off, comes round, and fires from about 2 GU. It takes no damage from its own rounds.
2. Order Attack Ground at a spot in range but 90° off its facing. It repositions and keeps shelling the spot, and doesn't circle forever.
3. Order Attack Ground right next to the Bike. It backs off before firing.
