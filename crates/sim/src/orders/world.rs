//! What the behaviours see of the game in phase 5a — a snapshot of every object, taken before any unit's behaviour
//! runs, so that every unit decides on the same world: the positions the last tick left and the vision its phase 5f
//! computed (`docs/movement.md > 4`). Also the scan for targets of `systems/controls/behaviors.md >
//! BaseAutoTargeting` and `systems/combat.md > ValidTarget`.

use std::collections::BTreeMap;

use glam::Vec2;

use crate::combat::{self, AttackState, domain_allows, object_domain, within_range};
use crate::defs::{AttackAttrs, Domain, ObjectKind, TargetDomain, UnitDef};
use crate::elevation::{self, Elevation};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::math;
use crate::mover::Mover;
use crate::object::{Object, ResourceState};
use crate::shape::Shape;
use crate::units::{SU_PER_GU, Tick, gu};
use crate::vision::Vision;

use super::OrderKind;
use super::state::{Behaviour, GatherPhase};
use crate::economy::Worker;

/// An object as the behaviours see it this tick.
#[derive(Copy, Clone, Debug)]
pub(crate) struct View {
    pub id: ObjectId,
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    /// Its centre.
    pub pos: Vec2,
    pub shape: Shape,
    pub domain: Domain,
    pub elevation: Elevation,
    /// `systems/combat.md > ValidTarget` 1.
    pub destructible: bool,
    /// Whether it is a unit: a Move to it follows it (`systems/controls/behaviors.md > MovingToObject > Completion`).
    pub is_unit: bool,
    /// Whether it is a Space Crystals patch (`core/map.md > SpaceCrystalsPatch`), which workers gather.
    pub is_patch: bool,
    /// The TargetDomain of its attack, if it has one: whom it threatens (`> TurretAutonomousScanning`, priority 1).
    threat: Option<TargetDomain>,
}

/// The side of a cell of the scan grid, tiles (technical: about a SightRange, so a scan looks at few cells).
const CELL_TILES: u32 = 4;

/// The unit whose behaviour runs: its own object, split into what the behaviours read and what they write.
pub(crate) struct Unit<'a> {
    pub id: ObjectId,
    pub owner: Option<PlayerId>,
    pub kind: ObjectKind,
    pub pos: Vec2,
    pub facing: f32,
    pub mover: &'a mut Mover,
    pub attack: Option<&'a mut AttackState>,
    /// Whether its mode lets it attack now: not while its Riot Shield is up (`riot_shield::allows_attack`).
    pub may_attack: bool,
    /// A worker's load, patch and drop-off (`economy::gathering`).
    pub worker: Option<&'a mut Worker>,
}

impl Unit<'_> {
    pub fn unit_def(&self) -> Option<&'static UnitDef> {
        self.kind.def().unit.as_ref()
    }

    /// Its attack, if it has one (`systems/combat.md > AttackAttributes`) and may use it now. A unit whose Riot
    /// Shield is up has none (`factions/gdo/units.md > Peacekeeper > Riot Shield`: it "cannot attack" and "does not
    /// auto-target enemies"): its behaviours pick no target and let go of the one they engaged.
    pub fn attrs(&self) -> Option<&'static AttackAttrs> {
        self.unit_def()?
            .attack
            .as_ref()
            .filter(|_| self.attack.is_some() && self.may_attack)
    }

    pub fn shape(&self) -> Shape {
        Shape::silhouette(self.kind, self.pos, self.facing)
    }

    /// `combat::may_move`: whether its attack sequence leaves its base free to move this tick.
    pub fn may_move(&self) -> bool {
        combat::base_is_free(self.kind, self.attack.as_deref())
    }

    /// Its behaviour engages `target`, or nothing (`AttackState::engage`).
    pub fn engage(&mut self, target: Option<ObjectId>) {
        if let Some(attack) = &mut self.attack {
            attack.engage(target);
        }
    }

    /// The mover stops if it has a goal; a unit standing anyway is left alone (it may be stepping aside for
    /// another, `docs/movement.md > 6.6`).
    pub fn halt(&mut self) {
        if self.mover.goal != crate::mover::MoveGoal::None {
            self.mover.stop();
        }
    }
}

