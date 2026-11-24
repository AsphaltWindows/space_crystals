# 0072 — Recruits are Groupable

- **Date**: 2026-11-24T00:35:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units.md > Recruit > Groupable` (false → true)
  - `factions/cults/units.md > Recruit ObjectInterfaceState` (the multi-select paragraph)
  - `systems/controls/selection_and_hud.md > SelectionGroup > No smart-casting` (which workers are Ungroupable)
  - `systems/controls/selection_and_hud.md > AwaitingTarget[CommandType] > Shift while placing` (Recruit example)
  - `core/entities.md > Object Type > Groupable` (pointer)

## What changed and why
This resolves the conflict noted in 0071:
- Recruits were Ungroupable, so the panel Construct sent only the active Recruit.
- `factions/cults/economy.md > Build Command` said all selected Recruits go.

The user chose to make Recruits **Groupable**:
- All selected Recruits are one SelectionGroup, so **Construct** and **Assist Construction** from the panel go to **every selected Recruit**. They build the same building together, which speeds it up (BaseConstructionTime scaling).
- This isn't smart-casting, because every member carries out the order.

Workers that are **used up** by a build stay Ungroupable, because each order needs one specific worker:
- the Agent goes inside the structure it builds;
- the Drone becomes its Beacon.

## Implementation impact
- Recruit: Groupable = true. It's now a single Tab stop and a single SelectionPanel group.
- Construct sends every Recruit in the ActiveGroup, as the Build Command doc already says. The cost is paid once per building, when the order is given.
- Shift while placing (0071): the whole Recruit group builds the queued buildings one after another.
- 0071 said to send only the active Recruit until this was resolved. That is **replaced**: send all of them.

## Verification
1. Select 4 Recruits, press A (Construct), choose a building and place it. All 4 walk to the site and build together. The build takes about a quarter of BaseConstructionTime once all 4 are there. The cost is deducted once.
2. With 4 Recruits selected, hold Shift and place 2 buildings. The group builds the first one together, then moves to the second.
3. Click one Recruit's portrait and Construct. Only that Recruit goes.
4. Pressing Tab with Recruits and Soldiers selected stops once on the Recruits as a group.
