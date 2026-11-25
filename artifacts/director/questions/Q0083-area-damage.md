# Q0083 — Area damage: friendly fire, the reveal, Attack Ground's end, and drawing

- **Date**: 2026-11-25T01:47:32-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, marked as placeholders in M4-O (the combat spike) and M4-U (the Grenadier).
- **Docs**: `systems/combat.md > AoEDamage`, `> DoublyDisjointed`; `systems/controls/behaviors.md > AttackingLocation`; `core/vision.md > Attacks from outside vision`; `factions/syndicate/units/grenadier.md`

## Question
1. **Friendly fire.** Does a blast damage the attacker's own units and structures, and the attacker itself?
2. **The reveal.** A grenade lands on your units from outside your vision. Is the thrower revealed to you, and when: at the blast, or at the throw? And if the blast damages nothing?
3. **Attack Ground's end.** Does an Attack Ground order fire one shot and complete, or keep firing at the point until given another order?
4. **Drawing.** Is a revealed thrower's grenade at your units drawn whole, as shots from a revealed attacker are (0244), or clipped to your vision?

## Context
- `> AoEDamage`: "AoE only affects units whose domain matches the attack's TargetDomain". It names no owner filter.
- `core/vision.md > Attacks from outside vision`: "a unit or structure that attacks one of your units or structures while outside your vision becomes **visible to you** for **1.5 seconds (24 frames)** ... after each shot". With a 2 s flight, the throw and the hit are far apart in time, and a dodged grenade hits nothing.
- `behaviors.md > AttackingLocation`: "Behavior complete after attack effect is applied". `> DoublyDisjointed`: "The unit is free after the projectile spawns".
- Vision draws a projectile "only along the part of its path that crosses tiles **Visible** to that player", with 0244's exception for revealed attackers' shots.

## Options
1. **Friendly fire:**
   - **A (proposed, literal).** Everything whose domain matches, any owner, the attacker included. Grenadiers must not throw into their own melee.
   - **B.** Enemies only (and neutral objects). Safer to use; needs an owner filter that the text doesn't state.
2. **The reveal:**
   - **A (proposed).** At the blast, if the thrower is alive, to the owner of each object the blast **damaged**. A dodged grenade reveals nothing.
   - **B.** At the throw, to the owner of the target. This reveals even when the grenade is dodged, and for a Ground target at a hidden expansion it would tell the thrower nothing new but give the target's owner warning.
3. **Attack Ground:**
   - **A (proposed, literal).** One shot, and the order completes at the blast; the next queued order follows.
   - **B.** It keeps firing at the point, like an attack on a unit that never dies, until another order. That is the usual RTS behaviour for shelling a spot, but "Behavior complete after attack effect is applied" says otherwise.
4. **Drawing:**
   - **A (proposed).** As for shots: a revealed thrower's grenade at your objects is drawn whole.
   - **B.** Clipped to your Visible tiles like any projectile.
