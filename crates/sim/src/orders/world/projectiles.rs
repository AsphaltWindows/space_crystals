//! (M4-O) What the behaviours read of the projectiles in flight (`crate::combat::projectiles`): whether a unit's shot
//! at a location is still on its way, for an Attack Ground that completes at its blast (`orders::attack_ground`).

use glam::Vec2;

use crate::combat::Projectile;
use crate::ids::ObjectId;

use super::World;

impl<'a> World<'a> {
    /// The snapshot with the projectiles in flight as the last tick left them (`Sim::projectiles`). Without this,
    /// none is.
    pub(crate) fn with_projectiles(mut self, projectiles: &'a [Projectile]) -> Self {
        self.projectiles = projectiles;
        self
    }

    /// Whether a projectile `shooter` launched at the location `at`, aimed at no object, is still in flight: it lands
    /// in phase 5e of a tick to come.
    pub(crate) fn in_flight(&self, shooter: ObjectId, at: Vec2) -> bool {
        self.projectiles
            .iter()
            .any(|p| p.shooter == shooter && p.to == at && p.target.is_none())
    }
}
