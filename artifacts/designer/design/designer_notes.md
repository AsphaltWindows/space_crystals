# Designer Notes

## Session - 2026-02-17

### Decisions Made
- **Unit entity formalized**: Added Unit as a mobile Object Type under the formal entity hierarchy, after Structure Instance
- **Unit common attributes**: All units share Faction, Silhouette, MaxHP, PointArmor, FullArmor, UnitBase
- **Silhouette**: Units use a 2D mask for collision outline that rotates with the unit. Exact rotation handling (discrete angles vs real-time) deferred to technical implementation
- **Armor system**: Two armor values — PointArmor (checked at hit location for single-target attacks) and FullArmor (checked for the whole unit against AoE attacks)
- **Data-only entity definitions**: Behaviors (movement, attacking) are NOT included in entity type definitions. Instead, UnitBaseAttributes[UnitBase] and TurretAttributes[UnitBase] serve as data containers for movement and attack attributes, parameterized by the unit's base type
- **TurretAttributes is optional**: struct | None, since infantry bases don't have turrets
- **Drill Unit**: Confirmed as essentially a tracked vehicle variant that operates underground, not a fully separate base category
- **Mech**: Confirmed can traverse rugged terrain (along with infantry)

- **Five movement models defined**: MovementModelEnum with TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, and Glider
  - **TurnRateMovement**: Simple turn-in-place model, responsive direct movement. Used by Light Infantry, Heavy Infantry, Mech
  - **FixedTurnRadiusMovement**: Car-like, can't turn in place, fixed radius, can reverse. No drift/slippage. Used by Wheeled
  - **SpeedTurnRadiusMovement**: Tank treads, can rotate in place, wider turns at speed due to tread mechanics. Used by Tracked, Drill Unit
  - **DragMovement**: Omni-directional thrust with drag equilibrium for max speed. Air units use high drag to stop passively; ground hovers thrust in opposing direction. Used by Hover Vehicle, Hover Craft
  - **GliderMovement**: Must always move. Idle circling vs active max speed. Turn radius governed by centripetal acceleration (r = v²/a). Used by Glider
- **Hover Craft gets Non-Forward Acceleration**: For consistency with Hover Vehicle in the Drag model (can be set to zero for air units that rely on drag alone)
- **Heavy Infantry vs Light Infantry differences confirmed**: (1) Size: Very Small vs Small, (2) Light gets rugged terrain defense bonus, Heavy doesn't, (3) Light is crushable by Tracked/Mechs, Heavy is not
- **Tracked vs Glider turn physics are different**: Tracked uses mechanical tread-based speed-to-turn-radius ratio; Glider uses aerodynamic centripetal acceleration (v²/r). Kept as separate models
- **UnitBase boolean properties confirmed** for all 9 bases: Turret, Directional Armor, Rugged Terrain, Crushable, Can Turn In Place, Can Reverse

- **All 9 UnitBase values formalized**: Generic UnitBase type defined with boolean properties and MovementModelAttributes[MovementModel] parameterized struct. Each specific base (LightInfantry, HeavyInfantry, WheeledVehicle, TrackedVehicle, DrillUnit, HoverVehicle, Mech, HoverCraft, Glider) written with concrete property values and descriptions.
- **LightInfantry has unique RuggedTerrainDefenseBonus** property not shared by other bases

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (deferred)
- Unit sizing: how granular is the silhouette compared to structure grid snapping?
- DrillUnit above-ground mode: how exactly does the transition work? Is it stationary or tracked above ground?

### Suggested Next Topics
- Formalize Attack Types and Attack Sources from old content into formal style (attack phases, types, sources sections)
- Formalize Unit Instance (like Structure Instance)
- Formalize Unit Commands, Actions, Behaviors, State

## Session 2 - 2026-02-17

### Decisions Made
- **TurretAttributes formalized**: Struct with TurnAngle (full arc in degrees, max 360, centered on unit facing, split equally clockwise/counter-clockwise) and TurnRate (independent of unit base turn rate)
- **Turret angle guidance added to base descriptions**: Wheeled, Tracked, Drill, Hover Vehicle typically have wide to full angles; Mech, HoverCraft, Glider typically have very narrow to narrow angles. Values are per-unit, guidance is just typical ranges.
- **AttackAttributes added to Unit entity**: `AttackAttributes - struct | None` for units that can attack
- **Damage model — Single-target**: Damage taken = Attack Damage - PointArmor
- **Damage model — AoE**: Damage is uniform across AoE circle. Unit damage share = Attack Damage × (unit overlap area / AoE area). Effective armor = FullArmor × (unit overlap area / unit total area). Damage taken = damage share - effective armor. Inspired by StarCraft's Small/Medium/Large damage type system but continuous and physics-based rather than discrete categories.
- **Directional armor applies to AoE**: Direction determined by vector from AoE center to unit center. For single-target attacks, direction is from attacker to target.
- **Head/Tail Disjointed names swapped**: "Disjointed" refers to separation from origin or target. Head Disjointed = disjointed at the head (origin), projectile spawns and unit is free, projectile tracks target, can't miss. Tail Disjointed = disjointed at the tail (target), effect applies to locked location, unit stays locked during firing, can be dodged.
- **Doubly Disjointed updated**: Projectile spawns (unit free), travels to locked location, can be dodged.
- **Projectile speed**: Only applies to Head Disjointed (cosmetic/timing) and Doubly Disjointed (determines dodge window). Tail Disjointed dodge window is purely FiringDuration.
- **AttackAttributes fields formalized**: AttackType, TargetType (SingleTarget/AoE), AoERadius, Damage, Range, MinRange, ProjectileSpeed, AimDuration, FiringDuration, CooldownDuration, ReloadDuration

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed: is it purely cosmetic or does it have any gameplay impact beyond feel?

