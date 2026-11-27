# 0423 — Low Power slows every consumer's activity: cooldowns, turret fire, rearming

- **Date**: 2026-11-26T18:15:00-05:00
- **Answers**: Q0157
- **Docs**: `factions/gdo/overview.md > GDOResources > Power` (Power producers are never slowed / Slowed)

## What changed and why
The user chose (b): "all power-consuming buildings operate slower" means **everything** they do. Beyond construction, production and mining, the Power ratio now also slows ability cooldowns (Scan), weapons (the Missile Turret's full attack cycle) and rearming at the Airfield. Running short of Power weakens defences, and Generators become raid targets.

## Implementation impact
Replaces the placeholder (a). Each frame, these timers advance by the speed ratio (generated ÷ consumed, capped at 100%), as production does. Applies to any future consuming building too. Producers (Deployment Center, Generators) are still never slowed.

## Verification
- Run Power to about 59% (e.g. 30 generated, 51 consumed): Scan's cooldown takes about 102 s; a Missile Turret fires about every 41 frames; a Hornet's full rearm takes about 17 s.
- Build a Generator back to positive: all return to full speed at once.
