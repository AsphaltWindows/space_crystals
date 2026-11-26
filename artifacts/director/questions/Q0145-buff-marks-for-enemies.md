# Q0145 — Does a player see the Shrine's buffs on enemy units?

- **Date**: 2026-11-26T12:17:36-05:00
- **Blocking**: no. M5-N5 proceeds with (a): a buffed unit shows a small mark beside its HP bar to every player who sees the unit, enemies included.
- **Docs**: `factions/cults/structures/shrine.md` (Health Regeneration, Energy Regeneration); `systems/combat.md > Shields > Display` (the bars over units)

## Question
When a Shrine buffs units (Q: health regeneration, W: energy regeneration), should the enemy see that those units are buffed?

## Context
- The client shows a buffed unit with a small mark beside its HP bar.
- HP bars are drawn over damaged and selected objects, whoever owns them. Shield and energy bars follow the same rule, so an enemy already sees your units' HP, Shields and energy.
- The Shrine's design doesn't say who sees a buff. The marks' look is a placeholder for the art pass (Q0001).

## Options
- **(a) Everyone who sees the unit sees its buffs** (built), as with the bars. The enemy can tell a healing army from a fresh one and react, for example by focusing it before the heal lands.
- **(b) Only the owner (and allies) see their buffs.** The enemy reads the buff only from its effect, HP climbing back. This is a small client change.
