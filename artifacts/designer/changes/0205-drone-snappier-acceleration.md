# 0205 — Drone: thrust and drag ×1.5 (same top speed, faster to reach it); please re-measure the Colonists economy

- **Date**: 2026-11-25T03:50:00-05:00
- **Answers**: — (step toward Q0055; Q0055 stays open)
- **Docs**: `factions/colonists/units.md > Drone > UnitBaseAttributes[HoverVehicle]`

## What changed and why
User: the Drone should reach its top speed faster. The values are now ForwardAcceleration **0.9**, OmniDirectionalAcceleration **0.3** and DragRatio **0.15** (were 0.6 / 0.2 / 0.1). Top speed stays 8 su/frame. The Drone gets to about 65% of it after ~7 frames instead of 10, and close to full speed after ~20 frames instead of ~30. Braking and turning are snappier too. Rugged terrain still doubles drag.

## Implementation impact
- Update the Drone's movement constants.
- **Request:** re-run the Colonists economy oracle (M3-K1 line and saturation curve) with the new values, and report the measured table in the same format as Q0055 (income at each time, deltas against the table, working Drones), so the user can decide Q0055 on the new figures.
- The Q0055 decision (table vs numbers vs benchmark line) is still pending. Don't change the table or targets yet.

## Verification
- A Drone starting from rest visibly reaches full speed in about 1.25 s.
- It stops with less drift than before.
