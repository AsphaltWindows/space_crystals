//! The gate of the construction commands (`ARCHITECTURE.md > Commands, events and replays`): the commands it
//! refuses, a command to several constructors carried out in id order (design change 0071), and the gate
//! fuzzed.

use super::*;

/// Checks that the gate rejects `player`'s `command` for `reason`, then runs the next tick with it and checks that
/// it changed nothing: the state and the events are those of the same tick without it.
fn assert_rejected(t: &mut TestSim, player: PlayerId, command: Command, reason: Reject) {
    let cmd = PlayerCommand {
        player,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Err(reason), "{command:?}");
    let mut without = t.sim().clone();
    without.step(&TickInputs::default());
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command).step();
    assert_eq!(t.sim().state_hash(), without.state_hash(), "{reason}");
    assert_eq!(t.sim(), &without, "{reason}");
}

/// The plan's acceptance, refusals: Construct is refused while the constructor constructs or holds a ready building,
/// for a building not on its list and without the crystals; the cancels and Place without their state; anyone
/// else's objects, none of the player's constructors, and two constructors for one Place. The bank stays as it was,
/// and so does everything else.
#[test]
fn refused_construction_commands_change_nothing() {
    let mut t = base()
        .spawn(DeploymentCenter, Some(P2), tile(16, 8)) // 5
        .start();
    t.debug_at(1, fund(P1, 150)).run_until(1);
    let dc = |command| construction(&[1], command);
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::InsufficientCrystals);
    assert_rejected(&mut t, P1, dc(CancelConstruction), Reject::NotConstructing);
    assert_rejected(&mut t, P1, dc(CancelReady), Reject::NothingReady);
    assert_rejected(&mut t, P1, dc(place(6, 5)), Reject::NothingReady);
    next(&mut t, dc(Construct(Generator)));
    assert_eq!(bank(&t, P1), 50);
    // Busy, whatever the bank holds.
    next(&mut t, Command::Debug(fund(P1, 1000)));
    assert_rejected(&mut t, P1, dc(Construct(Generator)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(CancelReady), Reject::NothingReady);
    assert_rejected(&mut t, P1, dc(place(6, 5)), Reject::NothingReady);
    until_ready(&mut t, 1);
    assert_rejected(&mut t, P1, dc(Construct(Barracks)), Reject::ConstructorBusy(id(1)));
    assert_rejected(&mut t, P1, dc(CancelConstruction), Reject::NotConstructing);
    // Not on the list.
    let ef = |command| construction(&[2], command);
    assert_rejected(&mut t, P1, ef(Construct(Generator)), Reject::CannotConstruct(Generator));
    assert_rejected(
        &mut t,
        P1,
        ef(Construct(Peacekeeper)),
        Reject::CannotConstruct(Peacekeeper),
    );
    assert_rejected(
        &mut t,
        P1,
        dc(Construct(ExtractionPlate)),
        Reject::CannotConstruct(ExtractionPlate),
    );
    // Whose objects, and which.
    assert_rejected(&mut t, P2, dc(CancelReady), Reject::NotOwned(id(1)));
    assert_rejected(
        &mut t,
        P1,
        construction(&[5], Construct(Generator)),
        Reject::NotOwned(id(5)),
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[2, 3], Construct(ExtractionPlate)),
        Reject::NotOwned(id(3)),
    );
    assert_rejected(&mut t, P1, construction(&[4], CancelReady), Reject::NoConstructor);
    assert_rejected(&mut t, P1, construction(&[99, 4], CancelReady), Reject::NoConstructor);
    assert_rejected(
        &mut t,
        P1,
        construction(&[], Construct(Generator)),
        Reject::NoConstructor,
    );
    assert_rejected(
        &mut t,
        P1,
        construction(&[1, 2], place(6, 5)),
        Reject::SeveralConstructors,
    );
    assert_rejected(&mut t, PlayerId(9), dc(CancelReady), Reject::UnknownPlayer(PlayerId(9)));
    let many = vec![1; MAX_ORDER_UNITS + 1];
    assert_rejected(&mut t, P1, construction(&many, CancelReady), Reject::TooManyUnits);
    // Repeated ids count once, and ids that are gone are dropped.
    next(&mut t, construction(&[1, 1, 99], CancelReady));
    assert_eq!(bank(&t, P1), 1000 + 75);
}

