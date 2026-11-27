# Q0179 — The Hand of Heaven: its MinRange back-off leaves its own sight, and a wall can stop its hull aim for good

- **Date**: 2026-11-27T03:34:04-05:00
- **Blocking**: no. M5-N10 built both as the docs read. It stays so until the answer, and the playtest notes say so.
- **Docs**:
  - `systems/controls/command_behaviors.md > AttackingObject` (steps 2–4, step 6 Target lost from vision, Failure) and `> Target inside MinRange`;
  - `factions/cults/units/hand_of_heaven.md` (TurnAngle "the vehicle must stop and turn its hull"; MinRange 4 GU, Range 14 GU, SightRange 7 GU);
  - design change 0363.

## Question
1. **The back-off.** On an explicit Attack at a target inside its MinRange, a lone Hand of Heaven backs off to the middle of the band, 9 GU. That is beyond its own 7 GU sight, so it loses the target and the Attack ends without a shot. Should it?
2. **A blocked hull aim.** The Hand of Heaven aims by stopping and turning its hull. When a wall or cliff beside it stops the turn, what should it do? Today it waits, turned short, and never fires.
3. **A doc check.** `AttackingObject`'s step 4 says a turret unit whose target is out of its arc "resume[s] MovingToObject". For the Hand of Heaven, N10 followed its own doc instead: it stops in range and turns its hull. Is that right? If so, should step 4 say so?

## Context
M5-N10 built the Hand of Heaven. Its tests cover 0363's Verification.

**1. The back-off.**
- `> Target inside MinRange` (Q0054) sends the unit to the middle of the band between MinRange and Range, which is (4 + 14) ÷ 2 = 9 GU.
- The Hand of Heaven sees 7 GU: "it needs spotters for its full range".
- **What happens alone:** it backs off, the target leaves its sight at about 6.8 GU, and step 6 (Target lost from vision) ends the behaviour at once. The unit is idle, unfired, and the enemy it was told to hit is close by in the fog.
- **With a spotter** that keeps the target in view, it backs off to 9 GU and fires as designed.

**2. A blocked hull aim.**
- **As built:** a spin cut by a contact backs off as V3's spin at rest does, else creeps toward its aim one tick's Acceleration at a time. An own unit in the way is asked to make way, and enemy infantry its hull sweeps are crushed (0418).
  - With a wall alongside it, it gets round in about 15 ticks instead of 5.
  - With the wall between it and the target, the creep goes into the wall. The hull never reaches the launcher's ±10°, so it never fires, and the Attack doesn't end.
- **Why it matters:** this is the case of a Hand of Heaven parked along a cliff edge and sent to fire across it, which a long-range missile platform is likely to meet.
- **The Failure rule** ("repeatedly fails to make progress toward the target, behavior complete") is written for the approach. Read for the aim, it would end the Attack and leave the unit idle.

**3. Step 4.** The Hand of Heaven's doc: "the launcher barely turns, so to aim at most targets the vehicle must **stop and turn its hull** to face them."
- With step 4 as written, it drove on along its path to the target. The path ended at a tile 14.7° off a building's centre, outside the ±10° arc, so it never fired.
- **As built:** a turret unit whose base turns in place (today only the Hand of Heaven) stops once the target is in range and beyond MinRange, wherever its arc is, and spins its hull to aim. Every other turret unit keeps step 4 as written.

## Options
**1. The back-off:**
- **(a) As built:** it backs off to the band's middle, and alone it loses the target. "It needs spotters" covers this case too.
- **(b) Not beyond its own sight:** it backs off to the band's middle or the edge of its sight, whichever is nearer (about 7 GU here), so a lone Hand of Heaven keeps the target in view and fires.
- **(c) Only past MinRange:** it backs off just beyond MinRange, about 4 GU, for units whose sight is shorter than the band's middle. The Q0054 rule stays for the rest.

**2. A blocked hull aim:**
- **(a) As built:** it waits, and the player moves it.
- **(b) Failure applies:** if the hull makes no progress toward its aim for a while (say 2 s), the Attack ends and the unit is idle, as a stuck approach does.
- **(c) It repositions:** it drives a short way clear of the obstacle (for example back along its path, or straight away from the contact), then turns. A new movement rule; more work.

**3. Step 4:**
- **(a) As built:** units that aim with their hull stop in range and turn. Step 4 gains a line saying so.
- **(b) Step 4 as written for every turret unit:** the Hand of Heaven then needs another way to get within its arc of a building it approaches.
