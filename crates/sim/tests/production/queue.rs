//! The queue and its costs, the commands and their gate, group production, the refund of a destroyed producer, and
//! a game with production replayed and saved.

use sc_sim::testing::{TestSim, dangling_references};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, ProductionCommand, QUEUE_LIMIT,
    Rally, Refusal, Reject, Sim, SimEvent, Target, Tick, UNIT_CONTROL_CAP, Vec2,
};

use super::{BARRACKS, P1, P2, PK, at, base, production, tile};

pub(super) fn train(objects: &[ObjectId]) -> Command {
    production(objects, ProductionCommand::Train(PK))
}

pub(super) fn rally(objects: &[ObjectId], target: Target) -> Command {
    production(objects, ProductionCommand::SetRally(target))
}

pub(super) fn crystals(player: PlayerId, crystals: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
    }
}

pub(super) fn bank(sim: &Sim, player: PlayerId) -> u32 {
    sim.player(player).unwrap().crystals
}

pub(super) fn queue(sim: &Sim, id: ObjectId) -> usize {
    sim.object(id).unwrap().production.as_ref().unwrap().queue.len()
}

pub(super) fn rally_of(sim: &Sim, id: ObjectId) -> Option<Rally> {
    sim.object(id).unwrap().production.as_ref().unwrap().rally
}

pub(super) fn validate(sim: &Sim, player: PlayerId, command: Command) -> Result<(), Reject> {
    sim.validate(&PlayerCommand { player, command })
}

/// The units produced so far: `(tick, producer, unit)`.
pub(super) fn produced(t: &TestSim) -> Vec<(Tick, u32, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::UnitProduced { producer, unit } => Some((*tick, producer.0, unit.0)),
            _ => None,
        })
        .collect()
}

/// Plan acceptance, production time: "One Peacekeeper every 80 ticks at full power."
#[test]
fn a_peacekeeper_takes_80_ticks_at_full_power() {
    let mut t = base().start();
    t.debug_at(1, crystals(P1, 500));
    t.at(1, P1, train(&[BARRACKS])).at(1, P1, train(&[BARRACKS]));
    t.run(1);
    assert_eq!(
        (bank(t.sim(), P1), queue(t.sim(), BARRACKS)),
        (400, 2),
        "50 each, paid when queued"
    );
    let status = t.sim().production_status(BARRACKS).unwrap();
    assert_eq!((status.kind, status.progress, status.total), (PK, 1 << 16, 80 << 16));
    t.run_until(79);
    assert_eq!(produced(&t), []);
    t.run_until(200);
    assert_eq!(
        produced(&t).iter().map(|p| p.0).collect::<Vec<_>>(),
        [80, 160],
        "the command's tick counts, and the second starts the tick after the first leaves"
    );
    assert_eq!(t.sim().production_status(BARRACKS), None);
    let units: Vec<&sc_sim::Object> = t.sim().objects().filter(|o| o.kind == PK).collect();
    assert!(units.len() == 2 && units.iter().all(|o| o.owner == Some(P1)));
}

/// Plan acceptance and design change 0089: production is slowed by the power ratio. With the Extraction Facility
/// and two plates the player consumes 51 of its 30: "at ratio 30/51 it takes exactly 136 ticks".
#[test]
fn production_is_slowed_by_the_power_ratio() {
    let mut t = base()
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(14, 10))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(20, 12))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(22, 12))
        .start();
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 51));
    t.debug_at(1, crystals(P1, 100));
    t.at(1, P1, train(&[BARRACKS])).at(1, P1, train(&[BARRACKS]));
    t.run_until(300);
    assert_eq!(produced(&t).iter().map(|p| p.0).collect::<Vec<_>>(), [136, 272]);
}

/// Plan acceptance, train and cancel; design change 0081: "Exactly 20 are accepted: 1 in production and 19
/// waiting."
#[test]
fn train_pays_cancel_refunds_and_the_queue_stops_at_twenty() {
    let mut t = base().start();
    t.debug_at(1, crystals(P1, 2000));
    for _ in 0..25 {
        t.at(2, P1, train(&[BARRACKS]));
    }
    t.run(2);
    assert_eq!(QUEUE_LIMIT, 20);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (20, 1000));
    assert_eq!(validate(t.sim(), P1, train(&[BARRACKS])), Err(Reject::QueueFull));
    assert_eq!(t.sim().unit_control(P1), 20, "design change 0082: reserved when queued");

    // "X: Cancel Production: removes last entry from BuildQueue, refunds full cost to player."
    let cancel = production(&[BARRACKS], ProductionCommand::CancelProduction);
    t.at(3, P1, cancel.clone());
    t.run(1);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (19, 1050));
    assert_eq!(t.sim().unit_control(P1), 19);
    assert!(t.events().contains(&(
        3,
        SimEvent::ProductionCancelled {
            producer: BARRACKS,
            kind: PK,
            refund: 50
        }
    )));
    // The unit in production goes on meanwhile, and is the last to be cancelled.
    assert_eq!(t.sim().production_status(BARRACKS).unwrap().progress, 2 << 16);
    for _ in 0..19 {
        t.at(4, P1, cancel.clone());
    }
    t.run(1);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (0, 2000));
    assert_eq!(t.sim().production_status(BARRACKS), None);
    assert_eq!(validate(t.sim(), P1, cancel), Err(Reject::NothingQueued));
    t.run_until(120);
    assert_eq!(produced(&t), [], "nothing is left to produce");
}

