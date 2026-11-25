//! What the behaviours see of the game in phase 5a — a snapshot of every object, taken before any unit's behaviour
//! runs, so that every unit decides on the same world: the positions the last tick left and the vision its phase 5f
//! computed (`docs/movement.md > 4`). Also the scan for targets of `systems/controls/behaviors.md >
//! BaseAutoTargeting` and `systems/combat.md > ValidTarget`, and what the workers' scans read ([`gathering`]).

mod gathering;
mod planning;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::combat::{self, AttackState, domain_allows, inside_min_range, object_domain, within_range};
use crate::defs::{AttackAttrs, Domain, ObjectKind, TargetDomain, UnitDef};
use crate::economy::Worker;
use crate::elevation::{self, Elevation};
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::math;
use crate::mover::Mover;
use crate::network::Paths;
use crate::object::{Object, ResourceState};
use crate::paths::NavState;
use crate::paths::knowledge::PathKnowledge;
use crate::shape::Shape;
use crate::targets::{self, Live, TargetView};
use crate::units::{SU_PER_GU, Tick, gu};
use crate::vision::{Hiding, Vision};

use gathering::Gathering;
pub(crate) use gathering::KnownPatch;

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
    /// Whom it is hidden from (`vision::hidden`, plan decision T32), which [`World::sees`] asks.
    hiding: Hiding,
}

/// What a unit's owner knows of the object its order names, this tick (`crate::targets::TargetView`, design change
/// 0139): the object as it is while the owner sees it, where its record has it while the owner only remembers it.
#[derive(Copy, Clone, Debug)]
pub(crate) enum Known<'a> {
    Seen(&'a View),
    /// Out of sight, remembered: the order goes to `position`, the last-known one (the centre of its footprint).
    Remembered {
        position: Vec2,
    },
    /// Neither: a unit out of sight, or an object the owner saw gone or never saw.
    Gone,
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
    /// The weapon in hand (`combat::weapon`): its form's, which a Legionnaire whose conversion waits still fights
    /// with (design change 0198).
    pub weapon: Option<&'static AttackAttrs>,
    /// Whether its mode lets it attack now: not while its Riot Shield is up (`riot_shield::allows_attack`).
    pub may_attack: bool,
    /// Whether it converts its weapon (`conversion::is_converting`): it holds still meanwhile.
    pub converting: bool,
    /// A worker's load, patch and drop-off (`economy::gathering`).
    pub worker: Option<&'a mut Worker>,
}

impl Unit<'_> {
    pub fn unit_def(&self) -> Option<&'static UnitDef> {
        self.kind.def().unit.as_ref()
    }

    /// Its attack, if it has one (`systems/combat.md > AttackAttributes`) and may use it now: the weapon in hand
    /// ([`Unit::weapon`]). A unit whose Riot Shield is up has none (`factions/gdo/units.md > Peacekeeper > Riot
    /// Shield`: it "cannot attack" and "does not auto-target enemies"): its behaviours pick no target and let go of
    /// the one they engaged.
    pub fn attrs(&self) -> Option<&'static AttackAttrs> {
        self.weapon.filter(|_| self.attack.is_some() && self.may_attack)
    }

    pub fn shape(&self) -> Shape {
        Shape::silhouette(self.kind, self.pos, self.facing)
    }

    /// `combat::may_move`: whether its attack sequence leaves its base free to move this tick, and it isn't
    /// converting its weapon.
    pub fn may_move(&self) -> bool {
        !self.converting && combat::base_is_free(self.kind, self.attack.as_deref())
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

/// Which enemies a scan takes. Never one inside the unit's MinRange (`> BaseAutoTargeting > Targets inside
/// MinRange`, design change 0182: auto-targeting "skips enemies inside the weapon's MinRange and picks one it can
/// hit").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scan {
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`, `> AttackMovingToLocation` 2:
    /// ValidTargets within the unit's SightRange, beyond its MinRange.
    Sight,
    /// `> HoldingPosition` 3: ValidTargets within weapon range (Range, beyond MinRange).
    Weapon,
}

/// A ValidTarget within the reach of a scan, as the scan's own filter sees it ([`World::pick`]).
pub(crate) struct Candidate<'v> {
    pub view: &'v View,
    /// The unit's attack reaches it from where the unit stands: within Range, beyond MinRange
    /// (`combat::within_range`).
    pub in_range: bool,
}

/// Design change 0179 (`systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 2):
/// the rotation a target needs is compared in steps of 45° (a placeholder), radians.
pub(crate) const ROTATION_STEP: f32 = math::PI / 4.0;

