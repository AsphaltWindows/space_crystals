# 0260 — Placeholders: Laser Turret, Grenadier, Chemical Plant (for M4)

- **Date**: 2026-11-25T13:40:00-05:00
- **Answers**: Q0071 (scope: option B; the rule gaps are being taken separately)
- **Docs**: `factions/syndicate/structures.md > Laser Turret`, `> Chemical Plant`, `> Hideout` (Grenadier cost); `factions/syndicate/units.md > Grenadier`

## What changed and why
User chose **B**: I fill placeholder numbers so the Laser Turret and the Grenadier can be planned into M4. Everything else stays deferred, as the Director proposed (vehicles wait for vehicle movement, the Phantom for air units, then the rest).
- **Chemical Plant:** like the Optics Lab: 2x2, 150 SC, 400 frames, HP 400, armor 1/4, Groupable, no commands.
- **Laser Turret:**
  - 100 SC, 320 frames, HP 300, sight 6;
  - armor 1/6 when raised and 6/30 underground;
  - laser, Universal, 24 damage per 24 frames, range 5 GU;
  - rises in 8 frames, and retracts after 32 frames without a target, taking 8 frames;
  - Agent hotkey D.
- **Grenadier:**
  - HeavyInfantry 36x36, HP 70, armor 1/1, sight 5, speed 5, TunnelSpaceCost 1;
  - grenade AoE: Damage 100 over a radius-40 su circle (about 26 to one fully covered infantry), range 4 GU, projectile 8 su/frame;
  - one grenade per 40 frames;
  - 150 SC, 160 frames, Hideout E.

All values are placeholders, to be tuned. Still open, as rule questions: whether a Tunnel-built Laser Turret uses the Tunnel's operation slot, and the AoE-underground gap.

## Implementation impact
- The Laser Turret and the Grenadier can be scheduled into M4.
- The Grenadier needs the projectile/AoE/Attack Ground combat spike.

## Verification
- A Grenadier's grenade visibly arcs about 2 s to a target 4 GU away; a target that keeps walking escapes.
- A Laser Turret kills a Guard (80 HP, PointArmor 1) in 4 shots, about 6 s.
