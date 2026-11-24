//! The movement invariants I1–I7 of `docs/movement.md > 8` (I2 with its one exception, the exit doorway of design
//! change 0080), checked from outside through the sim's public state,
//! the vision invariant V1 (the vision kept up tick by tick is the one a fresh build gives, and every Visible tile
//! is Explored: `vision`), the path knowledge invariant K1 (each player's planning grids are those a fresh build
//! from the state gives: `paths`), and an ASCII picture of the map with its units for readable failures.
//!
//! [`super::TestSim`] checks them after every tick (on by default). A test that steps a bare [`Sim`] keeps its own
//! [`Invariants`] and calls [`Invariants::check`] after each step.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use glam::Vec2;

use crate::command::{Command, DebugCommand, TickInputs};
use crate::defs::Domain;
use crate::ids::ObjectId;
use crate::mover::{Locomotion, MAX_REPATHS, MoveGoal, MoveStatus, Mover, PathState};
use crate::nav;
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::production::Doorway;
use crate::sim::Sim;
use crate::units::{SU_PER_GU, Tick, TilePos};

/// I6: a path request is served within this many ticks. Requests are served oldest first and at least one per
/// tick (`docs/movement.md > 5.5`), so this bounds how far a test's scenario may fall behind, not the sim.
pub const MAX_REQUEST_AGE: Tick = 32;

/// A broken invariant.
#[derive(Clone, Debug, PartialEq)]
pub struct Violation {
    /// "I1" … "I7", "V1", "K1".
    pub invariant: &'static str,
    pub message: String,
    /// Where on the map to look.
    pub at: Vec2,
}

/// What the checker remembers of a unit from the tick before.
#[derive(Clone, Debug)]
struct Seen {
    pos: Vec2,
    had_goal: bool,
    /// It was taking a sidestep for a blocked unit of its owner (design change 0022).
    sidestepping: bool,
    repaths_left: u8,
    /// The building whose exit doorway it was still in (`Mover::exit`, design change 0080).
    exit: Option<ObjectId>,
}

/// Checks I1–I7 between ticks. It remembers each unit's state of the tick before, so check after **every** step.
#[derive(Clone, Debug)]
pub struct Invariants {
    seen: BTreeMap<ObjectId, Seen>,
}

fn leg_of(o: &Object) -> Option<Tick> {
    o.orders.as_ref()?.current.as_ref().map(|active| active.since)
}

fn seen(o: &Object, mover: &Mover) -> Seen {
    Seen {
        pos: o.pos,
        had_goal: mover.goal != MoveGoal::None,
        sidestepping: mover.sidestep.is_some(),
        repaths_left: mover.repaths_left,
        exit: mover.exit,
    }
}

impl Invariants {
    /// Starts from the sim's current state.
    pub fn new(sim: &Sim) -> Self {
        let seen = sim
            .objects()
            .filter_map(|o| Some((o.id, seen(o, o.mover.as_ref()?))))
            .collect();
        Self { seen }
    }

    /// Checks the state after a step that applied `inputs` (teleports are read from them: a teleported unit may
    /// "move" without a goal). Returns the first violation.
    pub fn check(&mut self, sim: &Sim, inputs: &TickInputs) -> Result<(), Violation> {
        let teleported: Vec<ObjectId> = inputs
            .commands
            .iter()
            .filter_map(|c| match c.command {
                Command::Debug(DebugCommand::SetPosition { id, .. }) => Some(id),
                _ => None,
            })
            .collect();
        let result = self
            .check_units(sim, &teleported)
            .and_then(|()| no_overlaps(sim))
            .and_then(|()| vision(sim))
            .and_then(|()| path_knowledge(sim));
        self.seen = Self::new(sim).seen;
        result
    }