### Suggested Next Topics
- Formalize Attack Types, Attack Sources, and Attack Phases from old content into formal style (the definitions exist but need to match the new entity formatting)
- Formalize the damage model (single-target and AoE calculations) as a formal section
- Formalize Unit Instance (like Structure Instance)
- Formalize Unit Commands, Actions, Behaviors, State

## Session 3 - 2026-02-17

### Decisions Made
- **Object Instance**: New common parent for Structure Instance and Unit Instance. Holds Type (ObjectEnum), Location, Owner (PlayerNumber | None), HP (Destructible types only).
- **Structure Instance** now extends Object Instance, only adds Rotation (discrete 0/90/180/270).
- **Unit Instance** added, extends Object Instance, adds Rotation (continuous degrees).
- **Owner can be None**: Both structures and units can be neutral/unowned.
- **Destructible flag**: Already exists on Object Type, gates whether HP is relevant. Applies to both structures and units — there can be indestructible units.

- **Unit Command/State/Behavior/Action ontology formalized**:
  - **Command**: Player input layer. Translated directly from user input. Units maintain a command queue (shift-click queuing). Commands: Move, Attack, AttackGround, AttackMove, Patrol, HoldPosition, Stop.
  - **Base State**: Set by the currently executing command. Carries parameters (target location, target unit, etc.). Drives base behavior.
  - **Turret State**: NOT set by commands. Updated by the base behavior. Has a LockedTarget (specific target or None). When None, turret falls back to autonomous scanning.
  - **Base Behavior**: Algorithm driven by base state, responds to game circumstances, composed of base actions. Also responsible for updating turret state.
  - **Turret Behavior**: Algorithm driven by turret state, engages locked target or autonomously scans for best available target.
  - **Base Action**: Atomic engine-level operations (Moving, Turning, Decelerating, Idle).
  - **Turret Action**: Atomic engine-level operations running concurrently with base actions (TurretAiming, TurretFiring, TurretCooldown, TurretReloading, TurretIdle).

### Key Design Insights (things to remember for future sessions)
- **The user's design philosophy favors emergent behavior over hard-coded categories.** The damage model (continuous area-based AoE) over StarCraft's discrete size types is a pattern. Expect the user to prefer physics-based/continuous solutions over lookup tables.
- **The user values rewarding micro skill ceiling.** Directional armor applying to AoE (from AoE center) was chosen specifically to reward players who position artillery behind formations. When in doubt, lean toward "more player agency" over "simplify for the engine."
- **Turret independence is a key design pillar.** The turret persists its own state across base state changes. Move commands don't interrupt turret targeting. This came from a strong intuition that the turret should feel like a semi-autonomous subsystem. This independence is a major gameplay advantage of turret-bearing bases over infantry.
- **The ontology matters to this user.** They think carefully about the conceptual layers (command vs state vs behavior vs action) and want clean separation of concerns. Don't rush to concrete behaviors — establish the abstract model first.
- **Attack-related naming should reflect spatial meaning.** Head/Tail Disjointed was swapped because "head" means origin and "tail" means destination. The user thinks about naming in terms of what the words literally describe.
- **Infantry attacking needs separate attention.** For infantry (no turret), attack phases (aiming, firing, cooldown, reloading) are base actions that lock the unit. This hasn't been formalized yet — the current base actions (Moving, Turning, Decelerating, Idle) don't include attack phases for infantry.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed: is it purely cosmetic or does it have any gameplay impact beyond feel? (carried over)
- How do specific behaviors map to commands? e.g., what exactly does AttackMove base behavior do when it encounters an enemy — does it pursue to a certain distance? Does it give up after the target leaves range?
- Patrol behavior specifics: how does it interact with turret state when engaging enemies?
- What happens to turret state when a Stop or HoldPosition command is issued? Does it clear the locked target?

### Suggested Next Topics
- Define specific behavior algorithms for each command (Move, Attack, AttackMove, Patrol, HoldPosition)
- Define turret autonomous scanning behavior (target priority, range vs arc considerations)
- Begin formalizing old-format content for tiles, resources, factions
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 4 - 2026-02-17

