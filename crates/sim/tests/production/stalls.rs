//! Blocked doorways (`core/entities.md > Structure Type > Exit Doorway`, points 3 and 5; design change 0085):
//! production pauses while a unit stands in the doorway and resumes when it is free; saves in the middle of an exit;
//! and a long random game around the doorways.

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerId, ProductionCommand, Rng, Rotation, Sim, Target, Vec2,
};

use super::doors::{OUTSIDE, exit_of, produced, queue, rally, run_checked, step_checked, waiting};
use super::{BARRACKS, P1, P2, PK, at, base, flat, production, tile};

/// Design change 0080's Verification 3: "Place a building flush against side B. The first unit appears in the
/// doorway and stays there. The next unit waits, and the queue doesn't advance. Destroy the blocking building, and
/// the unit walks out and production resumes."
#[test]
fn a_building_against_the_exit_side_stalls_production_until_it_is_gone() {
    // A Generator on tiles (11, 4) to (12, 5): x 704..832, y 256..384, flush against side B.
    let mut t = base().spawn(ObjectKind::Generator, Some(P1), tile(11, 4)).start();
    let generator = ObjectId(3);
    queue(&mut t, BARRACKS, 3);
    rally(&mut t, 1, BARRACKS, Vec2::new(1300.0, 320.0));
    run_checked(&mut t, 80);
    let (_, first) = produced(&t)[0];
    let inside = t.pos(first.0);
    assert_eq!(inside, Vec2::new(692.0, 320.0), "in the doorway, fully inside");
    run_checked(&mut t, 500);
    assert_eq!(
        (t.pos(first.0), exit_of(t.sim(), first)),
        (inside, Some(BARRACKS)),
        "it stays there"
    );
    assert_eq!(produced(&t).len(), 1, "the next unit waits");
    // Finished and held at 100% since tick 160; the queue behind it hasn't moved.
    let status = t.sim().production_status(BARRACKS).unwrap();
    assert!(status.waiting && status.progress == status.total);
    assert_eq!(
        t.sim()
            .object(BARRACKS)
            .unwrap()
            .production
            .as_ref()
            .unwrap()
            .queue
            .len(),
        2
    );
    assert!(waiting(t.sim(), BARRACKS));
    assert_eq!(
        t.sim().unit_control(P1),
        2,
        "one on the map and one finished inside; the one queued behind takes none (design change 0143)"
    );

    t.debug_at(501, DebugCommand::Remove { id: generator });
    run_checked(&mut t, 520);
    assert_eq!(exit_of(t.sim(), first), None, "it walks out");
    let made = produced(&t);
    assert_eq!(made.len(), 2, "production resumes");
    assert!(
        made[1].0 > 501 && made[1].0 <= 506,
        "as soon as the doorway is free: tick {}",
        made[1].0
    );
    run_checked(&mut t, 700);
    let made = produced(&t);
    assert_eq!(made.len(), 3);
    assert_eq!(made[2].0, made[1].0 + 80, "the third unit takes its 80 ticks from then");
    // Its move order had failed in the doorway, so it only walked out, and then made way for the others.
    assert!(t.sim().object(first).unwrap().orders.as_ref().unwrap().is_idle());
    assert!(t.pos(first.0).distance(OUTSIDE) < 60.0, "{}", t.pos(first.0));
    // The two after it were never stuck: their rally order stands.
    for (_, unit) in &made[1..] {
        assert!(t.pos(unit.0).x > 1000.0, "{}", t.pos(unit.0));
        assert!(!t.sim().object(*unit).unwrap().orders.as_ref().unwrap().is_idle() || t.pos(unit.0).x > 1200.0);
    }
}

/// Design change 0085's Verification: with a unit stuck in the doorway, "let the next Peacekeeper reach 100%, then
/// destroy the Barracks. Its cost is refunded. The Peacekeeper that was in the doorway remains on the map."
#[test]
fn a_finished_unit_waiting_inside_is_refunded_with_the_queue() {
    let mut t = base().spawn(ObjectKind::Generator, Some(P1), tile(11, 4)).start();
    queue(&mut t, BARRACKS, 4);
    run_checked(&mut t, 200);
    let (_, first) = produced(&t)[0];
    assert!(waiting(t.sim(), BARRACKS));
    let bank = t.sim().player(P1).unwrap().crystals;
    assert_eq!(bank, 10_000 - 200);
    t.debug_at(201, DebugCommand::Remove { id: BARRACKS });
    run_checked(&mut t, 230);
    assert_eq!(
        t.sim().player(P1).unwrap().crystals,
        bank + 150,
        "the waiting unit and the two behind it"
    );
    assert_eq!(t.sim().unit_control(P1), 1);
    // The unit that stood in the doorway is an ordinary unit on open ground now.
    let o = t.sim().object(first).unwrap();
    assert_eq!((o.pos, exit_of(t.sim(), first)), (Vec2::new(692.0, 320.0), None));
    assert_eq!(dangling_references(t.sim()), []);
    t.order_at(
        231,
        P1,
        &[first],
        OrderKind::Move(Target::Point(Vec2::new(600.0, 600.0))),
        false,
    );
    run_checked(&mut t, 330);
    assert_eq!(t.pos(first.0), Vec2::new(600.0, 600.0));
}

