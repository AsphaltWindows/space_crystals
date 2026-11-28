//! (M6-SB) Patches on the ground, and the tanks that lay them (plan decision T93): the Shadebringer's Sludge
//! (`factions/syndicate/units/shadebringer.md > Sludge (toggle)`), and beside it (slice M6-SC) its Cover's smoke.
//!
//! - **The tanks** ([`Tanks`], `Object::tanks`): "Sludge and Cover each have their **own, separate charge**", "**Each
//!   charge is a tank** that **drains while its toggle is on**" (`> Charges and recharging`). A unit whose type has
//!   tanks (`UnitDef::tanks`) has them from the start, full: **Placeholder (not in the design: a new Shadebringer's
//!   charges)**. In fixed point, [`RATIO_ONE`] to a unit of charge, so that they drain and fill exactly: a tank of 100
//!   holds 6 553 600, drains 40 960 a tick ("drain **10 per second** while on (10 s of use from full)", 160 ticks) and
//!   fills 20 480 a tick ("refill **5 per second** inside the network with a Vault", 320 ticks from empty;
//!   `network::services`).
//! - **The toggles** ([`switch`]; `Ability::Sludge`, and `Ability::Cover`, which slice M6-SC makes live: one rule for
//!   both). "A toggle can be switched on with **any charge** left in its tank" (design change 0369), and is refused
//!   with none (`Reject::TankEmpty`). "Toggles follow the mode-switch rule (keep orders, not queueable)": a press
//!   applies in the commands phase of its tick (`Sim::use_ability`) and leaves the unit's orders be. On, a tank drains
//!   every tick, whether its unit moves or not, and "an empty tank switches its toggle off" (design change 0425: "it
//!   never stays "on" with nothing left"). "Entering the Tunnel Network switches both toggles off: it comes out with
//!   both off" (design change 0425; [`Tanks::switch_off`], `Sim::take_off_map`).
//! - **The trail** ([`Patches`], `Sim::patches`; [`lay`], after phase 5d): "While on, the Shadebringer leaves a **trail
//!   of sludge** on the ground in its wake. Each patch of sludge lasts **X seconds** and then disappears." A chain of
//!   discs ([`Patch`]), each as wide as the trail (`TankDef::width`: "trail width 48 su"), laid at the point behind
//!   the unit's motion where the tick's move left it: its rear edge, or its front while it backs up ([`trail_point`]:
//!   D44's placeholder, which design change 0425 keeps; "the sludge is spread **behind** it, so laying the trail
//!   doesn't slow it"); a new disc each time that point has moved the patches' spacing from the last one
//!   (`tuning::patches::Patches::spacing`). Until then the last disc is laid again, its life starting over, as is the
//!   disc where it stands when switched on again: "a Shadebringer with a toggle on that isn't moving keeps laying its
//!   effect **where it stands** (and keeps draining)" (design change 0425), so that a parked one holds its spot until
//!   its tank is empty, and its last patch lasts its life after that. A patch lasts its tank's `TankDef::life` ("each patch lasts **X = 10 s**"): laid in a tick, it is
//!   on the ground at the end of that tick and of the 159 after it, and gone in the next one's step.
//! - **The slow** ([`slow`], in phase 5d before the units move): "**Infantry and non-hover vehicles** that step into
//!   sludge are **slowed by Y%** for **Z seconds**", Mechs and Drill Units on the surface too, "**Affects
//!   everyone**". A unit of a base sludge slows (`SlowDef::bases`), on the ground (a Drill Unit's base is underground
//!   until slice M6-ST1 surfaces it: "unaffected while underground"), whose centre is in a sludge patch where the last
//!   tick left it (design change 0425: "a unit is in sludge [...] when its **centre** is inside the trail"; on its
//!   edge too, as the Shrine's area: **Placeholder (not in the design)**), holds the sludge's slow
//!   (`BuffKind::Sludge`) for the slow's time after (`SlowDef::after`: "slowed by **Y = 40%** for **Z = 2 s** after
//!   leaving it, and the slow **refreshes while they stay in it**"). While it holds it, its top speeds are 0.6 of what
//!   they would be ([`speed_factor`], `Object::movement`), from the tick it stepped in to the 32nd after it left. The
//!   Shadebringer is no exception ("if it drives back over sludge (its own or another Shadebringer's), it is slowed
//!   like any other vehicle").
//! - **The store**: `Sim::patches`, history, hashed and saved only while there is a patch (Sim tag 4), so that the
//!   states without one are as they were before the Shadebringer. The patches are kept by the cell of the coarse grid
//!   their centres lie in (T93: "indexed in a coarse grid (2 GU cells) for lookups", [`CELL`]), so that a lookup at a
//!   point reads the cells round it alone.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::buffs::{self, BuffKind};
use crate::defs::{Domain, ObjectKind, PatchKind, SlowDef, TankDef};
use crate::economy::RATIO_ONE;
use crate::hash::{StateHash, StateHasher};
use crate::ids::ObjectId;
use crate::math::facing_vector;
use crate::object::Object;
use crate::ruleset::rules;
use crate::units::{TICKS_PER_SECOND, Tick, seconds};

