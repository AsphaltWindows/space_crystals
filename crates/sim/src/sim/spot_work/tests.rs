//! The spots' step in a game, with test defs (`defs::test_defs`) for what the design's types can't show: a site that
//! lasts as long as a test needs, for the Recruits waiting further out and moving in as room frees (design change
//! 0339: "Recruits still walking, waiting for a spot or doing anything else add nothing"), and the oblong footprints
//! with ABAC lettering, 2 × 3 and 2 × 4, at all four rotations (the Director's note 2 of slice M5-E: the ten Cults
//! buildings to come are data alone).

use glam::Vec2;

use crate::command::{Command, DebugCommand, TickInputs};

use crate::defs::test_defs::TestDef;
use crate::defs::{Footprint, ObjectDef, ObjectKind, SymmetryType};
use crate::deploy::on_site;
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::mover::MoveStatus::Moving;
use crate::object::{Object, Placement, Rotation};
use crate::orders::OrderKind;
use crate::player::Faction;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::shape::Shape;
use crate::spot_work;
use crate::testing::{ScenarioBuilder, TestSim};
const BUILD_REACH_SU: f32 = crate::tuning::construction::BUILTIN.build_reach_su;
const RESPOT_TICKS: Tick = crate::tuning::construction::BUILTIN.respot_ticks;
use crate::units::{Tick, TilePos};

const P1: PlayerId = PlayerId(1);
const STORAGE: ObjectKind = ObjectKind::Storage;
/// Where the tests build: tiles (20, 14) on, x from 1280 and y from 896.
const SITE: TilePos = TilePos::new(20, 14);

/// The Storage, taking 100 000 frames with one Recruit.
fn slow_storage() -> TestDef {
    TestDef::structure(STORAGE, |structure| {
        structure.site.as_mut().expect("built by Recruits").frames = 100_000;
    })
}

/// The Storage with a `w` × `h` footprint lettered ABAC.
fn oblong_storage(w: u32, h: u32) -> TestDef {
    let mut def = STORAGE.def().clone();
    def.footprint = Footprint::Tiles { w, h };
    def.structure.as_mut().expect("a structure").symmetry = SymmetryType::ABAC;
    TestDef::set(ObjectDef { ..def })
}

/// A flat 40 × 30 map of the Cults' with `count` Recruits of player 1's, ids 1 to `count`, in columns of eight 32 su
/// apart west of [`SITE`], and 10 000 Space Crystals.
fn recruits(count: u32) -> TestSim {
    recruits_at((0..count).map(|i| {
        let (column, row) = (i / 8, i % 8);
        Vec2::new(1100.0 - 32.0 * column as f32, 848.0 + 32.0 * row as f32)
    }))
}

/// A flat 40 × 30 map of the Cults' with Recruits of player 1's at `points`, ids from 1 on, and 10 000 Space Crystals.
fn recruits_at(points: impl IntoIterator<Item = Vec2>) -> TestSim {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(40); 30].join("\n")).faction(P1, Faction::Cults);
    for at in points {
        scenario = scenario.spawn(ObjectKind::Recruit, Some(P1), SpawnAt::Point(at));
    }
    let mut t = scenario.start();
    t.debug_at(
        1,
        DebugCommand::SetResources {
            player: P1,
            crystals: 10_000,
            supplies: 0,
            alloys: 0,
            essence: 0,
            flux: 0,
        },
    );
    t
}

/// The group `1..=count`'s Build of a Storage on [`SITE`] turned by `rotation`, in tick 2.
fn build(t: &mut TestSim, count: u32, rotation: Rotation) {
    let orientation = BuildingOrientation {
        rotation,
        ..Default::default()
    };
    let units = (1..=count).map(ObjectId).collect();
    let kind = OrderKind::Build {
        kind: STORAGE,
        origin: SITE,
        orientation,
    };
    t.at(
        2,
        P1,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    );
}

/// The site `o` assists, if its current order is an Assist.
fn assisting(o: &Object) -> Option<ObjectId> {
    let active = o.orders.as_ref()?.current.as_ref()?;
    spot_work::assisted(active.order.kind)
}

/// The spot unit `o` claims for its work, if any.
fn spot_of(o: &Object) -> Option<Vec2> {
    spot_work::work_of(o)?.spot
}

