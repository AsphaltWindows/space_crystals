# 0482 — Mechanics doc: Gathering

- **Date**: 2026-11-28T13:55:00-05:00
- **Answers**: Q0193 (migration, part 10)
- **Docs**:
  - `systems/mechanics/gathering.md` (new)
  - `README.md > systems/mechanics/`
  - `factions/colonists/units/drone.md > Gathering` (its job rules moved out; a pointer is left)
  - `factions/syndicate/units/agent.md > Gathering rules`
  - `factions/cults/units/recruit.md > Gathering`

## What changed and why
The tenth mechanics doc.
- **Worker round trip:** a pick-up point and a drop-off chosen by a rule.
  - Parameters: carry capacity, durations, reaches, drop-off choice, patch sharing, collision while gathering, drop-off occupancy.
  - The **job rules** that were defined in the Drone's doc and applied to Agents from there:
    - Last load of a patch;
    - Interrupted mining;
    - Other orders keep the load;
    - Who sees the load;
    - Gather while carrying a load;
    - Drop Off order;
    - When a Gather ends.
- **Stationary extractor:** GDO plates.

**No rule changed.** The job rules moved word for word.

**Open point found by the audit** (not changed): the Recruit's doc doesn't say whether these job rules apply to Recruits. For example, does a Recruit ordered away mid-mining collect nothing, and does it keep its load on other orders? The Director should report what is built; the designer will raise it with the user.

## Implementation impact
One gathering module. No behaviour changes.

## Verification
No gameplay change.