/// The side of a cell of the patches' grid, space units: 2 grid units (plan decision T93). A technical number: it
/// changes how many patches a lookup reads, never what it finds.
pub const CELL: f32 = 128.0;

/// One of a unit's tanks (module docs): how much of it is used, and its toggle.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tank {
    /// How much of it is used, in fixed point ([`RATIO_ONE`] to a unit of charge): 0 full, its capacity's
    /// ([`capacity`]) empty.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub used: u32,
    /// Whether its toggle is on: it lays its patches and drains.
    #[serde(default, skip_serializing_if = "is_false")]
    pub on: bool,
    /// While on, the centre of the last patch it laid, the trail's newest ([`lay`]): the next is laid once the point
    /// behind the unit is the patches' spacing from it. `None` while off, and before the first patch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laid: Option<Vec2>,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl Tank {
    /// Whether it is full and off: as it starts, which its unit's hash and file leave out.
    pub fn is_default(&self) -> bool {
        *self == Tank::default()
    }

    /// How much of it is left, the share of its tank def `def`'s capacity, from 0 (empty) to 1 (full): what its bar
    /// shows.
    pub fn fraction(&self, def: &TankDef) -> f32 {
        let capacity = capacity(def);
        capacity.saturating_sub(self.used) as f32 / capacity as f32
    }

    /// Whether it is empty, of a tank def `def`.
    pub fn is_empty(&self, def: &TankDef) -> bool {
        self.used >= capacity(def)
    }

    /// Its toggle goes off, and its trail ends.
    fn off(&mut self) {
        self.on = false;
        self.laid = None;
    }
}

impl StateHash for Tank {
    fn hash(&self, h: &mut StateHasher) {
        let Tank { used, on, laid } = self;
        used.hash(h);
        on.hash(h);
        laid.hash(h);
    }
}

/// A unit's tanks (`Object::tanks`, module docs): one of each kind of patch, in [`PatchKind::ALL`]'s order, of which
/// its type's def has those it lays (`UnitDef::tanks`); the others stay full and off.
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tanks {
    /// Its Sludge's (R).
    #[serde(default, skip_serializing_if = "Tank::is_default")]
    pub sludge: Tank,
    /// (Slice M6-SC) Its Cover's (F).
    #[serde(default, skip_serializing_if = "Tank::is_default")]
    pub smoke: Tank,
}

impl Tanks {
    /// Its tank of `kind`.
    pub fn get(&self, kind: PatchKind) -> &Tank {
        match kind {
            PatchKind::Sludge => &self.sludge,
            PatchKind::Smoke => &self.smoke,
        }
    }

