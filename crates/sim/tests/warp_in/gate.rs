//! The gate of a Warp In (`Sim::validate`): its rules besides the footprint's, a Beacon that is itself
//! transforming, the gate in agreement with `Sim::warp_in_check`, and the gate fuzzed.

use super::*;

/// The gate's other rules for a Warp In (`Sim::validate`): one Beacon of the player's ([`Reject::NoConstructor`],
/// [`Reject::SeveralConstructors`], [`Reject::NotOwned`]), a building on its Constructs List
/// ([`Reject::CannotConstruct`]), and a bank that covers the cost as the command applies, after the earlier commands
/// of the tick ([`Reject::InsufficientCrystals`]). A refused command changes nothing (a refused footprint, which is
/// also reported, is the footprint tests').
#[test]
fn refused_warp_ins_change_nothing() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 249)); // one short of the Robo Portal's 250 (design change 0167)
    t.step();
    let robo = |objects: &[u32]| construction(objects, warp(RoboPortal));
    assert_rejected(&mut t, P1, robo(&[]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[3]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[99]), Reject::NoConstructor);
    assert_rejected(&mut t, P1, robo(&[1, 2]), Reject::SeveralConstructors);
    assert_rejected(&mut t, P1, robo(&[1, 4]), Reject::NotOwned(id(4)));
    assert_rejected(&mut t, P2, robo(&[1]), Reject::NotOwned(id(1)));
    for kind in [ObjectKind::Barracks, Beacon, Generator] {
        let command = construction(&[1], warp(kind));
        assert_rejected(&mut t, P1, command, Reject::CannotConstruct(kind));
    }
    assert_rejected(&mut t, P1, robo(&[1]), Reject::InsufficientCrystals);
    // Two Warp Ins in one tick, a bank for one: the second Beacon's is refused, and it stays a Beacon.
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, fund(P1, 200))
        .at(tick, P1, construction(&[1], warp(BioPortal)))
        .at(tick, P1, construction(&[2], warp(EgressNode)))
        .step();
    assert_eq!((object(&t, 1).kind, object(&t, 2).kind), (BioPortal, Beacon));
    assert_eq!(bank(&t, P1), 50, "the Egress Node's 75 wasn't there any more");
}

/// A Beacon that is itself transforming (being deployed, slice M3-F) warps nothing in ([`Reject::ConstructorBusy`]).
/// The test edits a saved game: Beacon 1 still deploying from a Drone (`tests/deploy/` deploys them by command).
#[test]
fn a_transforming_beacon_warps_nothing_in() {
    let mut t = base().start();
    t.debug_at(1, fund(P1, 1000));
    t.step();
    let deploying = Transform {
        from: Drone,
        from_placement: None,
        progress: Progress {
            elapsed: 10 * RATIO_ONE,
            total: 48,
        },
        paid: 0,
        load: 0,
    };
    let sim = edited(t.sim(), 1, "transform", serde_json::to_value(&deploying).unwrap());
    assert!(sim.object(id(1)).unwrap().is_transforming());
    let cmd = PlayerCommand {
        player: P1,
        command: construction(&[1], warp(EgressNode)),
    };
    assert_eq!(sim.validate(&cmd), Err(Reject::ConstructorBusy(id(1))));
}

/// The plan's acceptance, "Consistency": the gate refuses a Warp In for its footprint exactly when
/// `Sim::warp_in_check` finds a problem, with its first one, for both of player 1's Beacons, every building and every
/// orientation, with a unit, a patch and a structure about that block some of the footprints: Beacon 1's Robo Portal
/// and three corners of its Bio Portal, Beacon 2's Robo Portal and two corners of its Bio Portal.
#[test]
fn the_gate_and_the_warp_in_check_agree() {
    let mut t = base()
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(at(7, 6).center()))
        .spawn(CrystalPatch, None, tile(10, 8))
        .spawn(Generator, Some(P2), tile(17, 8))
        .start();
    t.debug_at(1, fund(P1, 1000));
    t.step();
    let (mut valid, mut refused) = (0, 0);
    for beacon in [1, 2] {
        for kind in [EgressNode, RoboPortal, BioPortal] {
            for orientation in every_orientation() {
                let check = t.sim().warp_in_check(id(beacon), kind, orientation).unwrap();
                let cmd = PlayerCommand {
                    player: P1,
                    command: construction(&[beacon], WarpIn { kind, orientation }),
                };
                let verdict = t.sim().validate(&cmd);
                match check.first_problem() {
                    None => {
                        valid += 1;
                        assert_eq!(verdict, Ok(()), "{beacon}: {kind:?} {orientation:?}");
                    }
                    Some(problem) => {
                        refused += 1;
                        let expected = Err(Reject::Unplaceable(problem));
                        assert_eq!(verdict, expected, "{beacon}: {kind:?} {orientation:?}");
                    }
                }
            }
        }
    }
    assert_eq!((valid, refused), (16 + 4 + 16 + 8, 16 + 12 + 16 + 8));
}

