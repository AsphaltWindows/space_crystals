# Q0164 — A shielded Peacekeeper in a Bunker, and whom the First Aid Tent heals

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GB, the Bunker and the First Aid Tent, builds the placeholders in Options.
- **Docs**: `factions/gdo/structures/bunker.md > Garrison`; `factions/gdo/structures/first_aid_tent.md > Healing`, `> Placeholder stats`; `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`; `factions/cults/structures/recruitment_hall.md`; `factions/colonists/economy.md`; `factions/cults/structures/README.md` (Entering pool buildings)

## Question
1. A Peacekeeper enters a Bunker with its Riot Shield raised. Does it stay shielded, and so not fire, or does entering lower the shield?
2. Does the First Aid Tent heal infantry inside a Bunker within its 3 grid units?
3. Does a First Aid Tent heal while it is still under construction?

## Context
- **The Bunker:** "Who can enter: all GDO infantry, up to 4 units, using the Enter command." "Units inside fire their own weapons from the Bunker." "Protection: units inside cannot be targeted and take no damage while the Bunker stands."
  - "No abilities from inside: units inside the Bunker cannot use their abilities (e.g. ... a Peacekeeper cannot raise its Riot Shield)."
  - "Evacuation on destruction: when the Bunker is destroyed, everyone inside comes out alive around its footprint."
- **The shield:** a shielded Peacekeeper can't attack: "Attack orders are not possible: the player must lower the shield before ordering an attack." Raising or lowering takes "6 frames (0.375 s), placeholder".
- **The Tent:** "Heals nearby infantry (no need to enter)": "own and allied infantry ... whose centre is within 3 grid units: +2 HP per second". Nothing says whether units inside a building count.
  - The Shrine's casts, as built in M5, leave out units inside a building. The docs don't say that either.
- **Buildings under construction:**
  - the Cults' Recruitment Hall: "A Hall under construction claims nothing, provides no capacity and produces nothing until it is complete.";
  - the Colonists: "None of the building's functions (production, accepting deliveries, research, abilities) work until the transformation completes.";
  - a Cults building "under construction can't be entered".

## Options
1. **(a) It stays shielded and doesn't fire** (the placeholder: nothing changes its mode for it), and it can't lower the shield inside. The shield does nothing there, but it comes out shielded if the Bunker falls. **(b) Entering lowers the shield** (the 6-frame toggle), so it fires from inside.
2. **(a) No** (the placeholder): only units on the map are healed, so wounded infantry must step out to heal. **(b) Yes:** a Bunker beside a Tent heals its occupants, e.g. a Peacekeeper from 25 to its 50 HP in about 13 s.
3. **(a) No: it heals only once complete** (the placeholder), as other buildings work only once complete. **(b) Yes,** from the start of construction.
