# 0282 — Design docs reorganized: one file per unit and per structure

- **Date**: 2026-11-25T18:00:00-05:00
- **Answers**: —
- **Docs**:
  - every `factions/<faction>/units.md` and `factions/<faction>/structures.md` is **removed** and split into `factions/<faction>/units/*.md` and `factions/<faction>/structures/*.md`;
  - `README.md` (index, reference convention);
  - cross-references updated in `core/entities.md`, `core/tech_and_research.md`, `core/vision.md`, `factions/colonists/economy.md`, `factions/syndicate/tunnels.md`, `systems/combat.md`, `systems/units.md` and `systems/controls/selection_and_hud.md`.

## What changed and why
User asked to split the design docs by object, so each unit or building can be read and changed on its own.
- **No design content changed.** Every line was moved verbatim; only cross-references were rewritten. This was checked line by line against the old files.
- **Files:**
  - each unit is now `factions/<faction>/units/<unit>.md`;
  - each structure is now `factions/<faction>/structures/<structure>.md`;
  - each file's title is the object's name, and its sections are one heading level higher than before (`## Gathering` instead of `### Gathering`); section names are unchanged.
- **Shared sections** go to each folder's `README.md`, which also holds the folder index. They are:
  - Syndicate: the red-lasers rule, the Tech Tree Overview, the Tunnel Expansions intro with Expansion Requirements and Rally Point Behavior, and the Surface Defensive Structures intro;
  - GDO: GDOBuildArea;
  - Cults: the Tech Tree Overview;
  - Colonists: the Tech Model and Portals.
- **Colonists Weapon Conversion** has its own file, `units/weapon_conversion.md`.
- **Reference convention** (`README.md`): a path-less reference starting with an object name means that object's file, e.g. `Recruit > Gathering` = `units/recruit.md > Gathering`.
- The Syndicate structure groupings ("Tunnel Expansions > X", "Surface Defensive Structures > Laser Turret") are dropped from references to the objects themselves.

## Implementation impact
- **Code comments and docs that cite design paths are now stale.** I count about 320 citations across about 150 files. Please update them using the table below. The section part after the path carries over unchanged, except that the entity name (and, for Syndicate structures, the grouping) is now the file.
  - Example: `factions/syndicate/structures.md > Tunnel Expansions > Null Chamber > Null Chamber Exit Buff` becomes `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`.
- No behavior changes.

**From the user, for the code side:** keep the code **organized and modular** as well. Once testing starts we want to experiment and make changes rapidly and accurately, so it's worth investing time now to keep things in order. Examples: per-unit and per-structure definitions living in their own modules, data and tunables kept apart from logic, and shared mechanics (deploy model, toggles, AoE) implemented once and reused.

