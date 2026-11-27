# Q0163 — How the Field Engineer builds the Bunker and the First Aid Tent, and what cancelling refunds

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GB, the Bunker and the First Aid Tent, builds the placeholders in Options.
- **Docs**: `factions/gdo/units/field_engineer.md > Abilities`; `factions/gdo/structures/{bunker,first_aid_tent}.md`; `factions/cults/economy.md > Cults Building Mechanics`, `> Cancel Construction`; `factions/cults/units/recruit.md`; `factions/gdo/structures/{deployment_center,defense_system}.md`; `core/entities.md > Object Type` (Groupable), `> ConstructionHP Rule`

## Question
1. How does a Field Engineer build? Must it keep working on the site, can other Engineers help, and is it used up?
2. What does cancelling a Bunker or a First Aid Tent under construction refund?

## Context
- "Build Bunker: constructs a Bunker (see factions/gdo/structures/bunker.md). Costs Space Crystals." "Set up First Aid Tent: sets up a First Aid Tent". Both buildings follow the Worker-Built Structures rules. Nothing else says how the Engineer builds.
- These are the GDO's first buildings built on site. Its others are built inside a producer and placed complete, or deployed (the Outpost).
- **Costs:** the Bunker "100 Space Crystals, 320 frames (20 seconds)"; the Tent "75 Space Crystals, 240 frames (15 seconds)".
- **Hints:**
  - repairers "repair faster, scaling linearly (2 repairers = twice the rate), as construction does";
  - the Groupable rule: "Caster-type units and workers that are used up by a build (Agent, Drone) are Ungroupable; Recruits, which share a build, are Groupable". The Field Engineer is "Groupable true".
- **The Cults' Recruits:** "Recruits construct Cults buildings from outside the site and can assist in-progress construction", and "Recruits are not consumed by construction". The time scales with the builders: "2 Recruits halve it". "If all Recruits are pulled off a site, progress stalls at its current value (no decay)".
  - Their sites follow the ConstructionHP Rule, which "applies only to structures that explicitly reference it". The Bunker and the Tent don't.
- **Cancelling:**
  - the Bunker's panel has "X: Cancel construction while being built". The Tent's doc gives no panel;
  - the Cults' Cancel Construction "refunds 75% of the building's Space Crystals cost (rounded down)";
  - GDO buildings cancelled while being built inside a producer are refunded in full: "Cancel during construction: full refund of Space Crystals cost" (the Deployment Center); "X: Cancel Construction (full refund)" (the Defense System).

## Options
1. **Building:**
   - **(a) As the Cults' Recruits do** (the placeholder): Engineers build from outside the site, and others can join, so 2 Engineers build a Bunker in 10 s and 4 in 5 s. None is used up. Progress stalls while no one works on the site. The site follows the ConstructionHP Rule.
   - **(b) One Engineer per site,** with no help: a Bunker always takes 20 s.
   - **(c) Used up into the building,** as the Colonists' Drone deploys: each Engineer makes one building and is gone, so it can't repair afterwards. By the Groupable rule it would then be Ungroupable.
2. **Cancelling:**
   - **(a) 75%, rounded down, as for the Cults' buildings** (the placeholder): 75 of the Bunker's 100, 56 of the Tent's 75. The Tent gets the same X: Cancel construction.
   - **(b) A full refund,** as for GDO buildings cancelled while being built inside a producer.