/// The 45° step ([`ROTATION_STEP`]) of a rotation of `rotation` radians, 0 to π (either direction): 0 for 0° to 45°,
/// 1 for over 45° to 90°, 2 for over 90° to 135°, 3 for over 135° to 180°. Targets in one step count as needing
/// equal rotation. **The bounds**: the design's steps ("0–45°, 45–90°, 90–135°, 135–180°") share their bounds; a
/// placeholder reading takes each step up to its upper bound, so a target exactly 45° off is in the first step, with
/// those dead ahead.
pub(crate) fn rotation_step(rotation: f32) -> u8 {
    let bounds = [ROTATION_STEP, 2.0 * ROTATION_STEP, 3.0 * ROTATION_STEP];
    bounds.iter().filter(|&&bound| rotation > bound).count() as u8
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
    /// The grids each player plans on, as the last tick left them: whether a worker can reach a patch
    /// ([`gathering`]).
    nav: &'a NavState,
    /// The patches, the drop-offs and who mines and waits where ([`gathering`]).
    gathering: Gathering,
    /// What each player knows of the obstacles, for the versions ([`planning`]).
    knowledge: Option<&'a PathKnowledge>,
    nav_version: u32,
    /// Side A's paths of every Tunnel, in id order (`crate::network::Paths`): where an Enter order walks to.
    tunnels: Vec<(ObjectId, Paths)>,
}

impl<'a> World<'a> {
    pub(crate) fn new(
        map: &'a Map,
        vision: &'a Vision,
        nav: &'a NavState,
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
                // The weapon in hand (`combat::weapon`: a Legionnaire's form's, design change 0198).
                threat: combat::weapon(o)
                    .filter(|_| o.attack.is_some())
                    .map(|attrs| attrs.target_domain),
                hiding: Hiding::of(o),
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
            nav,
            gathering: Gathering::new(objects),
            knowledge: None,
            nav_version: 0,
            tunnels: (objects.values())
                .filter_map(|o| Some((o.id, Paths::of_object(o)?)))
                .collect(),
        };
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

    /// Side A's paths of Tunnel `id` (`crate::network::Paths`); `None` for anything else.
    pub(crate) fn tunnel_paths(&self, id: ObjectId) -> Option<Paths> {
        let i = self.tunnels.binary_search_by_key(&id, |&(tunnel, _)| tunnel).ok()?;
        self.tunnels.get(i).map(|&(_, paths)| paths)
    }

    /// The object `id`, if it exists.
    pub(crate) fn get(&self, id: ObjectId) -> Option<&View> {
        let i = self.views.binary_search_by_key(&id, |v| v.id).ok()?;
        self.views.get(i)
    }

    /// `systems/combat.md > ValidTarget` 2: whether `owner` sees `view` now, by the vision the last tick left
    /// (`PlayerVision::object_box_state`: it isn't hidden from the owner, plan decision T32, and a tile its shape
    /// covers is Visible, the fog delay included, or it is an attacker revealed to the owner). A unit without an owner
    /// sees nothing.
    pub(crate) fn sees(&self, owner: Option<PlayerId>, view: &View) -> bool {
        let (lo, hi) = view.shape.bounds();
        owner
            .and_then(|owner| self.vision.player(owner))
            .is_some_and(|sight| sight.sees_object_box(view.id, lo, hi, view.hiding))
    }

