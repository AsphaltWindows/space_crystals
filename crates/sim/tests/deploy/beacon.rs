//! A Beacon under deployment, a structure from the first frame (design change 0109, `> A Beacon from the first
//! frame`): it keeps its owner in the game (0113), claims its footprint, is taken for a structure by enemy
//! auto-targeting, keeps the damage it takes (0110), and destroyed midway loses the one object
//! (`> Vulnerability`).

use super::*;

/// Design change 0113's Verification (`core/game_rules.md > Defeat Condition`; `> A Beacon from the first frame`: "It
/// is a structure"): player 1 loses Egress Node 1, its only other building, while Drone 2 deploys, and stays in the
/// game; cancelling the deployment then leaves player 1 no structure, and it is defeated in the cancel's tick: the
/// Drone it got back becomes neutral.
#[test]
fn a_beacon_under_deployment_keeps_its_owner_in_the_game_until_cancelled() {
    let mut t = base().start();
    let start = deployed(&mut t);
    t.debug_at(start + 1, DebugCommand::Remove { id: id(1) });
    t.run(10);
    assert!(t.sim().object(id(1)).is_none());
    assert!(!t.sim().player(P1).unwrap().is_defeated());
    assert_eq!(
        ticks_of(&t, |e| matches!(e, SimEvent::PlayerDefeated { .. })),
        [] as [Tick; 0]
    );
    next(&mut t, cancel(&[2]));
    let tick = t.sim().tick();
    assert_eq!(
        ticks_of(&t, |e| *e == SimEvent::PlayerDefeated { player: P1 }),
        [tick],
        "in the cancel's tick"
    );
    assert!(t.sim().player(P1).unwrap().is_defeated());
    assert_eq!(t.sim().outcome().map(|o| o.winners.clone()), Some(vec![P2]));
    assert_eq!(object(&t, 2).owner, None, "its units become neutral (`> On Defeat`)");
}

/// The plan's acceptance, "Destroyed mid-deploy" (`> Vulnerability`: "The transforming unit is vulnerable to attack
/// for the entire deployment duration. If destroyed mid-deployment, both the Drone and the in-progress Beacon are
/// lost"): player 2's four Peacekeepers 14 to 17 attack Beacon 2 from the tick after it began, at 10/10, and destroy
/// it before it is complete: one object dies and is removed, nothing else, and the capacity stays as the start left
/// it, 9/10 (the Drone's 1 freed, the Beacon's 5 never provided). Nothing refers to it any more.
#[test]
fn a_deployment_destroyed_midway_loses_the_one_object() {
    let mut builder = at_capacity();
    for (x, y) in [(6, 9), (11, 9), (8, 7), (9, 12)] {
        builder = builder.spawn(Peacekeeper, Some(P2), point(x, y));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    let start = deployed(&mut t);
    assert_eq!(capacity(&t, P1), (9, 10));
    let objects = t.sim().objects().count();
    t.order_at(
        start + 1,
        P2,
        &[id(14), id(15), id(16), id(17)],
        OrderKind::Attack(id(2)),
        false,
    );
    while t.sim().object(id(2)).is_some() {
        assert!(t.sim().tick() < start + 47, "destroyed before it is complete");
        assert!(object(&t, 2).is_deploying());
        t.step();
    }
    let died = t.sim().tick();
    let deaths: Vec<&SimEvent> = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::Died { .. } | SimEvent::Removed(_)))
        .map(|(_, e)| e)
        .collect();
    assert_eq!(deaths.len(), 2, "{deaths:?}");
    assert!(matches!(deaths[0], SimEvent::Died { id: dead, kind: Beacon, .. } if *dead == id(2)));
    assert_eq!(deaths[1], &SimEvent::Removed(id(2)));
    assert_eq!(ticks_of(&t, |e| *e == SimEvent::Removed(id(2))), [died]);
    assert_eq!(t.sim().objects().count(), objects - 1);
    assert_eq!(capacity(&t, P1), (9, 10));
    assert_eq!(completions(&t), []);
    assert_eq!(dangling_references(t.sim()), []);
}

/// Design change 0109 (`> A Beacon from the first frame`: "the footprint is claimed immediately, units can't walk onto
/// it") and the plan's acceptance, "Footprint claimed": from the tick it began no unit can be put on any tile of the
/// footprint, and player 1's Peacekeeper 4, sent in the next tick from the west of it to the east across it, walks
/// around it, its box never on it (`TestSim` also checks that no ground box overlaps a blocked tile after every
/// tick, invariant I2).
#[test]
fn the_footprint_is_claimed_from_the_first_frame() {
    let mut t = base().spawn(Peacekeeper, Some(P1), point(5, 9)).start();
    let start = deployed(&mut t);
    let footprint = deploy::site(SITE);
    for tile in footprint.tiles() {
        let spawn = PlayerCommand {
            player: P1,
            command: Command::Debug(DebugCommand::Spawn {
                kind: Peacekeeper,
                owner: Some(P2),
                at: SpawnAt::Point(tile.center()),
                orientation: Default::default(),
            }),
        };
        assert_eq!(t.sim().validate(&spawn), Err(Reject::Blocked), "{tile:?}");
    }
    let goal = at(12, 10).center();
    t.order_at(start + 1, P1, &[id(4)], OrderKind::Move(Target::Point(goal)), false);
    let (lo, hi) = (
        footprint.origin.corner(),
        footprint.origin.corner() + Vec2::splat(2.0 * SU_PER_GU),
    );
    // A Peacekeeper's box is 24 space units across.
    let on_footprint = |p: Vec2| p.x + 12.0 > lo.x && p.x - 12.0 < hi.x && p.y + 12.0 > lo.y && p.y - 12.0 < hi.y;
    for _ in 0..240 {
        t.step();
        assert!(!on_footprint(t.pos(4)), "tick {}, at {}", t.sim().tick(), t.pos(4));
    }
    assert!(t.pos(4).distance(goal) < 8.0, "around it, to {}", t.pos(4));
}