/// A command goes to every constructor it names that can carry it out, in id order (design change 0071: no
/// smart-casting), each paying from what the ones before it left: with 150 crystals two idle Deployment Centers start
/// one Generator, the lower id's.
#[test]
fn a_command_goes_to_each_constructor_that_can_in_id_order() {
    let mut t = ScenarioBuilder::new(&flat(24, 10))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(ExtractionFacility, Some(P1), tile(11, 1)) // 3
        .start();
    t.debug_at(1, fund(P1, 150)).run_until(1);
    next(&mut t, construction(&[2, 1], Construct(Generator)));
    assert_eq!(
        (state(&t, 1).current(), state(&t, 2).current()),
        (Some(Generator), None)
    );
    assert_eq!(bank(&t, P1), 50);
    // One busy, the other one starts; the Extraction Facility can't build a Generator and is passed over.
    next(&mut t, Command::Debug(fund(P1, 300)));
    next(&mut t, construction(&[1, 2, 3], Construct(Generator)));
    assert_eq!(state(&t, 2).current(), Some(Generator));
    assert_eq!(bank(&t, P1), 200);
    // A cancel goes to all of them that have something to cancel.
    next(&mut t, construction(&[3, 2, 1], CancelConstruction));
    assert!(state(&t, 1).is_idle() && state(&t, 2).is_idle());
    assert_eq!(bank(&t, P1), 400);
    // Each a building of its own list.
    next(&mut t, construction(&[1, 3], Construct(ExtractionPlate)));
    assert_eq!(
        (state(&t, 1).current(), state(&t, 3).current()),
        (None, Some(ExtractionPlate))
    );
    assert_eq!(bank(&t, P1), 325);
}

/// The gate (`ARCHITECTURE.md > Commands, events and replays`), fuzzed: random construction commands from any player
/// (in the game or not) to any ids, of any building and anywhere, never panic; a rejected one changes nothing, but a
/// Place the placement check refuses, which only reports `PlacementRefused`. Each command runs alone in a tick, next
/// to a copy of the game that runs the same tick without it. Twice from the same seed, the runs are identical, and
/// the recorded commands replay to the same state.
#[test]
fn random_construction_commands_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = construction_fuzz(0xc0de);
    // The stream exercises the valid paths too, not only rejections.
    assert!(counts.accepted > 120 && counts.rejected > 3000, "{counts:?}");
    assert!(
        counts.placed > 10 && counts.refused > 100 && counts.completed > 20,
        "{counts:?}"
    );
    let (_, again, _) = construction_fuzz(0xc0de);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    assert_eq!(replay.run().state_hash(), last);
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// `SimEvent::StructurePlaced`.
    placed: usize,
    /// `SimEvent::PlacementRefused`.
    refused: usize,
    /// `SimEvent::ConstructionCompleted`.
    completed: usize,
}

/// Both players' bases on a 32 × 16 map: player 1's Deployment Center 1 and Extraction Facility 2 in the west,
/// player 2's 3 and 4 in the east, patches 5 to 8, two by each base.
fn fuzz_base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(32, 16))
        .spawn(DeploymentCenter, Some(P1), tile(4, 2))
        .spawn(ExtractionFacility, Some(P1), tile(4, 9))
        .spawn(DeploymentCenter, Some(P2), tile(24, 2))
        .spawn(ExtractionFacility, Some(P2), tile(24, 9))
        .spawn(CrystalPatch, None, tile(9, 3))
        .spawn(CrystalPatch, None, tile(9, 6))
        .spawn(CrystalPatch, None, tile(22, 3))
        .spawn(CrystalPatch, None, tile(22, 6))
        .config(SimConfig::DEBUG)
}

