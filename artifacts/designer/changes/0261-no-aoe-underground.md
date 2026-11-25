# 0261 — Area damage never hits anything underground unless it is the direct target

- **Date**: 2026-11-25T13:55:00-05:00
- **Answers**: Q0071 (gap: area damage underground)
- **Docs**: `systems/combat.md > AoEDamage`; `systems/units.md > DrillUnit`; `factions/syndicate/units.md > Subterranean Transport > Underground: hidden unless detected`

## What changed and why
User: nothing underground is hit by area damage unless it is the attack's direct target. This covers:
- underground DrillUnits;
- Syndicate expansions;
- a retracted pop-up Laser Turret;
- GDO mines.

Blasts aimed elsewhere don't touch them, even Universal ones. Targeting them directly requires them to be revealed (detection). **Replaces 0218** and 0219's "AoE still affects it", and resolves the contradiction with combat.md (which already said only surfaced underground units are hit).

## Implementation impact
- AoE resolution excludes underground objects other than the attack's own target.
- Blind Attack Ground can't find hidden underground things.

## Verification
A Grenadier's Attack Ground over a hidden Hideout or a burrowed Transport does no damage to it.