    /// What `owner` knows of object `id` ([`Known`]), by the vision the last tick left: the one rule of
    /// `targets::view`, the gate's and the default command's, so that a behaviour never reads the truth about an
    /// object its owner doesn't see (design change 0139). Nobody's units know nothing.
    pub(crate) fn known(&self, owner: Option<PlayerId>, id: ObjectId) -> Known<'_> {
        let Some(player) = owner else {
            return Known::Gone;
        };
        let view = self.get(id);
        let live = view.map(|v| Live {
            owner: v.owner,
            seen: self.sees(owner, v),
        });
        match (targets::view(self.vision.player(player), player, id, live), view) {
            (TargetView::Visible, Some(view)) => Known::Seen(view),
            (TargetView::Remembered { position, .. }, _) => Known::Remembered { position },
            (TargetView::Visible | TargetView::Gone, _) => Known::Gone,
        }
    }

    /// The objects that stand where they are, structures and map resources, in id order.
    pub(crate) fn fixed(&self) -> impl Iterator<Item = &View> + '_ {
        self.views.iter().filter(|view| !view.is_unit)
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
    ///   ElevationModifier`), and beyond its MinRange (design change 0182). The design doesn't say how "within
    ///   SightRange" is measured for a scan; a placeholder reading: edge to edge, like Range.
    /// - [`Scan::Weapon`]: within Range and beyond MinRange (`combat::within_range`).
    pub(crate) fn within(&self, unit: &Unit, attrs: &AttackAttrs, scan: Scan, view: &View) -> bool {
        let gap = unit.shape().gap(&view.shape);
        Self::gap_within(unit.kind, attrs, scan, gap, self.elevation(unit), view)
    }

    /// [`World::within`] for a unit of `kind` on `source`, `gap` away from `view`.
    fn gap_within(kind: ObjectKind, attrs: &AttackAttrs, scan: Scan, gap: f32, source: Elevation, view: &View) -> bool {
        match scan {
            Scan::Sight => {
                !inside_min_range(attrs, gap)
                    && gap <= gu(elevation::sight_range(kind.def().sight_range, source, view.elevation))
            }
            Scan::Weapon => within_range(attrs, gap, source, view.elevation),
        }
    }

    /// The angle `unit` would turn to face `view`, radians, 0 to π: from the unit's own facing, the turret's being the
    /// unit's while turrets have no state of their own (design change 0179: "unit facing for infantry").
    fn rotation(unit: &Unit, view: &View) -> f32 {
        let to = view.pos - unit.pos;
        if to == Vec2::ZERO {
            return 0.0;
        }
        math::wrap_angle(math::vector_angle(to) - unit.facing).abs()
    }

    /// `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority` (`> TurretAutonomousScanning`):
    /// the best ValidTarget within the reach of `scan`, if any, among those `admit` lets through:
    /// 1. **threatening** targets first, those whose own attack can target this unit's domain;
    /// 2. then the **least rotation** from the unit's facing (the turret's, once turrets have state), in 45° steps
    ///    (design change 0179, [`rotation_step`]): targets in one step count as needing equal rotation;
    /// 3. then the **closest** (the gap between the shapes); then the lowest id, the fixed order of the remaining
    ///    ties.
    ///
    /// Enemy structures are ValidTargets too (`systems/combat.md > ValidTarget`, design change 0103), so units
    /// attack enemy buildings on their own; those that can't attack (all of them, so far) are non-threatening, and
    /// come after the targets that can shoot back. An enemy inside the unit's MinRange is never picked (design change
    /// 0182, [`Scan`]).
    ///
    /// Nothing while auto-targeting is off (`DebugCommand::SetAutoTargeting`).
    ///
    /// `facing_only`: a non-turret unit that can't turn in place engages only what it already faces
    /// (`> HoldingPosition` 4, within `combat::FACING_TOLERANCE`, the exact angle).
    pub(crate) fn pick(
        &self,
        unit: &Unit,
        scan: Scan,
        facing_only: bool,
        admit: impl Fn(&Candidate) -> bool,
    ) -> Option<ObjectId> {
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
        let mut best: Option<(bool, u8, f32, ObjectId)> = None;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.cells_wide + x) as usize;
                let range = self.cell_starts[cell] as usize..self.cell_starts[cell + 1] as usize;
                for &index in &self.candidates[range] {
                    let view = &self.views[index as usize];
                    // The cheap tests first: most candidates are the unit's own side.
                    if view.owner == unit.owner {
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
                    let in_range = within_range(attrs, gap, source, view.elevation);
                    if !admit(&Candidate { view, in_range }) {
                        continue;
                    }
                    let harmless = !view.threat.is_some_and(|threat| domain_allows(threat, my_domain));
                    let key = (harmless, rotation_step(rotation), gap, view.id);
                    let better = best.is_none_or(|b| {
                        key.0
                            .cmp(&b.0)
                            .then(key.1.cmp(&b.1))
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
