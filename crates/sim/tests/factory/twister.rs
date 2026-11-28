//! The Twister in a fight (`factions/gdo/units/twister.md`): "TurnAngle: **360°** (full rotation)", "TurretTurnRate
//! 12°/frame", "AttackType: FullyConnected (Ranged) — machine-gun type weapon", "TargetDomain: **Air** (anti-air
//! only)", "Machine gun: Damage 4; Range 6 grid units; Aim 1, Firing 1, Cooldown 1, Reload 1 frames". Player 1's
//! Twister against player 2's Sparrow (`factions/cults/units/sparrow.md`: MaxHP 50, armour 0) and Peacekeeper, on an
//! open field, auto-targeting off.

use sc_sim::combat::turret_facing;
use sc_sim::math;

use super::*;

const SPARROW: ObjectKind = ObjectKind::Sparrow;
/// The Twister of [`field`], and the unit it faces.
const TWISTER_ID: ObjectId = ObjectId(3);
const OTHER: ObjectId = ObjectId(4);
/// Where [`field`]'s Twister stands, facing east.
const STAND: Vec2 = Vec2::new(900.0, 900.0);

/// Player 1 the GDO's, with a Deployment Center, and player 2 the Cults', with a Storage, both far off, on a flat 30 ×
/// 24 map; player 1's Twister ([`TWISTER_ID`]) standing at [`STAND`] and player 2's `kind` ([`OTHER`]) at `at`,
/// spawned in tick 1, which has run; auto-targeting off.
fn field(kind: ObjectKind, at: Vec2) -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(30, 24))
        .faction(P1, Faction::Gdo)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::Storage, Some(P2), tile(26, 20))
        .start();
    t.without_auto_targeting();
    t.spawn_unit_at(1, TWISTER, P1, STAND).spawn_unit_at(1, kind, P2, at);
    t.run_until(1);
    t
}

/// "The Twister hits a Sparrow": player 2's Sparrow, on Hold, circles 200 su behind the Twister, which faces east;
/// ordered in tick 3 to attack it, the Twister never moves nor turns its hull: its turret, which turns all the way
/// round, swings onto the Sparrow behind it at its 12° a frame and fires from there, more than 150° off the hull's
/// facing at every hit. Each hit takes the machine gun's 4 (the Sparrow has no armour), one every 4 frames once it is on
/// the Sparrow (its Aim, Firing, Cooldown and Reload of 1 frame each), and the thirteenth kills it.
#[test]
fn its_machine_gun_hits_a_sparrow_behind_it_its_turret_turning_round() {
    let mut t = field(SPARROW, STAND + Vec2::new(-200.0, 0.0));
    t.at(2, P2, order(&[OTHER], OrderKind::Hold));
    t.at(3, P1, order(&[TWISTER_ID], OrderKind::Attack(OTHER)));
    let mut seen = 0;
    while t.sim().object(OTHER).is_some() {
        assert!(t.sim().tick() < 300, "the Sparrow shot down");
        t.step();
        let twister = object(&t, TWISTER_ID);
        assert_eq!(
            (twister.pos, twister.facing),
            (STAND, 0.0),
            "tick {}: its hull still",
            t.sim().tick()
        );
        let hits = hits_on(&t, OTHER).len();
        if hits > seen {
            let off = math::wrap_angle(turret_facing(twister).expect("a turret") - twister.facing);
            assert!(
                off.abs() > 150.0_f32.to_radians(),
                "tick {}: its turret {}° off its hull",
                t.sim().tick(),
                off.to_degrees()
            );
        }
        seen = hits;
    }
    let hits = hits_on(&t, OTHER);
    assert_eq!(hits.len(), 13, "{hits:?}");
    assert!((14..=18).contains(&hits[0].0), "its turret swung round first: {hits:?}");
    for pair in hits.windows(2) {
        assert_eq!(pair[1].0 - pair[0].0, 4, "one every 4 frames: {pair:?}");
    }
    assert!(
        hits.iter().all(|&(_, by, damage)| by == TWISTER_ID && damage == 4.0),
        "{hits:?}"
    );
}

