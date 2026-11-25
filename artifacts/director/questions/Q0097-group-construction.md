# Q0097 — Group construction: where Recruits build, when the site starts, and who counts

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: the proposal under each point.
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics`, `> Build Command`, `> Assist Construction Command`; `core/entities.md > Worker-Built Structures`, `> Lingering build ghosts`

## Question
Several rules of building in a group are silent:
1. **Where do builders stand?** "from outside the site", with no reach given.
2. **When does the site start:** at the first Recruit's arrival, or when all have arrived?
3. **Recruits of the order on the footprint.** "the building worker itself ... doesn't count" as a blocker. What happens to Recruits of the order who stand on the footprint when it starts?
4. **A failed check.** If the first arrival finds the site blocked, does the whole order fail, or does each later Recruit try again?
5. **Who counts as "assigned"?** Only the Recruits at the site, or also those still walking to it?
6. **Under attack.** Do builders keep building, or fight back?
7. **Assist targets.** Can Assist target a placed ghost that hasn't started yet, or only a building under construction?
8. **Limits.** Is there a limit to the builders, beyond room around the site?

## Context
The doc: "All selected Recruits walk to the location and begin construction from outside the site"; "Construction speed scales proportionally with the number of Recruits currently assigned"; Assist: "Once the selected Recruits reach the building they join construction"; "no cap on this scaling other than the practical limit of how many Recruits can be assigned to a site". Mining and drop-off use an 8 su reach of the object's edge. An Agent goes inside its site; a Drone becomes the Beacon. Recruits do neither. The nearest precedent for point 3 is the pop-up turret's rise: "The **owner's own idle units** over it are pushed off by the idle push ... Any other unit over it (an enemy, or an own unit with an order) keeps it **underground** until the spot is clear" (`factions/syndicate/structures/laser_turret.md > Pop-up rules`, 0296).

## Options
1. (a) Each builder at its own spot within 8 su of the footprint's edge, the spots searched in a fixed order around it. (b) Anywhere within 8 su; the group may crowd one side.
2. (a) At the first arrival, as the Agent's site does. (b) When all have arrived: slower, and one straggler holds up the group.
3. (a) They are pushed off, as the owner's idle units are pushed off a rising pop-up turret, although they have an order, and then walk to their spots. (b) They block, as any unit does, so the order fails.
4. (a) The whole order fails, and is refunded once. (b) Each arrival tries again until one succeeds.
5. (a) Only those at the site, as "Once the selected Recruits reach the building" suggests. (b) All holders of the order.
6. (a) Keep building, as gatherers keep gathering. (b) Fight back, which stops the building.
7. (a) Only a building under construction. (b) A pending ghost too: the Recruit joins the order.
8. (a) Only those within reach count; others wait further out and don't count. (b) A fixed cap.

Proposed: (a) for each.
