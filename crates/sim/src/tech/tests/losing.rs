//! Losing a prerequisite (design change 0335; `core/tech_and_research.md > Tech Prerequisites > Research
//! Prerequisites`: "The general principle: **losing a prerequisite never cancels or disables anything already started
//! or finished**, for structures, units and research alike"; `core/tech_and_research.md > Tech Prerequisites > Losing a
//! Prerequisite`: "Already-completed structures that required it keep functioning normally. Constructions already
//! started (including GDO buildings in progress or ReadyToPlace) continue and can be completed/placed normally"), one
//! table for the four factions. Each case is work behind a prerequisite, under way or done, as player 1 loses their
//! only structure of the kind it requires (a debug removal stands for a destruction); the work is then done in full,
//! and nothing is cancelled, refunded nor failed: the gate checks the prerequisite as the work starts, and nothing
//! checks it again (`crate::tech`). An order given and paid whose work hasn't started, a builder still on its way to
//! its site or a Build queued behind another order, goes on as work started (the plan's reading, `Sim::check_build`:
//! a worker's Build is checked as it is ordered, which is when it is paid; its arrival doesn't check again).
//!
//! The cases tested elsewhere are left out: the GDO's building in progress and units in production ([`super`]), and
//! the Field Engineers queued as the Career Campus is lost (`crates/sim/tests/field_engineer/prerequisites.rs`); the
//! Hideout's queued Guards and Grenadiers (`crates/sim/tests/prerequisites/`); the Armory's Gunners
//! (`crates/sim/tests/armory/gunner.rs`). The Colonists have no prerequisite of the design's among the types built so
//! far, so their case has a test one (`defs::test_defs`), as does the Agent's.

use glam::Vec2;

use super::*;
use crate::augmentation::RecruitKind;
use crate::events::SimEvent;
use crate::object::Object;
use crate::orders::Target;
use crate::units::Tick;

const CAREER_CAMPUS: ObjectKind = ObjectKind::CareerCampus;
const ROBO_PORTAL: ObjectKind = ObjectKind::RoboPortal;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const LASER_TURRET: ObjectKind = ObjectKind::LaserTurret;
const GUNWORKS: ObjectKind = ObjectKind::Gunworks;

/// The ticks a case's work may take after the loss.
const WITHIN: Tick = 1000;

/// One case: work behind a prerequisite, and when its player loses it.
struct Case {
    /// The work, as the report's table names it.
    name: &'static str,
    game: fn() -> Game,
    /// When the prerequisite goes: in the tick after this first holds.
    lost_when: fn(&TestSim) -> bool,
    /// Player 1's orders once it is gone, one a tick, each of which the gate takes.
    then: fn(&TestSim) -> Vec<Command>,
    /// Whether the work is done, as it is within [`WITHIN`] ticks of the loss.
    done: fn(&TestSim) -> bool,
}

/// A case's game, the work ordered: the game, player 1's only structure of the kind the work requires, complete, and
/// the test defs it plays with, for as long as it runs.
struct Game(TestSim, ObjectId, Vec<TestDef>);

/// Plays `case` (module docs): its game until the time comes, the prerequisite removed in the next tick, player 1's
/// orders after it, and the game until the work is done, without a cancel, a refund, a failure or a defeat from the
/// loss on.
fn play(case: &Case) {
    let name = case.name;
    let Game(mut t, prerequisite, _defs) = (case.game)();
    let kind = t.sim().object(prerequisite).map(|o| o.kind).expect("the prerequisite");
    while !(case.lost_when)(&t) {
        assert!(t.sim().tick() < 600, "{name}: the time to lose it never came");
        t.step();
    }
    assert!(t.sim().requirement_met(P1, kind), "{name}: the {kind:?} counts");
    next_debug(&mut t, remove(prerequisite));
    let lost = t.sim().tick();
    assert!(!t.sim().requirement_met(P1, kind), "{name}: the {kind:?} lost");
    for command in (case.then)(&t) {
        assert_eq!(gate(&t, P1, command.clone()), Ok(()), "{name}: {command:?}");
        next(&mut t, P1, command);
    }
    while !(case.done)(&t) {
        assert!(t.sim().tick() < lost + WITHIN, "{name}: not done in {WITHIN} ticks");
        t.step();
    }
    let undone: Vec<_> = (t.events().iter())
        .filter(|(tick, e)| *tick >= lost && undoes(e))
        .collect();
    assert!(undone.is_empty(), "{name}: {undone:?}");
}

