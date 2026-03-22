# armory_interface_and_production

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Implement the Armory's command panel interface, training production system, and eject mechanic.

### StructureMenuState
Add `StructureMenuState::ArmoryMenu` variant. Wire it up in:
- `update_command_panel_state` (ObjectEnum::CultsArmory → ArmoryMenu)
- The `get_structure_command_grid` match (or equivalent grid function)

### Command Panel Grid (3x3)
Following the design doc ObjectInterfaceState[Armory]:
- Q (0,0): Train Soldier — CommandIssuingTransition. Begins training a Soldier. Available only if StoredRecruits > 0 AND player has sufficient Space Crystals.
- W (0,1): Train Gunner — CommandIssuingTransition. Begins training a Gunner. Available only if StoredRecruits > 0 AND player has sufficient Space Crystals.
- E (0,2): Eject All — CommandIssuingTransition. Immediately ejects all stored Recruits from exit side. Available only if StoredRecruits > 0.
- C (2,2): Set Rally Point — StateOnlyTransition → AwaitingTarget[SetRallyPoint]. Left-click ground/object sets rally point.

### Right-click Resolution (Armory selected)
- Right-click Ground → SetRallyPoint to that location
- Right-click Object → SetRallyPoint to that object
(Same pattern as Barracks/HQ rally point)

### CommandButtonAction variants
Add new action variants for Armory commands:
- ArmoryTrainSoldier, ArmoryTrainGunner, ArmoryEjectAll
Wire into `execute_command_action` with crystal deduction for training.

### Training Production Tick System
Create `armory_training_tick_system`:
- If ArmoryState has active training (training_queue is Some):
  - Increment training_progress each frame
  - When training_progress reaches the training frames for the unit type:
    - Spawn the trained unit (Soldier or Gunner — use stub ObjectEnum variants if they don't exist yet, with placeholder type data similar to CultsRecruit)
    - Spawn at exit side (C side) of the Armory
    - Issue rally command if rally point is set (follow existing pattern from barracks/HQ)
    - Clear training state
- Note: Soldier and Gunner ObjectEnum variants likely don't exist yet. Create stubs: ObjectEnum::CultsSoldier, ObjectEnum::CultsGunner with placeholder stats and spawn functions.

### Eject System
Create `armory_eject_system`:
- When EjectAll is triggered, eject stored Recruits one at a time from exit side (C side) in rapid succession
- Use a cooldown/tick pattern similar to tunnel ejection (EjectionQueue pattern)
- Ejected Recruits return as normal field units (restore Visibility, place at exit side position)

### Info Panel
In the info panel section of command_panel.rs, show:
- StoredRecruits count: "Recruits: X/10"
- Training progress if active: "Training: [UnitName] X%"

### Availability Gating
- Train buttons greyed out when StoredRecruits == 0 or insufficient crystals
- Eject All greyed out when StoredRecruits == 0
- Follow existing `is_command_available` patterns

### Tests
- Train Soldier deducts crystal cost and consumes a stored Recruit
- Train Gunner works similarly
- Eject All returns all Recruits to the field
- Buttons unavailable when StoredRecruits is empty
- Rally point integration works
