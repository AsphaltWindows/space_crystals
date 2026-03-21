# factions-resources

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement all faction definitions and their resource systems as defined in `artifacts/designer/design/factions.md`.

**GlobalDefenseOrdinance (GDO):**
- DisplayHud: Space Crystals (current), Supplies (current), Power (current/total), Unit Control (used/200)
- Resources:
  - **Space Crystals**: Core resource. Gathered via Extraction Facilities/Plates.
  - **Supplies**: Tactical resource. Gathered via Supply Towers/Choppers from SDS.
  - **Power**: Flat capacity system. Each GDO building has a static Power value (positive=generator, negative=consumer). Total Power = sum across all buildings. If total Power is negative, all power-consuming buildings operate slower proportionally (available/required ratio).
  - **Unit Control**: Hard cap of 200, always fully available (no buildings needed). Each unit has a cost. Cannot build units exceeding cap.

**TheSyndicate:**
- DisplayHud: Space Crystals (current), Supplies (current), Tunnel Space (used/available, max 200)
- Resources:
  - **Space Crystals**: Core resource. Gathered by Agents from patches.
  - **Supplies**: Expansion/tech resource. Gathered by Agents from SDS. Spent on Tunnel construction, upgrades, research.
  - **Tunnel Space**: Unit control. Each Tunnel provides space based on tier. Total caps at 200. Each unit has a cost.

**TheCults:**
- DisplayHud: Space Crystals (current), Unit Control (used/available)
- Resources:
  - **Space Crystals**: Core resource. Gathered by Recruits.
  - **Unit Control**: Provided by Recruitment Centers proportional to Recruitable tiles recruited from. No hard cap — bounded by territorial control.

**Colonists:**
- DisplayHud: Space Crystals (current), Alloys (current), Essence (current), Conduits (current), Beacon Capacity (used/available, max 200)
- Resources:
  - **Space Crystals**: Core resource. Gathered by Prospectors.
  - **Alloys**: Refined from Space Crystals. For buildings and vehicles.
  - **Essence**: Refined from Space Crystals. For research and psychic abilities.
  - **Conduits**: Refined from Alloys and Essence. For advanced research and psionic weaponry.
  - **Beacon Capacity**: Unit control. Each Beacon provides capacity. Caps at 200.

## QA Instructions

1. Start a game as GDO — verify HUD shows Space Crystals, Supplies, Power (current/total), and Unit Control (X/200).
2. Build a PowerPlant (+20 power) and a Barracks (-30 power) — verify Power display updates correctly.
3. Make total Power negative — verify power-consuming buildings visibly slow down proportionally.
4. Try to build units past 200 Unit Control — verify the build option is unavailable.
5. Start as Syndicate — verify HUD shows Space Crystals, Supplies, and Tunnel Space (X/Y where Y is current available, max 200).
6. Build/upgrade Tunnels — verify Tunnel Space total increases per tier (T1=20, T2=30, T3=40).
7. Start as Cults — verify HUD shows Space Crystals and Unit Control (used/available with no fixed cap).
8. Start as Colonists — verify HUD shows Space Crystals, Alloys, Essence, Conduits, and Beacon Capacity.
