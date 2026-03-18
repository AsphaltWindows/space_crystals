# Ticket: Info Panel Does Not Update When Switching Control Groups

## Current State
When switching between control groups via hotkeys (e.g., pressing 1 then 2), the command panel updates correctly to reflect the new selection, but the info panel and portrait remain stale — they continue showing the previous selection's information. This occurs specifically when both control groups contain the same number of entities (e.g., 1 entity each), because the HUD rebuild condition at `src/ui/hud.rs:251` only compares entity count, not entity identity.

## Desired State
The info panel and portrait must update immediately to reflect the currently selected entity whenever the Selection changes — including when switching between control groups that contain the same number of entities. The rebuild condition should compare the set of selected entity identities, not just the count.

## Justification
The feature spec (`features/control_system.md`) states the CommandPanel "derives from ControlState + game state each tick." The info panel must follow the same contract: it must reflect the current Selection at all times. This bug was confirmed by QA, product_analyst, and task_planner in forum topic `info_panel_stale_on_control_group_switch.md` (archived, 6/6 close votes).

## QA Steps
1. [human] Spawn a GDO faction game with at least two different structures (e.g., Deployment Center and Extraction Facility)
2. [human] Assign the Extraction Facility to control group 2 (Ctrl+2)
3. [human] Assign the Deployment Center to control group 1 (Ctrl+1)
4. [human] Press 2 to select the Extraction Facility — verify info panel shows EF information
5. [human] Press 1 to select the Deployment Center — verify info panel and portrait immediately update to show DC information
6. [human] Rapidly alternate between 1 and 2 several times — verify info panel tracks the active selection each time without delay or stale data

## Expected Experience
When pressing control group hotkeys, the info panel and portrait should update instantly (same frame or next frame) to show the newly selected entity's name, icon, health, and other stats. There should be no lingering display of the previously selected entity's information. The command panel should continue to update correctly as it does today.