/// Design change 0082's Verification: at 199 of 200 one more Peacekeeper can be queued, the next is refused
/// although crystals are there, and cancelling gives the Unit Control back. (A Peacekeeper's UnitControlCost is 1.)
#[test]
fn unit_control_is_reserved_when_a_unit_is_queued() {
    let mut builder = base();
    for i in 0..199 {
        let (col, row) = (i % 25, i / 25);
        builder = builder.spawn(PK, Some(P1), at(800.0 + 30.0 * col as f32, 500.0 + 30.0 * row as f32));
    }
    let mut t = builder.start();
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP - 1);
    t.debug_at(1, crystals(P1, 1000));
    t.at(2, P1, train(&[BARRACKS])).at(2, P1, train(&[BARRACKS]));
    t.run(2);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (1, 950));
    assert_eq!(
        t.sim().unit_control(P1),
        200,
        "the HUD shows 200/200 as soon as it's queued"
    );
    assert_eq!(validate(t.sim(), P1, train(&[BARRACKS])), Err(Reject::UnitControlCap));
    t.at(3, P1, production(&[BARRACKS], ProductionCommand::CancelProduction));
    t.run(1);
    assert_eq!((t.sim().unit_control(P1), bank(t.sim(), P1)), (199, 1000));
    // Queued again and produced: the unit takes over the Unit Control its queue entry held.
    t.at(4, P1, train(&[BARRACKS]));
    t.run_until(90);
    assert_eq!((produced(&t).len(), t.sim().unit_control(P1)), (1, 200));
    // "When a unit dies, release its Unit Control as usual."
    t.debug_at(91, DebugCommand::Remove { id: ObjectId(10) });
    t.run(1);
    assert_eq!(t.sim().unit_control(P1), 199);
}

/// The gate: a production command that does nothing is rejected, and changes nothing.
#[test]
fn invalid_production_commands_are_rejected_and_change_nothing() {
    let mut t = base()
        .spawn(ObjectKind::Barracks, Some(P2), tile(20, 4))
        .spawn(PK, Some(P1), at(400.0, 700.0))
        .start();
    let (theirs, unit) = (ObjectId(3), ObjectId(4));
    t.debug_at(1, crystals(P1, 49));
    t.run(1);
    let off_map = Vec2::new(-5.0, 100.0);
    let cases = [
        (train(&[BARRACKS]), Reject::InsufficientCrystals),
        (train(&[theirs]), Reject::NotOwned(theirs)),
        (train(&[BARRACKS, theirs]), Reject::NotOwned(theirs)),
        (train(&[unit, ObjectId(1), ObjectId(77)]), Reject::NoProducers),
        (train(&[]), Reject::NoProducers),
        (
            production(&[BARRACKS], ProductionCommand::Train(ObjectKind::Guard)),
            Reject::CannotProduce(ObjectKind::Guard),
        ),
        (
            production(&[BARRACKS], ProductionCommand::Train(ObjectKind::Barracks)),
            Reject::CannotProduce(ObjectKind::Barracks),
        ),
        (
            production(&[BARRACKS], ProductionCommand::CancelProduction),
            Reject::NothingQueued,
        ),
        (rally(&[BARRACKS], Target::Point(off_map)), Reject::OffMap),
        (
            rally(&[BARRACKS], Target::Point(Vec2::new(f32::NAN, 1.0))),
            Reject::OffMap,
        ),
        (
            rally(&[BARRACKS], Target::Object(ObjectId(99))),
            Reject::UnknownObject(ObjectId(99)),
        ),
        // The enemy Barracks is far out of sight and was never seen.
        (rally(&[BARRACKS], Target::Object(theirs)), Reject::NotVisible(theirs)),
        (
            rally(&[unit], Target::Point(Vec2::new(100.0, 100.0))),
            Reject::NoProducers,
        ),
    ];
    let before = t.sim().state_hash();
    for (command, reject) in cases {
        assert_eq!(validate(t.sim(), P1, command.clone()), Err(reject), "{command:?}");
        let mut with = t.sim().clone();
        let mut without = t.sim().clone();
        with.step(&sc_sim::TickInputs {
            commands: vec![PlayerCommand { player: P1, command }],
        });
        without.step(&sc_sim::TickInputs::default());
        assert_eq!(with, without);
        assert!(with.events().is_empty());
    }
    assert_eq!(t.sim().state_hash(), before);
    // A player who isn't in the game.
    assert_eq!(
        validate(t.sim(), PlayerId(9), train(&[BARRACKS])),
        Err(Reject::UnknownPlayer(PlayerId(9)))
    );
    // Stale and repeated ids are dropped: the Barracks queues once.
    t.debug_at(2, crystals(P1, 500));
    t.at(3, P1, train(&[BARRACKS, ObjectId(77), BARRACKS, unit]));
    t.run(2);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (1, 450));
}