/// A Barracks walled in by units that don't make way (own units on Hold never yield): the first unit stays in the
/// doorway, production pauses, and both go on once the wall walks off.
#[test]
fn a_wall_of_units_stalls_production_until_it_moves_away() {
    let mut builder = base();
    // Six Peacekeepers flush against side B, covering y 248..392, more than the side.
    for i in 0..6 {
        builder = builder.spawn(PK, Some(P1), at(716.0, 260.0 + 24.0 * i as f32));
    }
    let mut t = builder.start();
    let wall: Vec<ObjectId> = (3..9).map(ObjectId).collect();
    t.order_at(1, P1, &wall, OrderKind::Hold, false);
    queue(&mut t, BARRACKS, 3);
    run_checked(&mut t, 600);
    let made = produced(&t);
    assert_eq!(made.len(), 1);
    let first = made[0].1;
    assert_eq!(exit_of(t.sim(), first), Some(BARRACKS));
    assert_eq!(
        t.pos(first.0),
        Vec2::new(692.0, 320.0),
        "fully inside, against the units in front"
    );
    assert!(waiting(t.sim(), BARRACKS), "the second unit is finished and held");
    for unit in &wall {
        assert_eq!(t.pos(unit.0).x, 716.0, "units on Hold don't step aside");
    }
    // A blocked doorway pauses production deterministically.
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());

    t.order_at(
        601,
        P1,
        &wall,
        OrderKind::Move(Target::Point(Vec2::new(1300.0, 800.0))),
        false,
    );
    run_checked(&mut t, 900);
    let made = produced(&t);
    assert_eq!(made.len(), 3, "the doorway was freed");
    assert!(made[1].0 < 640 && made[2].0 == made[1].0 + 80, "{made:?}");
    for (_, unit) in made {
        assert_eq!(exit_of(t.sim(), unit), None);
    }
}

