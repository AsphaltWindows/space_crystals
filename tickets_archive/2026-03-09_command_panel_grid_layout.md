# Ticket: CommandPanel 3x3 Grid Layout with Hotkeys

## Current State
The CommandPanel concept exists in the control system feature spec but has no defined spatial layout or hotkey mapping. Commands are described abstractly without specifying how they are arranged visually or which keys activate them.

## Desired State
Implement the CommandPanel as a 3x3 grid of command slots with the following default hotkey mapping:

```
| Q | W | E |
| A | S | D |
| Z | X | C |
```

Each slot can hold one command or be empty. The grid is populated each tick based on the current ControlState and game state:
- CommonCommands (available to every object in Selection) fill designated slots
- GroupCommands (available to ActiveGroup's object type) fill their designated slots
- Empty slots show as inactive/blank

The grid layout must support:
- Visual distinction between CommonCommands and GroupCommands
- Empty/inactive slots when no command occupies a position
- Hotkey activation: pressing the mapped key triggers the command in that slot (if occupied)
- Slot assignment is per-object-type — each object type's interface state defines which commands go in which slots

## Justification
`features/control_system.md` — CommandPanel / Grid Layout section. The 3x3 grid is the primary interface for issuing commands. Without a concrete layout and hotkey system, the command panel cannot be implemented.

## QA Steps
1. [human] Select a combat unit — verify the CommandPanel displays as a 3x3 grid with commands in their assigned slots
2. [human] Press Q/W/E/A/S/D/Z/X/C — verify each key activates the command in its corresponding slot (or does nothing if the slot is empty)
3. [human] Select multiple unit types — verify CommonCommands and GroupCommands are visually distinguished in the grid
4. [human] Select an object with fewer than 9 commands — verify unoccupied slots appear as inactive/blank
5. [human] Change ActiveGroup via Tab — verify the grid updates to show the new ActiveGroup's commands

## Expected Experience
The command panel appears as a familiar 3x3 grid (similar to StarCraft/Warcraft command cards). Each slot shows an icon or label for its command, with a visible hotkey indicator. Pressing a hotkey feels instant — the command activates or the state transitions immediately. Empty slots are visually muted. The grid updates fluidly when switching between selection groups.
