# Q0030: Structures as targets and in the interface

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `systems/combat.md > ValidTarget`;
  - `systems/controls/behaviors.md > BaseAutoTargeting`;
  - `factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`;
  - `core/entities.md > Object Type > InfoPanel`;
  - `systems/controls/selection_and_hud.md > CommandPanel > Grid Layout`.

## Question
1. **Do units attack enemy buildings on their own?**
   - ValidTarget begins "An enemy unit is a valid target", but its first condition is "a Destructible ObjectInstance".
   - **Proposal:** yes. Enemy buildings count as targets that can't shoot back, so units that can shoot back are picked first. M1 already behaves this way.
2. **Build menu slots.**
   - The DC's BuildMenu entries have no slots or hotkeys.
   - **Proposal:** Q Generator, W Barracks, E Extraction Facility, in the order of `> Constructs`, plus Z Back and X Cancel. When a building is ready, its own slot (for example, Q for a Generator) becomes "Select completed building".
3. **InfoPanels for buildings.**
   - Only the plate's InfoPanel is specified.
   - **Proposal:**
     - **For an own building:** name, HP, and what it is doing (construction or production with progress, queue length, the building ready to place, the rally point).
     - **For another player's building:** name, owner and HP.

## Context
Buildings become targets and get panels in M2.

## Options
- **Point 1:** A, enemy buildings are attacked automatically; B, only on an explicit Attack order.
- **Points 2 and 3:** A, as proposed; B, the Designer's layout.