/// The site in the game, if any.
fn site(t: &TestSim) -> Option<&Object> {
    t.sim().objects().find(|o| o.is_site())
}

/// The builders at work on `site`: the units on an Assist of it within reach of its footprint, their walk over.
fn builders(t: &TestSim, site: &Object) -> Vec<ObjectId> {
    let footprint = Shape::footprint(&site.placement.expect("placed"));
    let standing = |o: &Object| o.mover.as_ref().is_none_or(|m| m.status != Moving);
    (t.sim().objects())
        .filter(|o| assisting(o) == Some(site.id) && Shape::of(o).gap(&footprint) <= BUILD_REACH_SU && standing(o))
        .map(|o| o.id)
        .collect()
}

/// One tick, and the rule of a site built from outside: it gained one frame for each builder at work.
fn step(t: &mut TestSim) {
    let before = site(t).map(|o| (o.id, o.construction.expect("a site").progress.ticks_done()));
    t.step();
    let Some((id, frames)) = before else {
        return;
    };
    if let Some(o) = t.sim().object(id).filter(|o| o.is_site()) {
        let done = o.construction.expect("a site").progress.ticks_done();
        assert_eq!(done, frames + builders(t, o).len() as u32, "tick {}", t.sim().tick());
    }
}

/// Steps until `done` holds, at most `limit` ticks.
fn run_to(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) {
    for _ in 0..limit {
        step(t);
        if done(t) {
            return;
        }
    }
    for o in t.sim().objects().filter(|o| o.kind == ObjectKind::Recruit) {
        let status = o.mover.as_ref().map(|m| m.status);
        let order = o
            .orders
            .as_ref()
            .and_then(|orders| orders.current.as_ref())
            .map(|a| a.order.kind);
        eprintln!("{:?} at {} spot {:?} {status:?} {order:?}", o.id, o.pos, spot_of(o));
    }
    panic!("{what}: not within {limit} ticks");
}

/// The builders' wall round a Storage holds twenty-four Recruits, a room of 32 su each along its 768 su round its
/// 3x2 (design change 0380).
const ROOM_ROUND: usize = 24;

/// Twenty-eight Recruits on a Storage that lasts: twenty-four build, each on its own spot of the builders' wall, and
/// four wait out of reach further out (the row next to the builders left free), their Assist kept. Two builders gone,
/// two of those waiting move in, within the re-check (`RESPOT_TICKS`) and a walk.
#[test]
fn those_who_wait_further_out_move_in_as_room_frees() {
    let _slow = slow_storage();
    let mut t = recruits(28);
    build(&mut t, 28, Rotation::R0);
    run_to(&mut t, 400, "the site starts", |t| site(t).is_some());
    let full = |t: &TestSim| builders(t, site(t).unwrap()).len() == ROOM_ROUND;
    run_to(&mut t, 900, "the wall is full", full);
    let settled = |t: &TestSim| (t.sim().objects()).all(|o| o.mover.as_ref().is_none_or(|m| m.status != Moving));
    run_to(&mut t, 600, "everyone stands", settled);
    let s = site(&t).unwrap().clone();
    let builders_now = builders(&t, &s);
    assert_eq!(builders_now.len(), ROOM_ROUND);
    let waiting: Vec<ObjectId> = (1..=28).map(ObjectId).filter(|id| !builders_now.contains(id)).collect();
    let footprint = Shape::footprint(&s.placement.unwrap());
    for &id in &waiting {
        let o = t.sim().object(id).unwrap();
        assert_eq!(assisting(o), Some(s.id), "{id:?} waits on its Assist");
        assert!(Shape::of(o).gap(&footprint) > BUILD_REACH_SU, "{id:?} out of reach");
    }
    let tick = t.sim().tick();
    for &id in &builders_now[..2] {
        t.debug_at(tick + 1, DebugCommand::Remove { id });
    }
    step(&mut t);
    assert_eq!(builders(&t, &s).len(), ROOM_ROUND - 2);
    let start = t.sim().tick();
    run_to(&mut t, RESPOT_TICKS + 60, "two move in", full);
    println!("the wall full again {} ticks after", t.sim().tick() - start);
    let moved_in = builders(&t, &s).iter().filter(|id| waiting.contains(id)).count();
    assert_eq!(moved_in, 2);
}