    fn check_units(&self, sim: &Sim, teleported: &[ObjectId]) -> Result<(), Violation> {
        let tick = sim.tick();
        for o in sim.objects() {
            let fail = |invariant, message: String| {
                Err(Violation {
                    invariant,
                    message: format!("#{} ({:?}) {message}", o.id.0, o.kind),
                    at: o.pos,
                })
            };
            // I4: every float of the state is finite.
            if !floats_finite(o) {
                return fail("I4", format!("has a float that isn't finite: {o:?}"));
            }
            let def = o.def();
            let (Some(unit), Some(half), Some(mover)) = (&def.unit, def.half_extents(), &o.mover) else {
                continue;
            };
            let before = self.seen.get(&o.id);
            // I2: a ground box overlaps no blocked tile and stays on the map. The one exception (design change
            // 0080, `core/entities.md > Structure Type > Exit Doorway`): a unit that a producer released, while
            // its box still overlaps that building's footprint (`Mover::exit`), overlaps that footprint's tiles
            // and no other blocked tile; and from tick to tick it only comes out (`Doorway::allows`).
            let grid = unit
                .base_def()
                .nav_class()
                .and_then(|class| sim.nav().grid(class, nav::size_class(half * 2.0)));
            match mover.exit {
                None => {
                    if grid.is_some_and(|grid| !nav::box_clear(grid, half, o.pos)) {
                        return fail("I2", format!("at {} overlaps a blocked tile or the map's edge", o.pos));
                    }
                }
                Some(building) => {
                    let Some(door) = sim.object(building).and_then(Doorway::of) else {
                        return fail("I2", format!("is in the doorway of #{}, which has none", building.0));
                    };
                    if !door.holds(o.pos, half) {
                        return fail(
                            "I2",
                            format!("at {} is out of the doorway of #{} but still exempt", o.pos, building.0),
                        );
                    }
                    if grid.is_some_and(|grid| !door.box_clear(grid, half, o.pos)) {
                        return fail(
                            "I2",
                            format!("at {} in a doorway overlaps a blocked tile of another object", o.pos),
                        );
                    }
                    let came_from = before.filter(|b| b.exit == Some(building) && !teleported.contains(&o.id));
                    if let Some(b) = came_from
                        && !door.allows(b.pos, o.pos, half)
                    {
                        return fail("I2", format!("went from {} back into its doorway, to {}", b.pos, o.pos));
                    }
                    // Only the units a producer just released come into being inside a footprint.
                    if before.is_none_or(|b| b.exit != Some(building)) && o.prev_pos != o.pos {
                        return fail("I2", format!("entered the doorway of #{} from outside", building.0));
                    }
                }
            }
            // I3: a unit moves at most its speed per tick.
            let step = unit.movement.max_speed() * mover.speed_factor;
            let moved = o.pos.distance(o.prev_pos);
            if moved > step * 1.0001 + 1e-3 {
                return fail("I3", format!("moved {moved} su in one tick, more than its {step}"));
            }
            if let Some(before) = before
                && before.pos != o.prev_pos
                && !teleported.contains(&o.id)
            {
                return fail(
                    "I5",
                    format!("was moved outside the movement step: {} to {}", before.pos, o.prev_pos),
                );
            }
            // I5, I7: a unit without a goal doesn't move, so an Arrived unit never moves again without an order,
            // except by the one short sidestep it takes for a blocked unit of its owner (design change 0022).
            // It may move in a tick it ends without a goal only if it had one before, an order started, or it was
            // taking a sidestep.
            let had_goal = before.is_some_and(|b| b.had_goal);
            let sidestepping = before.is_some_and(|b| b.sidestepping);
            // A unit in an exit doorway walks out of it by itself (design changes 0080 and 0086).
            let leaving = before.is_some_and(|b| b.exit.is_some());
            let order_started = leg_of(o) == Some(tick);
            // A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase, a Patrol's next
            // leg, the way back to an IdleOrigin). `go_to` in this tick left a full count and its tick behind,
            // also when the move ended in the same tick.
            let new_leg = mover.repaths_left == MAX_REPATHS && mover.last_repath == tick;
            let began = order_started || new_leg;
            if o.pos != o.prev_pos && mover.goal == MoveGoal::None && !had_goal && !began && !sidestepping && !leaving {
                return fail(
                    "I5/I7",
                    format!(
                        "moved from {} to {} without a goal, a new order or move, a sidestep, or a doorway to leave",
                        o.prev_pos, o.pos
                    ),
                );
            }
            // A finished move leaves the unit standing (`docs/movement.md > 6.5`), and so does a finished sidestep.
            if mover.goal == MoveGoal::None
                && ((mover.sidestep.is_none()
                    && mover.exit.is_none()
                    && (mover.velocity != Vec2::ZERO || mover.locomotion != Locomotion::Stationary))
                    || mover.path_state != PathState::None)
            {
                return fail("I7", format!("has no goal but isn't at rest: {mover:?}"));
            }
            // Only a unit without a goal and without an order takes a sidestep (design change 0023).
            let idle = o.orders.as_ref().is_some_and(|orders| orders.is_idle());
            if mover.sidestep.is_some() && (mover.goal != MoveGoal::None || !idle) {
                return fail("I7", format!("sidesteps though it has a goal or an order: {mover:?}"));
            }
            if mover.goal != MoveGoal::None && mover.status != MoveStatus::Moving {
                return fail("I7", format!("has a goal with status {:?}", mover.status));
            }
            // I6: repaths per leg are capped, and a request doesn't wait for ever.
            if mover.repaths_left > MAX_REPATHS {
                return fail("I6", format!("has {} repaths left", mover.repaths_left));
            }
            // Only a new leg gives repaths back.
            if let Some(before) = before
                && mover.repaths_left > before.repaths_left
                && !new_leg
            {
                return fail(
                    "I6",
                    format!(
                        "got repaths back within one leg: {} to {}",
                        before.repaths_left, mover.repaths_left
                    ),
                );
            }
            if let PathState::Wanted { since, .. } = mover.path_state
                && tick.saturating_sub(since) > MAX_REQUEST_AGE
            {
                return fail("I6", format!("has waited for a path since tick {since}"));
            }
        }
        Ok(())
    }
}

