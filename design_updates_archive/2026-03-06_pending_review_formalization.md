# Design Update: Pending Review Formalization

**Date**: 2026-03-06
**Session**: Formalized 4 forum-originated design items into design docs

## Changes

### 1. Placement Validation (`design/entities.md`)

Added **Placement Validation** section under Structure Type defining three validation models:

- **Direct Placement** (GDO buildings, Tunnel underground expansions): All footprint tiles must pass validation at confirmation. Surface buildings require all tiles in **Visible** state; placement ghost shown red if not. Underground expansions validated against underground spatial rules (Tunnel Area bounds, no overlap), surface visibility not relevant. Standard spatial checks also apply.
- **Worker-Built Structures** (e.g., Agent building a Tunnel): Build command can be queued regardless of current visibility. Validation happens on worker arrival (Buildable, unoccupied, faction constraints). If validation fails, command cancelled, worker stops and idles. No visibility requirement on arrival.

### 2. Box Selection Priority (`design/control_system.md`)

Added **BoxSelection** section defining a 5-tier priority system for drag-box selection:

1. Own units — multi-select (all in box)
2. Own buildings — single-select (closest to box center)
3. Enemy units — single-select (closest to box center)
4. Enemy buildings — single-select (closest to box center)
5. Neutral objects — single-select (closest to box center)

Only own units produce multi-selection. Own buildings are never box-selected alongside own units.

### 3. Command Indicators (`design/control_system.md`)

Added **CommandIndicators** section defining persistent visual markers at command targets:

- Indicators only visible when a unit with that command is in the current Selection
- All active command indicators shown simultaneously
- Removed on deselect or command completion
- Two indicator types: Location (marker on ground) and Object (surrounds target perimeter)
- Color language: Green (peaceful movement: Move, Reverse, Enter), Red (hostile target: Attack, AttackGround), Orange (aggressive movement: AttackMove, Patrol)

### 4. Unit Collision (`design/units.md`)

Added **UnitCollision** section defining two collision models:

- **Ground Collision**: Hard collision using Silhouette rectangle. No overlap, no push forces. Idle units don't move aside. Moving units pathfind around.
- **Air Collision**: No collision with ground units or structures. Soft separation with other air units via circular SeparationRadius (per unit type, must be larger than Silhouette).
