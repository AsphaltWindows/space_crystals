//! The pop-up turrets in a game (`crate::popup`, plan decision T50, slice M4-S2;
//! `factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`, `factions/syndicate/structures/laser_turret.md
//! > Pop-up rules`, `factions/syndicate/structures/laser_turret.md > Conversion`): the step that takes each turret
//! through its cycle, and the form as the game asks it.
//!
//! [`Sim::step_popups`] runs right after phase 5e and its removals:
//! - **after 5e**, where a turret picks its target (`combat::structures`), so that a turret starts to rise in the tick
//!   it finds one, and after the tick's deaths, so that a Tunnel lost in the tick converts its turrets in it;
//! - **after 5d**, so that the footprint-clear test (design change 0296 point 3: "it can't rise while any unit stands
//!   over it") reads where the units stand as the tick ends: no unit moves again before the footprint blocks;
//! - **before 5f**, so that a turret is seen from the tick it starts to rise, and hidden from the tick it is down.
//!
//! **Navigation** follows as for any structure that appears or goes (`paths::sync`: the true grids here, each
//! player's known ones at the end of the tick): a turret that starts to rise blocks its footprint from that tick, one
//! fully lowered opens it. A path planned across a lowered turret's tile may meet it raised: the stuck and re-route
//! rules apply as for any new obstacle (`docs/movement.md > 6.5`).

use glam::Vec2;

use super::Sim;
use crate::defs::Domain;
use crate::ids::{ObjectId, PlayerId};
use crate::nav::TileRect;
use crate::network;
use crate::object::Object;
use crate::paths::{self, box_overlaps_rect};
use crate::popup;
use crate::tunnel;

/// The box of `o` if it is a unit that stands on the ground: what keeps a turret under it from rising (design change
/// 0296 point 3; air units fly over it, as they block no placement: design change 0094).
fn ground_box(o: &Object) -> Option<(Vec2, Vec2)> {
    let def = o.def();
    let unit = def.unit.as_ref()?;
    (unit.base_def().domain == Domain::Ground).then(|| Some((o.pos, def.half_extents()?)))?
}

impl Sim {
    /// `factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`: whether `o` is a pop-up turret by its form
    /// now (`popup::in_area`): a turret in operation inside the current Tunnel Area of one of its owner's Tunnels in
    /// operation. Derived, never stored (plan decision T50).
    pub fn is_popup(&self, o: &Object) -> bool {
        o.is_operational() && popup::in_area(o, self.objects.values().filter(|t| t.tunnel.is_some()))
    }

    /// The pop-up turrets' step (module docs): each turret in operation from before this tick (`first_new`), in id
    /// order, goes one tick through its cycle (`popup::step`), by its form now, the target phase 5e left it and the
    /// units over its footprint. The true grids follow if a footprint started or stopped blocking.
    pub(super) fn step_popups(&mut self, first_new: ObjectId) {
        let turret = |o: &Object| o.placement.is_some() && popup::def_of(o.kind).is_some() && o.is_operational();
        let turrets: Vec<ObjectId> = (self.objects.range(..first_new))
            .filter(|(_, o)| turret(o))
            .map(|(&id, _)| id)
            .collect();
        if turrets.is_empty() {
            return;
        }
        // The Tunnel Areas that make a pop-up: those of the Tunnels in operation, each at its current tier.
        let areas: Vec<(PlayerId, TileRect)> = (self.objects.values())
            .filter(|t| network::is_exit(t))
            .filter_map(|t| Some((t.owner?, tunnel::area::of(t)?)))
            .collect();
        // The ground units' boxes, listed once, for the first turret that asks.
        let mut units: Option<Vec<(Vec2, Vec2)>> = None;
        let mut changed = false;
        for id in turrets {
            let Some(o) = self.objects.get(&id) else {
                continue;
            };
            let (Some(def), Some(p)) = (popup::def_of(o.kind), o.placement) else {
                continue;
            };
            let footprint = TileRect::new(p.origin, p.w, p.h);
            let target = o.attack.as_ref().is_some_and(|attack| attack.target.is_some());
            let clear = || {
                let units = units.get_or_insert_with(|| self.objects.values().filter_map(ground_box).collect());
                !(units.iter()).any(|&(pos, half)| box_overlaps_rect(pos, half, footprint))
            };
            let after = popup::step(o.popup, def, popup::in_areas(o, &areas), target, clear);
            if after == o.popup {
                continue;
            }
            let blocked = popup::blocks(o);
            if let Some(o) = self.objects.get_mut(&id) {
                o.popup = after;
                changed |= popup::blocks(o) != blocked;
            }
        }
        if changed {
            paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
        }
    }
}
