# cults_gunner_unit_definition

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added the full unit definition for the Cults Gunner (trained at the Armory), completing the "Cults unit definitions needed" loose end alongside the previously-sent Soldier/Recruit definitions.

## artifacts/designer/design/cults_objects.md

**Armory "Available unit types" list**: Gunner's cost/time filled in — `SpaceCrystalsCost: 40`, `TrainingTime: 32 frames` (same training time as Soldier, 2 seconds at 16 FPS).

**New "Gunner" section** (full unit definition, same format as Soldier/Peacekeeper/Guard):
- Shares the Soldier's full physical profile: UnitBase LightInfantry, Silhouette 24x24 space units, MaxHP 40, PointArmor 0, FullArmor 0, SightRange 5 grid units, Groupable true, same LightInfantry movement attributes (TurnRate 180°/frame, infinite accel/decel, MaxSpeed 4 space units/frame, RuggedTerrainDefenseBonus 50%).
- Differs entirely in attack profile: FullyConnected, TargetDomain **Ground only** (unlike Soldier's Universal targeting — Gunner cannot hit air units), SingleTarget, Damage 16, Range 5 grid units, MinRange 2 (cannot fire at point-blank range, unlike Soldier), AimDuration 4 frames, FiringDuration 2 frames, CooldownDuration 8 frames, ReloadDuration 8 frames — a slower, harder-hitting, longer-range attack compared to the Soldier's faster/weaker/dual-domain attack.
- ObjectInterfaceState: BasicCombatUnitInterfaceState (standard, no special commands).

This completes all three previously-missing Cults units (Soldier, Gunner, Recruit) referenced in the Armory and RecruitmentCenter sections.

## QA Instructions

1. Queue and complete a Train Gunner order at the Armory (40 Space Crystals, 32 frames / 2 seconds — same training time as Soldier). Confirm the Gunner spawns with the same silhouette size, MaxHP (40), armor (0/0), and movement speed as the Soldier.
2. Confirm the Gunner can only attack ground targets — attempt to target an air unit and verify it's rejected/unavailable (unlike the Soldier, which can hit both).
3. Confirm the Gunner cannot fire at targets closer than 2 grid units (MinRange 2) — a target that walks adjacent to the Gunner should cause it to stop firing until the target is 2+ grid units away again.
4. Confirm the Gunner's attack cycle timing: AimDuration 4 frames, FiringDuration 2 frames, CooldownDuration 8 frames, ReloadDuration 8 frames (22 frames total per shot, ~1.375 seconds at 16 FPS), dealing 16 damage per hit at up to 5 grid units range.
5. In a side-by-side comparison with the Soldier, confirm the Gunner deals more damage per hit (16 vs 4) but attacks slower and cannot target air units — verify this differentiation is visible in actual combat behavior, not just stats.