/// Which enemies a scan takes.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scan {
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`, `> AttackMovingToLocation` 2:
    /// ValidTargets within the unit's SightRange.
    Sight,
    /// `> HoldingPosition` 3: ValidTargets within weapon range (Range, beyond MinRange).
    Weapon,
}

/// The snapshot (module docs).
pub(crate) struct World<'a> {
    pub map: &'a Map,
    pub vision: &'a Vision,
    pub tick: Tick,
    /// `Sim::auto_targeting`: off, no scan picks anything.
    auto_targeting: bool,
    /// Every object, in id order.
    views: Vec<View>,
    /// The scan grid: for each cell, row by row, where its candidates start in `candidates`; one more entry ends
    /// the last cell.
    cell_starts: Vec<u32>,
    /// Indices into `views` of the objects a scan can pick (destructible, with an owner), by cell, in id order
    /// within each.
    candidates: Vec<u32>,
    /// The largest half extent of a candidate's shape, space units: a structure's footprint can be much larger
    /// than a unit.
    candidate_half: f32,
    cells_wide: u32,
    cells_high: u32,
    /// Indices into `views` of the Space Crystals patches and of the drop-offs, in id order: what a worker's scans
    /// look at (`orders::gather`). They run on events only: an arrival at an occupied patch, a return without a
    /// drop-off.
    patches: Vec<u32>,
    drop_offs: Vec<u32>,
    /// `factions/colonists/units.md > Drone > Patch Occupancy (one Drone per patch)`: the patches a worker was
    /// mining as the phase began, sorted.
    mined: Vec<ObjectId>,
    /// The patches under a mining structure (a GDO plate: question Q0037 point 11), sorted.
    covered: Vec<ObjectId>,
    /// The workers waiting at an occupied patch, as `(patch, since, worker)`, sorted: a patch's first entry is next
    /// in line (question Q0037 point 6).
    waiters: Vec<(ObjectId, Tick, ObjectId)>,
}

impl<'a> World<'a> {
    pub(crate) fn new(
        map: &'a Map,
        vision: &'a Vision,
        objects: &BTreeMap<ObjectId, Object>,
        tick: Tick,
        auto_targeting: bool,
    ) -> Self {
        let views: Vec<View> = objects
            .values()
            .map(|o| View {
                id: o.id,
                kind: o.kind,
                owner: o.owner,
                pos: o.pos,
                shape: Shape::of(o),
                domain: object_domain(o),
                elevation: Elevation::of_object(map, o),
                destructible: o.hp.is_some(),
                is_unit: o.mover.is_some(),
                is_patch: matches!(o.resource, Some(ResourceState::CrystalPatch { .. })),
                threat: o
                    .def()
                    .unit
                    .as_ref()
                    .and_then(|unit| unit.attack.as_ref())
                    .filter(|_| o.attack.is_some())
                    .map(|attrs| attrs.target_domain),
            })
            .collect();
        let cells_wide = map.width().div_ceil(CELL_TILES).max(1);
        let cells_high = map.height().div_ceil(CELL_TILES).max(1);
        let mut world = Self {
            map,
            vision,
            tick,
            auto_targeting,
            views,
            cell_starts: vec![0; (cells_wide * cells_high) as usize + 1],
            candidates: Vec::new(),
            candidate_half: 0.0,
            cells_wide,
            cells_high,
            patches: Vec::new(),
            drop_offs: Vec::new(),
            mined: Vec::new(),
            covered: Vec::new(),
            waiters: Vec::new(),
        };
        for (index, o) in objects.values().enumerate() {
            if world.views[index].is_patch {
                world.patches.push(index as u32);
            }
            // An Egress Node being warped in takes nothing until it is complete (placeholder: question Q0039 point
            // 8, `crate::transform`).
            if o.def().structure.as_ref().is_some_and(|structure| structure.drop_off) && !o.is_transforming() {
                world.drop_offs.push(index as u32);
            }
            world.covered.extend(o.extractor.map(|extractor| extractor.patch));
            let job = o
                .worker
                .as_ref()
                .and(o.orders.as_ref())
                .and_then(|orders| orders.current.as_ref());
            if let Some(active) = job
                && let (OrderKind::Gather(patch), Behaviour::Gather(gather)) = (active.order.kind, &active.state)
            {
                match gather.phase {
                    GatherPhase::Mining { .. } => world.mined.push(patch),
                    GatherPhase::Waiting { since } => world.waiters.push((patch, since, o.id)),
                    GatherPhase::ToPatch
                    | GatherPhase::ToDropOff
                    | GatherPhase::DroppingOff { .. }
                    | GatherPhase::Delivered => {}
                }
            }
        }
        world.mined.sort_unstable();
        world.covered.sort_unstable();
        world.waiters.sort_unstable();
        // A counting sort by cell keeps the id order within each cell.
        let is_candidate = |v: &View| v.destructible && v.owner.is_some();
        for v in world.views.iter().filter(|v| is_candidate(v)) {
            let cell = world.cell_of(v.pos);
            world.cell_starts[cell + 1] += 1;
            let (lo, hi) = v.shape.bounds();
            world.candidate_half = world.candidate_half.max(((hi - lo) / 2.0).max_element());
        }
        for i in 1..world.cell_starts.len() {
            world.cell_starts[i] += world.cell_starts[i - 1];
        }
        let mut next = world.cell_starts.clone();
        world.candidates = vec![0; world.cell_starts.last().copied().unwrap_or(0) as usize];
        for (index, v) in world.views.iter().enumerate().filter(|(_, v)| is_candidate(v)) {
            let cell = world.cell_of(v.pos);
            world.candidates[next[cell] as usize] = index as u32;
            next[cell] += 1;
        }
        world
    }

    fn cell_coord(&self, v: f32, cells: u32) -> u32 {
        let cell = (v / (CELL_TILES as f32 * SU_PER_GU)).floor();
        (cell.max(0.0) as u32).min(cells - 1)
    }

    fn cell_of(&self, p: Vec2) -> usize {
        (self.cell_coord(p.y, self.cells_high) * self.cells_wide + self.cell_coord(p.x, self.cells_wide)) as usize
    }

    /// The object `id`, if it exists.
    pub(crate) fn get(&self, id: ObjectId) -> Option<&View> {
        let i = self.views.binary_search_by_key(&id, |v| v.id).ok()?;
        self.views.get(i)
    }

    /// `systems/combat.md > ValidTarget` 2: whether `owner` sees `view` now: a tile its shape covers is Visible,
    /// by the vision the last tick left. A unit without an owner sees nothing.
    pub(crate) fn sees(&self, owner: Option<PlayerId>, view: &View) -> bool {
        let (lo, hi) = view.shape.bounds();
        owner
            .and_then(|owner| self.vision.player(owner))
            .is_some_and(|sight| sight.sees_box(lo, hi))
    }

    /// Whether `owner` knows of `view`, a structure or a map resource: it sees it now, or remembers it (its
    /// vision's record, `core/vision.md > Explored`). Nobody's units know nothing.
    pub(crate) fn knows(&self, owner: Option<PlayerId>, view: &View) -> bool {
        let (lo, hi) = view.shape.bounds();
        owner
            .and_then(|owner| self.vision.player(owner))
            .is_some_and(|sight| sight.sees_box(lo, hi) || sight.record(view.id).is_some())
    }

    /// The objects that stand where they are, structures and map resources, in id order.
    pub(crate) fn fixed(&self) -> impl Iterator<Item = &View> + '_ {
        self.views.iter().filter(|view| !view.is_unit)
    }

    /// The Space Crystals patches, in id order.
    pub(crate) fn patches(&self) -> impl Iterator<Item = &View> + '_ {
        self.patches.iter().map(|&index| &self.views[index as usize])
    }

    /// The structures that take workers' loads, in id order.
    pub(crate) fn drop_offs(&self) -> impl Iterator<Item = &View> + '_ {
        self.drop_offs.iter().map(|&index| &self.views[index as usize])
    }

    /// `> Patch Occupancy (one Drone per patch)`: whether a worker, of any player, was mining `patch` as the
    /// phase began.
    pub(crate) fn patch_mined(&self, patch: ObjectId) -> bool {
        self.mined.binary_search(&patch).is_ok()
    }

    /// Whether a mining structure stands on `patch` (question Q0037 point 11).
    pub(crate) fn patch_covered(&self, patch: ObjectId) -> bool {
        self.covered.binary_search(&patch).is_ok()
    }

    /// The worker next in line at `patch`: the one that has waited longest, the lowest id among equals (question
    /// Q0037 point 6).
    pub(crate) fn first_waiter(&self, patch: ObjectId) -> Option<ObjectId> {
        let first = self.waiters.partition_point(|&(p, ..)| p < patch);
        self.waiters
            .get(first)
            .filter(|&&(p, ..)| p == patch)
            .map(|&(.., id)| id)
    }

    pub(crate) fn elevation(&self, unit: &Unit) -> Elevation {
        Elevation::of(self.map, unit.kind, unit.pos)
    }

    /// `systems/combat.md > ValidTarget`, for a target an auto-targeting behaviour may pick or keep: `view` is an
    /// **enemy** of `unit` (it belongs to another player; nobody's objects and the unit's own side are never
    /// picked, question Q0014 point 1), destructible, seen by the unit's owner, and in a domain the unit's attack
    /// can target.
    pub(crate) fn valid_target(&self, unit: &Unit, attrs: &AttackAttrs, view: &View) -> bool {
        view.owner.is_some()
            && unit.owner.is_some()
            && view.owner != unit.owner
            && view.destructible
            && domain_allows(attrs.target_domain, view.domain)
            && self.sees(unit.owner, view)
    }

    /// Whether `view` is within the reach of `scan` from `unit`, the gap between their shapes
    /// (`systems/combat.md > Range Measurement`).
    /// - [`Scan::Sight`]: within the unit's SightRange, with the ElevationModifier (`core/vision.md >
    ///   ElevationModifier`). The design doesn't say how "within SightRange" is measured for a scan; a placeholder
    ///   reading: edge to edge, like Range.
    /// - [`Scan::Weapon`]: within Range and beyond MinRange (`combat::within_range`).
    pub(crate) fn within(&self, unit: &Unit, attrs: &AttackAttrs, scan: Scan, view: &View) -> bool {
        let gap = unit.shape().gap(&view.shape);
        Self::gap_within(unit.kind, attrs, scan, gap, self.elevation(unit), view)
    }

    /// [`World::within`] for a unit of `kind` on `source`, `gap` away from `view`.
    fn gap_within(kind: ObjectKind, attrs: &AttackAttrs, scan: Scan, gap: f32, source: Elevation, view: &View) -> bool {
        match scan {
            Scan::Sight => gap <= gu(elevation::sight_range(kind.def().sight_range, source, view.elevation)),
            Scan::Weapon => within_range(attrs, gap, source, view.elevation),
        }
    }

    /// The angle `unit` would turn to face `view`, radians, 0 to π.
    fn rotation(unit: &Unit, view: &View) -> f32 {
        let to = view.pos - unit.pos;
        if to == Vec2::ZERO {
            return 0.0;
        }
        math::wrap_angle(math::vector_angle(to) - unit.facing).abs()
    }

    /// `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority` (`> TurretAutonomousScanning`):
    /// the best ValidTarget within the reach of `scan`, if any, leaving out `skip`:
    /// 1. **threatening** targets first, those whose own attack can target this unit's domain;
    /// 2. then the **least rotation** from the unit's facing (the turret's, once turrets have state);
    /// 3. then the **closest** (the gap between the shapes), and the lowest id.
    ///
    /// Enemy structures are ValidTargets too (`systems/combat.md > ValidTarget`, design change 0103), so units
    /// attack enemy buildings on their own; those that can't attack (all of them, so far) are non-threatening, and
    /// come after the targets that can shoot back.
    ///
    /// Nothing while auto-targeting is off (`DebugCommand::SetAutoTargeting`).
    ///
    /// `facing_only`: a non-turret unit that can't turn in place engages only what it already faces
    /// (`> HoldingPosition` 4, within `combat::FACING_TOLERANCE`).
    pub(crate) fn pick(&self, unit: &Unit, scan: Scan, skip: Option<ObjectId>, facing_only: bool) -> Option<ObjectId> {
        let attrs = unit.attrs().filter(|_| self.auto_targeting)?;
        unit.owner?;
        let my_domain = unit.unit_def()?.base_def().domain;
        // Everything a scan can reach lies within the unit's longest range, plus the modifier, of the unit's shape:
        // its centre lies within that and the two half shapes along each axis, in the cells this box touches.
        let longest = unit.kind.def().sight_range.max(attrs.range) + elevation::ELEVATION_MODIFIER;
        let own_half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO).max_element();
        let radius = gu(longest) + own_half + self.candidate_half;
        let (x0, x1) = (
            self.cell_coord(unit.pos.x - radius, self.cells_wide),
            self.cell_coord(unit.pos.x + radius, self.cells_wide),
        );
        let (y0, y1) = (
            self.cell_coord(unit.pos.y - radius, self.cells_high),
            self.cell_coord(unit.pos.y + radius, self.cells_high),
        );
        let (shape, source) = (unit.shape(), self.elevation(unit));
        let mut best: Option<(bool, f32, f32, ObjectId)> = None;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.cells_wide + x) as usize;
                let range = self.cell_starts[cell] as usize..self.cell_starts[cell + 1] as usize;
                for &index in &self.candidates[range] {
                    let view = &self.views[index as usize];
                    // The cheap tests first: most candidates are the unit's own side.
                    if view.owner == unit.owner || Some(view.id) == skip {
                        continue;
                    }
                    let gap = shape.gap(&view.shape);
                    if !Self::gap_within(unit.kind, attrs, scan, gap, source, view)
                        || !self.valid_target(unit, attrs, view)
                    {
                        continue;
                    }
                    let rotation = Self::rotation(unit, view);
                    if facing_only && rotation > combat::FACING_TOLERANCE {
                        continue;
                    }
                    let harmless = !view.threat.is_some_and(|threat| domain_allows(threat, my_domain));
                    let key = (harmless, rotation, gap, view.id);
                    let better = best.is_none_or(|b| {
                        key.0
                            .cmp(&b.0)
                            .then(key.1.total_cmp(&b.1))
                            .then(key.2.total_cmp(&b.2))
                            .then(key.3.cmp(&b.3))
                            .is_lt()
                    });
                    if better {
                        best = Some(key);
                    }
                }
            }
        }
        best.map(|(.., id)| id)
    }
}