/// (M5-J1) The site waits for the order's own Recruits to step off its footprint (design change 0362,
/// `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`: "the site **waits** for Recruits
/// of the same order who stand on the footprint to step off, then starts"), so none may stand still there. #1 stands at
/// its spot west of the footprint, #2 on its middle; as #2 walks off, its walk ends where it stands, still on the
/// footprint, as a walk that found no way left does. It doesn't take that place as its spot (`Sim::step_spots`): within
/// `RESPOT_TICKS` it looks for one again and walks off, and the site starts, with no failure.
#[test]
fn a_recruit_whose_walk_ended_on_the_footprint_looks_for_a_spot_again() {
    let mut t = recruits_at([Vec2::new(1264.0, 960.0), Vec2::new(1344.0, 960.0)]);
    build(&mut t, 2, Rotation::R0);
    let walking = |t: &TestSim| {
        (t.sim().object(ObjectId(2)))
            .is_some_and(|o| spot_of(o).is_some() && o.mover.as_ref().is_some_and(|m| m.status == Moving))
    };
    run_to(&mut t, 4, "#2 walks to its spot", walking);
    let footprint = Placement::new(SITE, STORAGE.def().footprint);
    let mut sim = t.sim().clone();
    let o = sim.objects.get_mut(&ObjectId(2)).expect("#2");
    assert!(on_site(o.kind, o.pos, &footprint), "#2 on the footprint at {}", o.pos);
    o.mover.as_mut().expect("a unit").stop();
    let (stopped, limit) = (sim.tick(), RESPOT_TICKS + 60);
    let mut started = None;
    for _ in 0..limit {
        sim.step(&TickInputs::default());
        let failed = sim.events().iter().any(|e| matches!(e, SimEvent::BuildFailed { .. }));
        assert!(!failed, "tick {}: no failure", sim.tick());
        if sim.events().iter().any(|e| matches!(e, SimEvent::SiteStarted { .. })) {
            started = Some(sim.tick());
            break;
        }
    }
    let started = started.unwrap_or_else(|| panic!("the site starts within {limit} ticks of tick {stopped}"));
    let o = &sim.objects[&ObjectId(2)];
    assert!(
        !on_site(o.kind, o.pos, &footprint),
        "#2 off the footprint at tick {started}"
    );
}

/// The Director's note 2: four Recruits build a Storage given an oblong footprint lettered ABAC, 2 × 3 and 2 × 4, at
/// each of the four rotations: each on its own spot beside the footprint and within reach, none on it (`TestSim`'s
/// invariants), and the building is complete, its footprint turned as ordered.
#[test]
fn oblong_footprints_are_built_at_all_four_rotations() {
    for (w, h) in [(2, 3), (2, 4)] {
        let _oblong = oblong_storage(w, h);
        for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
            let what = format!("{w} x {h} at {rotation:?}");
            let mut t = recruits(4);
            build(&mut t, 4, rotation);
            run_to(&mut t, 400, &what, |t| {
                site(t).is_some_and(|s| builders(t, s).len() == 4)
            });
            let s = site(&t).unwrap().clone();
            let placement = s.placement.unwrap();
            let turned = matches!(rotation, Rotation::R90 | Rotation::R270);
            assert_eq!(
                (placement.w, placement.h),
                if turned { (h, w) } else { (w, h) },
                "{what}"
            );
            let spots: Vec<Vec2> = (1..=4).filter_map(|n| spot_of(t.sim().object(ObjectId(n))?)).collect();
            assert_eq!(spots.len(), 4, "{what}: each has its spot");
            for (i, a) in spots.iter().enumerate() {
                assert!(
                    spots[i + 1..].iter().all(|b| (*a - *b).abs().max_element() >= 24.0),
                    "{what}: {spots:?}"
                );
            }
            let built = |t: &TestSim| t.events().iter().any(|(_, e)| matches!(e, SimEvent::SiteBuilt { .. }));
            run_to(&mut t, 400, &what, built);
            assert!(
                t.sim().object(s.id).is_some_and(|o| !o.is_site() && o.is_operational()),
                "{what}"
            );
        }
    }
}