    /// Its tank of `kind`, to change.
    pub fn get_mut(&mut self, kind: PatchKind) -> &mut Tank {
        match kind {
            PatchKind::Sludge => &mut self.sludge,
            PatchKind::Smoke => &mut self.smoke,
        }
    }

    /// Whether a toggle of its is on.
    pub fn any_on(&self) -> bool {
        self.sludge.on || self.smoke.on
    }

    /// Whether a tank of its isn't full: a Vault refills it (`network::services`).
    pub fn any_used(&self) -> bool {
        self.sludge.used > 0 || self.smoke.used > 0
    }

    /// Both toggles go off (module docs): its unit enters the Tunnel Network (design change 0425).
    pub(crate) fn switch_off(&mut self) {
        self.sludge.off();
        self.smoke.off();
    }
}

impl StateHash for Tanks {
    fn hash(&self, h: &mut StateHasher) {
        let Tanks { sludge, smoke } = self;
        // Each tank only while it isn't full and off, under a sub-tag of its own: a unit whose Cover (slice M6-SC) is
        // never used hashes as before it.
        for (tag, tank) in [(0, sludge), (1, smoke)] {
            if !tank.is_default() {
                h.tag(tag);
                tank.hash(h);
            }
        }
    }
}

/// The tanks of a new object of `kind` (module docs): full and off for a type with tanks, `None` for every other.
pub(crate) fn fresh(kind: ObjectKind) -> Option<Tanks> {
    let unit = kind.def().unit.as_ref()?;
    (!unit.tanks.is_empty()).then(Tanks::default)
}

/// The tank def of `kind` of a unit of type `of`, if it lays `kind`'s patches.
pub fn tank_def(of: ObjectKind, kind: PatchKind) -> Option<&'static TankDef> {
    of.def().unit.as_ref()?.tank(kind)
}

/// The tank def that lays `kind`'s patches, in the current rules: the Shadebringer's, the one type with tanks, which
/// structure keeps so (`Ruleset::structural_diff`: `UnitDef::tanks`). A patch's slow is read from it ([`slow`]), as
/// the Null Chamber's service is from the one type with one (`network::services`). `None` for a kind no type lays:
/// smoke until slice M6-SC.
pub fn def_of(kind: PatchKind) -> Option<&'static TankDef> {
    tank_def(ObjectKind::Shadebringer, kind)
}

/// `amount` units of charge in fixed point ([`RATIO_ONE`] to the unit), to the nearest, and at least 1, so that a tank
/// always drains and fills (`Ruleset::check` bounds a capacity to what fits: `ruleset/check.rs`).
fn fixed(amount: f32) -> u32 {
    ((amount * RATIO_ONE as f32).round() as u32).max(1)
}

/// A tank's capacity, of tank def `def`, in fixed point: "**Tanks**: 100 each", 6 553 600.
pub fn capacity(def: &TankDef) -> u32 {
    fixed(def.capacity)
}

/// What a tank of `def` drains a tick while its toggle is on, in fixed point: "drain **10 per second**", 40 960.
pub fn drain(def: &TankDef) -> u32 {
    fixed(def.drain / TICKS_PER_SECOND as f32)
}

/// What a Vault refills a tank of `def` a tick, in fixed point: "refill **5 per second**", 20 480.
pub fn refill(def: &TankDef) -> u32 {
    fixed(def.refill / TICKS_PER_SECOND as f32)
}

/// How many ticks a patch of `def` lasts: "each patch lasts **X = 10 s**", 160, at least 1.
pub fn life(def: &TankDef) -> Tick {
    seconds(def.life).max(1)
}

/// How many ticks the slow `slow` holds a unit after the last tick its centre was in a patch: "for **Z = 2 s** after
/// leaving it", 32.
pub fn slow_after(slow: &SlowDef) -> Tick {
    seconds(slow.after)
}

