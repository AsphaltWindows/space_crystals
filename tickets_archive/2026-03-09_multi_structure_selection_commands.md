# Ticket: Multi-Structure Selection Shows Unit Commands Instead of Structure Menu

## Current State
When selecting 2+ structures of the same type (e.g., two Barracks), the command panel shows default unit commands (Move, Stop, Attack) instead of the structure's menu (e.g., Train Peacekeeper, Cancel). The root cause is at `command_panel.rs:311`: a `struct_count != 1` guard causes the function to skip the entire structure menu branch when multiple structures are selected, falling through to the unit branch. Single-structure selection works correctly.

## Desired State
Multi-selection of same-type structures should show that structure type's command panel, identical to single-structure selection. The guard at `command_panel.rs:311` should change from `struct_count != 1` to `struct_count == 0`. When `struct_count >= 1`, the function should use the active group to determine which structure entity drives the menu. Commands issued from the shared menu should apply to all selected structures of that type.

## Justification
The feature spec (`features/control_system.md`) defines Selection as an array of SelectionGroups, each with a Type (ObjectEnum) and Instances. The CommandPanel derives GroupCommands from the ActiveGroup's type — nothing limits this to single-instance groups. `features/gdo_objects.md` confirms Barracks is groupable, meaning multiple Barracks can coexist in one SelectionGroup. This affects all groupable structures: Barracks, Power Plant, Supply Tower, Extraction Plate. Ungroupable structures (DC, EF) are unaffected. Bug confirmed in forum topic `multi_structure_selection_shows_unit_commands.md` (archived, 6/6 close votes).

## QA Steps
1. [human] Start a GDO game and build two Barracks
2. [human] Box-select both Barracks
3. [human] Verify the command panel shows Barracks commands (Train Peacekeeper, etc.) — not unit commands (Move, Stop, Attack)
4. [human] Press Q to train a Peacekeeper — verify the command is issued to both Barracks (both should begin training)
5. [human] Build two Power Plants and box-select them — verify Power Plant commands appear
6. [human] Select a single Barracks — verify single-selection still works correctly as before

## Expected Experience
When multiple structures of the same type are selected, the command panel should display that structure's full command menu, exactly as it appears for a single selection. Commands issued from this menu (train, cancel, etc.) should apply to all selected structures simultaneously. There should be no visible difference in the command panel between selecting one Barracks and selecting three Barracks — the same buttons appear in both cases.
