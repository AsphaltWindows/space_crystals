# 0127 — The Drop Off order: a delivery only; passes through gathering Drones

- **Date**: 2026-11-24T15:50:00-05:00
- **Answers**: Q0045 (point 2; Q0045 done); Q0037 (point 1)
- **Docs**: `factions/colonists/units.md > Drop Off order` (new); `> Gathering Drones pass through each other` (Drop Off bullet)

## What changed and why
- **Semantics (user):** an explicit Drop Off is only a delivery, not a Gather. After delivering, the order is complete: the next queued order starts (e.g. Shift-queued build), otherwise the Drone idles at the drop-off point. It does **not** return to its patch (Q0037 pt 1: B, not the proposed "back to the patch"). Reason: if it counted as a Gather, Shift-queued orders behind it would wait until the field is mined out (0123).
- **Collision (Q0045 pt 2: A):** a Drone on a Drop Off order passes through gathering Drones on its way, like a gathering Drone's return trip.

## Implementation impact
Replaces the Director's Q0037 pt 1 proposal (return to the patch). Collision group for pass-through = Drones on Gather or Drop Off orders.

## Verification
- Right-click the node with a carrying Drone: it passes through the mining crowd, delivers, then sits idle at the node.
- Right-click the node, then Shift-place a Beacon: after delivering it goes to deploy.
