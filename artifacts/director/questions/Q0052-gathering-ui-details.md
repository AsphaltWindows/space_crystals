# Q0052: Gathering UI details: a Drone's right-click on enemies, the Set Drop-off pointer, who sees a Drone's load, Gather indicators

- **Date**: 2026-11-24T17:40:27-05:00
- **Blocking**: no. M3-J2 is built with the current behaviour listed under each point.
- **Docs**: `factions/colonists/units.md > Drone > Right-Click Resolution`, `> Set Drop-off Location`; `systems/controls/selection_and_hud.md > PointerDisplayType`, `> InfoPanel`; change 0062.

## Question
1. **A Drone right-clicking an enemy.** The Drone's section says an enemy or other object gives "Move to that object's location". 0062's general rule, for a unit that can't carry out a right-click, is to follow an enemy unit and move to an enemy structure.
   - **Current behaviour:** the general rule. The Drone follows an enemy unit, and moves to an enemy structure, stopping nearby.
   - The other reading, a one-off Move to the point clicked, would leave a Drone standing where a moving enemy was.
2. **The pointer while targeting Set Drop-off.** PointerDisplayType defines no pointer for AwaitingTarget[SetDropOffLocation].
   - **Current behaviour:** ReturnResources (the drop-off pointer) over a valid target.
3. **Who sees what a Drone carries.**
   - **Current behaviour:** the load marker on a carrying Drone shows to anyone who sees the Drone. The InfoPanel lines "Carrying 8 Space Crystals" and "Drop-off: Egress Node" show only to its owner, in line with 0155's owner-only plate count.
4. **The indicator for Gather and Drop Off orders.** The design has none.
   - **Current behaviour:** the green Move marker at the target.

## Context
All four came up while building the gathering UI (M3-J2).

## Options
- **Point 1:** A, as now (follow a unit, move to a structure); B, a Move to the point under the click.
- **Point 2:** A, as now (ReturnResources); B, a dedicated pointer; C, the default pointer.
- **Point 3:** A, as now; B, the marker owner-only too; C, both lines visible to everyone.
- **Point 4:** A, as now (the Move marker); B, a distinct Gather marker on the patch and Drop Off marker on the node.
