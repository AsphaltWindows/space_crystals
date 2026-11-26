//! Training a Gunner (slice M5-G2): `factions/cults/structures/armory.md > Training`: "**Gunner** (SpaceCrystalsCost:
//! 40, TrainingTime: 32 frames). TechPrerequisite: player owns at least one Gunworks", KindMenu\[kind\]'s "**W: Train
//! Gunner**" (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`). The gate refuses it while its
//! player owns no Gunworks (`Reject::Requires`), and once one stands it is paid, reserved and trained as a Militant is;
//! the Gunworks lost, the Gunners queued and in training complete, and a new one is refused
//! (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "units already queued or in production
//! continue and complete normally; new ones cannot be queued until the player again owns a qualifying prerequisite
//! structure").

use super::*;

const GUNNER: ObjectKind = ObjectKind::Gunner;
const GUNWORKS: ObjectKind = ObjectKind::Gunworks;

/// KindMenu\[`from`\]'s "W: Train Gunner".
fn train_gunner(objects: &[ObjectId], from: RecruitKind) -> Command {
    production(objects, ProductionCommand::TrainFrom { kind: GUNNER, from })
}

/// A complete Gunworks of `player`'s on tiles `(x, 2)` to `(x + 1, 3)`, north of [`base`]'s Armory, spawned in `tick`
/// (a debug spawn: the Recruits' build of one is `tests/cults_construction/tech.rs`'s).
fn gunworks_at(t: &mut TestSim, tick: Tick, player: PlayerId, x: i32) {
    let spawn = DebugCommand::Spawn {
        kind: GUNWORKS,
        owner: Some(player),
        at: tile(x, 2),
        orientation: Default::default(),
    };
    t.debug_at(tick, spawn);
}

/// The Gunworks of `player`'s on the map.
fn gunworks_of(t: &TestSim, player: PlayerId) -> Vec<ObjectId> {
    (t.sim().objects())
        .filter(|o| o.kind == GUNWORKS && o.owner == Some(player))
        .map(|o| o.id)
        .collect()
}

/// The plan's acceptance (addendum G2): "W stays greyed, "Requires a Gunworks", until one is complete. Then a Gunner
/// costs 40 SC and 32 frames." With three Recruits pooled and no Gunworks, the gate refuses W and a W sent anyway
/// changes nothing; another player's Gunworks doesn't count. Once player 1's stands, W pays 40 Space Crystals and
/// reserves the Recruit that entered first; it comes out of side C as a Gunner 32 ticks later, the same unit
/// (`SimEvent::KindChanged`).
#[test]
fn w_trains_a_gunner_in_32_ticks_once_a_gunworks_stands() {
    let (mut t, order) = filled(3);
    let w = || train_gunner(&[A1], RecruitKind::Plain);
    assert_eq!(validate(&t, w()), Err(Reject::Requires(GUNWORKS)));
    let refused = t.sim().tick() + 1;
    t.at(refused, P1, w());
    gunworks_at(&mut t, refused, P2, 20);
    run_checked(&mut t, refused);
    assert_eq!(gunworks_of(&t, P2).len(), 1);
    assert_eq!((bank(&t, P1), reserved(&t, A1)), (BANK, Vec::new()), "refused");
    assert_eq!(validate(&t, w()), Err(Reject::Requires(GUNWORKS)), "player 2's");

    gunworks_at(&mut t, refused + 1, P1, 24);
    run_checked(&mut t, refused + 1);
    assert!(t.sim().requirement_met(P1, GUNWORKS));
    assert_eq!(validate(&t, w()), Ok(()));
    let start = refused + 2;
    t.at(start, P1, rally(&[A1], RALLY));
    t.at(start, P1, w());
    run_checked(&mut t, start);
    assert_eq!(bank(&t, P1), BANK - 40);
    assert_eq!(reserved(&t, A1), [[order[0]]]);
    let queued = SimEvent::ProductionQueued {
        producer: A1,
        kind: GUNNER,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start && *e == queued));
    run_checked(&mut t, start + 30);
    assert_eq!(produced(&t), [], "training");
    step_checked(&mut t);
    assert_eq!(
        produced(&t),
        [(start + 31, A1, order[0])],
        "32 ticks, the command's first"
    );
    let gunner = t.sim().object(order[0]).expect("out");
    assert_eq!(gunner.kind, GUNNER);
    assert!(
        gunner.pos.x > EXIT.x - 0.5 && gunner.pos.x < EXIT.x + 12.5 && (gunner.pos.y - EXIT.y).abs() <= 52.0,
        "in the doorway of side C: {}",
        gunner.pos
    );
    let changed = SimEvent::KindChanged {
        id: order[0],
        from: RECRUIT,
        to: GUNNER,
    };
    assert!(t.events().iter().any(|(tick, e)| *tick == start + 31 && *e == changed));
    assert_eq!(inside(&t, A1), order[1..]);
    assert_eq!(bank(&t, P1), BANK - 40);
}