### Decisions Made
- **State split into CommandState and BehaviorState**: CommandState is the "what" set by the command (objective, parameters). BehaviorState is the "how" — internal data the behavior needs to execute its algorithm (planned paths, progress tracking, cached results).
- **Both BaseCommandState and BaseBehaviorState parameterized by UnitBase**: Different bases need different internal data (e.g., Glider circling parameters vs Wheeled turn radius planning).
- **TurretCommandState and TurretBehaviorState are not parameterized**: Only one kind of turret so far, turret behavior is uniform regardless of base.
- **StructureState stubbed out**: Will be defined with structure capabilities, expected to follow a similar Command → State → Behavior → Action flow.

- **AttackTarget type defined**: UnitTarget (ObjectInstance) | LocationTarget (Coordinates). Constrained by AttackType: FullyConnected/HeadDisjointed require UnitTarget, TailDisjointed/DoublyDisjointed accept both.
- **CanMiss and CanTargetGround derived from AttackType**: Not removed but marked as derived — they're inherent to the type, not independent properties.

- **Action channel model**: Instead of a single base action, the unit base operates on three concurrent channels:
  - **LocomotionChannel**: Moving(path), Reversing(path), Stopping, Stationary
  - **OrientationChannel**: Turning(targetPosition), Maintaining
  - **BaseAttackChannel** (infantry only): Aiming(target), Firing(target), Cooldown, Reloading, None
- **Turret action channels** (turret units only):
  - **TurretOrientationChannel**: TurretTurning(targetPosition), TurretMaintaining
  - **TurretAttackChannel**: TurretAiming(target), TurretFiring(target), TurretCooldown, TurretReloading, TurretInactive
- **Valid channel combinations constrained by UnitBase and attack phase** (e.g., during BaseFiring, Locomotion must be Stationary, Orientation is locked).

- **Reverse is an explicit command**, not automatic behavior. Available to CanReverse bases (Wheeled, Tracked, Drill). Wheeled vehicles always approach forward for Move commands, loop around if needed.
- **Glider Move behavior**: Glider transitions to idle circling over target location since it can't stop.
- **Idle reserved for state, not action**: Renamed to Stationary (for base) and TurretInactive (for turret).
- **Turning and aiming parameterized by targetPosition**: The behavior sets the target position, the engine resolves the angle and applies turn rate each tick. Position rather than angle because targets move.

### Key Design Insights (continued)
- **The user thinks in terms of clean layered abstractions.** The CommandState/BehaviorState split came from wanting to separate "what the command asks for" from "how the behavior is accomplishing it." This mirrors the Command/State/Behavior/Action layering — each layer has a clear single responsibility.
- **Plan for extensibility even if not needed now.** TurretBehaviorState was added even though it may not be necessary, because "we never know what may be useful to cache." Same principle applied to parameterizing states by UnitBase.
- **Naming precision matters.** "Idle" was rejected for an action because it describes the absence of behavior (a state concept), not an atomic operation. "Stationary" and "TurretInactive" are more precise. Similarly, "Decelerating" was renamed to "Stopping" to avoid implying a missing "Accelerating" action.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed: is it purely cosmetic or does it have any gameplay impact beyond feel? (carried over)
- Structure capabilities and their command/state/behavior/action model

### Suggested Next Topics
- Define turret autonomous scanning behavior (target priority, range vs arc considerations)
- Begin formalizing old-format content for tiles, resources, factions
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 5 - 2026-02-17

### Decisions Made
- **DragMovement gets TurnRate**: Added TurnRate field to DragMovement struct, was previously missing.
- **SightRange added to Object Type**: Both structures and units have SightRange. Vision is provided to the owning player when the instance has an Owner. Also serves as acquisition range for AttackMove behavior.
- **AttackMoveLeashDistance is a global constant**: Rather than per-unit, keeps things simple and consistent.
- **Attack-move leash measured from path**: Perpendicular distance from the attack-move path, not from the acquisition point. Prevents lateral kiting of entire armies while allowing chase along the path direction.

- **Action channel model formalized**: Base operates on three concurrent channels (Locomotion, Orientation, BaseAttack). Turret operates on two concurrent channels (TurretOrientation, TurretAttack). All parameterized:
  - Turning(targetPosition), Aiming(AttackTarget), Firing(AttackTarget) — engine resolves positions to angles each tick
  - Moving(path), Reversing(path) — path comes from BehaviorState

- **LocomotionOrientationConstraints defined per MovementModel**: Each of the 5 movement models has explicit valid/invalid combinations with maxTurnRate constraints:
  - TurnRateMovement: all combinations valid, fixed TurnRate
  - FixedTurnRadiusMovement: can't turn while stationary/stopping, turn rate = currentSpeed / MinimumTurnRadius, reversing + turning is valid
  - SpeedTurnRadiusMovement: unconstrained when stationary, speed-dependent when moving
  - DragMovement: all combinations valid, fixed TurnRate
  - GliderMovement: only Moving states (can never stop)

- **Behaviors pre-compute action sequences**: Path planning outputs a sequence of (Locomotion, Orientation) channel state pairs stored in BehaviorState. Recomputation triggered when unit deviates from expected position on path.

