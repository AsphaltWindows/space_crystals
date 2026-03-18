# Ticket: Remove WASD Camera Movement

## Current State
The `camera_movement` system in `src/main.rs` (lines 83-97) uses both WASD and arrow keys for camera panning. WASD is conditionally active only when the command panel is hidden (`panel_hidden` variable). Arrow keys are always active. This creates conflicts: A (attack-move) and S (stop) are unit command hotkeys that share the same activation condition as WASD camera movement, and the command panel uses QWEASDZXC as a 3x3 button grid.

## Desired State
Remove all `KeyCode::KeyW`, `KeyCode::KeyA`, `KeyCode::KeyS`, `KeyCode::KeyD` branches from the four directional `if` statements in the `camera_movement` system. Camera panning uses arrow keys only. Clean up the `panel_hidden` variable and its associated comment if it is no longer referenced by any remaining logic in the function.

## Justification
User request via `forum/remove_wasd_camera_movement.md`. WASD bindings conflict with the control system design (`features/control_system.md`): A is attack-move, S is stop, and the command panel uses QWEASDZXC as hotkey grid. Removing WASD from camera movement eliminates state-dependent input behavior and frees these keys permanently for unit commands. Standard RTS conventions use arrow keys, edge-of-screen scrolling, and middle-mouse drag for camera panning — not WASD.

## QA Steps
1. Launch the game. Verify the camera does NOT move when pressing W, A, S, or D keys regardless of whether the command panel is visible or hidden.
2. Press arrow keys (Up, Down, Left, Right). Verify the camera pans correctly in all four directions.
3. With units selected and command panel visible, press A. Verify it triggers the attack-move command (or the appropriate command panel button), not camera movement.
4. With units selected and command panel visible, press S. Verify it triggers the stop command (or the appropriate command panel button), not camera movement.
5. With no units selected and command panel hidden, press W, A, S, D. Verify no camera movement occurs.
6. Verify the `panel_hidden` variable in `camera_movement` is removed if no other logic in the function references it.

## Expected Experience
- Arrow keys smoothly pan the camera in all directions, always.
- WASD keys have zero effect on camera movement in any game state.
- Unit command hotkeys (A, S, and the QWEASDZXC grid) work reliably without competing with camera controls.
- No behavioral change to any other input system.
