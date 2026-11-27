# Q0169 — GDO Supplies: what they buy, and the Supply Chopper's repair on its Tower

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GS, the Supply Tower and the Supply Chopper's supply runs, proceeds on the placeholders in Options: Supplies are gathered but buy nothing, and a landed Chopper is repaired at the repairers' rate.
- **Docs**: `factions/gdo/overview.md > GDOResources > Supplies`; `factions/gdo/economy.md > Starting Conditions`, `> SupplyTower > Landing and Attachment`; `factions/gdo/units/{supply_chopper,field_engineer}.md`; changes 0077, 0366

## Question
1. What does the GDO spend Supplies on?
2. How fast is a Supply Chopper repaired while landed on its Tower?

## Context
- **What Supplies buy:** they are "Spent on advanced buildings, units, and research", but no GDO cost in 0366 uses them.
  - 0077, answering Q0022: "the GDO supply loop (Supply Tower, Supply Chopper, deliveries) is deferred to the air milestone". "The missing design pieces will be filled in during the GDO numbers pass", the first of them "what GDO spends Supplies on". Q0022 had pointed out: "Collected supplies would have no use."
  - "Starting Supplies: 0 (placeholder; revisit when GDO Supply costs are designed, user 2026-11-24)".
  - 0366: "The only GDO TBD left is the Supply Chopper's transport mechanics, a design question."
  - M6 builds the GDO's aircraft, so I read 0077 as putting the Tower and the Chopper's supply runs in M6, before any Supply cost exists. The Chopper's transport stays out.
  - The Syndicate's Agents gather Supplies from the same Supply Delivery Stations.
- **The repair:** "A Supply Chopper that lands on its attached Supply Tower drops off all carried supplies and is gradually repaired at no cost." No rate is given.
  - A repairer takes "half the target's build time" from 0 to full HP. The Chopper has MaxHP 150 and is built in 160 frames (10 s).

## Options
1. **What Supplies buy:**
   - **(a) Nothing yet** (the placeholder): Supplies are gathered and shown, but buy nothing until the numbers pass. GDO Choppers can still empty the stations the Syndicate's Agents use.
   - **(b) Supply costs now,** on some of 0366's advanced buildings, units or research, as "Spent on advanced buildings, units, and research" reads.
2. **The repair rate:**
   - **(a) The repairers' rate, free** (the placeholder): 0 to full in 80 frames (5 s), 30 HP a second. A First Aid Tent heals 2 HP a second.
   - **(b) A slower rate,** e.g. the Tunnel Network's 1% of MaxHP a second: 1.5 HP a second, 100 s from 0 to full.