- **All base behaviors defined**:
  - **MovingToLocation**: pathfind, execute plan, stop on arrival. Glider circles over target.
  - **MovingToObject**: same but target moves, recomputation optimization stubbed out.
  - **ReversingToLocation**: MovingToLocation with Reversing locomotion.
  - **AttackingObject**: naive approach — move toward target, engage when in range/arc. No special pathfinding. Glider does strafing runs.
  - **AttackingLocation**: same as AttackingObject but for ground locations.
  - **AttackMovingToLocation**: move along path, scan SightRange for enemies, engage, leash back to path if too far. Glider does strafing runs along the path.
  - **Patrolling**: cycles AttackMovingToLocation between origin and destination.
  - **HoldingPosition**: Stationary at all times. CanTurnInPlace units rotate and fight. Turret units continue autonomous scanning. Non-turning units can only engage what's in front of them.
  - **StoppingBehavior**: decelerate, clear turret lock, complete when stopped.

- **Commands mapped to BaseCommandState mutations**: Each command formally sets CommandType, TargetLocation, TargetObject. Commands only mutate BaseCommandState — TurretCommandState managed by behaviors.
- **Attack targets Destructible ObjectInstance**: not just units, includes structures.
- **Reverse command added**: available to CanReverse bases only.

### Key Design Insights (continued)
- **Prefer simple/naive approaches first.** AttackingObject uses naive "move toward target, shoot when in range" rather than complex engagement pathfinding. The user prefers getting something working and iterating over over-engineering upfront.
- **Actions and behaviors must have distinct names.** Move is a command, Moving is an action, MovingToLocation is a behavior. Each layer has its own naming convention to avoid confusion.
- **"Hold position" means "don't move" not "don't fight."** Any unit that can turn in place should rotate to engage enemies. Only locomotion is locked.
- **Gliders are a special case for almost every behavior.** They can never stop, so every behavior that normally ends with stopping needs a Glider exception (circling, strafing runs). Worth keeping in mind when defining new behaviors.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure capabilities and their command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization — how to avoid replanning every tick when target moves
- Turret autonomous scanning behavior — target priority rules (closest? most threatening? lowest HP?)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange — closest? First spotted?
- Fog of war formalization — SightRange is defined but the vision system itself isn't

### Suggested Next Topics
- Define turret autonomous scanning behavior (target priority, range vs arc considerations)
- Formalize fog of war / vision system
- Structure capabilities and their command/state/behavior/action model
- Begin formalizing old-format content for tiles, resources, factions
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 6 - 2026-02-18

