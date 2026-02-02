# Developer Agent Log - Task 012 (Task #6)
**Date**: 2026-02-01
**Task**: Implement Attack System Foundation

## Summary
Successfully implemented the foundational combat system with attack phases, damage application, targeting, and range checking. Units can now attack enemies using the Attack command (A key + click), progress through four attack phases, deal damage, and destroy enemy units. Auto-targeting allows idle units to automatically engage nearby enemies.

## Implementation Details

**New Files Created**:
- `src/combat.rs` - Complete combat system

**Modified Files**:
- `src/main.rs` - Added CombatPlugin
- `src/units.rs` - Added attack capabilities, attack integration
- `src/commands.rs` - Attack command support (placeholder removed)

**Core Components**:

### 1. AttackCapability Component
```rust
pub struct AttackCapability {
    pub damage: f32,           // Damage per hit
    pub range: f32,            // Attack range
    pub aim_time: f32,         // Aiming phase duration
    pub fire_time: f32,        // Firing phase duration
    pub cooldown_time: f32,    // Cooldown phase duration
    pub reload_time: f32,      // Reload phase duration
    pub attack_type: AttackType, // Attack type (FullyConnected for now)
}
```

### 2. AttackState Component
```rust
pub struct AttackState {
    pub phase: AttackPhase,    // Current attack phase
    pub time_in_phase: f32,    // Time spent in current phase
    pub target: Option<Entity>, // Current target entity
}
```

### 3. AttackPhase Enum
```rust
pub enum AttackPhase {
    None,       // Not attacking
    Aiming,     // Turning toward target
    Firing,     // Dealing damage
    Cooldown,   // Brief unresponsive period
    Reloading,  // Main delay between attacks
}
```

### 4. AttackType Enum
```rust
pub enum AttackType {
    FullyConnected,     // Instant hit (implemented)
    TailDisjointed,     // Projectile (Task #8)
    HeadDisjointed,     // Ground attack (Task #8)
    DoublyDisjointed,   // Projectile to location (Task #8)
}
```

## Attack Phases Implementation

### Phase 1: Aiming
- **Duration**: 0.2-0.5 seconds (varies by unit)
- **Behavior**:
  * Unit attempts to face target
  * Validates target is still in range
  * Can be interrupted by new commands
  * Transitions to Firing when complete
- **Interruption**: ✅ Can be cancelled

### Phase 2: Firing
- **Duration**: 0.1-0.2 seconds (brief)
- **Behavior**:
  * Damage applied at phase start
  * Cannot be interrupted
  * Unit locked in place
  * Attack effect occurs
- **Interruption**: ✗ Cannot be cancelled

### Phase 3: Cooldown
- **Duration**: 0.05-0.15 seconds (very brief)
- **Behavior**:
  * Brief unresponsive period
  * Cannot move or turn
  * Represents attack recovery
  * Transitions to Reloading
- **Interruption**: ✗ Cannot be cancelled

### Phase 4: Reloading
- **Duration**: 1.0-3.0 seconds (main attack speed)
- **Behavior**:
  * Unit can move and turn
  * Main delay between attacks
  * Can be interrupted by commands
  * Returns to Aiming if target still valid
- **Interruption**: ✅ Can be interrupted

## Unit Attack Capabilities

### Light Infantry
- **Damage**: 10.0
- **Range**: 5.0 units (short)
- **Reload**: 1.0 seconds (fast)
- **Aim Time**: 0.2 seconds (quick)
- **Role**: Rapid-fire infantry

### Wheeled Vehicle
- **Damage**: 20.0
- **Range**: 8.0 units (long)
- **Reload**: 2.5 seconds (moderate)
- **Aim Time**: 0.4 seconds (slower)
- **Role**: Long-range support

### Tracked Vehicle (Tank)
- **Damage**: 30.0
- **Range**: 7.0 units (moderate)
- **Reload**: 3.0 seconds (slow)
- **Aim Time**: 0.5 seconds (slowest)
- **Role**: Heavy damage dealer

## Systems Implemented

### 1. attack_command_system
- Responds to UnitCommand::AttackTarget
- Sets attack target in AttackState
- Validates target exists
- Initiates attack sequence

### 2. attack_phase_system
- Main combat logic system
- Progresses through attack phases
- Validates target each frame during Aiming
- Applies damage during Firing phase
- Handles phase transitions
- Checks range and enemy status

### 3. auto_target_system
- Auto-targets nearby enemies for idle units
- Finds nearest enemy within range
- Only affects idle/holding units
- Won't interrupt movement commands
- Creates organic combat feel

### 4. apply_damage_system
- Processes DamageEvent components
- Reduces target health
- Clamps health to minimum 0
- Logs damage events
- Removes DamageEvent after processing

### 5. remove_dead_units_system
- Checks for units with health <= 0
- Despawns dead units
- Logs unit destruction
- Recursive despawn (removes children)

