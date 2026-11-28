# Q0194 — An empty Recruit's right-click on a damaged or unfinished own Storage

- **Date**: 2026-11-28T01:58:42-05:00
- **Blocking**: no. Proceeding on option (a), as built: the Recruit's right-click table decides, so the right-click repairs a damaged Storage and assists an unfinished one. Until the answer, the phase for 0484 tests only a completed, undamaged Storage.
- **Docs**:
  - `systems/mechanics/gathering.md > Job rules > Drop Off order`
  - `factions/cults/units/recruit.md > Right-Click Resolution:`
  - change 0484

## Question
An empty Recruit is right-clicked onto an own Storage that is damaged (completed) or still under construction. Does it:
- **repair or assist**, as the Recruit's Right-Click Resolution lists; or
- make a **plain Move** to the Storage, as the gathering doc's rule for empty workers says?

## Context
0484 applies the gathering job rules to the Recruits. They disagree only on this case.
- **The gathering doc's Drop Off order** (moved from the Drone's doc) says:
  - Drop Off is resolved only for a worker that carries something;
  - an empty worker's right-click on its drop-off structure is "a plain **Move**".
- **The Recruit's table** resolves an own Storage to Drop Off only while the Recruit carries crystals. It also lists:
  - "In-progress own building → Assist Construction";
  - "Damaged own building or vehicle (completed) → Repair".
- **Drones can't repair or assist**, so the question never arose while the rule was the Drone's.
- **The code follows the Recruit's table.** An empty Recruit's right-click repairs a damaged Storage and assists an unfinished one (`crates/sim/src/orders/resolution.rs`). Where both docs apply, they agree: an empty Recruit's right-click on a completed, undamaged Storage is a plain Move, and a carrying Recruit's right-click on any own Storage delivers.

## Options
- **(a) The unit's own table decides once Drop Off doesn't apply.**
  - The result: Repair on a damaged Storage, Assist on an unfinished one, a Move otherwise.
  - No code change. The gathering doc would say that the right-click goes to the worker's other rules, not "a plain Move".
- **(b) Always a plain Move for an empty worker on its drop-off structure, whatever the structure's state.**
  - A small change to the Recruit's right-click resolution.
  - The player repairs a damaged Storage with D: Repair, and assists an unfinished one with S: Assist Construction.
