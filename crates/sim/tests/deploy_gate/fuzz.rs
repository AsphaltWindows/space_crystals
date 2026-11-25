//! The gate and the arrival, fuzzed: random Deploys among other commands never panic, and a rejected one
//! changes nothing.

use super::*;

/// What a fuzz run did.
#[derive(Debug, Default)]
struct Counts {
    accepted: usize,
    rejected: usize,
    /// Drones that began deploying.
    started: usize,
    /// `SimEvent::DeployFailed`.
    failed: usize,
    completed: usize,
    cancelled: usize,
}

/// The gate and the arrival, fuzzed: random Deploys, Shift or not, from any player (in the game or not) to any ids,
/// one Drone, several or none, with any origin, among moves, stops, cancels, new Drones and removals, never panic; a
/// rejected one changes nothing, events included. Each command runs alone in a tick, next to a copy of the game that
/// runs the same tick without it. A transforming object is always a Beacon deploying from a Drone on a 2 × 2 site,
/// with nothing but its HP; the pending Deploys are the players' own Drones'. Twice from the same seed, the runs are
/// identical, and the recorded commands replay to the same state.
#[test]
fn random_deploys_never_panic_and_rejections_change_nothing() {
    let (counts, hashes, replay) = deploy_fuzz(0xde9_10ce_u64);
    assert!(
        counts.accepted > 400 && counts.rejected > 1500 && counts.failed > 20,
        "{counts:?}"
    );
    assert!(
        counts.started > 60 && counts.completed > 10 && counts.cancelled > 25,
        "{counts:?}"
    );
    let (_, again, _) = deploy_fuzz(0xde9_10ce_u64);
    assert_eq!(hashes, again);
    let last = *hashes.last().unwrap();
    let file: Replay = serde_json::from_str(&serde_json::to_string(&replay).unwrap()).unwrap();
    assert_eq!(file.run().state_hash(), last);
}

fn random_command(rng: &mut Rng, sim: &Sim) -> PlayerCommand {
    let player = PlayerId([1, 1, 1, 2, 2, 2, 0, 3][rng.below(8) as usize]);
    let own: Vec<&Object> = sim.objects().filter(|o| o.owner == Some(player)).collect();
    let drones: Vec<ObjectId> = own.iter().filter(|o| o.kind == Drone).map(|o| o.id).collect();
    let deploying: Vec<ObjectId> = own.iter().filter(|o| o.is_deploying()).map(|o| o.id).collect();
    let any = |rng: &mut Rng| ObjectId(rng.below(40) + 1);
    let pick = |rng: &mut Rng, ids: &[ObjectId]| ids[rng.below(ids.len() as u32) as usize];
    let command = match rng.below(40) {
        0..2 => {
            let at = Vec2::new(rng.below(24 * 64) as f32, rng.below(16 * 64) as f32);
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Drone,
                    owner: Some(PlayerId(1 + rng.below(2) as u8)),
                    at: SpawnAt::Point(at),
                    orientation: Default::default(),
                }),
            };
        }
        2 => {
            // Never the Egress Nodes 1 and 2, which keep both players in the game; often a Beacon under deployment.
            let deploying: Vec<ObjectId> = sim.objects().filter(|o| o.is_deploying()).map(|o| o.id).collect();
            let id = match rng.below(2) {
                0 if !deploying.is_empty() => pick(rng, &deploying),
                _ => ObjectId(rng.below(40) + 3),
            };
            return PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id }),
            };
        }
        3..6 => {
            let objects = match rng.below(4) {
                _ if deploying.is_empty() => vec![any(rng)],
                0 => vec![any(rng)],
                _ => vec![pick(rng, &deploying)],
            };
            Command::Construction {
                objects,
                command: CancelTransform,
            }
        }
        6..9 => {
            let units = if drones.is_empty() {
                vec![any(rng)]
            } else {
                vec![pick(rng, &drones)]
            };
            let kind = match rng.below(2) {
                0 => OrderKind::Stop,
                _ => OrderKind::Move(Target::Point(Vec2::new(
                    rng.below(24 * 64) as f32,
                    rng.below(16 * 64) as f32,
                ))),
            };
            Command::Order {
                units,
                kind,
                queue: rng.below(3) == 0,
            }
        }
        _ => {
            let units = match rng.below(8) {
                _ if drones.is_empty() => vec![any(rng)],
                0 => drones.clone(),
                1 => vec![any(rng)],
                2 => own.iter().map(|o| o.id).collect(),
                _ => vec![pick(rng, &drones)],
            };
            let origin = match rng.below(12) {
                0 => at(rng.next_u32() as i32, rng.next_u32() as i32),
                _ => at(rng.below(27) as i32 - 2, rng.below(19) as i32 - 2),
            };
            Command::Order {
                units,
                kind: OrderKind::Deploy { origin },
                queue: rng.below(3) == 0,
            }
        }
    };
    PlayerCommand { player, command }
}

