//! What the behaviours see of the game in phase 5a — a snapshot of every object, taken before any unit's behaviour
//! runs, so that every unit decides on the same world: the positions the last tick left and the vision its phase 5f
//! computed (`docs/movement.md > 4`). Also the scan for targets of `systems/controls/behaviors.md >
//! BaseAutoTargeting` and `systems/combat.md > ValidTarget` ([`scan`]), and what the workers' scans read
//! ([`gathering`]).

mod gathering;
mod planning;
pub(super) mod scan;

use std::collections::BTreeMap;

use glam::Vec2;

use crate::augmentation::Research;
use crate::combat::{self, AttackState, domain_allows, object_domain};
use crate::defs::{AttackAttrs, Domain, ObjectKind, TargetDomain, TurretAttrs, UnitDef};
use crate::economy::Worker;
use crate::elevation::Elevation;
use crate::ids::{ObjectId, PlayerId};
use crate::map::Map;
use crate::mover::Mover;
use crate::network::EntryPath;
use crate::object::{Object, ResourceState};
use crate::paths::NavState;
use crate::paths::knowledge::PathKnowledge;
use crate::shape::Shape;
use crate::targets::{self, Live, TargetView};
use crate::units::{SU_PER_GU, Tick};
use crate::vision::{Hiding, Vision};

use gathering::Gathering;
pub(crate) use gathering::{KnownPatch, KnownStation};
pub(crate) use scan::{Candidate, Scan};

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
    /// Whether it is a unit: a Move to it follows it (`systems/controls/command_behaviors.md > MovingToObject >
    /// Completion`).
    pub is_unit: bool,
    /// Whether it is a Space Crystals patch (`core/map.md > SpaceCrystalsPatch`), which workers gather.
    pub is_patch: bool,
    /// The TargetDomain of its attack, if it has one: whom it threatens (`systems/controls/behaviors.md >
    /// TurretAutonomousScanning`, priority 1); (M5-S2b) a building's units inside add theirs ([`World::with_occupants`]).
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
    /// with (design change 0198), (M5-N1a) as its Augmentations make it.
    pub weapon: Option<&'a AttackAttrs>,
    /// (M5-N1a) Its top speed with its Augmentations' at its owner's levels, and (M5-N11) its Inspire's
    /// (`Object::max_speed`), before its mode's speed factor ([`Unit::max_speed`]).
    pub max_speed: f32,
    /// Whether its mode lets it attack now: not while its Riot Shield is up (`riot_shield::allows_attack`).
    pub may_attack: bool,
    /// Whether it converts its weapon (`conversion::is_converting`): it holds still meanwhile.
    pub converting: bool,
    /// (M6-GD) Whether it holds still for its drone (`tether::holds`: an operator with its drone out).
    pub tethered: bool,
    /// A worker's load, patch and drop-off (`economy::gathering`).
    pub worker: Option<&'a mut Worker>,
    /// (M5-N4b) Its energy pool (`Object::energy`), which a channel drains (`super::shroud`).
    pub energy: Option<&'a mut u32>,
}

