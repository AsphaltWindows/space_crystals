# gdo_unit_control_cap_test

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-factions_resources_r1.md

## Task

Add an integration test verifying that GDO unit production is blocked when the Unit Control cap (200) would be exceeded. In-game QA of this behavior is currently blocked because the Extraction Facility is not buildable (tracked in a separate forum topic), so this test provides verification of the cap enforcement logic.

### Changes needed:

1. **Add test in tests/ or as a unit test in faction.rs**: Create a test that:
   - Sets up a GDO player with GdoPlayerResources (unit_control_used near cap)
   - Verifies that has_unit_control() returns false when cost would exceed cap
   - Verifies production systems (barracks_production_tick_system or HQ equivalent) respect the cap check
   - Specifically: set unit_control_used=199, verify has_unit_control(1) is true, set unit_control_used=200, verify has_unit_control(1) is false

2. **Verify production gating**: The production gating code already exists:
   - command_panel.rs ~line 1205: checks has_unit_control(control_cost) before allowing train action
   - faction.rs ~line 927: barracks_production_tick_system checks has_unit_control before spawning
   - The test should confirm these paths correctly block when at cap

3. **Also verify unit_control_used increments on spawn**: faction.rs ~line 273 increments unit_control_used when HQ produces a unit. Verify this accounting is correct.

### Verification:
- cargo test should pass with the new test(s)
- Tests should cover: cap boundary (199->200 transition), at-cap blocking, over-cap blocking, cost-0 units (like SupplyChopper) always allowed