/// Whether `e` takes back work, or defeats a player (each case keeps player 1 a structure).
fn undoes(e: &SimEvent) -> bool {
    matches!(
        e,
        SimEvent::ProductionCancelled { .. }
            | SimEvent::ConstructionCancelled { .. }
            | SimEvent::TransformCancelled { .. }
            | SimEvent::BuildFailed { .. }
            | SimEvent::BuildRefunded { .. }
            | SimEvent::SiteCancelled { .. }
            | SimEvent::ExpansionCancelled { .. }
            | SimEvent::PlayerDefeated { .. }
    )
}

fn none(_: &TestSim) -> Vec<Command> {
    Vec::new()
}

fn object(t: &TestSim, id: u32) -> &Object {
    t.sim().object(ObjectId(id)).expect("on the map")
}

/// Player 1's complete objects of `kind` on the map.
fn count(t: &TestSim, kind: ObjectKind) -> usize {
    let complete = |o: &&Object| o.kind == kind && o.owner == Some(P1) && !o.is_site();
    t.sim().objects().filter(complete).count()
}

/// The site in the game, if any.
fn site(t: &TestSim) -> Option<ObjectId> {
    t.sim().objects().find(|o| o.is_site()).map(|o| o.id)
}

fn is_build(kind: OrderKind) -> bool {
    matches!(kind, OrderKind::Build { .. })
}

/// Whether a Build is under way short of its site: a unit's current order is one, and no site has started.
fn on_the_way(t: &TestSim) -> bool {
    let current = |o: &Object| Some(o.orders.as_ref()?.current.as_ref()?.order.kind);
    site(t).is_none() && t.sim().objects().any(|o| current(o).is_some_and(is_build))
}

/// Whether a Build is queued behind another order, and no site has started.
fn queued(t: &TestSim) -> bool {
    let queue = |o: &Object| {
        o.orders
            .iter()
            .flat_map(|orders| orders.queue.iter())
            .any(|q| is_build(q.kind))
    };
    site(t).is_none() && t.sim().objects().any(queue)
}

fn construction(object: u32, command: ConstructionCommand) -> Command {
    Command::Construction {
        objects: vec![ObjectId(object)],
        command,
    }
}

fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    let units = units.iter().map(|&n| ObjectId(n)).collect();
    Command::Order { units, kind, queue }
}

/// A Build of a `kind` at `origin`, unturned, to `units` (`queue` is Shift).
fn build(units: &[u32], kind: ObjectKind, origin: TilePos, queue: bool) -> Command {
    let orientation = BuildingOrientation::default();
    let kind = OrderKind::Build {
        kind,
        origin,
        orientation,
    };
    order(units, kind, queue)
}

/// The GDO's: the Career Campus's Barracks (`factions/gdo/structures/career_campus.md`: "TechPrerequisite - player owns
/// at least one Barracks"; slice M5-N9).
const GDO: [Case; 1] = [Case {
    name: "GDO: a Career Campus ReadyToPlace at its Deployment Center, placed after the loss",
    game: gdo_construction,
    lost_when: |t| object(t, 1).constructor.and_then(|c| c.ready()).is_some(),
    then: |_| {
        let orientation = BuildingOrientation::default();
        let origin = TilePos::new(6, 2);
        vec![construction(1, ConstructionCommand::Place { origin, orientation })]
    },
    done: |t| count(t, CAREER_CAMPUS) == 1,
}];

/// Player 1's Deployment Center (1) building a Career Campus, behind their Barracks (2).
fn gdo_construction() -> Game {
    let scenario = ScenarioBuilder::new(&flat(40, 24))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(2, 2)) // 1
        .spawn(ObjectKind::Barracks, Some(P1), tile(10, 2)); // 2
    let mut t = started(scenario, 1000);
    next(
        &mut t,
        P1,
        construction(1, ConstructionCommand::Construct(CAREER_CAMPUS)),
    );
    Game(t, ObjectId(2), Vec::new())
}

