# 0019 — Aircraft rearm time is per unit of ammo, set per aircraft type

- **Date**: 2026-11-23T14:54:17-05:00
- **Answers**: —
- **Docs**: `factions/gdo/structures.md > Airfield > Aircraft Support > Returning and rearming`, `factions/gdo/units.md > Air-to-Ground Hovercraft` and `> Air-to-Air Fighter` (new RearmTimePerAmmo stat)

## What changed and why
User decision: rearming takes a time **per unit of ammo**, and the per-ammo time **differs per aircraft type**. Each aircraft has a RearmTimePerAmmo stat, and a rearm takes RearmTimePerAmmo × missing ammo.

## Implementation impact
New per-unit stat RearmTimePerAmmo (values TBD). Rearm duration scales with how much ammo is missing.

## Verification
(For later.) A Hovercraft that fired 2 of 5 missiles rearms in 2 × its RearmTimePerAmmo, and one that fired all 5 takes 5 ×. The Fighter uses its own per-missile time.
