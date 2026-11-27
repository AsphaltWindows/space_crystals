//! (M6-CM) Mirage in a game (`crate::mirage`): the cast, at once in the commands phase ([`Sim::cast_mirage`]); the
//! illusions that go with their parents through the removal path ([`Sim::illusions_of`]); and the energy an illusion
//! shows, its parent's ([`Sim::energy_shown`]).

use glam::Vec2;

use super::Sim;
use crate::energy;
use crate::events::SimEvent;
use crate::ids::ObjectId;
use crate::math::{self, PI};
use crate::mirage;
use crate::object::Object;
use crate::orders::OrderKind;
use crate::ruleset::rules;
use crate::units::SU_PER_GU;

impl Sim {
    /// `caster` casts Mirage (`Sim::use_ability`; the gate took it: its owner's unit, of a type that casts Mirage, no
    /// illusion, with the energy it costs; `crate::mirage`, module docs): it pays the cost, its Shroud channel ends, its
    /// old illusions vanish, and its new ones come round it, each on its compass spot or the nearest free one
    /// (`mirage::compass`, [`Sim::mirage_spot`]), in the order of the compass, from the north: of its kind and its
    /// owner, with its HP, its Augmentations and its facing, idle, without energy, linked to it
    /// (`Object::illusion_of`), each with its `SimEvent::Spawned`, then a `SimEvent::Mirage`. They are the tick's new
    /// objects: they act from the next tick on. Nothing for a unit that can't pay.
    pub(super) fn cast_mirage(&mut self, caster: ObjectId) {
        let Some(o) = self.objects.get_mut(&caster) else {
            return;
        };
        let illusion = mirage::is_illusion(o);
        let Some(def) = mirage::def_of(o.kind).copied().filter(|_| !illusion) else {
            return;
        };
        let Some(pool) = &mut o.energy else {
            return;
        };
        if !energy::spend(pool, energy::units(def.cost)) {
            return;
        }
        // "Casting it cancels Shroud": the channel ends (its cloud lingers, `crate::shroud`), and what is queued behind
        // it goes on, from this tick's 5a.
        if let Some(orders) = &mut o.orders
            && (orders.current.as_ref()).is_some_and(|active| active.order.kind == OrderKind::Shroud)
        {
            orders.current = None;
        }
        let (kind, owner, pos, facing, hp, augmentation) = (o.kind, o.owner, o.pos, o.facing, o.hp, o.augmentation);
        // "the old set vanishes and a new set of 4 appears; a Disciple never has more than one set".
        let (old, held) = self.illusions_of(&[caster]);
        self.remove_objects(&old);
        self.remove_held(&held);
        let mut made = Vec::new();
        for spot in mirage::compass(&def, pos) {
            if self.next_id == u32::MAX {
                break;
            }
            let Some(at) = self.mirage_spot(kind, pos, spot, facing) else {
                continue;
            };
            let id = self.insert_object(kind, owner, at, None, None);
            if let Some(illusion) = self.objects.get_mut(&id) {
                illusion.facing = facing;
                illusion.prev_facing = facing;
                illusion.hp = hp;
                illusion.augmentation = augmentation;
                illusion.energy = None;
                illusion.illusion_of = Some(caster);
            }
            made.push(id);
        }
        self.events.push(SimEvent::Mirage {
            disciple: caster,
            illusions: made,
        });
    }

    /// Where an illusion whose compass spot is `spot`, of a Disciple of `kind` at `from`, goes (`crate::mirage`, module
    /// docs): the spot if it fits there, facing `facing`, else the first free point of the rings round it, from the
    /// nearest out, `Spells::mirage_search_step_su` apart out to `Spells::mirage_search_radius_gu`, each ring's points
    /// that far apart (at least 8), from the one straight on from the Disciple round both ways in turn, the positive
    /// one (clockwise on screen) first. `None` if none is free.
    fn mirage_spot(&self, kind: crate::defs::ObjectKind, from: Vec2, spot: Vec2, facing: f32) -> Option<Vec2> {
        let fits = |p: Vec2| self.map.contains_point(p) && self.check_unit_fits_facing(kind, p, facing, None).is_ok();
        if fits(spot) {
            return Some(spot);
        }
        let tuning = &rules().tuning.spells;
        let step = tuning.mirage_search_step_su;
        if step <= 0.0 {
            return None;
        }
        let rings = (tuning.mirage_search_radius_gu * SU_PER_GU / step) as u32;
        let ahead = math::vector_angle(spot - from);
        (1..=rings).find_map(|ring| {
            let radius = ring as f32 * step;
            let count = ((2.0 * PI * radius / step) as u32 + 1).max(8);
            (0..count).find_map(|i| {
                // 0, +1, −1, +2, −2, … of the ring's points from straight on.
                let turns = if i % 2 == 1 {
                    i.div_ceil(2) as f32
                } else {
                    -((i / 2) as f32)
                };
                let p = spot + math::facing_vector(ahead + turns * 2.0 * PI / count as f32) * radius;
                fits(p).then_some(p)
            })
        })
    }

    /// The illusions of the objects `parents` (`crate::mirage`): those on the map, then those held off it, each in id
    /// order, but for those among `parents`. None without a parent of a type that casts Mirage among them, which is
    /// all the removal path asks of most removals.
    pub(super) fn illusions_of(&self, parents: &[ObjectId]) -> (Vec<ObjectId>, Vec<ObjectId>) {
        let caster = |id: &ObjectId| {
            let o = (self.objects.get(id)).or_else(|| self.held.get(id).map(|held| &held.object));
            o.is_some_and(|o| mirage::def_of(o.kind).is_some() && !mirage::is_illusion(o))
        };
        if !parents.iter().any(caster) {
            return (Vec::new(), Vec::new());
        }
        let of = |o: &Object| o.illusion_of.is_some_and(|parent| parents.contains(&parent)) && !parents.contains(&o.id);
        let on_map = (self.objects.values()).filter(|o| of(o)).map(|o| o.id).collect();
        let held = (self.held.values())
            .filter(|held| of(&held.object))
            .map(|held| held.object.id)
            .collect();
        (on_map, held)
    }

    /// (Design change 0452) The energy `o` shows, and its pool's maximum, in points: an illusion's parent's, wherever
    /// the parent is (on the map or held), live; every other object's own (`mirage::shown_energy`). What the energy bar
    /// and the InfoPanel show, to every player who sees the object.
    pub fn energy_shown(&self, o: &Object) -> Option<(f32, f32)> {
        mirage::shown_energy(o, |id| {
            (self.objects.get(&id)).or_else(|| self.held.get(&id).map(|held| &held.object))
        })
    }
}