/// Three Barracks for design change 0083: X (2) with an empty queue, Y (3) with one Peacekeeper nearly finished, Z
/// (4) with three queued.
fn three_barracks() -> TestSim {
    let mut t = base()
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 11))
        .spawn(ObjectKind::Generator, Some(P1), tile(4, 8))
        .start();
    assert!(!t.sim().power(P1).is_negative());
    let (y, z) = (ObjectId(3), ObjectId(4));
    t.debug_at(1, crystals(P1, 200));
    t.at(1, P1, train(&[y]));
    for _ in 0..3 {
        t.at(1, P1, train(&[z]));
    }
    t.run_until(70);
    assert_eq!(bank(t.sim(), P1), 0);
    t
}

/// Design change 0083's Verification 1: "With crystals for exactly 2 Peacekeepers, press Q. X and Y queue one
/// each, and Z doesn't."
#[test]
fn group_production_serves_the_least_work_first() {
    let mut t = three_barracks();
    let (x, y, z) = (BARRACKS, ObjectId(3), ObjectId(4));
    t.debug_at(71, crystals(P1, 100));
    // Whatever the order of the selection.
    t.at(72, P1, train(&[z, y, x]));
    t.run(2);
    assert_eq!(
        (queue(t.sim(), x), queue(t.sim(), y), queue(t.sim(), z)),
        (1, 2, 3),
        "X and Y queue one each"
    );
    assert_eq!(bank(t.sim(), P1), 0);
    let events: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(tick, _)| *tick == 72)
        .map(|(_, e)| e)
        .collect();
    assert_eq!(
        events,
        [
            &SimEvent::ProductionQueued { producer: x, kind: PK },
            &SimEvent::ProductionQueued { producer: y, kind: PK },
            &SimEvent::ProductionRefused {
                producer: z,
                kind: PK,
                reason: Refusal::Crystals
            },
        ],
        "the idlest pays first, and the one left out is told why"
    );
}

/// Design change 0083's Verification 2, and its tie-break: "With enough crystals for all, every selected Barracks
/// queues one"; with crystals for one of two equally idle Barracks, the lower id.
#[test]
fn group_production_queues_everywhere_when_the_player_can_pay() {
    let mut t = three_barracks();
    let (x, y, z) = (BARRACKS, ObjectId(3), ObjectId(4));
    t.debug_at(71, crystals(P1, 150));
    t.at(72, P1, train(&[x, y, z]));
    t.run(2);
    assert_eq!((queue(t.sim(), x), queue(t.sim(), y), queue(t.sim(), z)), (1, 2, 4));
    assert_eq!(bank(t.sim(), P1), 0);

    let mut tie = base().spawn(ObjectKind::Barracks, Some(P1), tile(8, 8)).start();
    tie.debug_at(1, crystals(P1, 50));
    tie.at(1, P1, train(&[ObjectId(3), BARRACKS]));
    tie.run(1);
    assert_eq!((queue(tie.sim(), BARRACKS), queue(tie.sim(), ObjectId(3))), (1, 0));
}

/// Design change 0083: "This applies to every faction and every cost, including ... GDO Unit Control": with room
/// for one more unit under the cap, the idlest Barracks takes it. A full queue is passed over.
#[test]
fn group_production_counts_unit_control_and_passes_over_full_queues() {
    let mut builder = base().spawn(ObjectKind::Barracks, Some(P1), tile(8, 8));
    for i in 0..198 {
        let (col, row) = (i % 25, i / 25);
        builder = builder.spawn(PK, Some(P1), at(800.0 + 30.0 * col as f32, 500.0 + 30.0 * row as f32));
    }
    let mut t = builder.start();
    let (a, b) = (BARRACKS, ObjectId(3));
    t.debug_at(1, crystals(P1, 5000));
    t.at(1, P1, train(&[a]));
    t.at(2, P1, train(&[a, b]));
    t.run(2);
    assert_eq!(
        (queue(t.sim(), a), queue(t.sim(), b)),
        (1, 1),
        "B, the idle one, took the last"
    );
    assert_eq!(t.sim().unit_control(P1), 200);
    assert!(t.events().contains(&(
        2,
        SimEvent::ProductionRefused {
            producer: a,
            kind: PK,
            reason: Refusal::UnitControl
        }
    )));

    let mut full = base().spawn(ObjectKind::Barracks, Some(P1), tile(8, 8)).start();
    full.debug_at(1, crystals(P1, 5000));
    for _ in 0..20 {
        full.at(1, P1, train(&[a]));
    }
    full.at(2, P1, train(&[a, b]));
    full.run(2);
    assert_eq!((queue(full.sim(), a), queue(full.sim(), b)), (20, 1));
    assert!(full.events().contains(&(
        2,
        SimEvent::ProductionRefused {
            producer: a,
            kind: PK,
            reason: Refusal::QueueFull
        }
    )));
}