/// How long the sludge's slow lasts, ticks, from the tick a unit is in a patch (`BuffKind::duration`): that tick and
/// the slow's time after it ([`slow_after`]), 33; 1 while no type lays sludge.
pub(crate) fn slow_duration() -> Tick {
    def_of(PatchKind::Sludge)
        .and_then(|def| def.slow)
        .map_or(0, |slow| slow_after(&slow))
        .saturating_add(1)
}

/// Whether sludge slows a unit of `kind` (module docs): what holds its slow (`BuffKind::affects`).
pub fn slows(kind: ObjectKind) -> bool {
    def_of(PatchKind::Sludge)
        .and_then(|def| def.slow.as_ref())
        .is_some_and(|slow| slowed_by(slow, kind))
}

/// Whether the slow `slow` slows a unit of `kind` (module docs): one of a base it slows (`SlowDef::bases`), whose base
/// is on the ground.
fn slowed_by(slow: &SlowDef, kind: ObjectKind) -> bool {
    let Some(unit) = kind.def().unit.as_ref() else {
        return false;
    };
    slow.bases.contains(&unit.base) && unit.base_def().domain == Domain::Ground
}

/// Whether `o` holds the sludge's slow.
pub fn slowed(o: &Object) -> bool {
    !o.buffs.is_empty() && (o.buffs.iter()).any(|buff| buff.ability == BuffKind::Sludge)
}

/// What `o`'s top speeds are multiplied by (module docs): 1 − the sludge's slow ("slowed by **Y = 40%**", 0.6) while
/// it holds it ([`slowed`]), else 1.
pub fn speed_factor(o: &Object) -> f32 {
    if !slowed(o) {
        return 1.0;
    }
    def_of(PatchKind::Sludge)
        .and_then(|def| def.slow)
        .map_or(1.0, |slow| 1.0 - slow.by)
}

/// The point `o` lays its trail at as the tick's move left it (module docs): behind its motion, its rear edge (half
/// its Silhouette's length behind its centre, along its facing), or its front while it backs up (its tick's move
/// against its facing, `Mover::velocity`). Standing, its rear edge.
pub fn trail_point(o: &Object) -> Vec2 {
    let half = o.def().silhouette().map_or(0.0, |silhouette| silhouette.x / 2.0);
    let ahead = facing_vector(o.facing);
    let backing = o.mover.as_ref().is_some_and(|mover| mover.velocity.dot(ahead) < 0.0);
    match backing {
        true => o.pos + ahead * half,
        false => o.pos - ahead * half,
    }
}

/// A patch on the ground (module docs): a disc of a kind's, laid by a unit, until a tick.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    /// What it is: sludge, or (slice M6-SC) smoke.
    pub kind: PatchKind,
    /// The unit that laid it, which it outlives.
    pub by: ObjectId,
    /// Its centre, space units.
    pub centre: Vec2,
    /// Its radius, space units: half the trail's width (`TankDef::width`) as it was laid.
    pub radius: f32,
    /// The tick whose step removes it ([`lay`]): it is on the ground at the end of every tick before it.
    pub until: Tick,
}

impl Patch {
    /// Whether `point` is in it: in the disc, or on its edge (module docs).
    pub fn covers(&self, point: Vec2) -> bool {
        point.distance_squared(self.centre) <= self.radius * self.radius
    }
}

impl StateHash for PatchKind {
    fn hash(&self, h: &mut StateHasher) {
        (self.index() as u32).hash(h);
    }
}

impl StateHash for Patch {
    fn hash(&self, h: &mut StateHasher) {
        let Patch {
            kind,
            by,
            centre,
            radius,
            until,
        } = self;
        kind.hash(h);
        by.hash(h);
        centre.hash(h);
        radius.hash(h);
        until.hash(h);
    }
}

/// The cell of the patches' grid that `point` lies in ([`CELL`]).
fn cell(point: Vec2) -> (i32, i32) {
    ((point.x / CELL).floor() as i32, (point.y / CELL).floor() as i32)
}