/// Steps `sim` on its own until object `n` has been a Beacon for `frames` frames of its deployment, its first in the
/// tick it began (its HP then interpolate by that many frames).
fn run_frames(sim: &mut Sim, n: u32, frames: u32) {
    while sim.object(id(n)).unwrap().kind == Drone {
        assert!(sim.tick() < 600, "it deploys");
        sim.step(&TickInputs::default());
    }
    let done = |sim: &Sim| {
        let o = sim.object(id(n)).unwrap();
        o.transform.as_ref().map_or(DEPLOYMENT_FRAMES, |t| t.frames_done())
    };
    while done(sim) < frames {
        sim.step(&TickInputs::default());
    }
    assert_eq!(done(sim), frames);
}

/// Design change 0110's Verification (`> HP/Shields Interpolation`; "current += newMax − oldMax each frame"):
/// "Drone at 10/20 HP deploys undisturbed: completes at 110/120"; "Hit a deploying Beacon for 30 at 50%: shows 40/70,
/// completes at 90/120". The damage is set by editing saves, as no command deals an exact amount: Drone 2 at 10 HP
/// when ordered, and Beacon 2 at 40 of its 70 after 24 frames. Each stays as short of its maximum all the way.
#[test]
fn damage_taken_before_or_while_deploying_stays_taken() {
    let close = |hp: Option<f32>, expected: f32| hp.is_some_and(|hp| (hp - expected).abs() < 1e-3);
    let mut t = base().start();
    t.step();
    let mut sim = edited(t.sim(), 2, "hp", serde_json::json!(10.0));
    let cmd = PlayerCommand {
        player: P1,
        command: order(&[2], deploy_at(SITE), false),
    };
    sim.step(&TickInputs { commands: vec![cmd] });
    for frames in [1, 24, 47, 48] {
        run_frames(&mut sim, 2, frames);
        let o = sim.object(id(2)).unwrap();
        let max = 20.0 + 100.0 * frames as f32 / 48.0;
        assert_eq!(o.max_hp(), Some(max), "{frames} frames");
        assert!(close(o.hp, max - 10.0), "{frames} frames: {:?}", o.hp);
    }
    assert!(!sim.object(id(2)).unwrap().is_transforming(), "complete at 110/120");
    let mut t = base().start();
    let start = deployed(&mut t);
    t.run_until(start + 23);
    let mut sim = edited(t.sim(), 2, "hp", serde_json::json!(40.0));
    assert_eq!(sim.object(id(2)).unwrap().max_hp(), Some(70.0), "50%");
    for frames in [25, 36, 48] {
        run_frames(&mut sim, 2, frames);
        let o = sim.object(id(2)).unwrap();
        let max = 20.0 + 100.0 * frames as f32 / 48.0;
        assert!(close(o.hp, max - 30.0), "{frames} frames: {:?}", o.hp);
    }
    let o = sim.object(id(2)).unwrap();
    assert!(
        !o.is_transforming() && close(o.hp, 90.0),
        "complete at 90/120: {:?}",
        o.hp
    );
}

/// Design change 0109's Verification, "Enemy auto-targeting treats it as a structure during the 3 s": the targeting
/// takes the Beacon's 2 × 2 footprint for its shape. Player 2's Peacekeeper 4 holds 310 space units east of the
/// centre of the site (`OrderKind::Hold`: it engages what its weapon reaches, 4 grid units, 256 space units edge to
/// edge). The walking Drone, 36 across, never comes nearer than that centre (280 between the boxes), and is never
/// shot; the Beacon under deployment, 128 across, is within reach (234), and is shot.
#[test]
fn enemy_auto_targeting_takes_a_deploying_beacon_for_the_structure() {
    let centre = deploy::site(SITE).center();
    let mut t = base()
        .spawn(Peacekeeper, Some(P2), SpawnAt::Point(centre + Vec2::new(310.0, 0.0)))
        .start();
    t.order_at(1, P2, &[id(4)], OrderKind::Hold, false);
    let start = deployed(&mut t);
    let hits = |t: &TestSim| ticks_of(t, |e| matches!(e, SimEvent::Hit { target, .. } if *target == id(2)));
    assert_eq!(hits(&t), [] as [Tick; 0], "the Drone never shot");
    t.run_until(start + 46);
    assert!(!hits(&t).is_empty(), "the Beacon under deployment shot");
    let o = object(&t, 2);
    assert!(o.is_deploying());
    assert!(o.hp < o.max_hp(), "{:?} of {:?}", o.hp, o.max_hp());
}