/// Design change 0084's Verification 1: "Queue 5 Peacekeepers (250 crystals) at a Barracks, then destroy the
/// Barracks. Crystals go up by 250, and Unit Control drops by the queued units' cost." Destroyed in combat here.
#[test]
fn a_destroyed_barracks_refunds_its_whole_queue() {
    let mut builder = base();
    // Six of player 2's Peacekeepers south of the Barracks, in range of its wall.
    for i in 0..6 {
        builder = builder.spawn(PK, Some(P2), at(530.0 + 30.0 * i as f32, 500.0));
    }
    let mut t = builder.start();
    let attackers: Vec<ObjectId> = (3..9).map(ObjectId).collect();
    t.debug_at(1, crystals(P1, 250));
    for _ in 0..5 {
        t.at(1, P1, train(&[BARRACKS]));
    }
    t.order_at(2, P2, &attackers, OrderKind::Attack(BARRACKS), false);
    t.run(3);
    assert_eq!((bank(t.sim(), P1), t.sim().unit_control(P1)), (0, 5));
    let died = (4..2000).find(|_| {
        t.step();
        t.sim().object(BARRACKS).is_none()
    });
    let died = died.expect("the Barracks is destroyed");
    // Whatever it produced before it died is on the map; the rest is refunded in full.
    let made = produced(&t).len() as u32;
    assert!(made < 5, "it died at tick {died} with units still queued");
    assert_eq!(bank(t.sim(), P1), (5 - made) * 50);
    let refunds = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::ProductionCancelled { refund: 50, .. }))
        .count() as u32;
    assert_eq!(refunds, 5 - made);
    let own_units = t
        .sim()
        .objects()
        .filter(|o| o.kind == PK && o.owner == Some(P1))
        .count() as u32;
    assert_eq!(
        t.sim().unit_control(P1),
        own_units,
        "the queue's Unit Control is released"
    );
    assert_eq!(dangling_references(t.sim()), []);
}

/// A debug removal stands for a destruction (as for the plates): the queue is refunded.
#[test]
fn a_removed_barracks_refunds_its_queue_too() {
    let mut t = base().start();
    t.debug_at(1, crystals(P1, 250));
    for _ in 0..5 {
        t.at(1, P1, train(&[BARRACKS]));
    }
    t.debug_at(40, DebugCommand::Remove { id: BARRACKS });
    t.run(40);
    assert_eq!((bank(t.sim(), P1), t.sim().unit_control(P1)), (250, 0));
}

/// Determinism: a game with production replays to the same hashes, and a save in the middle of it goes on like
/// the game itself.
#[test]
fn a_game_with_production_replays_and_saves() {
    // Two Generators, so that both Barracks work at full power.
    let mut t = base()
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 11))
        .start();
    let both = [BARRACKS, ObjectId(3)];
    t.debug_at(1, crystals(P1, 1000));
    t.at(1, P1, rally(&both, Target::Point(Vec2::new(1300.0, 500.0))));
    for tick in [1, 2, 3, 50] {
        t.at(tick, P1, train(&both));
    }
    t.at(100, P1, production(&both, ProductionCommand::CancelProduction));
    t.at(120, P1, rally(&[BARRACKS], Target::Object(BARRACKS)));
    t.run_until(82);
    let saved: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&saved, t.sim());
    assert_eq!(saved.state_hash(), t.sim().state_hash());
    t.run_until(400);
    // Plan acceptance: "Two Barracks that finish in the same tick spawn in id order, and a replay matches."
    let made = produced(&t);
    assert_eq!(made.len(), 6, "{made:?}");
    assert_eq!(&made[..2], [(80, 2, 6), (80, 3, 7)]);
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
    // The save, played on with the same inputs.
    let mut resumed = saved;
    for inputs in &t.replay().inputs[82..] {
        resumed.step(inputs);
    }
    assert_eq!(resumed.state_hash(), t.sim().state_hash());
}
