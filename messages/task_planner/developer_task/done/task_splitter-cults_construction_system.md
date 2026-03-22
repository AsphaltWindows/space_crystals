# cults_construction_system

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-cults_objects_formalized.md

## Task

Implement the Cults construction execution system — the mechanics of Recruits entering buildings under construction, proportional speedup, completion consumption, and cancellation.

### Overview

When Recruits reach a Cults building under construction, they enter it (become hidden/consumed into the building). The building's construction speed scales linearly with the number of Recruits inside. On completion, all Recruits inside are consumed (despawned). On cancellation, all Recruits are ejected and returned to play.

### New component: CultsConstructionState

Add a component to track Cults building construction state:
- `assigned_recruits: Vec<Entity>` — entities of Recruits inside the building
- `construction_progress: u32` — frames of progress completed
- `total_construction_frames: u32` — total frames needed (base, with 1 Recruit)

Attach this component when a Cults building is spawned under-construction (from the placement task).

### Recruit enter behavior

When a Recruit with a ConstructBuilding command reaches the target building:
- Add the Recruit entity to the building's `CultsConstructionState.assigned_recruits`
- Set the Recruit to Visibility::Hidden (or despawn — but keeping entity allows cancellation refund)
- Remove the Recruit's movement/command state

### Construction tick system (`cults_construction_tick_system`)

Each frame, for each entity with CultsConstructionState + ConstructionHP:
- If `assigned_recruits` is empty, do nothing (construction paused)
- Progress = number of assigned Recruits per frame (1 Recruit = 1 frame progress per frame, 2 Recruits = 2 frames per frame, etc.)
- Increment `construction_progress` by the number of assigned Recruits
- Update ConstructionHP proportionally (hp = max_hp * (construction_progress / total_construction_frames), minimum 10% per ConstructionHP Rule)
- When `construction_progress >= total_construction_frames`:
  - Set HP to full max HP, remove ConstructionHP component
  - Despawn all Recruit entities in `assigned_recruits` (they are consumed)
  - Remove CultsConstructionState component (building is complete)
  - The building becomes fully operational

### Cancellation

When a Cults building under construction is cancelled (via a cancel command or destruction):
- For each Recruit in `assigned_recruits`:
  - Restore Visibility to Visible
  - Place them near the building's position (find valid nearby tile)
  - Restore their command/movement state to idle
- Despawn the building entity
- No resource cost to refund (Cults buildings cost Recruits, not crystals — the Recruits themselves are the cost)

### Multiple Recruits speedup

The proportional speedup is inherent in the tick system: progress per frame = count of assigned Recruits. With base construction requiring N frames with 1 Recruit, having K Recruits completes in N/K frames.

### Tests

- Test that 1 Recruit completes construction in exactly `total_construction_frames` frames
- Test that 2 Recruits complete in half the frames
- Test that cancellation returns all assigned Recruits
- Test that Recruits are despawned on completion
- Test ConstructionHP scales correctly during construction
