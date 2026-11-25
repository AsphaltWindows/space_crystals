# 0262 — Area damage does hit underground objects; expansions and the pop-up turret have high armor (replaces 0261)

- **Date**: 2026-11-25T14:10:00-05:00
- **Answers**: Q0071 (gap: area damage underground; replaces 0261)
- **Docs**: `systems/combat.md > AoEDamage`; `systems/units.md > DrillUnit`; `factions/syndicate/units.md > Subterranean Transport`; `factions/syndicate/structures.md > Tunnel Expansions` (High armor underground)

## What changed and why
User reversed 0261: **all underground objects take area damage** from Ground/Universal blasts over them, even unseen. This covers:
- DrillUnits underground (0218/0219 stand);
- Syndicate expansions;
- the retracted pop-up Laser Turret;
- GDO mines.

**Expansions and the pop-up turret have high armor while underground.** Placeholders:
- **every expansion: PointArmor 6, FullArmor 200**, replacing the 1/4 in individual stat blocks;
- pop-up turret underground: 6/30 (0260).

Under the AoE formula, effective armor scales with the fraction of the structure covered, so a big structure soaks most of a blast. That is why underground FullArmor must be high. With a Grenadier blast (100, radius 40 su), a 2x2 expansion takes about 40.

## Implementation impact
- AoE includes underground objects.
- Expansion armor placeholders as above, to be tuned.
- **0261 is void.**

## Verification
Grenadier Attack Ground over a hidden Hideout (400 HP) deals about 40 per grenade.
