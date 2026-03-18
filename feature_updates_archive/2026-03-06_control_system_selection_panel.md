# Feature Update: Control System — SelectionPanel & GroupCycling

## Modified Feature File
`features/control_system.md`

## Relevant Design Files
- `design/control_system.md` (SelectionPanel section, GroupCycling section)

## Summary of Modifications

### New: SelectionPanel
Added SelectionPanel section — a grid of unit portraits displayed when 2+ units are selected. Includes:
- Visibility rules (hidden for 0-1 units)
- ActiveGroup sheer highlight
- 5 portrait click interactions (left-click, shift-click, ctrl-click, ctrl-shift-click, alt-click)
- Edge cases for selection reduction

### Updated: GroupCycling
Expanded GroupCycling from a single-direction description to include directional keybindings:
- Forward (Tab): advance to next group, wraps around
- Backward (Shift-Tab): advance to previous group, wraps around