/// The Colonists', behind an Egress Node of player 1's: only their defensive structures have a TechPrerequisite in the
/// design (`factions/colonists/structures/README.md > Tech Model`), none of them built yet.
const COLONISTS: [Case; 1] = [Case {
    name: "Colonists: a Warp In begun",
    game: colonists_warp_in,
    lost_when: |t| object(t, 1).is_transforming(),
    then: none,
    done: |t| object(t, 1).kind == ROBO_PORTAL && !object(t, 1).is_transforming(),
}];

/// Player 1's Beacon (1) warping in a Robo Portal, which a test TechPrerequisite has require their Egress Node (2).
fn colonists_warp_in() -> Game {
    let defs = vec![TestDef::structure(ROBO_PORTAL, |s| {
        s.requires = &[ObjectKind::EgressNode]
    })];
    let scenario = ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::Beacon, Some(P1), tile(4, 4)) // 1
        .spawn(ObjectKind::EgressNode, Some(P1), tile(20, 4)); // 2
    let mut t = started(scenario, 1000);
    let orientation = BuildingOrientation::default();
    let warp_in = ConstructionCommand::WarpIn {
        kind: ROBO_PORTAL,
        orientation,
    };
    next(&mut t, P1, construction(1, warp_in));
    Game(t, ObjectId(2), defs)
}

/// The Syndicate's: the Laser Turret's Optics Lab (`factions/syndicate/structures/laser_turret.md`: "TechPrerequisite -
/// player owns at least one Optics Lab"), and a test one on the Tunnel an Agent builds. The Tunnel's own queues hold
/// nothing behind a prerequisite (its eject queue; one operation at a time).
const SYNDICATE: [Case; 2] = [
    Case {
        name: "Syndicate: a Laser Turret under construction in its Tunnel's operation slot, complete and active",
        game: syndicate_turret,
        lost_when: |t| object(t, 1).tunnel.as_ref().and_then(|s| s.operation).is_some(),
        then: none,
        done: |t| (t.sim().objects()).any(|o| o.kind == LASER_TURRET && !o.is_site() && t.sim().is_active_turret(o)),
    },
    Case {
        name: "Syndicate: an Agent's Build of a Tunnel, ordered and paid, the Agent on its way",
        game: syndicate_build,
        lost_when: on_the_way,
        then: none,
        done: |t| count(t, TUNNEL) == 2,
    },
];

/// Player 1's Tunnel (1), on tiles (4, 8) to (7, 11), building a Laser Turret, and their Optics Lab (2) in its area.
fn syndicate_turret() -> Game {
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8)) // 1: its area (2, 6) to (9, 13)
        .spawn(OPTICS_LAB, Some(P1), tile(2, 6)); // 2
    let mut t = started(scenario, 1000);
    next(&mut t, P1, expand(ObjectId(1), LASER_TURRET, 8, 8));
    Game(t, ObjectId(2), Vec::new())
}

/// Player 1's Agent (3) sent from the map's south-west to build a Tunnel at (4, 4), which a test TechPrerequisite has
/// require their Hideout (2), in the area of their Tunnel (1) in the north-east.
fn syndicate_build() -> Game {
    let defs = vec![TestDef::structure(TUNNEL, |s| s.requires = &[HIDEOUT])];
    let agent = SpawnAt::Point(TilePos::new(4, 17).center());
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(30, 4)) // 1: its area (28, 2) to (35, 9)
        .spawn(HIDEOUT, Some(P1), tile(28, 8)) // 2
        .spawn(ObjectKind::Agent, Some(P1), agent); // 3
    let mut t = started(scenario, 1000);
    next(&mut t, P1, build(&[3], TUNNEL, TilePos::new(4, 4), false));
    Game(t, ObjectId(2), defs)
}

/// Where the Cults' cases build: a 2 × 2 footprint, tiles (20, 14) to (21, 15).
const SITE: TilePos = TilePos::new(20, 14);
/// The Recruits of [`cults`] round [`SITE`].
const GROUP: [u32; 4] = [3, 4, 5, 6];

