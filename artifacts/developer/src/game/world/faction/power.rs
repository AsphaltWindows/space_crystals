use bevy::prelude::*;
use crate::types::{Owner, ObjectEnum};
use crate::game::types::*;

// =====================================================
// POWER GRID SYSTEM
// =====================================================

/// System to compute power grid for GDO players each tick
pub fn compute_power_grid(
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    buildings: Query<(&Owner, &PowerValue, &ObjectInstance)>,
) {
    for (player, mut resources) in players.iter_mut() {
        let mut generated: i32 = 0;
        let mut consumed: i32 = 0;
        let mut found_power_plant = false;

        for (owner, power, obj) in buildings.iter() {
            if owner.player_number() == Some(player.player_number) {
                if power.0 > 0 {
                    generated += power.0;
                } else if power.0 < 0 {
                    consumed += power.0.abs();
                }
                if obj.object_type == ObjectEnum::PowerPlant {
                    found_power_plant = true;
                }
            }
        }

        resources.power_generated = generated;
        resources.power_consumed = consumed;
        resources.has_power_plant = found_power_plant;
    }
}

// =====================================================
// HELPER FUNCTIONS
// =====================================================

/// Get power ratio for a given owner from the player resources (works with both mutable and immutable queries)
pub(super) fn get_power_ratio_for_owner(
    owner: &Owner,
    players: &Query<(&Player, &mut GdoPlayerResources)>,
) -> f32 {
    for (player, res) in players.iter() {
        if Some(player.player_number) == owner.player_number() {
            return res.power_ratio();
        }
    }
    1.0 // Default if no matching player found
}