/// 3000 ticks of one random command each, on a 24 × 16 map with four Drones a player and a patch: what happened, the
/// state hash of every tick, and the replay.
fn deploy_fuzz(seed: u64) -> (Counts, Vec<u64>, Replay) {
    let mut builder = colonists(ScenarioBuilder::new(&flat(24, 16)))
        .spawn(EgressNode, Some(P1), tile(1, 1))
        .spawn(EgressNode, Some(P2), tile(21, 13))
        .spawn(CrystalPatch, None, tile(11, 7));
    for (x, y) in [(4, 4), (6, 9), (9, 3), (3, 12)] {
        builder = builder
            .spawn(Drone, Some(P1), point(x, y))
            .spawn(Drone, Some(P2), point(23 - x, 15 - y));
    }
    let mut t = builder.start();
    let mut rng = Rng::from_seed(seed);
    let mut counts = Counts::default();
    let mut hashes = Vec::new();
    for _ in 0..3000 {
        let cmd = random_command(&mut rng, t.sim());
        let verdict = t.sim().validate(&cmd);
        let mut without = t.sim().clone();
        without.step(&TickInputs::default());
        let tick = t.sim().tick() + 1;
        t.at(tick, cmd.player, cmd.command.clone()).step();
        match verdict {
            Ok(()) => counts.accepted += 1,
            Err(reason) => {
                counts.rejected += 1;
                assert_eq!(t.sim().state_hash(), without.state_hash(), "{cmd:?}: {reason}");
                assert_eq!(t.sim().events(), without.events(), "{cmd:?}: {reason}");
            }
        }
        for e in t.sim().events() {
            match e {
                SimEvent::KindChanged {
                    from: Drone,
                    to: Beacon,
                    ..
                } => counts.started += 1,
                SimEvent::DeployFailed { .. } => counts.failed += 1,
                SimEvent::TransformCompleted { .. } => counts.completed += 1,
                SimEvent::TransformCancelled { .. } => counts.cancelled += 1,
                _ => {}
            }
        }
        let sim = t.sim();
        for o in sim.objects().filter(|o| o.is_transforming()) {
            let deploying = o.transform.as_ref().unwrap();
            let site = o.placement.expect("a structure");
            assert_eq!(
                (
                    o.kind,
                    deploying.from,
                    deploying.from_placement,
                    deploying.progress.total,
                    deploying.paid
                ),
                (Beacon, Drone, None, 48, 0),
                "#{}",
                o.id.0
            );
            assert_eq!((site.w, site.h, o.pos), (2, 2, site.center()), "#{}", o.id.0);
            assert!(o.orders.is_none() && o.mover.is_none() && o.worker.is_none() && o.production.is_none());
        }
        for player in [P1, P2] {
            for (drone, _) in pending_deploys(sim, player) {
                let o = sim.object(drone).expect("a pending Deploy's Drone exists");
                assert_eq!((o.kind, o.owner), (Drone, Some(player)));
            }
        }
        assert_eq!(dangling_references(sim), []);
        hashes.push(sim.state_hash());
    }
    (counts, hashes, t.replay().clone())
}
