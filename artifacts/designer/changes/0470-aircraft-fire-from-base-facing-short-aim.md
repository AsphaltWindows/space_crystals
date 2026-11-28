# 0470 — Limited-ammo aircraft fire from their base, facing the target; Specter loses its turret; short Aim

- **Date**: 2026-11-28T10:20:00-05:00
- **Answers**: Q0192
- **Docs**:
  - `factions/gdo/units/dragonfly.md > Placeholder stats` (Attack)
  - `factions/gdo/units/hornet.md > Placeholder stats` (Attack)
  - `factions/syndicate/units/specter.md` (No turret; Placeholder stats)
  - `factions/gdo/structures/airfield.md > Returning and rearming` (Glider landing approach)

## What changed and why
1. **No launch cone** (user; option a, as built). The Dragonfly must **face its target** to launch. The same goes for the Hornet and the Specter.
   - **Why no turret** (user): turrets keep engaging during a Move. Limited-ammo aircraft fire **from their base, with no turret**, so they never spend their payload on enemies they pass on a Move; base engagement isn't active during Move. On Attack Move they engage as usual.
   - **The Specter loses its turret** (was 20°). It drops from its base, facing its target, nearly over it.
   - **Shorter Aim** (user's idea; values are designer placeholders) keeps the runs smooth:

     | Aircraft | Aim (frames) |
     |---|---|
     | Dragonfly | 2 (was 8) |
     | Hornet | 2 (was 4) |
     | Specter | 2 (was 4) |

2. **Landing approach** (designer placeholder, as built). The aircraft slows to IdleSpeed 4 GU out, lines up straight, comes round again if it overflies, and lands facing the way it came. With no free slot, it circles over the Airfield's centre.

## Implementation impact
- **Specter:** remove its turret. It attacks via the base channel and the glider strafing pattern.
- **Aim changes:** as in the table above.
- **Hornet:** it already had no turret. It is a HoverCraft and turns in place onto its target.
- **The circling-Sparrow case from the test:** the glider attack pattern (command_behaviors.md > AttackingObject > GliderMovement exception) flies past until the target's centre is out of range, then loops back. From several GU out, a Sparrow on a 1 GU circle sits within a few degrees of a straight approach, so head-on passes should line up. If the Dragonfly still can't launch in practice, report it with numbers; it's a tuning matter for its turn and speed.

## Verification
1. Order a Move past enemy ground units and aircraft with Hornets, Dragonflies and Specters. None of them fires or drops.
2. Attack Move: they engage.
3. A Dragonfly attacking a circling Sparrow flies out, comes back head-on and launches within a pass or two.
4. A Specter on Attack flies straight over its target and drops.
