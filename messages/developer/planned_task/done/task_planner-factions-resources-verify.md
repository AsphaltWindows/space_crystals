# factions-resources-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Verify (no changes expected)

1. **`artifacts/developer/src/game/types/factions.rs`** — All four faction resource structs
   - `GdoPlayerResources` (line 84): fields `space_crystals`, `supplies`, `power_generated`, `power_consumed`, `unit_control_used`, `unit_control_cap`, `has_power_plant` — matches spec (SC, Supplies, Power, Unit Control)
   - `SyndicatePlayerResources` (line 139): fields `space_crystals`, `supplies`, `tunnel_space_provided`, `tunnel_space_used` — matches spec (SC, Supplies, Tunnel Space)
   - `CultsPlayerResources` (line 170): fields `space_crystals`, `unit_control_used`, `unit_control_available` — matches spec (SC, Unit Control with no hard cap)
   - `ColonistsPlayerResources` (line 198): fields `space_crystals`, `alloys`, `essence`, `conduits`, `beacon_capacity_provided`, `beacon_capacity_used` — matches spec (SC, Alloys, Essence, Conduits, Beacon Capacity)
   - Cap constants (lines 76-80): GDO=200, Syndicate=200, Colonists=200 — correct
   - `power_ratio()` (line 124): returns 1.0 when sufficient, `generated/consumed` when in deficit — correct per spec
   - Cap enforcement methods: `has_unit_control()`, `has_tunnel_space()`, `has_beacon_capacity()` all use `used + cost <= available` — correct
   - **35 unit tests** (lines 234-572) covering: defaults, cap enforcement, over-cap blocking, power ratio edge cases, stockpile independence, cross-faction checks

2. **`artifacts/developer/src/ui/hud.rs`** — HUD resource bar
   - `resource_bar_fields_for_faction()` (line 1239): spawns correct fields per faction
     - GDO: Crystals, Supplies, Power, UnitControl — matches spec
     - Syndicate: Crystals, Supplies, TunnelSpace — matches spec
     - Cults: Crystals, UnitControl — matches spec
     - Colonists: Crystals, Alloys, Essence, Conduits, BeaconCapacity — matches spec
   - `update_resource_bar_system()` (line 1279): updates text per faction with correct formatting
     - GDO Power display shows `net / generated` with green/red coloring based on deficit — correct

3. **`artifacts/developer/src/game/world/faction.rs`** — Power ratio integration
   - `get_power_ratio_for_owner()` (line 974): queries `GdoPlayerResources` for matching player, returns `power_ratio()`
   - Used in production tick systems at lines 205, 251, 769, 1907 — adds `power_ratio` to construction/production progress each tick, causing proportional slowdown when in deficit — correct per spec

### Verification Approach

Run `cargo test` from `artifacts/developer/` to confirm all 35 faction tests pass. Visually compare each struct's fields against `artifacts/designer/design/factions.md`. No code changes are expected unless a discrepancy is found.

### Design Spec Reference
- `artifacts/designer/design/factions.md` — authoritative spec for all faction resources and HUD fields

## Dependencies

None. This is a standalone verification task with no code changes expected.