/// A save with a unit in a doorway and one waiting inside goes on like the game itself.
#[test]
fn a_save_in_the_middle_of_an_exit_plays_on_alike() {
    let mut t = base()
        .spawn(PK, Some(P1), at(716.0, 320.0))
        .spawn(PK, Some(P1), at(716.0, 344.0))
        .start();
    queue(&mut t, BARRACKS, 4);
    rally(&mut t, 200, BARRACKS, Vec2::new(1200.0, 900.0));
    for save_at in [81, 162, 243] {
        t.run_until(save_at);
        let saved: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
        assert_eq!(&saved, t.sim());
        assert!(
            saved
                .objects()
                .any(|o| o.mover.as_ref().is_some_and(|m| m.exit.is_some())),
            "tick {save_at}: a unit is in the doorway"
        );
        let mut resumed = saved;
        let mut game = t.sim().clone();
        for _ in 0..120 {
            resumed.step(&Default::default());
            game.step(&Default::default());
            assert_eq!(resumed.state_hash(), game.state_hash());
        }
    }
    t.run_until(500);
    assert_eq!(produced(&t).len(), 4);
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// A long random game around two bases' doorways: production commands of every kind, orders to the produced
/// units, buildings put in front of exits and removed again, units and producers removed. The invariants hold in
/// every tick (the doorway's included), nothing dangles, units keep coming out, and the game replays.
#[test]
fn a_random_game_around_the_doorways_keeps_every_invariant() {
    let mut scenario = ScenarioBuilder::new(&flat(30, 16))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 4))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 9))
        .spawn(ObjectKind::Barracks, Some(P1), tile(14, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 11))
        .spawn(ObjectKind::Generator, Some(P1), tile(4, 8))
        .spawn(ObjectKind::DeploymentCenter, Some(P2), tile(25, 11))
        .spawn(ObjectKind::Barracks, Some(P2), tile(21, 4))
        .build();
    // Exits to the south, the north and the west.
    scenario.map.spawns[2].orientation.rotation = Rotation::R90;
    scenario.map.spawns[3].orientation.rotation = Rotation::R270;
    scenario.map.spawns[8].orientation.rotation = Rotation::R180;
    let mut t = TestSim::new(scenario);
    let producers = [ObjectId(2), ObjectId(3), ObjectId(4), ObjectId(9)];
    // A game with many stalls in the doorways. (Design change 0169's arrival radius of 3 grid units changed the
    // game of seed 0xd00a, which then had none.)
    let mut rng = Rng::from_seed(0xd00d);
    let mut out = Vec::new();
    let (mut waits, mut in_doorways) = (0u32, 0u32);
    for tick in 1..=3000u32 {
        let units: Vec<(ObjectId, PlayerId)> = t
            .sim()
            .objects()
            .filter(|o| o.mover.is_some())
            .filter_map(|o| Some((o.id, o.owner?)))
            .collect();
        let highest = t.sim().objects().last().map_or(1, |o| o.id.0);
        let any_id = |rng: &mut Rng| ObjectId(rng.below(highest + 2));
        let point = |rng: &mut Rng| Vec2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1024.0);
        let player = if rng.below(4) == 0 { P2 } else { P1 };
        let producer = producers[rng.below(4) as usize];
        if tick % 200 == 1 {
            for player in [P1, P2] {
                t.debug_at(
                    tick,
                    DebugCommand::SetResources {
                        player,
                        crystals: 400,
                        supplies: 0,
                        alloys: 0,
                        essence: 0,
                        flux: 0,
                    },
                );
            }
        }
        match rng.below(40) {
            0..=5 => {
                let both = [producer, producers[rng.below(4) as usize]];
                t.at(tick, player, production(&both, ProductionCommand::Train(PK)));
            }
            6 => {
                t.at(
                    tick,
                    player,
                    production(&[producer], ProductionCommand::CancelProduction),
                );
            }
            7 => {
                let target = Target::Point(point(&mut rng));
                t.at(
                    tick,
                    player,
                    production(&[producer], ProductionCommand::SetRally(target)),
                );
            }
            8 => {
                let target = Target::Object(any_id(&mut rng));
                t.at(
                    tick,
                    player,
                    production(&[producer], ProductionCommand::SetRally(target)),
                );
            }
            9 | 10 if !units.is_empty() => {
                let (unit, owner) = units[rng.below(units.len() as u32) as usize];
                let kind = match rng.below(4) {
                    0 => OrderKind::Hold,
                    1 => OrderKind::Stop,
                    2 => OrderKind::Move(Target::Object(any_id(&mut rng))),
                    _ => OrderKind::Move(Target::Point(point(&mut rng))),
                };
                t.order_at(tick, owner, &[unit], kind, false);
            }
            11 => {
                // A plate or a Generator somewhere near the doorways: often refused, sometimes in the way.
                let kind = [ObjectKind::ExtractionPlate, ObjectKind::Generator][rng.below(2) as usize];
                let at = tile(7 + rng.below(18) as i32, 2 + rng.below(13) as i32);
                t.debug_at(
                    tick,
                    DebugCommand::Spawn {
                        kind,
                        owner: Some(P1),
                        at,
                        orientation: Default::default(),
                    },
                );
            }
            12 => {
                // Anything but the producers and the Deployment Centers goes.
                let id = any_id(&mut rng);
                if id.0 > 9 || (5..=7).contains(&id.0) {
                    t.debug_at(tick, DebugCommand::Remove { id });
                }
            }
            13 if tick > 2400 && rng.below(8) == 0 => {
                t.debug_at(tick, DebugCommand::Remove { id: producer });
            }
            _ => {}
        }
        step_checked(&mut t, &mut out);
        assert_eq!(dangling_references(t.sim()), [], "tick {tick}");
        waits += u32::from(producers.iter().any(|&p| waiting(t.sim(), p)));
        in_doorways += t
            .sim()
            .objects()
            .filter(|o| o.mover.as_ref().is_some_and(|m| m.exit.is_some()))
            .count() as u32;
    }
    let made = produced(&t).len();
    assert!(made > 40, "{made} units produced");
    assert!(
        waits > 50 && in_doorways > 200,
        "{waits} ticks with a unit held, {in_doorways} unit-ticks in doorways"
    );
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
}
