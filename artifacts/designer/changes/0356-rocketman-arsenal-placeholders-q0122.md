# 0356 — Q0122: the Sparrow stays; GDO anti-air via Rocketman and Arsenal placeholders; Target Lock works on structures; Colonists wait

- **Date**: 2026-11-26T23:45:00-05:00
- **Answers**: Q0122
- **Docs**:
  - `factions/gdo/units/rocketman.md`; `factions/gdo/structures/arsenal.md`;
  - `factions/gdo/structures/barracks.md > Produces`, `> ObjectInterfaceState`; `factions/gdo/structures/deployment_center.md > Constructs`.

## What changed and why
- **Q0122:** the Sparrow stays in the playtest.
  - The GDO gets its anti-air answer now through the Rocketman, with placeholder numbers, and the Arsenal it needs.
  - The **Colonists wait** until the Cults roster is done (user: "Let's wait with colonists until we're done with the Cults"). The playtest notes should name their gap.
- **Target Lock can target structures** (user).
- **Placeholders** (designer):
  - **Rocketman:**
    - LightInfantry 24x24, HP 45, armor 1/1, sight 6, speed 3;
    - missile: Damage 20, Range 6, MinRange 2, projectile speed 16 su/f, one missile every 32 f;
    - Target Lock: LockOn 48 f, LockRange 7;
    - cost: 100 SC, 160 f, Barracks W; panel R: Target Lock.
  - **Arsenal:** 2x2, HP 350, 1/4, Power −20, 150 SC, 320 f, no panel yet.
- The Missile Turret needs the Defense System, which needs a Radar Station. Those come in the general GDO placeholder pass, not needed for Q0122.

## Implementation impact
New GDO content: the Arsenal and the Rocketman, with the DoublyDisjointed missile and Target Lock.

## Verification
- Rocketmen can shoot down a Sparrow.
- A circling Sparrow can sometimes dodge the normal missile; a locked one can't.