/// The gate, fuzzed: random Warp Ins and cancels from any player (in the game or not) to any ids, of any kind,
/// turned anyhow, with Beacons spawned anywhere and objects removed now and then, never panic; a rejected one
/// changes nothing, but a footprint the placement check refuses, which only reports `PlacementRefused`. Each command
/// runs alone in a tick, next to a copy of the game that runs the same tick without it. Twice from the same seed, the
/// runs are identical, and the recorded commands replay to the same state.
#[test]
fn random_warp_ins_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = warp_in_fuzz(0xbeac);
    assert!(
        counts.accepted > 300 && counts.rejected > 1000 && counts.refused > 50,
        "{counts:?}"
    );
    assert!(counts.completed > 10 && counts.cancelled > 50, "{counts:?}");
    let (_, again, _) = warp_in_fuzz(0xbeac);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// `SimEvent::PlacementRefused`.
    refused: usize,
    /// `SimEvent::TransformCompleted`.
    completed: usize,
    /// `SimEvent::TransformCancelled`.
    cancelled: usize,
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    let own: Vec<&Object> = sim.objects().filter(|o| o.owner == Some(player)).collect();
    let beacons: Vec<&Object> = own.iter().copied().filter(|o| o.kind == Beacon).collect();
    let turned = |rng: &mut Rng| BuildingOrientation {
        rotation: [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270][rng.below(4) as usize],
        flip_h: rng.below(2) == 0,
        flip_v: rng.below(2) == 0,
    };
    // Few cancels and removals, so that Warp Ins get to complete.
    let command = match rng.below(40) {
        // Now and then a new Beacon somewhere, or an object removed (a destruction, to the removal path).
        0..2 => {
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Beacon,
                    owner: Some(PlayerId(1 + rng.below(2) as u8)),
                    at: tile(rng.below(22) as i32, rng.below(14) as i32),
                    orientation: BuildingOrientation::default(),
                }),
            };
        }
        2 => {
            let ids = sim.objects().map(|o| o.id.0).max().unwrap_or(0) + 1;
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove {
                    id: ObjectId(rng.below(ids) + 1),
                }),
            };
        }
        3..6 => {
            // Mostly one of the player's objects, sometimes all of them, else any id.
            let objects = match rng.below(8) {
                _ if own.is_empty() => vec![ObjectId(rng.below(30) + 1)],
                0 => own.iter().map(|o| o.id).collect(),
                1 => vec![ObjectId(rng.below(30) + 1)],
                _ => vec![own[rng.below(own.len() as u32) as usize].id],
            };
            Command::Construction {
                objects,
                command: CancelTransform,
            }
        }
        _ => {
            let kind = if rng.below(6) == 0 {
                ObjectKind::ALL[rng.below(ObjectKind::ALL.len() as u32) as usize]
            } else {
                [EgressNode, RoboPortal, BioPortal][rng.below(3) as usize]
            };
            let objects = match beacons.is_empty() || rng.below(5) == 0 {
                true => vec![ObjectId(rng.below(30) + 1)],
                false => vec![beacons[rng.below(beacons.len() as u32) as usize].id],
            };
            Command::Construction {
                objects,
                command: WarpIn {
                    kind,
                    orientation: turned(rng),
                },
            }
        }
    };
    PlayerCommand { player, command }
}

/// 4000 ticks of one random command each on the base, after a first tick that fills both banks: what happened, the
/// state hash of every tick, and the replay.
fn warp_in_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let scenario = base().config(SimConfig::DEBUG).build();
    let mut sim = Sim::new(&scenario);
    let mut replay = Replay::new(scenario);
    let funds = TickInputs {
        commands: [P1, P2]
            .map(|player| PlayerCommand {
                player: P1,
                command: Command::Debug(fund(player, 1_000_000)),
            })
            .to_vec(),
    };
    sim.step(&funds);
    replay.record(funds);
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..4000 {
        let cmd = random_command(&mut rng, &sim);
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
                            command: WarpIn { .. }, ..
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
                SimEvent::PlacementRefused { .. } => counts.refused += 1,
                SimEvent::TransformCompleted { .. } => counts.completed += 1,
                SimEvent::TransformCancelled { .. } => counts.cancelled += 1,
                _ => {}
            }
        }
        // A transforming object is always the building of its Warp In, on the footprint of its Beacon's.
        for o in sim.objects().filter(|o| o.is_transforming()) {
            let (w, p) = (o.transform.as_ref().unwrap(), o.placement.unwrap());
            let from = w.from_placement.unwrap();
            assert_eq!(
                warp_in_footprint(&from, o.kind, p.orientation()),
                Some(p),
                "#{}: over {from:?}",
                o.id.0
            );
            assert_eq!(w.from, Beacon);
            assert!(
                o.production.is_none() && o.constructor.is_none(),
                "#{}: held back",
                o.id.0
            );
        }
        assert_eq!(dangling_references(&sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, replay)
}
