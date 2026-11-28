# Q0189 — The Airfield's landings: firing while landed, queued orders, the slot layout, and a full Airfield

- **Date**: 2026-11-27T20:39:02-05:00
- **Blocking**: no. M6-GH2 built landing and rearming on the readings below, marked as placeholders. A sweep changes them if the answers differ.
- **Docs**:
  - `factions/gdo/structures/airfield.md > Landing slots`, `> Returning and rearming`;
  - `factions/gdo/units/hornet.md` (its Attack line);
  - `systems/controls/commands.md` (Shift-queued orders);
  - design change 0458.

## Question
1. **Does a landed aircraft fire?** A Hornet sits on its slot, rearming or rearmed, and an enemy ground unit walks into its range. Does it fire its missiles from the ground, or hold fire until it takes off?
2. **Does a queued order wait for the rearm?** The player gives Return to Base, then Shift-queues an Attack Move. The aircraft lands and rearms. Does the queued Attack Move wait until the rearm is complete, or does it take off at once, as an order given during rearming does?
3. **Is a landed aircraft out of the other aircraft's separation?** Flying aircraft push each other apart (SeparationRadius). Should a Hornet flying low over the Airfield push a landed one off its slot, and should a landed one push those flying over it?
4. **Where are the four slots?** The doc gives the number but not the layout.
5. **What happens when a new aircraft finds no free slot?** An Airfield never queues more aircraft than it has slots for. A slot can still be taken, though, by an aircraft that was reassigned to another Airfield while it was landed here. Does the finished aircraft wait inside, as behind a blocked exit, or appear somewhere else?
6. **The Hornet's damage:** is 45 the number, and the change entry's example shares the rough ones? 0458 and the Hornet's doc give 45 over 32 su, "roughly 30 to a 64x40 vehicle hit square-on, 7 to a 24x24 infantry unit". By the AoE formula, those shares fit a damage of 40. A damage of 45 gives 33.3 and 8.1.

## Context
M6-GH2 built landing and rearming at the Airfield (0428 parts 1–4, 0423's rearming, 0458's burst). The doc settles the main rules: landed aircraft are ground targets, an order during rearming takes off with the ammo so far, and new aircraft appear landed on a free slot. The points above are what it doesn't say, and GH2's readings are:
1. **Landed aircraft hold fire.** They fire only in the air.
2. **A queued order waits for the full rearm.** Only an order *given* during rearming takes off early (0428 point 4); one queued before the landing runs when the rearm is done.
3. **Landed aircraft are out of the air separation.** GH2's golden caught a Hornet flying over and pushing a landed one off its slot. Landed aircraft still count as air for collision layers; only the push is off.
4. **The layout:** a 2x2 grid of slots, one near each corner of the Airfield's footprint.
5. **It waits inside** until a slot frees, as production behind a blocked exit does.
6. **Damage 45**, as the doc says. The test pins 33.3 and 8.1.

Also built as placeholders, which I take as technical or obvious and don't ask about: an aircraft ordered off mid-descent rises from the height it has reached, and "arrived over the slot" means within 8 su.

## Options
1. (a) Hold fire while landed (as built). (b) Fire from the ground, at ground targets, using its ammo.
2. (a) A queued order waits for the full rearm (as built). (b) It takes off at once, with the ammo rearmed so far, as an order given during rearming does.
3. (a) Out of the separation while landed (as built). (b) In it: flying aircraft may push landed ones off their slots, and the landing is retried.
4. (a) The 2x2 grid (as built). (b) Another layout: a row along one side, say.
5. (a) Wait inside (as built). (b) Appear over the Airfield, airborne. (c) Something else.
6. (a) 45, with the example shares only rough (as built). (b) 40, so that the example shares are exact.
