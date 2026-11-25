# Space Crystals — Design Documents

Index of the design docs. Paths are relative to this folder.

**References:** cross-file references are written as `path/to/file.md > Section > Subsection`. A reference without a file path (e.g. "see Recruit > Gathering") points into the same faction's folder: it starts with a unit or structure name, meaning that object's file (`units/recruit.md > Gathering`), or with a section of a faction file or folder README (or the same `systems/` / `core/` area).

## core/ — game-wide foundations
| File | Contents |
|---|---|
| `core/scale.md` | Units of measure: SimulationFrame (16/s), GridUnit, SpaceUnit (64 per grid unit) |
| `core/lore.md` | Space Crystals lore, faction lore |
| `core/camera.md` | Zoom, panning, starting position, HUD layout |
| `core/art_direction.md` | 2.5D pixel art: square grid + 3/4 view, sprites pre-rendered from 3D models, 32 px per tile, building orientations, unit facings (16/32), elevation display |
| `core/game_rules.md` | Defeat Condition (shared across factions) |
| `core/entities.md` | Entity hierarchy: Faction, Player, Object Type, Structure Type, Placement Validation (incl. Worker-Built Structures), instances |
| `core/map.md` | Tiles, tile presets (Buildable/Traversible/Rugged/Drillable/Recruitable), Elevation (corner heights, slopes, flat-only building), Space Crystals patches, Supply Delivery Stations |
| `core/map_editor.md` | Map editor: map creation settings, map types (Skirmish Default / Custom), per-object resource values, per-faction Starting Layouts (overlay, Skirmish Default restriction, outdated flags), Symmetry (chosen at creation, generated), custom tile presets |
| `core/vision.md` | Vision, visibility states, elevation |
| `core/tech_and_research.md` | Shared Tech Prerequisite rules and Research rules (with faction exceptions) |

## systems/ — shared gameplay systems
| File | Contents |
|---|---|
| `systems/units.md` | Unit attributes, collision (incl. crushing), movement models, unit bases (infantry, vehicles, air), turrets |
| `systems/combat.md` | Attack phases, targeting, attack types, damage calculation (armor, directional, AoE, minimum), shields |
| `systems/controls/selection_and_hud.md` | Selection, control groups, interface states, HUD bar, selection/command panels, right-click resolution, cursor, command indicators |
| `systems/controls/commands.md` | Unit commands (+ group destinations / magic box): Move, Attack, Attack Ground, Attack Move, Patrol, Hold, Stop, Reverse, Enter |
| `systems/controls/behaviors.md` | Unit/turret behaviors, auto-targeting, action channels (locomotion, orientation, attack) |