/// Every patch on the ground (`Sim::patches`, module docs), by the cell of its centre, each cell's in the order they
/// were laid. In files as one list, cell by cell.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<Patch>", into = "Vec<Patch>")]
pub struct Patches {
    cells: BTreeMap<(i32, i32), Vec<Patch>>,
    /// How many there are of each kind, in [`PatchKind::ALL`]'s order.
    counts: [u32; 2],
    /// The largest radius among them, space units: how far round a point a lookup reads.
    reach: f32,
}

impl Patches {
    /// No patch: what the hash and the files leave out.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// How many there are.
    pub fn len(&self) -> usize {
        self.counts.iter().map(|&count| count as usize).sum()
    }

    /// How many there are of `kind`.
    pub fn count(&self, kind: PatchKind) -> usize {
        self.counts[kind.index()] as usize
    }

    /// Every patch, cell by cell, each cell's in the order they were laid.
    pub fn iter(&self) -> impl Iterator<Item = &Patch> {
        self.cells.values().flatten()
    }

    /// Whether `point` is in a patch of `kind` ([`Patch::covers`]): one whose centre lies in the cells round it, as
    /// far as the largest radius reaches, or, should those be more than the cells with a patch (a trail tuned wider
    /// than the map), in any.
    pub fn covers(&self, kind: PatchKind, point: Vec2) -> bool {
        if self.counts[kind.index()] == 0 {
            return false;
        }
        let in_it = |patches: &Vec<Patch>| (patches.iter()).any(|patch| patch.kind == kind && patch.covers(point));
        let reach = Vec2::splat(self.reach);
        let ((x0, y0), (x1, y1)) = (cell(point - reach), cell(point + reach));
        let around = (i64::from(x1) - i64::from(x0) + 1) * (i64::from(y1) - i64::from(y0) + 1);
        if around > self.cells.len() as i64 {
            return self.cells.values().any(in_it);
        }
        (x0..=x1).any(|x| (y0..=y1).any(|y| self.cells.get(&(x, y)).is_some_and(in_it)))
    }

    /// `patch` is laid.
    fn add(&mut self, patch: Patch) {
        self.counts[patch.kind.index()] += 1;
        self.reach = self.reach.max(patch.radius);
        self.cells.entry(cell(patch.centre)).or_default().push(patch);
    }

    /// The patch of `kind` that `by` laid at `centre` lasts until `until` from now on, laid again where it stands
    /// (module docs). Whether there was one.
    fn renew(&mut self, by: ObjectId, kind: PatchKind, centre: Vec2, until: Tick) -> bool {
        let mut patches = self.cells.get_mut(&cell(centre)).into_iter().flatten();
        match patches.find(|p| p.by == by && p.kind == kind && p.centre == centre) {
            Some(patch) => {
                patch.until = until;
                true
            }
            None => false,
        }
    }

    /// The patches whose last tick on the ground was the one before `tick` go (module docs).
    fn expire(&mut self, tick: Tick) {
        let (mut counts, mut reach) = ([0; 2], 0.0_f32);
        self.cells.retain(|_, patches| {
            patches.retain(|patch| patch.until > tick);
            for patch in patches.iter() {
                counts[patch.kind.index()] += 1;
                reach = reach.max(patch.radius);
            }
            !patches.is_empty()
        });
        self.counts = counts;
        self.reach = reach;
    }
}

impl From<Vec<Patch>> for Patches {
    fn from(list: Vec<Patch>) -> Self {
        let mut patches = Patches::default();
        for patch in list {
            patches.add(patch);
        }
        patches
    }
}

impl From<Patches> for Vec<Patch> {
    fn from(patches: Patches) -> Self {
        patches.cells.into_values().flatten().collect()
    }
}

impl StateHash for Patches {
    fn hash(&self, h: &mut StateHasher) {
        h.len(self.len());
        for patch in self.iter() {
            patch.hash(h);
        }
    }
}