fn random_construction(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    // Mostly one of the player's constructors, sometimes two; else any ids, some of which don't exist (any more, or
    // yet).
    let own: Vec<ObjectId> = sim
        .objects()
        .filter(|o| o.owner == Some(player) && o.constructor.is_some())
        .map(|o| o.id)
        .collect();
    let pick = |rng: &mut Rng| own[rng.below(own.len() as u32) as usize];
    let objects = if !own.is_empty() && rng.below(4) != 0 {
        let extra = rng.below(4) == 0;
        let mut objects = vec![pick(rng)];
        if extra {
            objects.push(pick(rng));
        }
        objects
    } else {
        let ids = sim.objects().map(|o| o.id.0).max().unwrap_or(0) + 3;
        (0..rng.below(3) + 1).map(|_| ObjectId(rng.below(ids) + 1)).collect()
    };
    // Few cancels, so that constructions finish and get placed.
    let command = match rng.below(40) {
        0..14 => {
            let kinds = [Generator, Barracks, ExtractionFacility, ExtractionPlate];
            let kind = if rng.below(4) == 0 {
                ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]
            } else {
                kinds[rng.below(4) as usize]
            };
            Construct(kind)
        }
        14 => CancelConstruction,
        15 => CancelReady,
        _ => {
            // Mostly in sight of the player's Deployment Center or on one of its patches, sometimes anywhere or off
            // the map.
            let (x0, patches) = if player == P2 {
                (20, [(22, 3), (22, 6)])
            } else {
                (0, [(9, 3), (9, 6)])
            };
            let (x, y) = match rng.below(6) {
                0 => (rng.below(40) as i32 - 4, rng.below(24) as i32 - 4),
                1 => patches[rng.below(2) as usize],
                _ => (x0 + rng.below(12) as i32, rng.below(11) as i32),
            };
            let orientation = BuildingOrientation {
                rotation: [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270][rng.below(4) as usize],
                flip_h: rng.below(2) == 0,
                flip_v: rng.below(2) == 0,
            };
            Place {
                origin: at(x, y),
                orientation,
            }
        }
    };
    PlayerCommand {
        player,
        command: Command::Construction { objects, command },
    }
}

/// 6000 ticks of one random construction command each, after a first tick that fills both banks: what happened,
/// the state hash of every tick, and the replay.
fn construction_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let scenario = fuzz_base().build();
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    let funds = TickInputs {
        commands: [P1, P2]
            .map(|player| PlayerCommand {
                player: P1,
                command: Command::Debug(fund(player, 100_000)),
            })
            .to_vec(),
    };
    sim.step(&funds);
    replay.record(funds);
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..6000 {
        let cmd = random_construction(&mut rng, &sim);
        let verdict = sim.validate(&cmd);
        let mut without = sim.clone();
        without.step(&TickInputs::default());
        let inputs = TickInputs {
            commands: vec![cmd.clone()],
        };
        sim.step(&inputs);
        replay.record(inputs);
        match verdict {
            Ok(()) => counts.accepted += 1,
            Err(reason) => {
                counts.rejected += 1;
                assert_eq!(sim.state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                let reported = match (&cmd.command, reason) {
                    (
                        Command::Construction {
                            command: Place { .. }, ..
                        },
                        Reject::Unplaceable(_),
                    ) => 1,
                    _ => 0,
                };
                assert_eq!(&sim.events()[reported..], without.events(), "{cmd:?}: {reason}");
                assert_eq!(sim.events().len(), without.events().len() + reported);
            }
        }
        for e in sim.events() {
            match e {
                SimEvent::StructurePlaced { .. } => counts.placed += 1,
                SimEvent::PlacementRefused { .. } => counts.refused += 1,
                SimEvent::ConstructionCompleted { .. } => counts.completed += 1,
                _ => {}
            }
        }
        assert_eq!(dangling_references(&sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, replay)
}
