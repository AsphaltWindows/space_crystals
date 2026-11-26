# Q0143 — Does an attack absorbed by a Shroud cloud reveal its attacker?

- **Date**: 2026-11-26T10:46:00-05:00
- **Blocking**: no. M5-N4b proceeds with (a): an absorbed hit reveals its attacker as if it had landed, and a release reveals nobody.
- **Docs**: `core/vision.md > Attacks from outside vision` ("The attacker is revealed"); `factions/cults/units/disciple.md > Spells > Shroud`

## Question
When an attack from outside the defender's vision hits an object under a Shroud cloud, its damage goes into the cloud's buffer instead of the object. Does the attacker still become visible to the defender? And does the later release reveal anyone?

## Context
- `core/vision.md > Attacks from outside vision`: an attacker outside your vision "becomes **visible to you** for **1.5 seconds** ... after each shot". "**For projectiles and area damage** ... the reveal happens **when the damage lands**, not at the throw, and only to the owners of the objects it **damaged**; a dodged projectile or a blast that damages nothing reveals nothing."
- The Shroud section doesn't mention reveals.
- When a hit is absorbed, the object it was aimed at takes no damage then. Its share of the release comes up to 8 frames later, as one area hit with no attacker of its own.
- For a direct shot, "after each shot" reads as revealing regardless. For a projectile or an area hit, "only to the owners of the objects it damaged" could read as no reveal, since nothing is damaged when it is absorbed.

## Options
- **(a) An absorbed hit reveals its attacker as if it had landed** (built). The defender sees who is shooting into the cloud. The release reveals nobody.
- **(b) Shots reveal, absorbed projectiles and area hits don't.** "After each shot" holds for direct fire; a projectile or blast whose damage was absorbed damaged nothing, so it reveals nothing. This is a small change in the absorption hook.
- **(c) Nothing absorbed reveals.** A cloud also hides who is shooting into it, which strengthens Shroud as a defence against attackers outside vision.