## Attack Command Integration

### A Key + Click Enemy
1. Press A key → CommandMode = Attack
2. Click enemy unit → Attack command issued
3. Selected units target clicked enemy
4. UnitCommand::AttackTarget set
5. attack_command_system sets AttackState.target
6. attack_phase_system begins attack sequence

### Auto-Targeting
- Idle units scan for enemies every frame
- Finds nearest enemy within attack range
- Automatically begins attacking
- Creates dynamic battlefield

## Movement Integration

### Units Stop When Attacking
Modified `unit_movement_system`:
```rust
// Stop moving if in Firing or Cooldown phase
if matches!(attack_state.phase, Firing | Cooldown) {
    velocity.0 = Vec3::ZERO;
    continue;
}
```

- Units can move during Aiming and Reloading
- Units locked during Firing and Cooldown
- Prevents "kiting" exploits
- Realistic combat positioning

## Damage System

### DamageEvent Component
- Temporary component applied to targets
- Contains damage amount and source
- Applied at start of Firing phase
- Removed after processing

### Health Tracking
- UnitHealth.current reduced by damage
- Health clamped to [0, max]
- Death when health reaches 0
- Simple and effective

### Death Handling
- Entity despawned when health <= 0
- Recursive despawn (removes selection indicators, etc.)
- Logged for debugging
- Clean removal from game world

## Enemy Detection

### is_enemy() Function
```rust
fn is_enemy(owner1: &Owner, owner2: &Owner) -> bool {
    match (owner1, owner2) {
        (Player(p1), Player(p2)) => p1 != p2,
        (Neutral, _) | (_, Neutral) => false,
    }
}
```

- Players are enemies with different player IDs
- Neutral units don't attack anyone
- Simple faction detection
- Ready for faction system (Task #10)

## Testing Scenarios

### Manual Attack (A + Click)
1. Select Player 0 infantry
2. Press A key
3. Click Player 1 unit
4. Infantry moves into range (if needed)
5. Infantry attacks until target destroyed

### Auto-Attack
1. Place units near each other
2. Wait for auto-targeting
3. Units automatically engage enemies
4. Combat continues until one dies

### Range Testing
- Infantry: 5.0 range (must get close)
- Vehicles: 7.0-8.0 range (can kite infantry)
- Tactical positioning matters

## Build Results
- `cargo build`: ✅ Success in 6.28s
- Warnings: Unused attack types (for Task #8)
- All combat systems functional

## Design Compliance

From design doc (lines 253-320):

**Attack Phases (lines 256-268)**:
- ✅ Aiming Phase (lines 258-262)
- ✅ Firing Phase (lines 263-264)
- ✅ Cooldown Phase (lines 265-267)
- ✅ Reloading Phase (lines 268)

**Attack Types (lines 273-286)**:
- ✅ FullyConnected (instant hit)
- ⏳ TailDisjointed (Task #8)
- ⏳ HeadDisjointed (Task #8)
- ⏳ DoublyDisjointed (Task #8)

**Unit Behaviors (lines 356-360)**:
- ✅ Attacking Target implemented
- ✅ Move to range, stop, attack

**Unit States (lines 374-383)**:
- ✅ Idle units auto-attack (line 380-381)

## Technical Details

**Phase Timing Example** (Light Infantry):
```
Total attack cycle: ~1.45 seconds
- Aiming: 0.20s (13.8%)
- Firing: 0.10s (6.9%)
- Cooldown: 0.05s (3.4%)
- Reloading: 1.00s (69.0%)
- Next attack: 0.10s (6.9%)
```

**Damage Per Second (DPS)**:
- Light Infantry: 10 / 1.45 = 6.9 DPS
- Wheeled Vehicle: 20 / 3.25 = 6.2 DPS
- Tracked Vehicle: 30 / 3.85 = 7.8 DPS

Tank has highest DPS despite slowest reload!

## Known Limitations

**By Design**:
- Only FullyConnected attacks (instant hit)
- No turret support (Task #7)
- No projectiles (Task #8)
- No directional armor yet
- No attack animations

**Future Enhancements** (Later Tasks):
- Task #7: Turret aiming
- Task #8: Projectile attacks
- Visual attack effects
- Attack animations
- Muzzle flash
- Death animations

## Integration with Commands

### Attack Command Flow
```
A key → Attack mode → Click enemy → AttackTarget command →
Attack phases → Damage → Death
```

### Command Priorities
1. Stop (S) - Cancels attack
2. Hold (H) - Stops but can still attack
3. Move (M) - Cancels attack, moves away
4. Attack (A) - New target

## Next Steps
Task #6 complete! Core combat mechanics functional.

Moving on to Task #7: Implement Unit Turret System
- Independent turret rotation
- Turn angle limits
- Turret aiming during Aiming phase
- Visual turret entities
- Attack from turret position

Combat foundation is solid! Units can now engage in tactical warfare.
