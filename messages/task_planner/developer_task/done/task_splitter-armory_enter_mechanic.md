# armory_enter_mechanic

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Implement the mechanic for Recruits to enter the Cults Armory building.

### Recruit Enter Command for Armory
When a Recruit is ordered to enter an Armory (via right-click on own Armory or explicit Enter command):
1. The Recruit walks to the entrance side (A side — one of the short ends of the 3x2 footprint)
2. Upon reaching the entrance, the Recruit entity is stored in the Armory's `stored_recruits` list
3. The Recruit entity should be hidden (Visibility::Hidden or similar pattern used by tunnel enter)
4. StoredRecruits count increments (visible in info panel)
5. If the Armory already has 10 stored Recruits (ARMORY_INTERNAL_RECRUIT_CAPACITY), the enter command should be rejected/unavailable

### Right-click Resolution
Add to the right-click resolution system (core.rs right_click_move_command):
- Recruit right-clicks own CultsArmory → Enter command (similar to how units enter tunnels)
- Only if stored_recruits.len() < ARMORY_INTERNAL_RECRUIT_CAPACITY

### Behavior System
Create an `entering_armory_behavior_system` (or reuse/extend existing entering behavior patterns):
- Move toward the entrance side position of the Armory
- On arrival: remove from field, add to ArmoryState.stored_recruits, hide entity
- Follow the same patterns as EnteringTunnelBehavior but target the Armory entrance side

### Tests
- Recruit enters Armory and is added to stored_recruits
- StoredRecruits caps at 10 (11th Recruit cannot enter)
- Non-Recruit units cannot enter the Armory
