# Q0016: Command UI edge cases the docs don't cover

- **Date**: 2026-11-23T20:15:00-05:00
- **Blocking**: no. The command panel, targeting and right-click are built with the behaviour listed under each point. Each one is a single named rule in the code.
- **Docs**: `systems/controls/selection_and_hud.md` (ObjectInterfaceState, InterfaceTransition, CommandPanel), `systems/controls/commands.md` (right-click behaviour)

## Question
Please confirm or correct each of these:
1. **Who carries out a right-click.** The docs say what a right-click resolves to: ground gives Move, an enemy gives Attack, and an own unit, crystal patch or station gives Move to that object. They don't say who carries it out. Current behaviour: every selected unit that has the resolved command carries it out. Units without it do nothing; a future harvester, for example, ignores a right-click on an enemy. In StarCraft II, units that can't attack the target move to or follow it instead.
2. **An invalid target while choosing one.** An example is a left-click on the ground while Attack waits for a target it can use. Current behaviour: the click is ignored and the game keeps waiting for a valid target. The alternatives are to cancel targeting, or to show an error message and keep waiting.
3. **Other input while a target is awaited.** Current behaviour:
   - the other command buttons are disabled;
   - dragging a box selects nothing and draws no box;
   - changing the selection, or the active group, cancels targeting.
4. **Shift while choosing a target.** The docs return to the default state after every issuing click. Current behaviour follows them: a Shift-click queues the order and then leaves targeting. In StarCraft II, targeting stays active while Shift is held, so several waypoints can be placed with one key press. Is the docs' version intended?

## Context
These came up while building the command panel and targeting (M1). None blocks anything, but each one is something the player feels in every game. Points 1 and 4 matter most.

## Options
For each point: the current behaviour (A), or a different rule you give. Main alternatives:
- **Point 1 (B):** units without the command move to, or follow, the target instead.
- **Point 4 (B):** targeting stays active while Shift is held.
