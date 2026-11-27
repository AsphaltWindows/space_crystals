# Q0161 — The Outpost's HP when a damaged Prospector deploys

- **Date**: 2026-11-26T19:12:15-05:00
- **Blocking**: no — GP, the Prospector and the Outpost, builds the placeholder in Options.
- **Docs**: `factions/gdo/units/prospector.md > Deploy into Outpost (ability)`, `> Placeholder stats`; `factions/gdo/structures/outpost.md > Placeholder stats`; `factions/gdo/units/artillery_rig.md` (the deploy model); `factions/colonists/economy.md > Beacon Deployment > HP/Shields Interpolation`, `> Cancel Deployment`; `factions/syndicate/tunnels.md > Upgrade Costs`; changes 0330, 0331

## Question
A damaged Prospector deploys into an Outpost. What HP does the Outpost start with?

## Context
- The Prospector has MaxHP 300. The Outpost is "Size 3x3 (the Prospector deploys in place)", with MaxHP 600.
- "Deploy 64 frames (4 s)", by the vehicle deploy model, in which "the deploying process is what actually changes the unit". So it is hit as a Prospector until the Outpost appears.
- **Precedents that keep the damage taken:**
  - a Colonists' Drone deploying into a Beacon: "damage taken before or during deployment is carried through unchanged";
  - a Tunnel's upgrade: "max HP rises (600 → 800 → 1000) and current HP rises by the same amount, so damage taken stays";
  - 0331: "When MaxHP goes up, HP goes up by the same amount, so the damage stays the same."
- **Precedents that don't:**
  - a cancelled Beacon: "the Drone returns with its own maxima and the same fraction of HP and Shields the Beacon had at the moment of cancelling";
  - 0330: "vehicles and aircraft come out at full HP, regardless of the Recruits' damage".

## Options
- **(a) The damage carries over unchanged** (the placeholder): a Prospector at 200/300 becomes an Outpost at 500/600.
- **(b) The same fraction:** 400/600.
- **(c) Full HP:** 600/600, so deploying also repairs it.