## Old → new paths
| Old reference | New file |
|---|---|
| `factions/colonists/structures.md` (shared sections / no section) | `factions/colonists/structures/README.md` |
| `factions/colonists/structures.md > Bio Portal` | `factions/colonists/structures/bio_portal.md` |
| `factions/colonists/structures.md > Exophysics Engine` | `factions/colonists/structures/exophysics_engine.md` |
| `factions/colonists/structures.md > Fusion Core` | `factions/colonists/structures/fusion_core.md` |
| `factions/colonists/structures.md > Heavy Portal` | `factions/colonists/structures/heavy_portal.md` |
| `factions/colonists/structures.md > Navigation Spire` | `factions/colonists/structures/navigation_spire.md` |
| `factions/colonists/structures.md > Plasma Turret` | `factions/colonists/structures/plasma_turret.md` |
| `factions/colonists/structures.md > Psionic Amplifier` | `factions/colonists/structures/psionic_amplifier.md` |
| `factions/colonists/structures.md > Relay Array` | `factions/colonists/structures/relay_array.md` |
| `factions/colonists/structures.md > Robo Portal` | `factions/colonists/structures/robo_portal.md` |
| `factions/colonists/structures.md > Skylance` | `factions/colonists/structures/skylance.md` |
| `factions/colonists/structures.md > Star Portal` | `factions/colonists/structures/star_portal.md` |
| `factions/colonists/structures.md > Structural Analyzer` | `factions/colonists/structures/structural_analyzer.md` |
| `factions/colonists/units.md` (shared sections / no section) | `factions/colonists/units/README.md` |
| `factions/colonists/units.md > Drone` | `factions/colonists/units/drone.md` |
| `factions/colonists/units.md > Lancer` | `factions/colonists/units/lancer.md` |
| `factions/colonists/units.md > Rifle Legionnaire` | `factions/colonists/units/rifle_legionnaire.md` |
| `factions/colonists/units.md > Sabre Legionnaire` | `factions/colonists/units/sabre_legionnaire.md` |
| `factions/colonists/units.md > Scout` | `factions/colonists/units/scout.md` |
| `factions/colonists/units.md > Weapon Conversion` | `factions/colonists/units/weapon_conversion.md` |
| `factions/cults/structures.md` (shared sections / no section) | `factions/cults/structures/README.md` |
| `factions/cults/structures.md > Airstrip` | `factions/cults/structures/airstrip.md` |
| `factions/cults/structures.md > Armory` | `factions/cults/structures/armory.md` |
| `factions/cults/structures.md > Cathedral` | `factions/cults/structures/cathedral.md` |
| `factions/cults/structures.md > Chop Shop` | `factions/cults/structures/chop_shop.md` |
| `factions/cults/structures.md > Garrison` | `factions/cults/structures/garrison.md` |
| `factions/cults/structures.md > Gunworks` | `factions/cults/structures/gunworks.md` |
| `factions/cults/structures.md > Machine Shop` | `factions/cults/structures/machine_shop.md` |
| `factions/cults/structures.md > Scrap Hangar` | `factions/cults/structures/scrap_hangar.md` |
| `factions/cults/structures.md > Shrine` | `factions/cults/structures/shrine.md` |
| `factions/cults/structures.md > Temple` | `factions/cults/structures/temple.md` |
| `factions/cults/units.md` (shared sections / no section) | `factions/cults/units/README.md` |
| `factions/cults/units.md > Acolyte` | `factions/cults/units/acolyte.md` |
| `factions/cults/units.md > Gunner` | `factions/cults/units/gunner.md` |
| `factions/cults/units.md > Recruit` | `factions/cults/units/recruit.md` |
| `factions/cults/units.md > Soldier` | `factions/cults/units/soldier.md` |
| `factions/gdo/structures.md` (shared sections / no section) | `factions/gdo/structures/README.md` |
| `factions/gdo/structures.md > Advanced Mechanics` | `factions/gdo/structures/advanced_mechanics.md` |
| `factions/gdo/structures.md > Airfield` | `factions/gdo/structures/airfield.md` |
| `factions/gdo/structures.md > Arsenal` | `factions/gdo/structures/arsenal.md` |
| `factions/gdo/structures.md > Barracks` | `factions/gdo/structures/barracks.md` |
| `factions/gdo/structures.md > Bunker` | `factions/gdo/structures/bunker.md` |
| `factions/gdo/structures.md > Career Campus` | `factions/gdo/structures/career_campus.md` |
| `factions/gdo/structures.md > Defense System` | `factions/gdo/structures/defense_system.md` |
| `factions/gdo/structures.md > DeploymentCenter` | `factions/gdo/structures/deployment_center.md` |
| `factions/gdo/structures.md > Factory` | `factions/gdo/structures/factory.md` |
| `factions/gdo/structures.md > First Aid Tent` | `factions/gdo/structures/first_aid_tent.md` |
| `factions/gdo/structures.md > Industrial Powerplant` | `factions/gdo/structures/industrial_powerplant.md` |
| `factions/gdo/structures.md > Missile Turret` | `factions/gdo/structures/missile_turret.md` |
| `factions/gdo/structures.md > Outpost` | `factions/gdo/structures/outpost.md` |
| `factions/gdo/structures.md > Propulsion Labs` | `factions/gdo/structures/propulsion_labs.md` |
| `factions/gdo/structures.md > Radar Station` | `factions/gdo/structures/radar_station.md` |
| `factions/gdo/units.md` (shared sections / no section) | `factions/gdo/units/README.md` |
| `factions/gdo/units.md > Artillery Rig` | `factions/gdo/units/artillery_rig.md` |
| `factions/gdo/units.md > Dragonfly` | `factions/gdo/units/dragonfly.md` |
| `factions/gdo/units.md > Drone Operator` | `factions/gdo/units/drone_operator.md` |
| `factions/gdo/units.md > Field Engineer` | `factions/gdo/units/field_engineer.md` |
| `factions/gdo/units.md > Hornet` | `factions/gdo/units/hornet.md` |
| `factions/gdo/units.md > Peacekeeper` | `factions/gdo/units/peacekeeper.md` |
| `factions/gdo/units.md > Prospector` | `factions/gdo/units/prospector.md` |
| `factions/gdo/units.md > Raptor` | `factions/gdo/units/raptor.md` |
| `factions/gdo/units.md > Rocketman` | `factions/gdo/units/rocketman.md` |
| `factions/gdo/units.md > SupplyChopper` | `factions/gdo/units/supply_chopper.md` |
| `factions/gdo/units.md > Twister` | `factions/gdo/units/twister.md` |
| `factions/syndicate/structures.md` (shared sections / no section) | `factions/syndicate/structures/README.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Aerospace Assembly` | `factions/syndicate/structures/aerospace_assembly.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Chemical Plant` | `factions/syndicate/structures/chemical_plant.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Forge` | `factions/syndicate/structures/forge.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Hideout` | `factions/syndicate/structures/hideout.md` |
| `factions/syndicate/structures.md > Surface Defensive Structures > Laser Turret` | `factions/syndicate/structures/laser_turret.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Null Chamber` | `factions/syndicate/structures/null_chamber.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Optics Lab` | `factions/syndicate/structures/optics_lab.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Orbital Command` | `factions/syndicate/structures/orbital_command.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Vault` | `factions/syndicate/structures/vault.md` |
| `factions/syndicate/structures.md > Tunnel Expansions > Vehicle Assembly` | `factions/syndicate/structures/vehicle_assembly.md` |
| `factions/syndicate/units.md` (shared sections / no section) | `factions/syndicate/units/README.md` |
| `factions/syndicate/units.md > Agent` | `factions/syndicate/units/agent.md` |
| `factions/syndicate/units.md > Assault Bike` | `factions/syndicate/units/assault_bike.md` |
| `factions/syndicate/units.md > Grenadier` | `factions/syndicate/units/grenadier.md` |
| `factions/syndicate/units.md > Guard` | `factions/syndicate/units/guard.md` |
| `factions/syndicate/units.md > Laser Battery` | `factions/syndicate/units/laser_battery.md` |
| `factions/syndicate/units.md > Phantom` | `factions/syndicate/units/phantom.md` |
| `factions/syndicate/units.md > Prism` | `factions/syndicate/units/prism.md` |
| `factions/syndicate/units.md > Shadebringer` | `factions/syndicate/units/shadebringer.md` |
| `factions/syndicate/units.md > Specter` | `factions/syndicate/units/specter.md` |
| `factions/syndicate/units.md > Subterranean Transport` | `factions/syndicate/units/subterranean_transport.md` |
## Verification
- Every old `units.md`/`structures.md` reference resolves through the table.
- `README.md` lists every file.
