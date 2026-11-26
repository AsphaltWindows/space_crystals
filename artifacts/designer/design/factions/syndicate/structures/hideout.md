# Hideout

The Hideout (renamed from Headquarters) is the primary accommodations for Syndicate infantry — a Tier 1 Tunnel expansion that produces Agents and Guards. The Syndicate player starts with one pre-built in their starting Tunnel. Additional Hideouts can be constructed in any Tier 1+ Tunnel.

## Entity Type - Structure Type (Underground)
## Size - 2x2
## Tier Requirement - 1
## TechPrerequisite - None
## Cost - 200 Space Crystals
## Build Time - 400 frames (25 seconds)
## HP - 400
## PointArmor - 6 (underground expansion, n = 6; README > Tunnel Expansions > High armor underground)
## FullArmor - 24
## Produces:
- Agent: 100 Space Crystals, 240 frames (15 seconds)
- Guard: 125 Space Crystals, 120 frames (7.5 seconds). Unit Prerequisite: player owns at least one Optics Lab (2026-11-24).
- Grenadier: 150 Space Crystals, 160 frames (10 seconds), placeholder. Unit Prerequisite: player owns at least one Chemical Plant (2026-11-25).

## HideoutInstanceState:
- RallyPoint: Coordinates | ObjectInstance | None
- BuildQueue: array of ObjectEnum (max 20, including the unit currently in production; user, 2026-11-24)
- CurrentBuild: ObjectEnum | None
- CurrentBuildProgress: number (frames elapsed) | None

## ObjectInterfaceState[Hideout]:

DefaultState commands:

Right-click resolution:
- Right-click Ground: issues SetRallyPoint command to that location
- Right-click Object: issues SetRallyPoint command to that object

Immediate commands (CommandIssuingTransition):
- **Q: Build Agent**: deducts 100 Space Crystals from player, adds Agent to BuildQueue. Only available if BuildQueue has fewer than 20 entries and player has sufficient Space Crystals. Tunnel Space is checked when the Agent is about to start, not when queued (see factions/syndicate/tunnel_area_and_space.md > Tunnel Space > Production and Tunnel Space).
- **W: Build Guard**: deducts 125 Space Crystals from player, adds Guard to BuildQueue. Only available if the player owns at least one Optics Lab, BuildQueue has fewer than 20 entries and player has sufficient Space Crystals. Tunnel Space is checked when the Guard is about to start, not when queued.
- **E: Build Grenadier**: deducts 150 Space Crystals (placeholder), adds Grenadier to BuildQueue. Only available if the player owns at least one Chemical Plant, BuildQueue has fewer than 20 entries and the player can pay.
- **X: Cancel Production**: removes last entry from BuildQueue, refunds full cost to player. Only available if BuildQueue is not empty.

Target commands (StateOnlyTransition):
- **C: Set Rally Point**: enters AwaitingTarget[SetRallyPoint]. Left-click ground or object sets the rally point (CommandIssuingTransition, returns to DefaultState).
