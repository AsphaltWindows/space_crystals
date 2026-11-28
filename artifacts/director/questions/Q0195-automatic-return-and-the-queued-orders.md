# Q0195 — The automatic return and the orders queued behind the one the aircraft ran out in

- **Date**: 2026-11-28T02:11:43-05:00
- **Blocking**: no. Proceeding on option (a), as built: the automatic return replaces all of the aircraft's orders. After landing, the aircraft rearms fully and stays landed.
- **Docs**:
  - `systems/mechanics/aircraft_landing.md > Rearming archetypes > Assigned base`: the automatic return; Shift-queued orders run as soon as it has landed;
  - `> Network base`: the Specter's queued orders;
  - change 0466.

## Question
A Hornet is given an Attack Move to A, then a Shift-queued Move to B. It runs out of ammo during the Attack Move and flies home by itself. What becomes of the queued Move to B?

## Context
- **The Assigned base** says: "Automatic return when out of ammo: it flies to its base, lands and rearms. Any other order interrupts it."
- **0466** says that an order Shift-queued behind an automatic return runs right after landing, with the ammo rearmed so far. It doesn't say whether the orders already queued behind the one the aircraft ran out in stay in the queue, behind the return.
- **Q0189** asked only about orders that the player queues behind a Return to Base.
- **The code drops them** (a placeholder since slice GH2), as the Network base's rules do for the Specter: "it ignores later queued orders and goes home to rearm", and "on entering, its orders are wiped".

## Options
- **(a) Dropped, as built.** The same as the Specter's. The aircraft lands, rearms fully and stays landed, and the player gives it new orders. No code change.
- **(b) Kept, and run on landing, by 0466's rule.** The aircraft takes off almost at once, with almost no ammo.
  - The automatic return fires when the last round goes. With no rounds, it would fly the queued orders unarmed and never return by itself, unless that rule changes too.
  - A small code change.
- **(c) Kept, and run after a full rearm.** The automatic return becomes "go home, rearm fully, then carry on with the queue", so the aircraft resumes the player's plan armed.
  - A small code change.
  - It differs from 0466's rule for orders that the player queues behind a return.
