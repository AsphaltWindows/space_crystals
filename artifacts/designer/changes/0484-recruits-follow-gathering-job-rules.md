# 0484 — The gathering job rules apply to Recruits

- **Date**: 2026-11-28T14:20:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/cults/units/recruit.md > Gathering`
  - `systems/mechanics/gathering.md > Used by`

## What changed and why
The user confirmed the open point from 0482. The gathering **job rules** apply to Cults **Recruits** as they do to Drones and Agents:
- Last load of a patch;
- Interrupted mining collects nothing;
- Other orders keep the load;
- Who sees the load;
- Gather while carrying a load;
- Drop Off order;
- When a Gather ends.

Recruits keep their own parameters: shared patches, normal collision, nearest Storage.

## Implementation impact
Make Recruits follow the job rules wherever they don't already.

## Verification
1. Order a Recruit away during mining. It carries nothing.
2. A loaded Recruit given a Move keeps its 2 crystals and delivers them on its next trip.
3. Right-clicking a Storage with a loaded Recruit delivers the load, then the Recruit idles.
4. Opponents see that a Recruit is carrying, but not how much.
