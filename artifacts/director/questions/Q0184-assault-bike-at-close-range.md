# Q0184 — The Assault Bike at close range: its turning circle, its own blast, and Attack Ground

- **Date**: 2026-11-27T10:43:20-05:00
- **Blocking**: no. M6-SV merges as described below, with tests that record each case. It stays so until the answer.
- **Docs**:
  - `factions/syndicate/units/assault_bike.md` and its `> Placeholder stats`;
  - `systems/controls/command_behaviors.md > AttackingObject` (steps 2 and 4) and `> Target inside MinRange` (Q0054);
  - `systems/combat.md > AoEDamage` (friendly fire, Q0083: "the attacker itself");
  - Q0179, part 3 (the Hand of Heaven and step 4).

## Question
How should the Assault Bike handle a target close beside it?
1. **A target it can't point at:** a target in range inside its turning circle, which it can't bring into its arc by driving on.
2. **Its own blast:** it fires from closer than its blast's radius, and the blast hits it.
3. **Attack Ground** at a spot in range but out of its arc.

## Context
**The numbers:**
- The Bike's turret turns **40°** in all (±20° of its facing).
- It is wheeled, with a **128 su** turning radius, so its turning circle is 256 su across. It can't turn on the spot.
- Its mortar reaches **5 GU (320 su)**, with MinRange 0 and a blast of **48 su**. Area damage hits "the attacker itself" (Q0083).

So the Bike can't point at most targets near its side without driving a long way around, and its range is little more than its turning circle.

**What M6-SV built:**
- **A target in range but out of its arc:** step 4 says "resume MovingToObject", so it drives on toward the target, turning, and stops to fire once the target will still be in its arc where it comes to rest.
  - This was a control fix. At first it braked as the target entered its arc, and coasting straight, it lost the target again.
  - With the fix, targets at 45°, 60° and 90° off, 4 GU away, are hit.

**What happens still:**
1. **Inside its turning circle,** near its rim, it can't get the target into its arc. In a grid of 168–224 su and 50–130° off its facing, 28 of 112 cases fail.
   - Its move toward the target ends in contact (Arrived), 49–57 su from the target and 20–38° off its facing.
   - It then gives the Attack up and stands idle. It never loops around.
2. **Its own blast:** at 60° and 90° off, it reaches the target's side before the target is in its arc. It then fires from 48 su, and its blast takes 13–15 of its own 100 HP each round. This is the design's friendly-fire rule as written.
3. **Attack Ground** at a spot in range but out of its arc, 96–256 su away and 30–150° off its facing.
   - **As built** (step 4 as written): it circles the spot forever, stopping once a lap, and never fires.
   - **With the fix applied there,** it drives onto the spot and shells itself: HP 100 → 53–90.
   - It is left as built until the answer.

## Options
These can combine.
- **(a) A MinRange for the mortar,** for example 1–1.5 GU, beyond its blast. The existing rule (`> Target inside MinRange`) then sends it back to the middle of its band before firing. For a wheeled Bike, that means driving out and coming round again. This also covers question 2, and Attack Ground inside it is refused or backs off as the rule says.
- **(b) A Bike that can't point at a target drives out and comes back:** a loop wide enough to bring the target into its arc. This is new behaviour for wheeled vehicles with a narrow arc (today only the Bike, and the Raptor when it goes live).
- **(c) Give the target up** when it can't be reached, as built for Attack. For Attack Ground, end the order the same way, rather than circling.
- **(d) A wider turret or a tighter turning circle** for the Bike, so that fewer targets fall outside what it can reach.
- **For its own blast:** keep the friendly-fire rule as written (the Bike shells itself at point-blank range, a cost of the weapon), or exempt a shooter from its own blast.
