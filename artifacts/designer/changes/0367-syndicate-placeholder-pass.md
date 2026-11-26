# 0367 — Syndicate placeholder pass: every designed Syndicate unit and expansion gets numbers

- **Date**: 2026-11-27T02:55:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/syndicate/structures/{vault,vehicle_assembly,forge,null_chamber,aerospace_assembly,orbital_command}.md`;
  - `factions/syndicate/units/{assault_bike,phantom,specter,laser_battery,shadebringer,subterranean_transport,prism}.md`.

## What changed and why
Under the user's standing permission, the remaining Syndicate objects get placeholders, marked as such. Expansions follow n = 6. Research and the Chemical Plant and Optics Lab commands wait for the techs pass.

**Expansions:**

| Expansion | Size | HP | Cost (SC) / time (f) | Other |
|---|---|---|---|---|
| Vault | 2x2 | 400 | 150 / 400 | |
| Vehicle Assembly | 3x3 | 600 | 250 / 480 | panel Q Bike, W Shadebringer, E Laser Battery, R Transport |
| Forge | 2x2 | 400 | 200 / 480 | |
| Null Chamber | 2x2 | 400 | 200 / 480 | buff 20 s, +2 sight, −20% damage |
| Aerospace Assembly | 3x3 | 600 | 300 / 640 | panel Q Phantom, W Specter, E Prism |
| Orbital Command | 2x2 | 400 | 250 / 640 | |

**Units** (cost SC / time f / TunnelSpaceCost):

| Unit | Cost / time / TS | Key numbers |
|---|---|---|
| Assault Bike | 125 / 240 / 2 | 12 su/f, turn radius 128; mortar 120 AoE over 48 su, range 5 |
| Shadebringer | 150 / 320 / 2 | sludge 10 s, −40% for 2 s; smoke 12 s, 1 GU wide; tanks 100, drain 10/s, refill 5/s; R and F |
| Laser Battery | 200 / 400 / 3 | battery 100 at +5/s; surge each 1 s using 10; damage 4 × charge^(2/3) (about 18.6; Amplify from full about 86); range 8; deploy 48 f; relay ranges 8/8, armed Prism 6 |
| Subterranean Transport | 200 / 400 / 3 | burrow and surface 48 f; capacity 8 Tunnel Space |
| Phantom | 175 / 320 / 2 | HP 70, 9 su/f, cone 75°, laser 12 dps, range 5 |
| Specter | 200 / 400 / 3 | lift-off and landing 24 f; bomb 400 AoE over 80 su, 24 f fall; re-arm 160 f |
| Prism | 250 / 480 / 2 | 1 su/f; launch and transform 32 f each; 3 thrusts of 3 GU |

**Designer's readings that are really rules**, flagged in the docs "to confirm" and being asked:
- **Throughput:** the Vault serves 2 units at a time; the Null Chamber buffs 4 at a time, 2 s each, and a unit leaving before its turn goes unbuffed.
- **Null Chamber buff:** it ends abruptly, and needs no research.
- **Laser Battery:** Amplify is usable mobile and deployed, and is instant and not queueable.
- **Shadebringer:** sludge slow refreshes while a unit stays in it; a toggle needs at least 10 to switch on; its toggles follow the mode-switch rule.
- **Transport:** capacity counts TunnelSpaceCost; loading is by right-click above ground; F unloads all.
- **Specter:** can be hit while lifting off or landing.

## Implementation impact
All of these can be built on placeholders.

## Verification
—
