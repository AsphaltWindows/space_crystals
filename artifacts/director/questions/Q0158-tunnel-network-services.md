# Q0158 — Inside the network: two services at once, and which unit the EjectMenu picks

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — SN (the Null Chamber), SB (the Vault and the Shadebringer) and SA (the Specter) build the placeholders in Options.
- **Docs**: `factions/syndicate/structures/{vault,null_chamber}.md > Placeholder stats`; `factions/syndicate/tunnel_menus.md > EjectMenu`; `factions/syndicate/tunnels.md > Tunnel Network` (Transit delay)

## Question
1. Can one unit be re-armed or recharged at a Vault and take its Null Chamber turn at the same time?
2. Should the EjectMenu prefer units whose services are done?

## Context
- **The services:**
  - "each Vault re-arms or recharges 2 units at a time; others wait their turn (network-wide, first come first served). Specter re-arming: 160 frames (10 s). Shadebringer tanks refill at 5 per second each." Empty tanks (100) take 20 s.
  - "each Null Chamber buffs 4 units at a time, each buff taking 2 s inside before the unit can leave buffed; a unit that leaves before its turn leaves unbuffed."
  - A Specter or a Shadebringer can need both.
- **Ejecting:** of the chosen type, the EjectMenu takes "the one that has been inside the network longest. (Not "the healthiest": other per-unit factors such as re-arming at a Vault or the Null Chamber's re-buff may matter too.)"
  - Services go by arrival, so the unit inside longest has usually been served. Not always: a Shadebringer that came in empty refills for 20 s, and a Specter re-arms for 10 s, while one of the same type that came in later may need nothing.
  - The Null Chamber's 2 s turn matches the "2-second transit delay" before a unit can be ejected at all, for the first 4 units at each chamber.

## Options
1. **(a) Yes, they are separate queues** (the placeholder): a Specter is re-armed and buffed in 10 s, a Shadebringer with empty tanks in 20 s. **(b) One at a time:** 12 s and 22 s.
2. **(a) No: the one inside longest, as now** (the placeholder). **(b) Units whose services are done come first,** then the one inside longest: an eject never takes a half-refilled Shadebringer while a ready one waits.
