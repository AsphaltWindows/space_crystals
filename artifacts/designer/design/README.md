# Space Crystals — Design Documents

Index of the design docs. Paths are relative to this folder.

**References:** cross-file references are written as `path/to/file.md > Section > Subsection`. A reference without a file path (e.g. "see Recruit > Gathering") points to a section in the same faction's folder (or the same `systems/` / `core/` area).

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
| `systems/controls/commands.md` | Unit commands: Move, Attack, Attack Ground, Attack Move, Patrol, Hold, Stop, Reverse, Enter |
| `systems/controls/behaviors.md` | Unit/turret behaviors, auto-targeting, action channels (locomotion, orientation, attack) |

## factions/ — one folder per faction
| File | Contents |
|---|---|
| `factions/gdo/overview.md` | GDO identity and resources (Space Crystals, Supplies, Power, Unit Control) |
| `factions/gdo/economy.md` | Starting Conditions, Opening Economy Baseline, Economy Targets (GDO vs Colonists), Generator, Extraction Facility, Extraction Plate, Supply Tower |
| `factions/gdo/structures.md` | Deployment Center, GDO Build Area, Barracks, tech buildings, Defense System, Missile Turret, Bunker, First Aid Tent (Field Engineer-built), Outpost (Prospector-deployed build radius) |
| `factions/gdo/units.md` | Peacekeeper (incl. Riot Shield), Rocketman (incl. Target Lock), Drone Operator (suicide drones), Field Engineer (repair buildings/vehicles, Bunker, First Aid Tent), Raptor (Factory homing-missile vehicle), Twister (Factory AA + mines), Artillery Rig (Factory, name TBD), Hornet (Airfield, air-to-ground hover chopper) + Dragonfly (Airfield, air-to-air glider fighter), Supply Chopper |
| `factions/syndicate/overview.md` | Syndicate identity and resources (Space Crystals, Supplies, Tunnel Space) |
| `factions/syndicate/tunnels.md` | Tunnel Network (incl. losing Tunnels, neutral expansions), Tunnel (tiers, sides, Tunnel Area, Tunnel Space) |
| `factions/syndicate/structures.md` | Tech tree overview, Tunnel Expansions (Hideout, Null Chamber, …), Laser Turret |
| `factions/syndicate/units.md` | Guard, Agent (incl. crystal/supply gathering) |
| `factions/cults/overview.md` | Cults identity and resources (Space Crystals, Unit Control) |
| `factions/cults/economy.md` | Starting Conditions, building mechanics (Recruit-time principles), Recruitment Hall, Storage |
| `factions/cults/structures.md` | Tech tree overview, Armory, Gunworks, Garrison, Chop Shop, Machine Shop, Airstrip, Scrap Hangar, Temple, Cathedral, Shrine |
| `factions/cults/augmentation.md` | Augmentation system, Altar |
| `factions/cults/units.md` | Recruit (incl. gathering), Soldier, Gunner, Acolyte |
| `factions/colonists/overview.md` | Colonists identity and resources (Space Crystals, Alloys, Essence, Flux, Beacon Capacity) |
| `factions/colonists/economy.md` | Beacon Deployment, Beacon, Warp In, Starting Conditions, Egress Node, refining (Alloy Furnace, Essence Refinery, Flux Reactor) |
| `factions/colonists/structures.md` | Tech Model, Portals (Robo/Bio/Heavy/Star), tech buildings, Skylance, Plasma Turret |
| `factions/colonists/units.md` | Drone (incl. jobs, patch occupancy), Scout, Lancer, Sabre/Rifle Legionnaire, Weapon Conversion |

## archive/ — not live design
| File | Contents |
|---|---|
| `archive/designer_notes.md` | Early session notes (Feb–Mar 2026). Historical only. |
| `archive/to_be_converted.md` | Old-format content from before the formal entity style. Mostly superseded; not yet verified that every unique detail was carried over. |

Planned: once most units exist, faction `units.md` / `structures.md` will be split into one file per unit / building.
