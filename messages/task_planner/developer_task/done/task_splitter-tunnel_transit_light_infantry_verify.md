# tunnel-transit-light-infantry-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-tunnel-transit-light-infantry.md

## Task

Verify that tunnel transit tier filtering correctly includes LightInfantry in Tier 1+. The implementation already handles this correctly in `TransitTier::allows_unit_base()` (structures.rs) where Tier1 matches both `UnitBaseEnum::LightInfantry` and `UnitBaseEnum::HeavyInfantry`. Existing tests confirm this at `transit_tier1_allows_infantry`.

No code changes needed. This is a verification-only task to confirm alignment between the updated design doc (syndicate_objects.md line 8: 'Tier 1+: Infantry (Light Infantry, Heavy Infantry)') and the implementation.

Verification steps:
1. Confirm `TransitTier::Tier1.allows_unit_base(&UnitBaseEnum::LightInfantry)` returns true (already tested)
2. Confirm `can_enter_tunnel` in units/utils.rs works for LightInfantry with Tier1 tunnels (already tested at line 631)
3. Run `cargo test` to ensure all transit tier tests pass
