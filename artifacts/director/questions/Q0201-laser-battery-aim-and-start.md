# Q0201 — The Laser Battery: aiming between surges and at a new target, and its charge when trained

- **Date**: 2026-11-28T07:27:36-05:00
- **Blocking**: no — proceeding on (a) for all three, as built in SLa
- **Docs**: `factions/syndicate/units/laser_battery.md > Battery`, `> Attack: constant beam with surges` and its **Placeholder stats** ("surge interval **16 frames** (1 s)", "AimDuration 8 frames"); `systems/combat.md > Aiming`, and its attack cycle ("Aim + Firing + Cooldown + Reload")

## Question
1. Is the 16-frame surge interval the time between surges while the beam stays on its target, so that the Battery aims only when it engages? Or is it the whole attack cycle, with an 8-frame aim before every surge?
2. The Battery's target changes (it died, or the player ordered another), and the turret already faces the new one. Does the Battery aim again, 8 frames, before its next surge?
3. What charge does a newly trained Laser Battery start with? The Disciple's pool has a stated start (50 of 200); the battery has none.

## Context
SLa built the battery as the unit's energy pool, and the surge attack that spends from it.

The general combat model aims before every attack (`systems/combat.md`: the cycle is Aim + Firing + Cooldown + Reload). The Battery's doc describes a constant beam with periodic surges. Its arithmetic needs surges 16 frames apart: a normal surge draws twice one interval's charge, so the battery drains at 1× its charge rate while attacking, and an empty battery's surge uses half a normal one's. Both readings of Question 1 give that spacing:
- **as built:** an 8-frame aim when the Battery engages, then a surge every 16 frames for as long as the beam holds (the target valid, seen, in range and in the turret's arc);
- **the general cycle:** 8 frames of aim and 8 more of firing and reload, aiming again before each surge.

Questions 2 and 3 are details the docs don't settle. SLa marked each as a placeholder.

## Options
1. **(a)** As built: aim on engaging, then a surge every 16 frames while the beam holds. A rule for surge attacks only. **(b)** The general cycle: an 8-frame aim before each surge. The rule for surge attacks is dropped. In play, the two differ only in what interrupts the Battery between surges, since the Aiming phase is interruptible.
2. **(a)** As built: no new aim if the turret already faces the new target, so the next surge comes 16 frames after the last. **(b)** An 8-frame aim at any new target: 24 frames between the last surge at the old target and the first at the new.
3. **(a)** Full, 100 (as built). **(b)** Empty. **(c)** Another start, as the Disciple's 50 of 200.