impl<'a> Unit<'a> {
    pub fn unit_def(&self) -> Option<&'static UnitDef> {
        self.kind.def().unit.as_ref()
    }

    /// (M5-N1a) Its top speed with its Augmentations' and (M5-N11) its Inspire's (`Object::max_speed`), before its mode's
    /// speed factor.
    pub fn max_speed(&self) -> f32 {
        self.max_speed
    }

    /// Its attack, if it has one (`systems/combat.md > AttackAttributes`) and may use it now: the weapon in hand
    /// ([`Unit::weapon`]). A unit whose Riot Shield is up has none (`factions/gdo/units/peacekeeper.md > Riot
    /// Shield`: it "cannot attack" and "does not auto-target enemies"): its behaviours pick no target and let go of
    /// the one they engaged.
    pub fn attrs(&self) -> Option<&'a AttackAttrs> {
        self.weapon.filter(|_| self.attack.is_some() && self.may_attack)
    }

    pub fn shape(&self) -> Shape {
        Shape::silhouette(self.kind, self.pos, self.facing)
    }

    /// `combat::may_move`: whether its attack sequence leaves its base free to move this tick, and it isn't
    /// converting its weapon, nor (M6-GD) holding still for its drone.
    pub fn may_move(&self) -> bool {
        !self.converting && !self.tethered && combat::base_is_free(self.kind, self.attack.as_deref())
    }

    /// Its behaviour engages `target`, or nothing (`AttackState::engage`).
    pub fn engage(&mut self, target: Option<ObjectId>) {
        if let Some(attack) = &mut self.attack {
            attack.engage(target);
        }
    }

    /// (M4-O) Its behaviour engages the ground location `at` (`AttackState::engage_ground`).
    pub fn engage_ground(&mut self, at: Vec2) {
        if let Some(attack) = &mut self.attack {
            attack.engage_ground(at);
        }
    }

    /// (M5-H1) Whether it moves by GliderMovement (`systems/units.md > GliderMovement`, `movement::glider`): it can't
    /// stop, and its behaviours fly it where another unit would stand (the GliderMovement exceptions of
    /// `systems/controls/command_behaviors.md`).
    pub fn glides(&self) -> bool {
        self.unit_def().is_some_and(UnitDef::glides)
    }

    /// (M5-U1) Its TurretAttributes, if it has a turret (`UnitDef::turret`, design change 0154): its attack comes
    /// from the turret (`combat::turret`).
    pub fn turret(&self) -> Option<TurretAttrs> {
        self.unit_def()?.turret
    }

    /// (M5-U1) The direction its attack faces, from which a scan measures the rotation a target needs
    /// (`systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`: "unit facing for infantry,
    /// turret angle for turret units"): its turret's, on its own facing (`combat::turret::aim_facing`), else its own.
    pub fn aim_facing(&self) -> f32 {
        let turret = self.attack.as_ref().and_then(|attack| attack.turret);
        combat::turret::aim_facing(turret, self.facing)
    }

    /// (M5-U1) Whether the point `at` is within its turret's arc (`combat::turret::in_arc`); always for a unit
    /// without a turret.
    pub fn in_arc(&self, at: Vec2) -> bool {
        (self.turret()).is_none_or(|turret| combat::turret::in_arc(&turret, self.facing, self.pos, at))
    }

    /// (M5-N10) Whether, the point `at` in range, it takes it on from where it stands: within its turret's arc
    /// ([`Unit::in_arc`]), or outside it if it aims with its hull as well (`combat::turret::aims_by_hull`: the Hand of
    /// Heaven), whose hull then turns toward it ([`Unit::aim_hull`]): "the vehicle must **stop and turn its hull** to
    /// face them" (`factions/cults/units/hand_of_heaven.md > TurretAttributes`).
    pub fn takes_aim(&self, at: Vec2) -> bool {
        self.in_arc(at) || self.unit_def().is_some_and(combat::turret::aims_by_hull)
    }

    /// (M5-N10) Stopped to take on the point `at` ([`Unit::takes_aim`]), a unit that aims with its hull turns it toward
    /// `at` while `at` is outside its turret's arc: it asks phase 5d to turn it there in place (`Mover::turn_to`,
    /// `movement::turn_in_place`), as infantry turns toward its target ("Orientation = Maintaining (or Turning toward
    /// target for infantry)", `systems/controls/command_behaviors.md > AttackingObject` 3). Within the arc, its turret
    /// turns the rest of the way.
    pub fn aim_hull(&mut self, at: Vec2) {
        if !self.in_arc(at) && self.unit_def().is_some_and(combat::turret::aims_by_hull) {
            self.mover.turn_to = Some(at);
        }
    }

    /// (M5-U1) Its turret, if it has one, locks `target`, or nothing (`AttackState::lock`): what the design calls
    /// setting TurretCommandState.LockedTarget (`systems/controls/commands.md > TurretCommandState`).
    pub fn lock(&mut self, target: Option<ObjectId>) {
        if self.turret().is_some()
            && let Some(attack) = &mut self.attack
        {
            attack.lock(target);
        }
    }

    /// (M5-K) Its Target Lock under way, if any, ends (`AttackState::end_target_lock`).
    pub fn end_target_lock(&mut self) {
        if let Some(attack) = &mut self.attack {
            attack.end_target_lock();
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
    /// The entry path of every Tunnel and every building that holds units, in id order (`crate::network::EntryPath`):
    /// where an Enter order walks to.
    entries: Vec<(ObjectId, EntryPath)>,
    /// (M5-N1a) Every player's research, which the units' stats follow (`augmentation::Research`): their weapons and
    /// top speeds (`orders::update`).
    pub research: Research<'a>,
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
                // (M5-H1) Where it counts as being: a Glider past the map's edge at the nearest edge tile.
                pos: o.map_pos(map),
                shape: Shape::on_map(o, map),
                domain: object_domain(o),
                elevation: Elevation::of_object(map, o),
                destructible: o.hp.is_some(),
                is_unit: o.mover.is_some(),
                is_patch: matches!(o.resource, Some(ResourceState::CrystalPatch { .. })),
                // The weapon in hand (`combat::weapon`: a Legionnaire's form's, design change 0198), a structure's
                // once it is complete (slice M4-S1).
                threat: combat::threat(o),
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
            entries: (objects.values())
                .filter_map(|o| Some((o.id, EntryPath::of_object(o)?)))
                .collect(),
            research: Research::NONE,
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

    /// (M5-N1a) The snapshot with every player's research (`augmentation::Research`), which the units' weapons and
    /// top speeds follow. Without this, everyone's levels are "No research".
    pub(crate) fn with_research(mut self, research: Research<'a>) -> Self {
        self.research = research;
        self
    }

    /// (M5-S2b) The snapshot with what the units inside the buildings threaten (`combat::Threats`; design change 0377,
    /// answering Q0139, `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` 1): a
    /// building that fires through its occupants is a threatening target to a scan ([`World::pick`]) while one of
    /// them could attack the scanner's domain. Without this, no building but one with an attack of its own is.
    pub(crate) fn with_occupants(mut self, inside: &combat::Threats) -> Self {
        inside.add_to(&mut self.views, |v| v.id, |v| &mut v.threat);
        self
    }

    fn cell_coord(&self, v: f32, cells: u32) -> u32 {
        let cell = (v / (CELL_TILES as f32 * SU_PER_GU)).floor();
        (cell.max(0.0) as u32).min(cells - 1)
    }

    fn cell_of(&self, p: Vec2) -> usize {
        (self.cell_coord(p.y, self.cells_high) * self.cells_wide + self.cell_coord(p.x, self.cells_wide)) as usize
    }

    /// The entry path of object `id` (`crate::network::EntryPath`): a Tunnel's on Side A, a building's that holds units
    /// on its entrance side; `None` for anything else.
    pub(crate) fn entry_path(&self, id: ObjectId) -> Option<EntryPath> {
        let i = self.entries.binary_search_by_key(&id, |&(target, _)| target).ok()?;
        self.entries.get(i).map(|&(_, path)| path)
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

    /// The units, in id order.
    pub(crate) fn units(&self) -> impl Iterator<Item = &View> + '_ {
        self.views.iter().filter(|view| view.is_unit)
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
}
