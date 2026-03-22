# command_panel_ownership_guard

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# command_panel_ownership_guard

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added an ownership guard to the CommandPanel in `artifacts/designer/design/control_system.md`. The CommandPanel section now explicitly states:

- The CommandPanel is only displayed when **all objects in the Selection are owned by the player**
- When selecting enemy or neutral objects, only the InfoPanel is shown
- No commands are available, no right-click resolution is performed, and no InterfaceTransitions can be initiated for non-owned selections
- Commands can only be issued to objects owned by the player

This was implicit in the design but never stated explicitly, leading to a bug where players could issue commands to enemy units and buildings.

## QA Instructions

1. **Own units — commands work**: Select your own units. Verify the CommandPanel appears with the expected commands. Issue commands (move, attack, etc.) and verify they work normally.

2. **Own buildings — commands work**: Select your own buildings (e.g., Deployment Center, Barracks). Verify the CommandPanel appears and build commands function.

3. **Enemy unit — no commands**: Single-select an enemy unit. Verify only the InfoPanel is shown (HP, type, etc.). Verify the CommandPanel is hidden. Verify right-clicking does nothing (no move/attack command issued to the enemy unit). Verify hotkeys (Q, A, S, etc.) do not trigger any commands.

4. **Enemy building — no commands**: Single-select an enemy building. Same checks as above — InfoPanel only, no CommandPanel, no right-click resolution, no hotkey commands.

5. **Neutral object — no commands**: Single-select a neutral object (e.g., Supply Delivery Station). Verify only the InfoPanel is shown, no commands available.

6. **Mixed selection not possible**: Verify that selecting an enemy or neutral object always results in a single-selection (this is already enforced by Selection constraints), and confirm the CommandPanel is hidden for that selection.
