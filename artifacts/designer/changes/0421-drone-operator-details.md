# 0421 — Drone Operator: R at the Barracks; drone details; no rebuy while a drone is out

- **Date**: 2026-11-26T18:05:00-05:00
- **Answers**: Q0155
- **Docs**: `factions/gdo/units/drone_operator.md > Drone Stock`, `> Launching and piloting`, `> Placeholder stats`; `factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`

## What changed and why
User decisions:
- **3 (b)**: Build Drone can start only once the operator's drone has **ended** (detonated, fallen, out of battery, lost to the leash). Drones stay timed bursts.
- **4 (a)**: launching **doesn't change the selection**; the player clicks the drone to fly it.

Designer placeholders (standing permission):
- **1**: Barracks **R: Build Drone Operator** (100 SC, Career Campus).
- **2**: drone UnitBase **HoverCraft** (DragMovement numbers from 0420), **Unit Control 0**, PointArmor 0 / FullArmor 0, **Groupable true**.
- **5**: leash measured **edge to edge**, like all ranges.

## Implementation impact
Replaces the Director's placeholder for 3 (buy at once) and 5 (centre to centre). 1, 2, 4 match the placeholders.

## Verification
- Barracks with a Career Campus: R queues a Drone Operator.
- Launch a drone: the operator stays selected; Build Drone (F) is greyed until the drone ends; then it can be bought (10 s).
- Fly the drone just past 8 GU from the operator's edge: it falls and explodes.
- Drones don't change the Unit Control count; two drones selected together form one group.
