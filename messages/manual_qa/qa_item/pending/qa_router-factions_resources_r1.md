# factions_resources_r1

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# factions_resources_r1

## Metadata
- **From**: manual_qa
- **To**: task_splitter

## Content

Rework for factions_resources: The GDO and Syndicate resource systems passed QA (HUD displays, Power system with deficit slowdown, Tunnel Space per tier). However, three aspects could not be verified:

1. **Unit Control cap (GDO)**: Could not test the 200 unit cap because the Extraction Facility is not buildable, preventing resource gathering to produce enough units. This is tracked in a separate forum topic. Once Extraction Facility is functional, the Unit Control cap (build option unavailable at 200/200) needs verification.

2. **The Cults faction**: Not yet available in-game. Needs full implementation: Space Crystals resource, Unit Control provided by Recruitment Centers proportional to Recruitable tiles, no hard cap, and HUD showing Space Crystals and Unit Control (used/available).

3. **Colonists faction**: Not yet available in-game. Needs full implementation: Space Crystals, Alloys (refined from SC), Essence (refined from SC), Conduits (refined from Alloys+Essence), Beacon Capacity (max 200), and HUD showing all five resources.

## QA Instructions

1. As GDO, build units until reaching 200 Unit Control — verify the build option becomes unavailable and the cap cannot be exceeded.
2. Start as Cults — verify HUD shows Space Crystals and Unit Control (used/available with no fixed cap, bounded by territorial control via Recruitment Centers).
3. Start as Colonists — verify HUD shows Space Crystals, Alloys, Essence, Conduits, and Beacon Capacity (X/Y, max 200).
