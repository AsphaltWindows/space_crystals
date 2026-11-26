# 0386 — A hit absorbed by Shroud reveals its attacker to the targeted object's owner

- **Date**: 2026-11-27T11:40:00-05:00
- **Answers**: Q0143
- **Docs**: `factions/cults/units/disciple.md > Spells > Shroud` (Reveals); `core/vision.md > Attacks from outside vision`

## What changed and why
User: **(a)**. "If it's your units being shot at, the attacker is revealed." An absorbed hit, whether a direct shot, a projectile or an area hit, counts as landing on the object it was aimed at. It reveals the attacker to that object's owner for the usual 1.5 s. The cloud's later release reveals nobody.

## Implementation impact
None; this is as built in M5-N4b.

## Verification
- An unseen enemy shoots a unit standing under a Shroud cloud.
- The defender sees the attacker for 1.5 s, although the damage is buffered.
