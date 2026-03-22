use bevy::prelude::*;
use crate::types::AppState;
use crate::simulation::types::DiagCategory;

pub mod types;
pub mod utils;
pub mod systems;
mod projectile;
mod turret;

/// Startup system to initialize the CombatAssetCache resource.
fn init_combat_asset_cache(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(types::CombatAssetCache::new(&mut meshes, &mut materials));
}

/// Plugin for combat systems
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), init_combat_asset_cache);
        app.add_systems(Update, (
            // Combat behavior systems run before attack_phase_system
            // so they can set AttackState.target before phase progression
            systems::attacking_object_behavior_system,
            systems::attacking_location_behavior_system,
            systems::attack_move_behavior_system,
            systems::hold_position_behavior_system,
            systems::patrol_scanning_system,
            systems::attack_command_system,
            systems::attack_phase_system,
            systems::turret_autonomous_scanning_system,
            systems::turret_engagement_system,
            systems::base_auto_target_system,
            systems::attack_channel_sync_system,
            systems::idle_leash_system,
            systems::apply_damage_system,
            systems::cults_unit_death_tracking_system.before(systems::remove_dead_entities_system),
            crate::game::world::faction::cults_construction_cancel_system.before(systems::remove_dead_entities_system),
            systems::remove_dead_entities_system,
        ).in_set(DiagCategory::Combat));
    }
}

/// Plugin for turret systems
pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            turret::turret_aiming_system,
            turret::turret_rotation_system,
            turret::update_turret_visual_system,
        ).in_set(DiagCategory::Turrets));
    }
}

/// Plugin for projectile systems
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            projectile::projectile_movement_system,
            projectile::projectile_impact_system,
            projectile::explosion_effect_system,
            projectile::attack_line_decay_system,
            projectile::target_highlight_decay_system,
        ).in_set(DiagCategory::Projectiles));
    }
}
