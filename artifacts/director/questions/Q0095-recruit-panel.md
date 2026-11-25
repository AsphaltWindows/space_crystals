# Q0095 — The Recruit's panel letters, and its pointer over a building under construction

- **Date**: 2026-11-25T14:03:45-05:00
- **Blocking**: no — proceeding on assumption: Q Move, A Construct, S Assist Construction, X Stop; a new Assist pointer.
- **Docs**: `factions/cults/units/recruit.md > Recruit ObjectInterfaceState`; `systems/controls/selection_and_hud.md > CommandPanel` (the grid), `> PointerDisplayType`; `factions/syndicate/units/agent.md > Agent ObjectInterfaceState` (0257)

## Question
1. The Recruit's panel gives "**B: Assist Construction**", but B isn't a slot of the 12-key grid (Q W E R / A S D F / Z X C V). Which letter does Assist take?
2. "Stop — panel button" has no letter, and Move has no panel button. Do they take X and Q, as on the Agent?
3. Right-clicking an own building under construction Assists, but no pointer type says so. Which pointer is shown?

## Context
The Agent's panel (0257): "**Q** Move, **A** Build Tunnel, **S** Drop Off Resources, ... **X** Stop", with Q Move and X Stop "as in BasicCombatUnitInterfaceState". The pointer types are Inactive, Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources, Enter and Mixed.

## Options
1. Assist on **S** (the slot after Construct), **D**, or **F**.
2. **Q** Move and **X** Stop, as on the Agent; or no Move button (right-click only) and **X** Stop.
3. A new **Assist** pointer (placeholder look); or the **Move** pointer; or the **Enter** pointer.

Proposed: Q Move, A Construct, S Assist Construction, X Stop; a new Assist pointer. Each is one line in the client.
