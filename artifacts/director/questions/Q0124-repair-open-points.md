# Q0124 — Repair: when it ends, an empty bank, and which targets

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a) on all six.
- **Docs**: `factions/cults/units/recruit.md > Repair`; `factions/syndicate/units/agent.md > Repair`, `> Agent ObjectInterfaceState`; `factions/syndicate/tunnels.md`; `factions/cults/units/jackal.md`, `sparrow.md`; entries 0321, 0325, 0326

## Question
Repair is being built from 0321. Some points are open:
1. **When the order ends.** Do repairers stop once the target is at full HP, or stay on it and repair it again when it takes more damage?
2. **An empty bank.** Repair is "paid gradually as HP is restored". When the player runs out of Space Crystals, do the repairers wait and go on when crystals come in, or does the order end?
3. **Aircraft.** The Sparrow "never lands". Can it be repaired?
4. **Tunnel expansions.** Can Agents repair a Hideout, Chemical Plant or Optics Lab? If so, for free like the Tunnel, or for their price?
5. **A Tunnel's build time at Tiers 2 and 3** (MaxHP 800 and 1000): is it its construction's 480 frames, or construction plus its upgrades (960 and 1440)? It only sets the rate, since Tunnel repair is free.
6. **The Jackal's price** is "2 Recruits, 75 Space Crystals". Is half of it 37.5 crystals, with no Recruits?

## Context
- **The numbers:**
  - one Recruit repairs an Armory (300 HP, 480 frames, 200 SC) from 0 to full in 240 frames, for 100 SC;
  - an Agent repairs a Laser Turret in 160 frames for 50 SC, and a Tier 1 Tunnel in 240 frames for nothing.
- **The bank holds whole crystals,** so repair pays one crystal per equal share of the HP: a Jackal costs 38 crystals from 0 to full.
- **Settled since this was drafted:** 0326 puts the Agent's Repair on D, and 0325 keeps Enter for a right-click on a damaged building that can be entered.

## Options
1. **(a)** stop at full HP; **(b)** stay on the target until given another order.
2. **(a)** wait, and go on when crystals come in; **(b)** the order ends.
3. **(a)** no, aircraft aren't repaired; **(b)** yes, by a repairer under it.
4. **(a)** no: the doc names only Tunnels and Laser Turrets; **(b)** yes, for free; **(c)** yes, for half their price.
5. **(a)** 480 frames at every tier; **(b)** construction plus the upgrades.
6. **(a)** yes, crystals only; **(b)** something else.

Proposed: (a) on all six.
