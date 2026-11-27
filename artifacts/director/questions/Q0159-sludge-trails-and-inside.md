# Q0159 — The Shadebringer's trails: standing still, when a unit is in one, and the Tunnel Network

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — SB (the Vault, the Shadebringer and Sludge) and SC (its Cover, smoke) build the placeholders in Options.
- **Docs**: `factions/syndicate/units/shadebringer.md` (`> Sludge (toggle)`, `> Cover (toggle)`, `> Charges and recharging`, `> Placeholder stats`); `systems/units.md > LightInfantry` ("Which tile counts"); `systems/combat.md > AoEDamage`; `factions/cults/structures/shrine.md`; `factions/gdo/structures/first_aid_tent.md`; `factions/cults/units/prayer_mobile.md`; `factions/syndicate/tunnels.md > Tunnel Network`; `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)`

## Question
1. Does a Shadebringer standing still with a toggle on lay sludge or smoke where it stands?
2. When is a unit in sludge, or inside smoke: when its centre is, or when any part of it is?
3. Does entering the Tunnel Network switch its toggles off?

## Context
- **Laying:** "while one is on, the Shadebringer leaves its effect behind it as it drives". Each tank "drains while its toggle is on": "Tanks: 100 each; drain 10 per second while on (10 s of use from full)".
  - Sludge lasts 10 s, smoke 12 s.
- **Being in it:**
  - "Infantry and non-hover vehicles that step into sludge are slowed": "units are slowed by Y = 40% for Z = 2 s after leaving it, and the slow refreshes while they stay in it". The trail is 48 su wide.
  - "Units inside the smoke can't see anything". The wall is 64 su wide.
  - Several area rules count a unit's centre: Rugged terrain ("a unit is "on Rugged terrain" when the tile under its centre is Rugged"), the Shrine ("a unit is affected when its centre is inside"), the First Aid Tent ("whose centre is within 3 grid units") and the Prayer-mobile's Suppress ("units whose centre is inside can't attack"). Area damage goes by the overlap instead.
- **The network:** on entering, "the unit's orders are cleared, queued ones included; it is idle inside". "Toggles follow the mode-switch rule (keep orders, not queueable)", and for the Peacekeeper "raising or lowering the shield is a mode switch, not an order".
  - The tanks refill only "inside the network with a Vault", at 5 a second. If they drained inside at 10 a second, they would empty.
  - Sludge affects everyone: "The Shadebringer has to be driven so it doesn't trap its own army."

## Options
1. **(a) No: it lays only while moving, though the tank still drains** (the placeholder), as "as it drives" reads. **(b) It lays a patch where it stands:** a parked Shadebringer holds one spot of smoke for about 22 s (10 s of tank, then 12 s for the last smoke).
2. **(a) Its centre** (the placeholder), as those rules: a 36x36 unit can overlap the trail's edge by up to 18 su without being slowed, or stand half in smoke and still see. **(b) Any part of it.**
3. **(a) Yes, both switch off** (the placeholder): it comes out with both off, and the player switches them on again. **(b) They stay on, paused inside without draining, and lay again as it comes out:** onto the exit path its own units use, so units following it out walk through its sludge.