/// The Cults': the Gunworks' Armory (`factions/cults/structures/gunworks.md`: "TechPrerequisite - player owns at least
/// one Armory") and the Armory's Storage (`factions/cults/structures/armory.md`: "TechPrerequisite - player owns at
/// least one Storage").
const CULTS: [Case; 4] = [
    Case {
        name: "Cults: a site under construction, its builders stopped and sent back by an Assist after the loss",
        game: || cults_build(false),
        lost_when: |t| site(t).is_some(),
        then: |t| {
            let assist = OrderKind::Assist(site(t).expect("the site"));
            vec![order(&GROUP, OrderKind::Stop, false), order(&GROUP, assist, false)]
        },
        done: |t| count(t, GUNWORKS) == 1,
    },
    Case {
        name: "Cults: a group's Build, ordered and paid, the Recruits on their way",
        game: || cults_build(false),
        lost_when: on_the_way,
        then: none,
        done: |t| count(t, GUNWORKS) == 1,
    },
    Case {
        name: "Cults: a group's Build queued (Shift) behind a Move, ordered and paid",
        game: || cults_build(true),
        lost_when: queued,
        then: none,
        done: |t| count(t, GUNWORKS) == 1,
    },
    Case {
        name: "Cults: a complete Armory, its Storage lost: it trains a Militant and still counts for a Gunworks",
        game: cults_armory,
        lost_when: |t| t.sim().held_in(ObjectId(2)).len() == 1,
        then: |_| {
            let militant = ProductionCommand::TrainFrom {
                kind: ObjectKind::Militant,
                from: RecruitKind::Plain,
            };
            let objects = vec![ObjectId(2)];
            let train = Command::Production {
                objects,
                command: militant,
            };
            vec![train, build(&GROUP, GUNWORKS, SITE, false)]
        },
        done: |t| count(t, ObjectKind::Militant) == 1 && count(t, GUNWORKS) == 1,
    },
];

/// A flat 40 × 30 map on which both players play the Cults: player 1's Storage (1) on tiles (2, 2) to (4, 3), their
/// Armory (2) on tiles (4, 20) to (6, 21), its entrance facing east, four Recruits of theirs ([`GROUP`]) 150 su off
/// each side of [`SITE`], so that they get there together, and a fifth (7) east of the Armory.
fn cults() -> TestSim {
    let centre = Vec2::new(1344.0, 960.0);
    let mut scenario = ScenarioBuilder::new(&flat(40, 30))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::Storage, Some(P1), tile(2, 2)) // 1
        .spawn(ObjectKind::Armory, Some(P1), tile(4, 20)); // 2
    for out in [Vec2::NEG_X, Vec2::X, Vec2::NEG_Y, Vec2::Y] {
        let at = SpawnAt::Point(centre + out * (64.0 + 150.0));
        scenario = scenario.spawn(ObjectKind::Recruit, Some(P1), at);
    }
    let fifth = SpawnAt::Point(Vec2::new(600.0, 1344.0));
    started(scenario.spawn(ObjectKind::Recruit, Some(P1), fifth), 1000) // 7
}

/// [`cults`], [`GROUP`] ordered to build a Gunworks at [`SITE`] behind player 1's Armory (2); with Shift behind a Move
/// south if `queued`.
fn cults_build(queued: bool) -> Game {
    let mut t = cults();
    if queued {
        let south = OrderKind::Move(Target::Point(Vec2::new(1344.0, 1500.0)));
        next(&mut t, P1, order(&GROUP, south, false));
    }
    next(&mut t, P1, build(&GROUP, GUNWORKS, SITE, queued));
    Game(t, ObjectId(2), Vec::new())
}

/// [`cults`], player 1's fifth Recruit (7) sent into their Armory (2), behind their Storage (1).
fn cults_armory() -> Game {
    let mut t = cults();
    next(&mut t, P1, order(&[7], OrderKind::Enter(ObjectId(2)), false));
    Game(t, ObjectId(1), Vec::new())
}

#[test]
fn the_gdo() {
    GDO.iter().for_each(play);
}

#[test]
fn the_colonists() {
    COLONISTS.iter().for_each(play);
}

#[test]
fn the_syndicate() {
    SYNDICATE.iter().for_each(play);
}

#[test]
fn the_cults() {
    CULTS.iter().for_each(play);
}