/// V1: the vision phase 5f keeps up incrementally is the one a fresh build from the objects gives, every Visible
/// tile is Explored, and each player's records are those it should keep (`vision::memory`).
fn vision(sim: &Sim) -> Result<(), Violation> {
    sim.vision()
        .check(sim.map(), sim.objects(), sim.tick())
        .map_err(|(at, message)| Violation {
            invariant: "V1",
            message,
            at,
        })
}

/// K1: the grids each player plans on are those of what it knows at the end of the tick
/// (`systems/controls/behaviors.md > PathKnowledge`), exactly as a loaded save rebuilds them.
fn path_knowledge(sim: &Sim) -> Result<(), Violation> {
    sim.check_path_knowledge().map_err(|message| Violation {
        invariant: "K1",
        message,
        at: Vec2::ZERO,
    })
}

/// I1: no two units of a layer with hard collision overlap. Touching is free.
fn no_overlaps(sim: &Sim) -> Result<(), Violation> {
    for domain in [Domain::Ground, Domain::Underground] {
        // (low x, id, position, half extents), by low x: a unit only meets those starting before its high x.
        let mut boxes: Vec<(f32, ObjectId, Vec2, Vec2)> = sim
            .objects()
            .filter_map(|o| {
                let def = o.def();
                let half = def.half_extents()?;
                (def.unit.as_ref()?.base_def().domain == domain).then_some((o.pos.x - half.x, o.id, o.pos, half))
            })
            .collect();
        boxes.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        for (i, &(_, a_id, a, a_half)) in boxes.iter().enumerate() {
            for &(low, b_id, b, b_half) in &boxes[i + 1..] {
                if low >= a.x + a_half.x {
                    break;
                }
                if (a.x - b.x).abs() < a_half.x + b_half.x && (a.y - b.y).abs() < a_half.y + b_half.y {
                    return Err(Violation {
                        invariant: "I1",
                        message: format!("#{} at {a} and #{} at {b} overlap", a_id.0, b_id.0),
                        at: (a + b) / 2.0,
                    });
                }
            }
        }
    }
    Ok(())
}

fn floats_finite(o: &Object) -> bool {
    let core = o.pos.is_finite()
        && o.prev_pos.is_finite()
        && o.facing.is_finite()
        && o.prev_facing.is_finite()
        && o.hp.is_none_or(f32::is_finite);
    let mover = o.mover.as_ref().is_none_or(|m| {
        let goal = match m.goal {
            MoveGoal::None => true,
            MoveGoal::Point(p) => p.is_finite(),
            MoveGoal::Object { reach, .. } => reach.is_finite(),
        };
        goal && m.velocity.is_finite()
            && m.speed_factor.is_finite()
            && m.planned_target.is_finite()
            && m.best_remaining.is_finite()
            && m.sidestep.is_none_or(|s| s.dir.is_finite() && s.left.is_finite())
            && m.path.iter().all(|p| p.is_finite())
    });
    let orders = o.orders.as_ref().is_none_or(|orders| {
        let current = orders.current.iter().map(|active| &active.order);
        current.chain(&orders.queue).all(|order| {
            order.slot.is_none_or(Vec2::is_finite)
                && match order.kind {
                    OrderKind::Move(Target::Point(p)) | OrderKind::AttackMove(p) | OrderKind::Patrol(p) => {
                        p.is_finite()
                    }
                    OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => {
                        true
                    }
                }
        })
    });
    let rally = o
        .production
        .as_ref()
        .and_then(|production| production.rally)
        .is_none_or(|rally| rally.point().is_finite());
    core && mover && orders && rally
}

