# Feature: Factions and Resources

## Overview
Four playable factions, each with unique resource systems, army size mechanics, and DisplayHuds.

## Design Sources
- `design/factions.md`

## Specifications

### Global Defense Ordinance (GDO)
Military faction with centralized construction and global power grid.

**Resources**:
- **Space Crystals**: Core resource. Gathered via Extraction Facilities/Plates from Space Crystal Patches.
- **Supplies**: Tactical/tech resource. Gathered via Supply Towers/Choppers from Supply Delivery Stations.
- **Power**: Flat capacity system. Each building has static Power value (+generators, -consumers). Total Power = sum across all buildings. If negative: all consumers operate slower (proportional to available/required).
- **Unit Control**: Hard cap 200, always fully available (no infrastructure needed). Each unit costs Unit Control.

**DisplayHud**: Space Crystals, Supplies, Power (current/total), Unit Control (used/200)

### The Syndicate
Covert faction with underground tunnel networks.

**Resources**:
- **Space Crystals**: Core resource. Gathered by Agents from Space Crystal Patches.
- **Supplies**: Expansion/tech resource. Gathered by Agents from Supply Delivery Stations.
- **Tunnel Space**: Unit control. Each Tunnel provides space based on upgrade level. Total caps at 200.

**DisplayHud**: Space Crystals, Supplies, Tunnel Space (used/available, max 200)

### The Cults
Territorial faction spreading through recruitment from the land.

**Resources**:
- **Space Crystals**: Core resource. Gathered by Recruits from Space Crystal Patches.
- **Unit Control**: Provided by Recruitment Centers proportional to Recruitable tiles recruited from. No hard cap. Army size bounded by territorial control.

**DisplayHud**: Space Crystals, Unit Control (used/available)

### Colonists
Psionic faction with deep refining economy.

**Resources**:
- **Space Crystals**: Core resource. Gathered by Prospectors from Space Crystal Patches.
- **Alloys**: Refined from Space Crystals. For buildings and vehicles.
- **Essence**: Refined from Space Crystals. For research and psychic abilities.
- **Conduits**: Refined from Alloys and Essence. For advanced research and psionic weaponry.
- **Beacon Capacity**: Unit control. Each Beacon provides capacity. Total caps at 200.

**DisplayHud**: Space Crystals, Alloys, Essence, Conduits, Beacon Capacity (used/available, max 200)

## Dependencies
- `entity_system` (Faction, Player entities)
- `entity_system` (SpaceCrystalsPatch, SupplyDeliveryStation resources)

## Notes
- Three factions have hard unit caps (GDO 200, Syndicate 200, Colonists 200). Cults have no hard cap.
- Each faction has a unique economy loop: GDO is building-based, Syndicate is worker-based with tunnels, Cults are territory-based, Colonists have a refining chain.
- GDO objects are fully specified. Syndicate Tunnel mechanics are formalized (see `syndicate_objects.md`), but the full Syndicate roster (expansions, units, defenses) remains unformalized. Cults and Colonist objects exist only in `design/to_be_converted.md` as unformalized content.
