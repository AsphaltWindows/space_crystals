# Q0039: Beacon Deployment and Warp In: what the transforming object is

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: no. E and F build the proposals. Answers to points 1–3 before F starts avoid rework.
- **Docs**: `factions/colonists/economy.md > Beacon Deployment`, `> Warp In`, `> Beacon Capacity Retention`; `units.md > Drone` intro (the Drone's 1 capacity is freed when it deploys); `core/game_rules.md > Defeat Condition`; `core/tech_and_research.md`.

## Question
1. **During deployment, is the object a unit or a structure, and when is the Beacon's footprint claimed?** **Proposal:** from the first frame it is a Beacon under deployment.
   - It is a structure on its 2x2 footprint: units can't walk onto it, it can't move, and it counts as a structure.
   - It keeps the Drone's identity, so it stays selected.
   - A cancel turns it back into the Drone at the centre of the footprint.

   Warp In works the same way: the larger footprint is claimed at the start.
2. **HP and Shields interpolation when damaged.** The formulas give the values of an undamaged object. **Proposal:** the formula sets the maximum, and the current value rises by the same amount each frame, so damage taken stays taken. Shields regenerate up to the current maximum.
3. **What a cancel restores** ("no HP/Shields loss"). **Proposal:** the original form's maxima, with the same fraction of HP and Shields it has when cancelled. Damage taken during the transformation stays.
4. **Beacon Capacity during transformations.** **Proposal:** a deploying Drone still uses 1, and its Beacon provides nothing until complete. A building being warped in keeps providing 5 (`> Beacon Capacity Retention`).
5. **Defeat.** Beacons count. **Proposal:** a Beacon under deployment counts too. A player who cancels their last such Beacon, owning no other structure, is defeated in that tick.
6. **A Drone carrying a load deploys.** **Proposal:** the load is lost when the deployment completes, and given back on a cancel.
7. **Destroyed during a Warp In: any refund?** **Proposal:** none. The docs say only that the Beacon is destroyed with it.
8. **A building being warped in** shows only Cancel, and its functions (production, drop-off) start at completion. **Proposal:** yes.

## Context
The docs fix the duration, the formulas, vulnerability, cancel and the 75% refund. They don't say what the object is in between. These answers decide how E and F model it (one object whose kind changes at the start, plan decision T15), and they are costly to change after F.

## Options
- **Point 1:**
  - A, a structure from the first frame, as proposed;
  - B, a unit until complete. Units may then walk onto the footprint during the 3 s, and completion would need a second check.
- **Point 2:**
  - A, the maximum interpolates and the current value rises with it, as proposed;
  - B, the current value keeps its fraction of the maximum.
- **Point 3:**
  - A, keep the fraction, as proposed;
  - B, restore the values from when the transformation started (damage during it is forgiven);
  - C, take away what the interpolation added, which could kill it.
- **Other points:** the proposal, or a rule you give.
