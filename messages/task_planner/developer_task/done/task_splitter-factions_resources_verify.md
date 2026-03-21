# factions-resources-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-factions-resources.md

## Task

Verify the existing factions and resources implementation against the design spec. All faction resource structs (GdoPlayerResources, SyndicatePlayerResources, CultsPlayerResources, ColonistsPlayerResources), HUD display, power ratio mechanics, unit control caps, and tunnel space tracking are already implemented. This is a verification-only task.

**What already exists (verify these are correct):**
- `game/types/factions.rs`: All four faction resource component structs with correct fields, defaults, cap checks, and power_ratio()
- `ui/hud.rs`: `update_resource_bar_system` with faction-specific HUD field rendering (SC, Supplies, Power, UC, TS, Alloys, Essence, Conduits, BC)
- `game/world/faction.rs`: `get_power_ratio_for_owner` used in production tick systems for proportional slowdown
- Constants: GDO_UNIT_CONTROL_CAP=200, SYNDICATE_MAX_TUNNEL_SPACE=200, COLONISTS_MAX_BEACON_CAPACITY=200
- Extensive unit tests covering caps, over-cap blocking, power ratios, stockpile independence

**Verify:**
1. All resource fields match `artifacts/designer/design/factions.md` spec
2. HUD displays correct fields per faction (GDO: SC+Supplies+Power+UC, Syndicate: SC+Supplies+TS, Cults: SC+UC, Colonists: SC+Alloys+Essence+Conduits+BC)
3. Power ratio correctly causes proportional slowdown when negative
4. Unit control/tunnel space/beacon capacity enforcement blocks unit production when at cap
5. All existing tests pass (`cargo test`)

If everything matches spec, no code changes needed — just confirm with a passing test run.
