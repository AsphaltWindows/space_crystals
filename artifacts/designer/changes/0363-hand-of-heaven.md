# 0363 — The Cults missile platform is the Hand of Heaven: ground only, MinRange, placeholder stats

- **Date**: 2026-11-27T01:25:00-05:00
- **Answers**: —
- **Docs**: `factions/cults/units/hand_of_heaven.md` (renamed from `missile_platform.md`); indexes; `factions/cults/structures/chop_shop.md`

## What changed and why
User:
- The missile platform is named **Hand of Heaven**: righteous help from the sky.
- It is **ground only**, and **has a MinRange**.
- It stays **tracked**. The user weighed wheeled, but a minimal turret on wheels would be frustrating to aim.

Placeholders (designer):

| Stat | Value |
|---|---|
| Size and toughness | 64x40, HP 200, armor 2/2, sight 7 |
| Movement | speed 5 su/f |
| Turret | 20° at 4°/f |
| Missile | 600 total AoE over a 1.5 GU radius, a siege profile (a 3x3 building takes it all, a lone infantry about 12) |
| Range | 14 GU, MinRange 4 |
| Flight | projectile 8 su/f (about 7 s at full range) |
| Attack cycle | Aim 48 / Firing 2 / Cooldown 4 / Reload 96 |
| Cost | 150 SC + 3 Recruits, 480 f |

It uses the shared reveal-on-landing rule.

## Implementation impact
New unit: a DoublyDisjointed area attack with a narrow-turret tracked base.

## Verification
- Fire at a building 12 tiles away: it aims for 3 s, launches, and can drive off at once.
- The missile lands about 6 s later.
- A unit inside 4 tiles can't be targeted.
