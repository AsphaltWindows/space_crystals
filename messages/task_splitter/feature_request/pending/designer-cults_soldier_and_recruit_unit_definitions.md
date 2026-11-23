# cults_soldier_and_recruit_unit_definitions

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added full unit definitions for two previously-undefined Cults units: the Soldier (trained at the Armory) and the Recruit (the Cults' basic worker unit). Both were on the "Cults unit definitions needed" loose end. Gunner is still pending (separate follow-up).

## artifacts/designer/design/cults_objects.md

**Armory "Available unit types" list**: Soldier's cost/time filled in — `SpaceCrystalsCost: 20`, `TrainingTime: 32 frames` (2 seconds at 16 FPS). Gunner's remains TBD.

**New "Soldier" section** (full unit definition, follows the same format as GDO's Peacekeeper / Syndicate's Guard):
- UnitBase: LightInfantry, same silhouette/movement profile as the GDO Peacekeeper (24x24 space units, TurnRate 180°/frame, infinite accel/decel, MaxSpeed 4 space units/frame, RuggedTerrainDefenseBonus 50%)
- MaxHP: 40, PointArmor: 0, FullArmor: 0, SightRange: 5 grid units, Groupable: true
- Attack: FullyConnected, TargetDomain **Universal** (hits both Ground and Air — unlike Peacekeeper/Guard which are Ground-only), SingleTarget, Damage 4, Range 4 grid units, MinRange 0, AimDuration 2 frames, FiringDuration 2 frames, CooldownDuration 1 frame, ReloadDuration 3 frames
- ObjectInterfaceState: BasicCombatUnitInterfaceState (standard, no special commands)

**New "Recruit" section** (full unit definition, follows the same format as Syndicate's Agent — a worker unit with build/gather/enter-building behaviors):
- UnitBase: LightInfantry, same silhouette as Peacekeeper/Soldier (24x24 space units), same movement profile as LightInfantry base
- MaxHP: 40, PointArmor: 0, FullArmor: 0, SightRange: 5 grid units, Groupable: false
- Self-defense attack: FullyConnected (Melee), TargetDomain Ground, SingleTarget, Damage 3, Range melee (adjacent contact), AimDuration 2 frames, FiringDuration 8 frames, CooldownDuration 1 frame, ReloadDuration 1 frame
- Gathering: CarryCapacity 4 Space Crystals/load, MiningDuration 8 frames, DropOffDuration 2 frames, drop-off at own Storage
- Building: constructs Cults buildings from outside the site and can assist in-progress construction (references the reworked Cults Building Mechanics from the previous feature_request `cults_recruit_outside_construction` — Recruits are not consumed)
- Armory Entry: Recruits can enter an Armory to join its StoredRecruits pool (up to InternalRecruitCapacity), making them available for training
- Full ObjectInterfaceState defined: **A: Construct** (opens BuildMenu, same flow as existing Build Command), **B: Assist Construction** (targeted, click an in-progress building). Unit commands: Move, Stop, Attack, Enter Armory, Gather, Drop Off Resources, Construct, Assist Construction. Right-click resolution order: Space Crystal patch → Gather; own Storage while carrying → Drop off; own Armory → Enter Armory; in-progress own building → Assist Construction; enemy → Attack; ground → Move. Multiple Recruits can be multi-selected and commanded together despite being ungroupable (mirrors the Agent's note on ungroupable multi-select).

## QA Instructions

**Soldier:**
1. Queue and complete a Train Soldier order at the Armory (20 Space Crystals, 32 frames / 2 seconds). Confirm the Soldier spawns with 40 MaxHP, 0/0 armor, and matches the Peacekeeper's silhouette size and movement speed.
2. Confirm the Soldier can attack both ground and air targets (TargetDomain Universal) — test against a ground unit and an air unit and verify both take damage (4 damage per hit, 4 grid unit range).

**Recruit:**
1. Confirm Recruits spawn from a Recruitment Center with 40 MaxHP, 0/0 armor, and are NOT groupable (no shared control group panel across multiple selected Recruits, but multi-select + right-click commands still works for all of them).
2. Order a Recruit to gather Space Crystals: confirm it carries up to 4 Space Crystals per load, mining takes 8 frames, drop-off at Storage takes 2 frames.
3. Order a Recruit to attack an enemy at point-blank range: confirm melee-only (must be adjacent), 3 damage per hit.
4. Right-click an own in-progress building with a Recruit: confirm it resolves to Assist Construction. Right-click an own Armory: confirm it resolves to Enter Armory and the Recruit joins the StoredRecruits pool. Right-click a Space Crystal patch: confirm it resolves to Gather. Right-click own Storage while carrying crystals: confirm it resolves to Drop Off.
5. Use panel button A (Construct) to open the build menu and place a new building — confirm it starts construction with the selected Recruit(s) working from outside, per the previously-implemented Recruit construction rework.
