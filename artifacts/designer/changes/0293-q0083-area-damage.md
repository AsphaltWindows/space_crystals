# 0293 — Area damage: friendly fire, reveal on damage, Attack Ground keeps firing, projectiles clipped to vision

- **Date**: 2026-11-26T10:15:00-05:00
- **Answers**: Q0083
- **Docs**: `systems/combat.md > AoEDamage` (Friendly fire); `core/vision.md > Attacks from outside vision`; `systems/controls/behaviors.md > AttackingLocation`; `systems/controls/commands.md > AttackGround`

## What changed and why
User:
1. **Friendly fire (A).** A blast hits everything whose domain matches, whoever owns it, the attacker included.
2. **Reveal (A).** A projectile or area attacker is revealed when the damage lands, only to the owners of the objects it damaged, and only if the attacker is still alive. A dodged grenade reveals nothing.
3. **Attack Ground keeps firing (B)** at the location until another order. This replaces "complete after the attack effect is applied", so queued orders after it never start on their own. The Specter follows its ammo rules: after its bomb it flies home to re-arm, and its orders are wiped there.
4. **No exception for drawing.** Every shot, tracer or projectile is drawn **only over the viewer's Visible tiles**, including a revealed attacker's shots at you. The user's reason: you should only see the attacker when it damages you, and only the visible part of projectiles in general. **This replaces the Q0056/Q0075 exception (0244).**

## Implementation impact
- Remove the whole-path drawing of revealed attackers' shots.
- Add the AoE owner filter: none, so everything in the circle is hit.
- Reveal timing moves to damage time for projectiles.
- AttackingLocation loops.

## Verification
- A Grenadier throwing into its own melee damages its own units.
- A grenade dodged from the fog reveals nothing.
- An Attack Ground keeps shelling until Stop.
- A revealed attacker's shots appear only from the edge of your vision.
