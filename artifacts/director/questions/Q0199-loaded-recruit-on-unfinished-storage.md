# Q0199 — A loaded Recruit's right-click on an unfinished own Storage: deliver or assist?

- **Date**: 2026-11-28T04:51:32-05:00
- **Blocking**: no. Proceeding as built: an unfinished Storage takes no delivery, so a loaded Recruit's right-click on it assists the construction, as an empty Recruit's does.
- **Docs**:
  - `factions/cults/units/recruit.md > Right-Click Resolution:`;
  - `systems/mechanics/gathering.md > Job rules > Drop Off order`;
  - changes 0487 (Verification 4), 0116 and 0234.

## Question
Can an unfinished Storage take a delivery? That decides whether a loaded Recruit's right-click on an unfinished own Storage delivers its load or assists the construction.

## Context
- **0487** says: "A carrying worker still delivers." Its Verification lists a damaged, an unfinished and a complete Storage, then: "Do the same with a loaded Recruit. It delivers its load."
- **The Recruit's right-click table** lists "Own Storage (carrying Space Crystals) → Drop off resources" above "In-progress own building → Assist Construction".
- **The build takes deliveries only at finished drop-offs,** for every faction.
  - The design says so for two of them: "A Drone can't deliver to an Egress Node under Warp In" (0116), and a Tunnel under construction has "no drop-offs" (0234).
  - It says nothing about the Storage.
- **So the build's loaded Recruit assists an unfinished Storage,** as an empty one does. Phase A3's test asserts this, marked `Placeholder (Q0199)`.

## Options
- **(a) As built:** no unfinished drop-off takes a delivery, whatever its faction.
  - A loaded Recruit's right-click on an unfinished Storage assists it, and the Recruit keeps its load.
  - 0487's Verification 4 would then cover only the damaged and the complete Storage.
  - No change.
- **(b) An unfinished Storage takes deliveries,** unlike the Egress Node and the Tunnel.
  - A loaded Recruit's right-click on one delivers.
  - The drop-off gate and the right-click change for the Storage only. A small change.
