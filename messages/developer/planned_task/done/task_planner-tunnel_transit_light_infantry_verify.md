# tunnel-transit-light-infantry-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

**No code changes required.** This is a verification-only task. The developer should confirm existing code and tests, then run the test suite.

### Key files to review (read-only verification):

- **`artifacts/developer/src/game/types/structures.rs`** (line 478): `TransitTier::allows_unit_base()` — Tier1 matches `UnitBaseEnum::LightInfantry | UnitBaseEnum::HeavyInfantry`. Test at line 981 (`transit_tier1_allows_infantry`) asserts both.

- **`artifacts/developer/src/game/units/utils.rs`** (line 175): `can_enter_tunnel()` — validates faction, owner, and tier. Test at line 629 (`can_enter_tunnel_valid_syndicate_light_infantry_t1`) asserts LightInfantry + Tier1 succeeds.

- **`artifacts/developer/src/game/units/systems/core.rs`** (lines 327, 570, 607): Three call sites of `can_enter_tunnel()` — right-click AwaitingTarget[Enter], agent right-click tunnel, guard right-click tunnel. All pass `unit_base` directly, so LightInfantry flows through correctly.

- **`artifacts/developer/src/game/units/systems/behaviors.rs`** (line 497): `enter_tunnel_dispatch_system` validates via `can_enter_tunnel()` before inserting `EnteringTunnelBehavior`.

### Verification command:
```bash
cargo test -p space_crystals -- transit_tier can_enter_tunnel
```
This runs all transit tier and tunnel entry tests. All should pass green.

## Dependencies

None. This is a standalone verification task with no code changes or dependencies on other tasks.
