# Q0200 — The Prism's thrust: orders given during a burst, and a thrust at the Prism's own position

- **Date**: 2026-11-28T05:56:14-05:00
- **Blocking**: no. Proceeding on (a) for both, as built by phase SP2a and marked as placeholders in `crates/sim/src/thrust.rs`.
- **Docs**: `factions/syndicate/units/prism.md > Movement`; change 0434 (point 2).

## Question
1. **What does an order given during a burst do to the burst?** A burst carries the Prism its 3 GU over 16 ticks.
2. **What does a thrust aimed at the point the Prism stands on do?**

## Context
- **0434** says only this: "Thrust is targeted (click a point; always the full 3 GU toward it) and, as every command, Shift-queues", and each queued thrust "spends its charge when it starts".
- **As built,** a burst is separate from the Prism's own movement. The Prism's own move (its very slow drift) goes on during the burst, and the burst's step is added to it.
  - A Move or a Stop given during a burst changes the drift only. The burst runs its course.
  - A thrust that starts during another's burst, queued or not, waits for that burst to end before it spends its charge. So each charge moves the Prism its full 3 GU.
- **A click on the Prism's own position** gives no direction. As built, the Prism thrusts along its facing.

## Options
**Point 1:**
- **(a) As built: a burst can't be stopped.** Orders steer the drift only, and a new thrust waits for the burst to end.
- **(b) A new order (not Shift-queued) ends the burst where the Prism is.** A thrust given during a burst then starts at once, from there. Its charge is spent; the unfinished burst's distance is lost.

**Point 2:**
- **(a) As built: along the Prism's facing.**
- **(b) Refused,** like an order with no target, and no charge is spent.