/// The plan's acceptance (addendum G2): "When the last Gunworks is lost, queued Gunners complete and no new one can be
/// queued." Two Gunners queued, the first in training, and a Gunworks of player 1's two lost: the other still counts.
/// Then the last one lost: nothing is refunded nor cancelled, both Gunners come out, one after the other, and the gate
/// refuses a third while it takes a Militant, which needs no Gunworks. A new Gunworks lets W train again.
#[test]
fn losing_the_last_gunworks_keeps_the_queued_gunners() {
    let (mut t, order) = filled(3);
    let now = t.sim().tick() + 1;
    gunworks_at(&mut t, now, P1, 20);
    gunworks_at(&mut t, now, P1, 24);
    run_checked(&mut t, now);
    let works = gunworks_of(&t, P1);
    assert_eq!(works.len(), 2);
    let start = now + 1;
    t.at(start, P1, train_gunner(&[A1], RecruitKind::Plain));
    t.at(start, P1, train_gunner(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start);
    assert_eq!(reserved(&t, A1), [[order[0]], [order[1]]]);
    assert_eq!(bank(&t, P1), BANK - 80);

    remove_at(&mut t, start + 5, works[0]);
    run_checked(&mut t, start + 5);
    assert_eq!(
        validate(&t, train_gunner(&[A1], RecruitKind::Plain)),
        Ok(()),
        "the other counts"
    );
    remove_at(&mut t, start + 10, works[1]);
    run_checked(&mut t, start + 10);
    assert!(!t.sim().requirement_met(P1, GUNWORKS));
    assert_eq!(
        validate(&t, train_gunner(&[A1], RecruitKind::Plain)),
        Err(Reject::Requires(GUNWORKS))
    );
    assert_eq!(
        validate(&t, train(&[A1], RecruitKind::Plain)),
        Ok(()),
        "a Militant needs none"
    );
    t.at(start + 11, P1, train_gunner(&[A1], RecruitKind::Plain));
    run_checked(&mut t, start + 11);
    assert_eq!(reserved(&t, A1).len(), 2, "the third refused");
    assert_eq!(bank(&t, P1), BANK - 80, "nothing refunded");
    let cancelled = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ProductionCancelled { .. }));
    assert!(!cancelled, "nothing cancelled");

    run_for(&mut t, 100);
    let out: Vec<ObjectId> = produced(&t).iter().map(|&(_, _, unit)| unit).collect();
    assert_eq!(out, order[..2], "both queued Gunners trained, in order");
    for &unit in &order[..2] {
        assert_eq!(t.sim().object(unit).map(|o| o.kind), Some(GUNNER), "{unit:?}");
    }
    assert_eq!((inside(&t, A1), reserved(&t, A1)), (order[2..].to_vec(), Vec::new()));
    assert_eq!(bank(&t, P1), BANK - 80);

    let again = t.sim().tick() + 1;
    gunworks_at(&mut t, again, P1, 20);
    run_checked(&mut t, again);
    assert_eq!(
        validate(&t, train_gunner(&[A1], RecruitKind::Plain)),
        Ok(()),
        "a Gunworks again"
    );
}