## factions/ — one folder per faction
| File | Contents |
|---|---|
| `factions/gdo/overview.md` | GDO identity and resources (Space Crystals, Supplies, Power, Unit Control) |
| `factions/gdo/economy.md` | Starting Conditions, Opening Economy Baseline, Economy Targets (GDO vs Colonists), Generator, Extraction Facility, Extraction Plate, Supply Tower |
| `factions/gdo/structures/README.md` | GDO Build Area; index of GDO structures |
| `factions/gdo/structures/deployment_center.md` | Primary construction structure; build queue, cancellation |
| `factions/gdo/structures/barracks.md` | Infantry production |
| `factions/gdo/structures/arsenal.md` | Combat research |
| `factions/gdo/structures/career_campus.md` | Advanced personnel |
| `factions/gdo/structures/factory.md` | Vehicle production |
| `factions/gdo/structures/radar_station.md` | Reconnaissance |
| `factions/gdo/structures/advanced_mechanics.md` | Vehicle upgrades |
| `factions/gdo/structures/airfield.md` | Aircraft production; Aircraft Support |
| `factions/gdo/structures/industrial_powerplant.md` | Power generation |
| `factions/gdo/structures/propulsion_labs.md` | Advanced aircraft |
| `factions/gdo/structures/defense_system.md` | Constructs defenses |
| `factions/gdo/structures/missile_turret.md` | Defensive turret |
| `factions/gdo/structures/bunker.md` | Field Engineer-built garrison |
| `factions/gdo/structures/first_aid_tent.md` | Field Engineer-built healing |
| `factions/gdo/structures/outpost.md` | Prospector-deployed build radius |
| `factions/gdo/units/README.md` | Index of GDO units |
| `factions/gdo/units/peacekeeper.md` | Basic infantry; Riot Shield |
| `factions/gdo/units/rocketman.md` | Missile infantry (Arsenal); Target Lock, MinRange |
| `factions/gdo/units/drone_operator.md` | Suicide-drone infantry (Career Campus) |
| `factions/gdo/units/field_engineer.md` | Repairs buildings/vehicles; builds Bunker, First Aid Tent |
| `factions/gdo/units/raptor.md` | Fast wheeled homing-missile vehicle (Factory) |
| `factions/gdo/units/twister.md` | Wheeled AA vehicle with mines (Factory) |
| `factions/gdo/units/artillery_rig.md` | Deployable artillery (Factory, Advanced Mechanics); deploy model |
| `factions/gdo/units/prospector.md` | Expansion vehicle; deploys into an Outpost |
| `factions/gdo/units/hornet.md` | Air-to-ground hover chopper (Airfield) |
| `factions/gdo/units/dragonfly.md` | Air-to-air glider fighter (Airfield, Propulsion Labs) |
| `factions/gdo/units/supply_chopper.md` | Supply transport aircraft |
| `factions/syndicate/overview.md` | Syndicate identity and resources (Space Crystals, Supplies, Tunnel Space) |
| `factions/syndicate/tunnels.md` | Tunnel Network (incl. losing Tunnels, neutral expansions), Tunnel (tiers, sides, Tunnel Area, Tunnel Space) |
| `factions/syndicate/structures/README.md` | Tech tree overview, Tunnel Expansion rules (requirements, rally points), Surface Defensive Structures; index |
| `factions/syndicate/structures/hideout.md` | Infantry production expansion |
| `factions/syndicate/structures/optics_lab.md` | Laser tech building |
| `factions/syndicate/structures/chemical_plant.md` | Explosives/propulsion tech building |
| `factions/syndicate/structures/vault.md` | Re-arming/recharging, network-wide |
| `factions/syndicate/structures/vehicle_assembly.md` | Ground vehicle production (Tier 2) |
| `factions/syndicate/structures/forge.md` | Advanced materials (Tier 2) |
| `factions/syndicate/structures/null_chamber.md` | Exit buff for manned units |
| `factions/syndicate/structures/aerospace_assembly.md` | Aircraft and satellite production (Tier 3) |
| `factions/syndicate/structures/orbital_command.md` | Orbital coordination; Prism prerequisite (Tier 3) |
| `factions/syndicate/structures/laser_turret.md` | Upright / pop-up defensive turret |
| `factions/syndicate/units/README.md` | Shared rules (red lasers); index of Syndicate units |
| `factions/syndicate/units/agent.md` | Worker: pistol, crystal/Supplies gathering, building (Tunnel Construction Flow) |
| `factions/syndicate/units/guard.md` | Basic laser infantry, ground+air (Optics Lab) |
| `factions/syndicate/units/grenadier.md` | Grenade AoE infantry (Chemical Plant) |
| `factions/syndicate/units/assault_bike.md` | Fastest wheeled vehicle, mortar turret |
| `factions/syndicate/units/shadebringer.md` | Support vehicle: Sludge and Cover (smoke) toggles (Vault) |
| `factions/syndicate/units/laser_battery.md` | Battery-powered laser tank; Amplify, Deploy (AA), Relay through Prisms |
| `factions/syndicate/units/subterranean_transport.md` | Infantry transport that burrows (DrillUnit, Forge) |
| `factions/syndicate/units/phantom.md` | Hovering laser aircraft, ground+air (Optics Lab) |
| `factions/syndicate/units/specter.md` | Single-payload glider bomber; re-arms in Tier 3 Tunnels (Vault + Chemical Plant) |
| `factions/syndicate/units/prism.md` | Satellite: laser relay, one-way launch, thrust charges (Orbital Command) |
| `factions/cults/overview.md` | Cults identity and resources (Space Crystals, Unit Control) |
| `factions/cults/economy.md` | Starting Conditions, building mechanics (Recruit-time principles), Recruitment Hall, Storage |
| `factions/cults/structures/README.md` | Tech tree overview; index of Cults structures |
| `factions/cults/structures/armory.md` | Trains Recruits into combat units |
| `factions/cults/structures/gunworks.md` | Tech/Weaponry T2 support |
| `factions/cults/structures/garrison.md` | Defensive structure |
| `factions/cults/structures/chop_shop.md` | Tech/Weaponry T2 vehicle production |
| `factions/cults/structures/machine_shop.md` | Tech/Weaponry T2 support |
| `factions/cults/structures/airstrip.md` | Tech/Weaponry T3 aircraft production |
| `factions/cults/structures/scrap_hangar.md` | Tech/Weaponry T3 support |
| `factions/cults/structures/temple.md` | Mystical T2; research, Recruit staffing |
| `factions/cults/structures/cathedral.md` | Mystical T3; trains Disciples |
| `factions/cults/structures/shrine.md` | Mystical T3; rituals, energy, abilities |
| `factions/cults/augmentation.md` | Augmentation system, Altar |
| `factions/cults/units/README.md` | Index of Cults units |
| `factions/cults/units/recruit.md` | Worker: gathering, building, Armory entry |
| `factions/cults/units/militant.md` | Basic combat infantry |
| `factions/cults/units/gunner.md` | Heavier-hitting infantry |
| `factions/cults/units/disciple.md` | Spell caster (Cathedral) |
| `factions/colonists/overview.md` | Colonists identity and resources (Space Crystals, Alloys, Essence, Flux, Beacon Capacity) |
| `factions/colonists/economy.md` | Beacon Deployment, Beacon, Warp In, Starting Conditions, Egress Node, refining (Alloy Furnace, Essence Refinery, Flux Reactor) |
| `factions/colonists/structures/README.md` | Tech Model, Portals (upgrade line, exits); index of Colonists structures |
| `factions/colonists/structures/robo_portal.md` | Drone production (standalone) |
| `factions/colonists/structures/bio_portal.md` | Basic military production |
| `factions/colonists/structures/heavy_portal.md` | Portal upgrade stage 2 |
| `factions/colonists/structures/star_portal.md` | Portal upgrade stage 3 |
| `factions/colonists/structures/exophysics_engine.md` | Tech building; unlocks Skylance |
| `factions/colonists/structures/skylance.md` | Anti-air defense |
| `factions/colonists/structures/fusion_core.md` | Tech building; unlocks Plasma Turret |
| `factions/colonists/structures/plasma_turret.md` | Anti-ground defense |
| `factions/colonists/structures/relay_array.md` | Communications tech building |
| `factions/colonists/structures/navigation_spire.md` | Large-aircraft navigation tech building |
| `factions/colonists/structures/psionic_amplifier.md` | Psionic tech building |
| `factions/colonists/structures/structural_analyzer.md` | Materials-analysis tech building |
| `factions/colonists/units/README.md` | Index of Colonists units |
| `factions/colonists/units/drone.md` | Worker: jobs, gathering, patch occupancy, deploying into Beacons |
| `factions/colonists/units/scout.md` | Flying scout (Robo Portal) |
| `factions/colonists/units/lancer.md` | Bio Portal infantry |
| `factions/colonists/units/sabre_legionnaire.md` | Melee form of the Legionnaire |
| `factions/colonists/units/rifle_legionnaire.md` | Ranged form of the Legionnaire |
| `factions/colonists/units/weapon_conversion.md` | Sabre ↔ Rifle Legionnaire conversion rules |

## archive/ — not live design
| File | Contents |
|---|---|
| `archive/designer_notes.md` | Early session notes (Feb–Mar 2026). Historical only. |
| `archive/to_be_converted.md` | Old-format content from before the formal entity style. Mostly superseded; not yet verified that every unique detail was carried over. |

**One file per unit and per structure** (2026-11-25): each faction has `units/` and `structures/` folders; each folder's `README.md` holds the rules shared by its objects and an index.