/// The plan's acceptance: "An Attack on a Peacekeeper is refused, with feedback" (the client's half,
/// `crates/game/tests/commands/factory.rs`). A Twister's machine gun hits Air only (`systems/combat.md > ValidTarget` 3,
/// `combat::can_hit`): the gate refuses its Attack on player 2's Peacekeeper (`Reject::CannotHit`), and one sent
/// anyway leaves it idle; its right-click on the Peacekeeper follows it, as a unit's that can't attack its target.
/// With a Raptor, whose missile hits the ground too, the group's Attack is taken, the Raptor's alone: it fires, the
/// Twister stays idle. (The client sends such a Twister after the target instead, design change 0461: below.)
#[test]
fn an_attack_on_a_peacekeeper_is_refused() {
    let mut t = field(PK, STAND + Vec2::new(256.0, 0.0));
    let attack = order(&[TWISTER_ID], OrderKind::Attack(OTHER));
    assert_eq!(gate(&t, P1, attack.clone()), Err(Reject::CannotHit(OTHER)));
    assert_eq!(
        t.sim().default_order(TWISTER_ID, Target::Object(OTHER)),
        Some(OrderKind::Move(Target::Object(OTHER)))
    );
    t.at(2, P1, attack);
    run_for(&mut t, 20);
    assert_eq!(current(&t, TWISTER_ID), None, "idle");
    assert_eq!(hits_on(&t, OTHER), []);

    let raptor = id(next_id(&t));
    t.spawn_unit_at(t.sim().tick() + 1, RAPTOR, P1, STAND + Vec2::new(0.0, 100.0));
    t.step();
    let both = order(&[TWISTER_ID, raptor], OrderKind::Attack(OTHER));
    assert_eq!(gate(&t, P1, both.clone()), Ok(()));
    t.at(t.sim().tick() + 1, P1, both);
    run_for(&mut t, 60);
    assert_eq!(current(&t, TWISTER_ID), None, "the Twister ignores it");
    let hits = hits_on(&t, OTHER);
    assert!(
        !hits.is_empty() && hits.iter().all(|&(_, by, _)| by == raptor),
        "{hits:?}"
    );
}

/// Design change 0461's Verification 1, the sim's half (the client's: `command_ui::resolution::target_orders`): the
/// orders the client gives Twisters and Raptors A-clicked on an enemy tank, an Attack to the Raptor and a Move on the
/// tank to the Twister, whose machine gun hits air only ("it **moves to the target (follows it) with the group**",
/// `systems/controls/interface.md > CommandPanel`, "Attack goes to everyone"). As player 2's Hand of Heaven drives off
/// east, 10 grid units, the Raptor hits it, and the Twister follows it all the way
/// (`systems/controls/command_behaviors.md > MovingToObject`), never more than 5 grid units behind, and stands by it at
/// the end, its order going on.
#[test]
fn a_twister_sent_after_a_tank_with_raptors_follows_it_as_it_drives_off() {
    let start = STAND + Vec2::new(256.0, 0.0);
    let mut t = field(ObjectKind::HandOfHeaven, start);
    let raptor = id(next_id(&t));
    t.spawn_unit_at(2, RAPTOR, P1, STAND + Vec2::new(0.0, 100.0));
    let away = start + Vec2::new(640.0, 0.0);
    t.at(3, P2, order(&[OTHER], OrderKind::Move(Target::Point(away))));
    t.at(3, P1, order(&[raptor], OrderKind::Attack(OTHER)));
    let follow = OrderKind::Move(Target::Object(OTHER));
    t.at(3, P1, order(&[TWISTER_ID], follow));
    run_for(&mut t, 2);
    let mut farthest = 0.0_f32;
    while t.sim().tick() < 400 {
        t.step();
        farthest = farthest.max(shape(&t, TWISTER_ID).gap(&shape(&t, OTHER)));
    }
    assert!(object(&t, OTHER).pos.distance(away) < 32.0, "the tank drove off");
    assert!(
        farthest < 5.0 * 64.0,
        "the Twister kept with it: {farthest} su behind at most"
    );
    let gap = shape(&t, TWISTER_ID).gap(&shape(&t, OTHER));
    assert!(gap < 2.0 * 64.0, "it stands by it: {gap} su off");
    assert_eq!(current(&t, TWISTER_ID), Some(follow), "it follows on");
    let hits = hits_on(&t, OTHER);
    assert!(
        !hits.is_empty() && hits.iter().all(|&(_, by, _)| by == raptor),
        "{hits:?}"
    );
}
