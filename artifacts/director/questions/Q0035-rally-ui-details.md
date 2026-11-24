# Q0035: Rally point UI details: Shift, rallying on a selected Barracks, pressing an unavailable command

- **Date**: 2026-11-24T02:09:35-05:00
- **Blocking**: no. M2-H1 is built with the current behaviour listed under each point.
- **Docs**: `systems/controls/selection_and_hud.md > CommandPanel > Production Building Default Right-Click`; changes 0070 (Shift keeps targeting active), 0086 (a rally point on the building itself clears it), 0087.

## Question
1. **Shift and the rally point.** A Barracks has a single rally point, so queuing doesn't apply. What does Shift do?
   - **Current behaviour:** after pressing Rally (C), a Shift-click acts as a plain click and ends targeting. A Shift+right-click sets the rally point at once.
   - **Alternative:** 0070's rule. Targeting stays active while Shift is held, and each Shift-click moves the rally point.
2. **A rally point set on one of several selected Barracks.** The player selects three Barracks and right-clicks one of them.
   - **Current behaviour:** under 0086, the clicked Barracks clears its rally point, and the other two rally on the clicked Barracks.
   - **Alternatives:** all three clear their rally points; or all three rally on the clicked one, except that one, which clears.
3. **Pressing a command that's unavailable right now**, for example Train with too few Space Crystals or a full queue.
   - **Current behaviour:** the button is greyed out, and pressing it, or its hotkey, gives no feedback.
   - **Alternative:** the same message as when the sim refuses the order, e.g. "Not enough Space Crystals".

## Context
The Barracks panel, the production right-click and the rally point indicators were built in M2-H1. The docs don't cover these three cases.

## Options
For each point: A, the current behaviour; B, the alternative given, or a rule you give.