/// Characters per tile across and down: a character cell is 16 × 32 su, roughly square on a terminal.
const CELLS: (i32, i32) = (4, 2);
/// The picture shows at most this many tiles; a larger map is cut to a window around the focus.
const WINDOW: (i32, i32) = (30, 20);

/// An ASCII picture of the map and its units (`docs/movement.md > 8`): four characters by two per tile. `.` is
/// open ground, `:` Rugged Terrain, `#` a tile nothing walks on, `%` a blocking footprint. Each unit is a letter
/// (`A`–`Z`, then `a`–`z`, by id) on the cells its box covers, `*` where two units claim one cell; the legend
/// below lists every unit shown. A map larger than 30 × 20 tiles is cut to a window around `focus` (the map's
/// middle without one).
pub fn ascii(sim: &Sim, focus: Option<Vec2>) -> String {
    let map = sim.map();
    let (w, h) = (map.width() as i32, map.height() as i32);
    let focus = TilePos::containing(focus.unwrap_or(Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0)));
    let origin = |centre: i32, size: i32, window: i32| (centre - window / 2).clamp(0, (size - window).max(0));
    let (x0, y0) = (origin(focus.x, w, WINDOW.0), origin(focus.y, h, WINDOW.1));
    let (cols, rows) = (w.min(WINDOW.0) * CELLS.0, h.min(WINDOW.1) * CELLS.1);
    let cell = Vec2::new(SU_PER_GU / CELLS.0 as f32, SU_PER_GU / CELLS.1 as f32);
    let corner = TilePos::new(x0, y0).corner();

    let mut picture = vec![vec!['.'; cols as usize]; rows as usize];
    for (row, line) in picture.iter_mut().enumerate() {
        for (col, c) in line.iter_mut().enumerate() {
            let tile = TilePos::new(x0 + col as i32 / CELLS.0, y0 + row as i32 / CELLS.1);
            let preset = map.tile_preset(tile).expect("inside the window");
            *c = if !preset.traversible {
                '#'
            } else if preset.rugged {
                ':'
            } else {
                '.'
            };
        }
    }
    let mut paint = |lo: Vec2, hi: Vec2, centre: Vec2, with: char, over_units: bool| {
        let inside = |v: Vec2| ((v - corner) / cell).floor();
        let (c0, c1, at) = (
            inside(lo + cell / 2.0 - 0.01),
            inside(hi - cell / 2.0 + 0.01),
            inside(centre),
        );
        let (c0, c1) = (c0.min(at), c1.max(at));
        for row in c0.y as i32..=c1.y as i32 {
            for col in c0.x as i32..=c1.x as i32 {
                if row < 0 || col < 0 || row >= rows || col >= cols {
                    continue;
                }
                let c = &mut picture[row as usize][col as usize];
                *c = if over_units && (c.is_ascii_alphabetic() || *c == '*') {
                    '*'
                } else {
                    with
                };
            }
        }
    };
    for o in sim.objects() {
        if let Some(placement) = &o.placement {
            let lo = placement.origin.corner();
            let hi = lo + Vec2::new(placement.w as f32, placement.h as f32) * SU_PER_GU;
            paint(lo, hi, placement.center(), '%', false);
        }
    }
    let mut legend = String::new();
    let letters: Vec<char> = ('A'..='Z').chain('a'..='z').collect();
    let mut units = 0;
    for o in sim.objects() {
        let (Some(half), Some(mover)) = (o.def().half_extents(), &o.mover) else {
            continue;
        };
        let letter = letters[units % letters.len()];
        units += 1;
        let tile = TilePos::containing(o.pos);
        if tile.x < x0 || tile.y < y0 || tile.x >= x0 + WINDOW.0 || tile.y >= y0 + WINDOW.1 {
            continue;
        }
        paint(o.pos - half, o.pos + half, o.pos, letter, true);
        let _ = writeln!(
            legend,
            "{letter} #{} {:?} p{} at ({:.2}, {:.2}) {:?} goal {:?} next {}/{} blocked {} repaths left {}",
            o.id.0,
            o.kind,
            o.owner.map_or(0, |p| p.0),
            o.pos.x,
            o.pos.y,
            mover.status,
            mover.goal,
            mover.next,
            mover.path.len(),
            mover.blocked_ticks,
            mover.repaths_left,
        );
    }
    let mut out = format!(
        "tick {}, tiles ({x0}, {y0}) to ({}, {}):\n",
        sim.tick(),
        x0 + cols / CELLS.0 - 1,
        y0 + rows / CELLS.1 - 1
    );
    for line in picture {
        out.extend(line);
        out.push('\n');
    }
    out + &legend
}