### Decisions Made
- **TargetDomain added to AttackAttributes**: Ground | Air | Universal. Ground attacks hit Ground and surfaced Underground units, Air attacks hit Air units, Universal hits all.
- **AoE domain filtering**: AoE only affects units whose domain matches the attack's TargetDomain. Prevents ground explosions from hitting air units overhead.
- **ValidTarget concept defined**: Reusable filter for target selection — enemy must be (1) Destructible, (2) visible to attacker's owner, (3) domain-compatible with attacker's TargetDomain. Range and arc applied separately by context.
- **TurretAutonomousScanning formalized**: Priority order: (1) threatening units (enemy TargetDomain can hit this unit's domain), (2) least turret rotation, (3) closest distance. References ValidTarget.
- **AttackMovingToLocation and HoldingPosition** updated to scan for ValidTarget enemies.
- **Vision system formalized**: Three states per tile per player — Unexplored (fully black), Explored (terrain + last-known structures, no enemy units), Visible (real-time everything). Vision provided by owned units/structures via SightRange.
- **ElevationModifier**: Binary +1/-1 to sight range and attack range based on relative elevation. Any elevation difference triggers it. Air units exempt in both directions. Underground units use terrain elevation above them.
- **DrillUnit no longer invisible**: Changed from invisible to visible while underground. Invisibility removed as a mechanic for now.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure capabilities and their command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange — closest? First spotted?
- Detection mechanics — deferred, not needed now that DrillUnits are visible

### Suggested Next Topics
- Structure capabilities and their command/state/behavior/action model
- Begin formalizing old-format content for tiles, resources, factions
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 7 - 2026-02-21

### Decisions Made
- **Tiles formalized as preset system**: Tile defines the property schema (Buildable, Traversible, Rugged, Drillable, Recruitable). TilePreset is a named configuration with a Texture and specific property values. Five default presets defined (Plane, Rugged Terrain, Cliff, Mountain, Water). Custom presets can be created via map editor.
- **Elevation is per-placement, not per-preset**: TilePlacement holds Type (TilePresetEnum), Location, and Elevation (0-16). Presets don't include elevation.
- **Recruitable is independent per preset**: No default assumption — each preset explicitly sets Recruitable. Map designers can create non-recruitable versions of any terrain type.
- **Resource Object Types formalized**: SpaceCrystalsPatch (1x1, indestructible, unowned, depletes and disappears) and SupplyDeliveryStation (2x2, indestructible, unowned, delivery cycle with size/interval/current supplies).
- **All four factions formalized with resources and DisplayHuds**:
  - **GDO**: Space Crystals, Supplies, Power (flat capacity, proportional slowdown when negative), Unit Control (fixed 200, no infrastructure)
  - **Syndicate**: Space Crystals, Supplies, Tunnel Space (up to 200, provided by Tunnels based on level)
  - **Cults**: Space Crystals, Unit Control (uncapped, provided by Recruitment Centers proportional to Recruitable tiles)
  - **Colonists**: Space Crystals, Alloys (refined from SC), Essence (refined from SC), Conduits (refined from Alloys + Essence), Beacon Capacity (up to 200, provided by Beacons)
- **Colonist resource renamed**: "Extracts" → "Essence", "Ascension Credits" → "Conduits"

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure capabilities and their command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange (carried over)
- Fog of war formalization — SightRange defined but vision system details incomplete (carried over)

### Suggested Next Topics
- Formalize faction-specific building mechanics (Deployment Center, Tunnels, Recruitment Centers, Beacons)
- Formalize faction-specific economy structures (Extraction Facility/Plates, Supply Tower/Chopper, Agents, Recruits, Prospectors)
- Structure capabilities and their command/state/behavior/action model
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 8 - 2026-02-22

### Decisions Made
- **Control System formalized as a client-side layer upstream of Unit Commands**: ControlState holds Selection and ObjectInterfaceState, lives entirely outside the game simulation. Validated against game state each tick — dead objects removed from selection, invalid interface state resets.
- **ControlState has two parts**: Selection (which objects are selected) and ObjectInterfaceState[ObjectEnum] (the current command flow state, parameterized by object type).
- **Selection is an array of SelectionGroups**: Each SelectionGroup has an ObjectEnum type and an array of ObjectInstance references. ActiveGroup tracks which group's commands are displayed.
- **Selection constraints**: Enemy/unowned objects can only be selected alone (exactly one object, one group). Own objects can be mixed freely with no count limit.
- **Multi-building selection supported**: No restriction on selecting multiple buildings or mixing units and buildings.
- **Command panel has common and group-specific commands**: All commands shown are the active group's commands, but commands common to the entire selection are visually distinguished. Common commands go to all selected objects, group-specific commands go only to the active group.
- **GroupCycling transition**: Rotates ActiveGroup through the selection groups.
- **InterfaceTransition has two types**: StateOnlyTransition (modifies only ControlState) and CommandIssuingTransition (modifies ControlState and issues a Command to game objects).
- **Interface state is not synchronized in multiplayer**: Only Commands enter the game simulation. Interface state is client-side only, consistent with standard RTS lockstep architecture.
- **InterfaceDisplay is a function of ControlState + game state**: Not a state machine, just a view rendered each tick. Not formalized separately.
- **DefaultState and AwaitingTarget[CommandType] defined as generic interface state templates**: DefaultState shows commands and handles right-click. AwaitingTarget is parameterized by command type and carries a resolution function from CursorTarget to Command.
- **Right-click is the context-sensitive default command**: Right-click enemy → Attack, right-click ground → Move, right-click friendly/neutral object → Move to object.
- **AttackMove is not a separate command button**: It's the ground-click resolution of the Attack command. Player presses Attack, clicks ground → AttackMove. Clicks enemy → Attack.
- **CursorTarget defined**: Ground, EnemyObject, FriendlyObject, NeutralObject — with Location and optional Object reference.
- **BasicCombatUnitInterfaceState template defined**: Immediate commands (HoldPosition, Stop), target commands (Attack, Move, Patrol, AttackGround, Reverse), and full resolution tables for right-click and each AwaitingTarget state.
- **Unit Command description updated**: Now references CommandIssuingTransition as its source rather than "direct translation of player input."
- **Control System placed before Unit Commands in the document**: Topological ordering — control system is upstream of the command/behavior/action pipeline.

### Key Design Insights
- **The user thinks about the control system as a state machine with clean separation from game state.** The interface accumulates intent (StateOnly transitions) before committing it to the simulation (CommandIssuing transitions). This mirrors the Command/Behavior/Action layering philosophy.
- **Bevy integration was considered early.** ObjectInterfaceState[ObjectEnum] maps to a Rust enum with per-type variant structs. ControlState maps to a Bevy Resource. Commands map to Bevy events. The design was validated against ECS patterns before committing.
- **AttackMove as a resolution of Attack rather than a separate command simplifies the interface** while preserving the strategic option. The player just "attacks" and the target determines what happens.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure capabilities and their command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange (carried over)
- Worker unit ObjectInterfaceState — right-click resolution for resource gathering, building construction
- Structure ObjectInterfaceState — multi-step flows for Deployment Center, Tunnels, Recruitment Centers, Beacons

### Suggested Next Topics
- Define structure-specific ObjectInterfaceStates (Deployment Center multi-step build/place flow, Tunnel expand flow, Beacon warp-in flow)
- Define worker unit ObjectInterfaceStates (Agents, Recruits, Prospectors — with resource and construction interactions)
- Structure command/state/behavior/action model (the game-side counterpart to the interface states)
- Formalize faction-specific building and economy mechanics
- Clean up old bullet-point content that has been superseded by formal definitions

## Session 8 continued - 2026-02-22

### Decisions Made
- **Scale system formalized**: SimulationFrame (16 fps), GridUnit (structure placement, range, sight), SpaceUnit (unit silhouettes, movement). 64 space units per grid unit.
- **Two levels of size**: Structures use grid units (1x1, 2x2, etc.), units use space unit silhouettes for finer granularity.
- **Movement speed in space units per frame**, range/sight in grid units.

- **Groupable property added to Object Type**: Boolean that determines whether multiple selected instances of the same type are combined into one SelectionGroup (true) or each get their own group (false). Ungroupable objects always have exactly one instance in their SelectionGroup.
- **ObjectInterfaceState validates against active SelectionGroup's game state each tick**: Available transitions depend on instance state. Resets to default if current state becomes invalid.
- **Selection constraints updated**: Ungroupable objects always occupy their own SelectionGroup.

- **Six concrete GDO objects defined**:
  - **Peacekeeper**: Light Infantry, FullyConnected Ground SingleTarget attack, 50 HP, 24x24 silhouette, 4 su/frame speed, infinite accel/decel, 180°/frame turn rate, 50% rugged terrain defense bonus, 1 Unit Control. BasicCombatUnitInterfaceState.
  - **Power Plant**: 2x2 AAAA, 350 HP, 1/4 armor, sight 3, power +20, BuildRadiusExtension 1. No commands (info display only).
  - **Barracks**: 3x2 ABAC, 300 HP, 1/6 armor, sight 4, power -30, BuildRadiusExtension 2. Produces Peacekeepers (50 SC, 80 frames). Build queue (max 5), rally point, cancel with full refund. Units exit B side.
  - **Deployment Center**: 4x4 AAAA, 1000 HP, 1/16 armor, sight 6, power +20, BuildRadiusExtension 12. Ungroupable. Constructs Power Plant (150 SC, 160 frames) and Barracks (200 SC, 160 frames). Multi-step interface: DefaultState → BuildMenu → AwaitingPlacement. Full refund during construction, 75% (rounded down) when ready to place.

- **RallyPoint behavior**: Spawned units receive the default right-click command resolved against the rally target (Move to ground, Attack enemy, Move to friendly/neutral object). If rally target object no longer exists, rally resets to None. If rally is None, unit spawns with no command.
- **AttackMove is not a separate command button**: It's the ground-click resolution of the Attack awaiting-target state. Player presses Attack → clicks ground → AttackMove command.
- **Deployment Center Build menu is a sub-menu**: Player must select "Build" first to enter the build menu, then select a specific building. Cancel and place options also live inside the build menu. Available options depend on instance game state (idle/constructing/ready).
- **BuildRadiusExtension is per-building**: Each GDO building specifies how much it extends the build radius when placed.
- **Deployment Center generates power (+20)**: Provides starting power infrastructure.
- **Barracks consumes power (-30)**: Requires two Power Plants to support, creating macro pressure.

### Key Design Insights (continued)
- **Production costs and build times belong to the producer, not the produced unit.** The Peacekeeper doesn't know its own cost — the Barracks defines what it costs to produce a Peacekeeper. This keeps unit definitions clean and allows the same unit to potentially be produced by different structures at different costs.
- **Rally points use the same resolution as right-click.** This is elegant — rally is just "auto-right-click on spawn." Keeps behavior predictable for the player.
- **The Deployment Center's interface state is independent of its game state.** The player can navigate to the build menu whether the DC is idle, constructing, or has a building ready. The game state determines which options are available/greyed out, but doesn't force the interface into a particular state.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- Unit sizing: how granular is the silhouette compared to structure grid snapping? (partially resolved — 64 su per grid unit)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange (carried over)
- Worker unit ObjectInterfaceState (carried over)
- GDO Extraction Facility / Extraction Plate mechanics and interface
- GDO Supply Tower / Supply Chopper mechanics and interface
- What other units can the Barracks produce beyond Peacekeeper?
- Peacekeeper UnitControlCost confirmed as 1 — what about other units?

### Suggested Next Topics
- Define remaining GDO structures (Extraction Facility, Supply Tower, and 2 more unique buildings user mentioned)
- Define worker/economy unit interfaces
- Define other faction objects (Syndicate, Cults, Colonists)
- Structure command/state/behavior/action model
- Clean up old bullet-point content

## Session 8 continued (part 2) - 2026-02-22

### Decisions Made
- **GDOBuildArea formalized**: A single shared buildable area per GDO player. Seeded by Deployment Center's BuildRadiusExtension, grown by every placed building's BuildRadiusExtension. Placement rule: at least 1 grid cell of the new building must be within the current build area. After placement, the area expands outward from the placed building by its BuildRadiusExtension in all directions.
- **BuildRadiusExtension is the unified concept**: The Deployment Center's large extension (12) seeds the initial area. Each subsequent building grows it further. No separate "initial build radius" concept needed.
- **Extraction Facility uses the shared GDO build area**: Originally considered giving it a local-only plate radius, but simplified to using the shared GDO build area for placing Extraction Plates.
- **Extraction Facility formalized**: 3x3 AAAA, 500 HP, 1/9 armor, sight 3, power -15, BuildRadiusExtension 2. Ungroupable. Constructs Extraction Plates (75 SC, 96 frames). Same construct-then-place flow as Deployment Center but without a build sub-menu (only builds one thing). Same cancellation rules (full refund during construction, 75% rounded down when ready).
- **Extraction Plate formalized**: 1x1 AAAA, 85 HP, 2/2 armor, sight 0, BuildRadiusExtension 0 (explicitly does not extend build area). Mines 10 SC per 48 frames, residual rate of 1 SC per 48 frames when patch depleted. On destruction: patch uncovered if not depleted, patch removed if depleted. Info display only, no commands.
- **Extraction Plate placement constraint**: Must be placed on a Space Crystal Patch within the GDO build area that doesn't already have a plate. Since it's 1x1, the "at least 1 grid within" rule simplifies to just "within."

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- What value should AttackMoveLeashDistance be? (deferred to tuning)
- AttackMovingToLocation target selection when multiple enemies in SightRange (carried over)
- Worker unit ObjectInterfaceState (carried over)
- GDO Supply Tower / Supply Chopper mechanics and interface
- What other units can the Barracks produce beyond Peacekeeper?
- Does the Extraction Facility have power consumption? (confirmed: -15)

### Suggested Next Topics
- Define GDO Supply Tower and Supply Chopper
- Define additional GDO units and production buildings
- Define other faction objects (Syndicate, Cults, Colonists)
- Worker/economy unit interfaces
- Structure command/state/behavior/action model
- Clean up old bullet-point content

## Session 9 - 2026-02-23

### Decisions Made
- **Control Groups formalized**: 10 saved selections (indexed 0-9) added to Control System section. Four operations: Assign (replace group with current selection), Add (merge selection into group), Recall (replace selection with group), Recall and Center (recall + center camera on centroid). Entities can belong to multiple groups. Dead entities silently removed on recall.
- **BaseAutoTargeting formalized**: New section defining when units auto-acquire targets. Active during Idle and HoldPosition only. NOT active during Move, AttackTarget, AttackGround, or Stop. AttackMove/Patrol have their own scanning logic.
- **Idle auto-targeting includes chase with leash**: Idle units chase acquired targets but leash back to their IdleOrigin position. IdleLeashDistance = 4 grid units.
- **HoldPosition auto-targeting is stationary**: Unit never moves, only engages what's in range. Turret units use turret arc, infantry with CanTurnInPlace rotate to face, infantry without CanTurnInPlace can only engage what's in front.
- **AttackMoveLeashDistance set to 6 grid units**: Was previously deferred to tuning, now has a concrete value.
- **Move command does NOT trigger base auto-targeting**: Turret units' TurretAutonomousScanning still operates independently during Move (turrets shoot while base moves), but the base does not acquire targets. Infantry do not engage during Move at all.
- **AttackGround does NOT trigger auto-targeting**: Deliberate location-targeted attack, auto-acquiring would override player intent.
- **AwaitingPlacement visual feedback formalized** for both DeploymentCenter and ExtractionFacility:
  - Ghost preview follows cursor, snapped to grid, tinted green (valid) or red (invalid)
  - Build area overlay highlights all valid cells as semi-transparent ground overlay
  - R/Shift+R rotation during placement, side labels (A/B/C/D per SymmetryType) on ghost
  - Non-square buildings swap footprint on 90°/270° rotation

- **DragMovement model updated**: NonForwardAcceleration renamed to OmniDirectionalAcceleration. Total forward thrust = OmniDirectionalAcceleration + ForwardAcceleration (additive). MaxSpeed = (Omni + Forward) / DragRatio.
- **Supply Tower formalized**: 3x3 AAAA, 400 HP, 1/9 armor, sight 4, power -15, BRE 1, groupable. Built by DC for 200 SC / 240 frames. Requires Power Plant tech prerequisite. Comes with one free Supply Chopper on placement. Produces additional Supply Choppers (100 SC / 160 frames). Has attach mechanic and scheduled delivery system.
- **Supply Chopper formalized**: 60x60 HoverCraft, 150 HP, 1/1 armor, sight 5, unarmed (no attack, no turret). Movement: ForwardAccel 0.9, OmniAccel 0.1, DragRatio 0.1 (max speed 10 su/frame), TurnRate 10 deg/frame. Picks up all supplies automatically when landing on SDS.
- **Scheduled Delivery mechanic**: Attached chopper auto-departs timed to arrive at SDS when delivery lands. If distance too long or deliveries too frequent, departs immediately after drop-off. Multiple towers targeting same SDS: closest tower with ready chopper goes first, only one chopper in flight per SDS at a time.
- **Supply Chopper is first unarmed HoverCraft**: No turret, no attack — unique among air units.

### Items reviewed but NOT added to design
- **Grid hotkeys (Q/W/E/A/S/D/Z/X/C)**: Implementation detail for testing, intended to be remappable. Not a design concern.
- **Force-attack on own buildings**: Implemented but not discussed for design. May want to formalize later.
- **Map defaults (64x64, DC at 30,30, 1000 SC start)**: Temporary test scenario, not design.

### Open Questions
- How exactly does silhouette rotation work at non-90-degree angles? (carried over)
- DrillUnit above-ground mode transition details (carried over)
- Head Disjointed projectile speed gameplay impact? (carried over)
- Structure command/state/behavior/action model (carried over)
- MovingToObject recomputation optimization (carried over)
- Force-attack on friendly units/structures — should this be formalized? (new)
- Current implementation has auto-targeting active during Move — needs to be corrected to match new design

### Suggested Next Topics
- Define additional GDO combat units and production buildings
- Define other faction objects (Syndicate, Cults, Colonists)
- Worker/economy unit interfaces
- Structure command/state/behavior/action model
- Clean up old bullet-point content superseded by formal definitions
- Supply Chopper transport capability (carry infantry/vehicles) — deferred, add later

## Session 10 - 2026-03-06

### Decisions Made
- **Tunnel fully formalized**: ABCD symmetry (A=entrance, B=crystal drop-off, C=supply drop-off, D=back wall), HP 600/800/1000 by tier, 1/16 armor, sight 5, Tunnel Space 20/30/40 by tier
- **Tunnel is Ungroupable**: Multi-stage interface warrants individual selection
- **Tunnel cost formulas**: Construction = current Tunnels owned (Supplies). Upgrade T2 = 2 + 2x(T2+ owned). Upgrade T3 = 3 + 3x(T3 owned). Higher tiers count for lower-tier scaling. Construction time 480 frames
- **Drop-off bottleneck is per-side**: Crystal and supply deliveries can occur simultaneously
- **Player starts with 1 T1 Tunnel + 1 pre-built Headquarters**
- **Headquarters**: T1 expansion, produces Agents only (Guards removed — will come from separate expansion)
- **Agent formalized**: Heavy Infantry cyborg, 36x36, 75 HP, 1/1 armor, sight 5, speed 6 su/frame, Tunnel Space 2, Ungroupable
- **Agent melee attack**: FullyConnected Melee (new subtype), Ground, 6 damage, Aim 2/Firing 4/Cooldown 1/Reload 9
- **Agent gathering**: 50 SC per load (mine 48f, drop-off 48f at Side B), 1 Supply per trip (pick up 48f, drop-off 48f at Side C)
- **Agent production**: 100 SC, 160 frames from Headquarters
- **FullyConnected subtypes**: Ranged (benefits from ElevationModifier on range) and Melee (adjacent contact, exempt from elevation range modifiers)
- **Agent Tunnel construction flow**: Agent walks to location, embeds in partially-built Tunnel (untargetable), on completion Agent is in Tunnel Network, on destruction Agent survives and emerges
- **ConstructionHP Rule**: Opt-in rule — buildings start at 10% HP and gain linearly to 100% during construction. Referenced by Tunnel construction
- **Building placement flipping**: All factions can rotate AND flip (horizontal/vertical axis) structures during placement. System-wide change
- **Enter command**: New unit command — unit walks to Tunnel Side A and enters Tunnel Network. Right-click on own Tunnel with sufficient tier resolves to Enter, otherwise Move
- **Tunnel interface**: DefaultState has A=Upgrade, B=Expand (underground building placement), C=Eject (unit type grid with counts, click to eject one). Greyed-out types for tier-insufficient units. Eject queue spawns units every 8 frames min, actual throughput limited by speed/collision

### Key Design Insights
- **The user doesn't want smart-casting for groupable units building structures.** This drove the decision to make Agent ungroupable. Worker units that can build may need to be individually commanded.
- **Syndicate economy is bottlenecked by Tunnel drop-off capacity, not worker count.** Per-side drop-off limits (1 Agent at a time per side) mean scaling income requires more Tunnels near resources, not just more Agents. This creates natural expansion pressure.
- **Agent construction is a one-way trip.** The Agent embeds in the Tunnel during construction and ends up in the network. This is thematically fitting (going underground) and mechanically clean (no need to pathfind back).
- **Tunnel Network value scales quadratically** with number of nodes (fully connected network). Cost scaling in Supplies reflects this increasing strategic value.

### Open Questions
- Headquarters stats (size, HP, armor, cost, build time) — not yet defined
- Guard unit and its production building — removed from HQ, needs a new home
- Tunnel upgrade times — construction time defined (480f) but upgrade durations not discussed
- Other T1 underground expansions — what else goes underground at T1?
- Syndicate combat unit roster — no combat units defined yet beyond the Agent's melee
- Agent ObjectInterfaceState — needs formalization (build commands, gather commands, right-click resolution for resources)
- How does rally point work for Tunnel expansions? Units emerge from Tunnel Side A or stay in network — how does the player choose?
- What happens when a Tunnel is destroyed? Do underground expansions survive? Are units in the network affected?
- Cults & Colonists: No objects defined yet
- `to_be_converted.md`: Old content awaiting formal structuring

### Suggested Next Topics
- Guard unit + its T1 production expansion
- Headquarters stats (size, HP, armor, cost)
- Agent ObjectInterfaceState (build/gather command flow)
- Tunnel destruction consequences
- Other T1 underground expansions
- Syndicate defensive structures (Agent-built on surface)
- Begin Cults or Colonists objects
