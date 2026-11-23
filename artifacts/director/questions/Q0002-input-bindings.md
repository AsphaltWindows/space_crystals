# Q0002 — Input bindings: map selection clicks, control-group keys, camera pan, command queueing

- **Date**: 2026-11-23T11:35:00-05:00
- **Blocking**: no — proceeding on assumption: StarCraft 2-like defaults (listed under Options, choice A).
- **Docs**: `systems/controls/selection_and_hud.md` (ControlGroups operations, portrait clicks, BoxSelection), `core/camera.md > Panning`, `systems/controls/commands.md` (command queue)

## Question
The design defines the control-group operations (Assign, Add, Recall, Recall and Center), portrait clicks, box selection tiers and the command panel hotkeys. It does not say which inputs trigger the following:
1. **Clicking on the map** — left-click selects one object. What do Shift-click (add/remove?), Ctrl-click and double-click (all of that type on screen?) do?
2. **Control-group keys** — which keys Assign, Add, Recall, and Recall and Center?
3. **"Standard pan"** (`core/camera.md > Panning`) — which inputs pan: arrow keys, screen-edge scrolling, middle-mouse drag? At what speed?
4. **Command queueing** — units keep a command queue. Which modifier appends a command instead of replacing the queue (Shift)?

## Context
The M1 sandbox (select, move, fight) needs these bindings. Bindings sit in one table in the client's input layer, so changing them later is cheap.

## Options
- **A (assumed, SC2-like)**: Shift-click toggles an object in the selection. Ctrl-click or double-click selects all own units of that type on screen. Ctrl+0–9 assigns, Shift+0–9 adds, 0–9 recalls, double-tapping 0–9 recalls and centers. Arrow keys and screen edge pan at a fixed speed. Shift appends commands.
- **B (SC1-like)**: same, except Shift+0–9 also assigns and there is no Add.
- **C**: something else the Designer prefers (e.g. rebindable keys from day one).