/// `o` switches its toggle of `kind` on or `off` (module docs; `Ability::Sludge`, `Ability::Cover`): on, if its tank
/// has any charge left (the gate has refused the press for an empty one); off, its trail ends. A unit without the tank
/// is left be.
pub(crate) fn switch(o: &mut Object, kind: PatchKind, on: bool) {
    let Some(def) = tank_def(o.kind, kind) else {
        return;
    };
    let Some(tanks) = &mut o.tanks else {
        return;
    };
    let tank = tanks.get_mut(kind);
    match on {
        true if !tank.is_empty(def) => tank.on = true,
        true => {}
        false => tank.off(),
    }
}

/// A Vault's tick for `o` (`network::services`): each of its tanks fills a tick's refill ([`refill`]), up to full.
pub(crate) fn refill_tanks(o: &mut Object) {
    let Some(tanks) = &mut o.tanks else {
        return;
    };
    for kind in PatchKind::ALL {
        if let Some(def) = tank_def(o.kind, kind) {
            let tank = tanks.get_mut(kind);
            tank.used = tank.used.min(capacity(def)).saturating_sub(refill(def));
        }
    }
}

/// The patches' step of tick `tick`, after phase 5d (module docs): those whose time is over go, then every object
/// before `first_new` with a toggle on, in id order, lays its trail where the tick's move left it, a new patch or its
/// last one again, and its tank drains, its toggle going off once it is empty.
pub(crate) fn lay(objects: &mut BTreeMap<ObjectId, Object>, patches: &mut Patches, first_new: ObjectId, tick: Tick) {
    if !patches.is_empty() {
        patches.expire(tick);
    }
    let spacing = rules().tuning.patches.spacing;
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        if !o.tanks.is_some_and(|tanks| tanks.any_on()) {
            continue;
        }
        let at = trail_point(o);
        let (id, kind_of) = (o.id, o.kind);
        let Some(tanks) = &mut o.tanks else {
            continue;
        };
        for kind in PatchKind::ALL {
            let tank = tanks.get_mut(kind);
            if !tank.on {
                continue;
            }
            let Some(def) = tank_def(kind_of, kind) else {
                tank.off();
                continue;
            };
            let until = tick.saturating_add(life(def));
            let last = match tank.laid {
                Some(last) if last.distance_squared(at) < spacing * spacing => Some(last),
                Some(_) => None,
                // Switched on again where it stood: its patch there, if it is still on the ground.
                None => Some(at),
            };
            if last.is_some_and(|last| patches.renew(id, kind, last, until)) {
                tank.laid = last;
            } else {
                patches.add(Patch {
                    kind,
                    by: id,
                    centre: at,
                    radius: def.width / 2.0,
                    until,
                });
                tank.laid = Some(at);
            }
            tank.used = tank.used.saturating_add(drain(def)).min(capacity(def));
            if tank.is_empty(def) {
                tank.off();
            }
        }
    }
}

/// The sludge's slow in tick `tick`, in phase 5d before the units move (module docs): every unit before `first_new`
/// that sludge slows ([`slows`]) whose centre is in a sludge patch, where the last tick left it, holds the slow from
/// this tick for the slow's time after it, one that holds it already from this tick again.
pub(crate) fn slow(objects: &mut BTreeMap<ObjectId, Object>, patches: &Patches, first_new: ObjectId, tick: Tick) {
    if patches.count(PatchKind::Sludge) == 0 {
        return;
    }
    let Some(slow) = def_of(PatchKind::Sludge).and_then(|def| def.slow) else {
        return;
    };
    let until = tick.saturating_add(1).saturating_add(slow_after(&slow));
    for o in objects.range_mut(..first_new).map(|(_, o)| o) {
        // The cheap tests first: most objects of a map, its crystals and its buildings, have no mover.
        if o.mover.is_none() || !slowed_by(&slow, o.kind) || !patches.covers(PatchKind::Sludge, o.pos) {
            continue;
        }
        buffs::hold(o, BuffKind::Sludge, until);
    }
}
