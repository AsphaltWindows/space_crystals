# Q0065 — Expansions under construction

- **Date**: 2026-11-24T22:01:11-05:00
- **Blocking**: no — proceeding on assumption: the proposals below, which slice M4-F builds.
- **Docs**: `factions/syndicate/tunnels.md > ExpandMenu`, `> AwaitingPlacement (Expansion)`, `> Construction and Upgrade Rules`, `> Losing Tunnels`; `factions/syndicate/structures.md > Hideout`; `core/entities.md > ConstructionHP Rule`.

## Question
1. **When an expansion is paid for.** **Proposal:** at placement, the left-click that "places expansion, begins construction".
2. **ConstructionHP.** The rule "applies only to structures that explicitly reference it", and the Hideout doesn't reference it. **Proposal:** it applies to every expansion, since expansions are built on site like the Tunnel.
3. **Cancelling an expansion under construction**, which nothing defines. **Proposal:** the site shows only X Cancel Construction, with a full refund, like the Tunnel's cancel and the upgrade's. The Tunnel's X cancels its current operation, whether an upgrade or an expansion.
4. **Tunnel Space.** **Proposal:** the expansion occupies its cells from placement, not from completion.
5. **SightRange of expansions**, not given. **Proposal:** none, since they are underground.
6. **Groupable**, not given for the Hideout. **Proposal:** true, like the Barracks.
7. **A Tunnel destroyed while one of its expansions is under construction.** **Proposal:** the unfinished expansion is destroyed, with no refund ("In-progress work is destroyed").
8. **The Tunnel during an expansion's construction.** **Proposal:** its panel shows the progress, and Eject still works. Eject isn't an operation.

## Context
The Hideout has full stats and the menus are defined, but nothing says what an expansion is between the click and completion.

## Options
- **Point 2:** A, it applies, as proposed. B, full HP from the start, which is the literal reading.
- **Point 3:** A, a full refund, as proposed. B, 75% rounded down, as for the Cults' cancel and the GDO's ready building.
- **Point 7:**
  - A, destroyed, as proposed;
  - B, it becomes neutral unfinished and decays;
  - C, refunded.
