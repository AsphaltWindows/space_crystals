//! A Hall's commands (`factions/cults/economy.md > RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`): the
//! rally point, and the X and the `Train` it has no use for; and a game with Halls replayed and saved.

use sc_sim::{
    DebugCommand, ObjectId, OrderKind, PlayerCommand, Rally, Reject, Sim, SpawnSettings, Target, TilePos, Vec2,
};

use super::*;

fn validate(t: &TestSim, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player: P1, command })
}

fn cancel(hall: ObjectId) -> Command {
    production(&[hall], ProductionCommand::CancelProduction)
}

fn rally(hall: ObjectId, target: Target) -> Command {
    production(&[hall], ProductionCommand::SetRally(target))
}

/// Design change 0342 point 2 (`factions/cults/economy.md > RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`:
/// "No cancel: Recruit production can't be cancelled"): a `CancelProduction` finds nothing to cancel at a Hall, before
/// its first step or with its Recruit in production, and one sent anyway changes nothing: the Recruit comes out in
/// tick 192, as without it.
#[test]
fn a_hall_refuses_a_cancel() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    assert_eq!(validate(&t, cancel(hall)), Err(Reject::NothingQueued));
    t.at(100, P1, cancel(hall));
    t.run_until(99);
    assert_eq!(progress(&t, hall), Some(99 * 100));
    assert_eq!(validate(&t, cancel(hall)), Err(Reject::NothingQueued));
    t.run(1);
    assert_eq!(progress(&t, hall), Some(100 * 100), "not cancelled");
    let cancelled = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ProductionCancelled { .. }));
    assert!(!cancelled);
    t.run_until(FRAMES);
    assert_eq!(ticks(&t, hall), [FRAMES]);
}

/// A Hall trains nothing on command: `factions/cults/economy.md > RecruitmentHall > ObjectInterfaceState[RecruitmentHall]`
/// has no Train, and the Hall recruits by itself (`factions/cults/economy.md > RecruitmentHall > Production
/// Behavior`). A `Train` of a Recruit is refused and changes nothing.
#[test]
fn a_hall_refuses_a_train() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    let train = production(&[hall], ProductionCommand::Train(RECRUIT));
    assert_eq!(validate(&t, train.clone()), Err(Reject::CannotProduce(RECRUIT)));
    t.run(5);
    assert_eq!(validate(&t, train.clone()), Err(Reject::CannotProduce(RECRUIT)));
    let before = t.sim().clone();
    t.at(6, P1, train);
    t.run(1);
    let queue = |sim: &Sim| sim.object(hall).and_then(|o| o.production.clone()).map(|p| p.queue);
    assert_eq!(queue(t.sim()), queue(&before));
    assert_eq!(queue(t.sim()).map(|q| q.len()), Some(1));
}

/// Plan acceptance, the rally point: "Produced Recruits spawn at the hall and move to its rally point"
/// (`factions/cults/economy.md > RecruitmentHall > Production Behavior`), C setting it as for every producer.
#[test]
fn the_recruits_walk_to_the_rally_point() {
    let hall = ObjectId(1);
    let point = Vec2::new(1700.0, 640.0);
    let mut t = flat(40, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    t.at(1, P1, rally(hall, Target::Point(point)));
    t.run_until(FRAMES);
    let production = t
        .sim()
        .object(hall)
        .and_then(|o| o.production.clone())
        .expect("a producer");
    assert_eq!(production.rally, Some(Rally::Point(point)));
    let (_, unit) = made(&t, hall)[0];
    let order = |t: &TestSim| {
        let orders = t.sim().object(unit).and_then(|o| o.orders.clone()).expect("a unit");
        orders.queue.front().map(|order| order.kind)
    };
    assert_eq!(order(&t), Some(OrderKind::Move(Target::Point(point))));
    // Out of side B, the west one, and round the Hall to the east, where it arrives before the next one can.
    for _ in 0..600 {
        if t.pos(unit.0) == point {
            break;
        }
        t.step();
    }
    assert_eq!(t.pos(unit.0), point);
    assert!(t.sim().tick() < 2 * FRAMES + 300, "in tick {}", t.sim().tick());
}

/// Plan acceptance (slice M5-D landed first): a rally point on a Space Crystals patch makes each new Recruit gather
/// from it (`systems/controls/selection_and_hud.md > What a rally point makes a new unit do`: the default right-click
/// command on the target; `factions/cults/units/recruit.md > Gathering`), delivering at the Storage.
#[test]
fn a_rally_on_a_patch_makes_new_recruits_gather() {
    let (hall, patch) = (ObjectId(1), ObjectId(3));
    let patch_of = SpawnSettings::CrystalPatch { starting_amount: 1000 };
    let mut t = flat(40, 20)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Storage, Some(P1), tile(16, 9))
        .spawn_with(patch_of, tile(19, 9))
        .start();
    t.at(1, P1, rally(hall, Target::Object(patch)));
    t.run_until(2 * FRAMES);
    for (_, unit) in made(&t, hall) {
        let orders = t.sim().object(unit).and_then(|o| o.orders.clone()).expect("a unit");
        let current = orders.current.map(|active| active.order.kind);
        let next = orders.queue.front().map(|order| order.kind);
        assert_eq!(current.or(next), Some(OrderKind::Gather(patch)), "Recruit {}", unit.0);
    }
    t.run_until(2 * FRAMES + 600);
    let delivered = (t.events().iter()).any(|(_, e)| {
        matches!(e, SimEvent::ResourcesDelivered { worker, .. } if made(&t, hall).iter().any(|&(_, u)| u == *worker))
    });
    assert!(delivered, "the Hall's Recruits deliver at the Storage");
}

/// Determinism: a game with Halls replays to the same hashes, and a save goes on like the game. The Hall spawned in
/// tick 10 has its completion tick in the save, which decides the shared tiles (`Production::completed_at`).
#[test]
fn a_game_with_halls_replays_and_saves() {
    let (first, late) = (ObjectId(1), ObjectId(2));
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(14, 8)).start();
    t.debug_at(
        10,
        DebugCommand::Spawn {
            kind: HALL,
            owner: Some(P2),
            at: SpawnAt::Tile(TilePos::new(8, 8)),
            orientation: Default::default(),
        },
    );
    t.at(120, P1, rally(first, Target::Point(Vec2::new(1500.0, 300.0))));
    t.run_until(150);
    assert_eq!(
        t.sim()
            .object(late)
            .and_then(|o| o.production.as_ref())
            .map(|p| p.completed_at),
        Some(10)
    );
    assert_eq!((hall_state(&t, first).claimed, hall_state(&t, late).claimed), (100, 60));
    let json = serde_json::to_string(t.sim()).expect("saves");
    assert!(json.contains("\"completed_at\":10"));
    let saved: Sim = serde_json::from_str(&json).expect("loads");
    assert_eq!(&saved, t.sim());
    assert_eq!(saved.state_hash(), t.sim().state_hash());
    t.run_until(700);
    assert!(made(&t, first).len() >= 2 && made(&t, late).len() >= 2);
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
    let mut resumed = saved;
    for inputs in &t.replay().inputs[150..] {
        resumed.step(inputs);
    }
    assert_eq!(resumed.state_hash(), t.sim().state_hash());
}
