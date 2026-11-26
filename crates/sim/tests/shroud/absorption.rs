//! What a cloud absorbs and what its release does (`super`): "anti-ground damage aimed at objects under the cloud goes
//! into the shared buffer instead of hitting its target. The buffer is released **0.5 s (8 frames), placeholder, after
//! the first hit it absorbs** ... as **one area hit over the whole connected cloud** ... Each ground object under it
//! takes its share by **overlap area** (systems/combat.md > AoEDamage), with armor applied once per release; the share
//! falling on empty cloud area is **lost**" (`factions/cults/units/disciple.md > Spells > Shroud`).

use sc_sim::popup::PopUp;
use sc_sim::{Target, TilePos};

use super::*;

/// The plan's acceptance, "A lone Militant under one cloud, shot once for 4 from outside, takes 0.5 at the next
/// release, the area minimum, instead of 4. The rest is lost": player 1's Militant (2), 80 su from its Disciple (1),
/// shot once by player 2's (3) from 300 su off: nothing at the shot, the shot's whole Damage in the buffer, and 8 ticks
/// later (design change 0334: "after the first hit it absorbs") a release of 4 that takes 0.5 from each of the two
/// under the cloud, wholly under it, by the Disciple, and nothing from the shooter outside it. Then nothing more: the
/// next release waits for the next absorbed hit.
#[test]
fn a_lone_militant_shot_once_from_outside_takes_the_area_minimum_8_ticks_later() {
    let mut t = quiet(
        flat(30, 12)
            .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
            .spawn(MILITANT, Some(P1), point(480.0, 400.0))
            .spawn(MILITANT, Some(P2), point(700.0, 400.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 2, P2, &[3], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 60, "the shot absorbed", |t| {
        (t.sim().clouds().list().first()).is_some_and(|c| c.buffer > 0.0)
    });
    let shot = t.sim().tick();
    assert_eq!(buffer(&t, 1), 4.0, "the Militant's Damage, whole");
    assert_eq!(t.sim().clouds().list()[0].release_at, Some(shot + 8));
    assert_eq!((hp(&t, 2), hits_on(&t, 2)), (40.0, vec![]), "absorbed");
    order(&mut t, shot + 1, P2, &[3], OrderKind::Stop);
    t.run_until(shot + 7);
    assert_eq!((hp(&t, 2), releases(&t).len()), (40.0, 0), "not yet");
    t.step();
    assert_eq!(releases(&t), [(shot + 8, vec![Vec2::new(400.0, 400.0)], 4.0)]);
    assert_eq!(
        hits_on(&t, 2),
        [(shot + 8, 1, 0.5)],
        "the area minimum, by the cloud's Disciple"
    );
    assert_eq!((hp(&t, 1), hp(&t, 2), hp(&t, 3)), (39.5, 39.5, 40.0));
    t.run(40);
    assert_eq!(releases(&t).len(), 1, "no hit, no release");
    assert_eq!((buffer(&t, 1), t.sim().clouds().list()[0].release_at), (0.0, None));
}

/// "**Attackers under the cloud do full damage** ... Designer's reading: under the same connected cloud as its target,
/// so units fighting inside the cloud (e.g. melee) deal full damage to each other, while fire from outside is spread":
/// player 2's Militant under player 1's cloud (3, 120 su from the Disciple) hits player 1's Militant (2) there for its
/// full 4, nothing into the buffer; its fellow outside (4) only into the buffer.
#[test]
fn fire_from_under_the_same_cloud_hits_in_full() {
    let mut t = quiet(
        flat(30, 12)
            .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
            .spawn(MILITANT, Some(P1), point(480.0, 400.0))
            .spawn(MILITANT, Some(P2), point(400.0, 520.0))
            .spawn(MILITANT, Some(P2), point(700.0, 400.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 2, P2, &[3], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 60, "the inside shot", |t| !hits_on(t, 2).is_empty());
    let shot = t.sim().tick();
    assert_eq!(hits_on(&t, 2), [(shot, 3, 4.0)], "in full");
    assert_eq!((hp(&t, 2), buffer(&t, 1)), (36.0, 0.0));
    order(&mut t, shot + 1, P2, &[3], OrderKind::Stop);
    order(&mut t, shot + 1, P2, &[4], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 60, "the outside shot absorbed", |t| buffer(t, 1) > 0.0);
    assert_eq!((hp(&t, 2), buffer(&t, 1)), (36.0, 4.0), "into the buffer");
    assert_eq!(hits_on(&t, 2).len(), 1);
}

/// Plan decision T77: "For a projectile, that is judged by its position at launch". Player 2's Grenadier (3) throws
/// from under player 1's cloud at player 1's Militant (2) there and walks out of the cloud while the grenade flies: its
/// blast hits in full, its share of 100 over its 40-su circle on the Militant's 24x24 box. A second one (4) throws from
/// outside: its blast goes into the buffer.
#[test]
fn a_grenade_is_judged_by_where_it_was_thrown_from() {
    let mut t = quiet(
        flat(30, 16)
            .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
            .spawn(MILITANT, Some(P1), point(480.0, 400.0))
            .spawn(ObjectKind::Grenadier, Some(P2), point(400.0, 560.0))
            .spawn(ObjectKind::Grenadier, Some(P2), point(700.0, 400.0))
            .start(),
    );
    let launched = |t: &TestSim, by: u32| {
        (t.events().iter()).find_map(|(tick, e)| match e {
            SimEvent::ProjectileLaunched { shooter, .. } if *shooter == ObjectId(by) => Some(*tick),
            _ => None,
        })
    };
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 2, P2, &[3], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 60, "the first throw", |t| launched(t, 3).is_some());
    let away = Vec2::new(400.0, 900.0);
    let next = t.sim().tick() + 1;
    order(&mut t, next, P2, &[3], OrderKind::Move(Target::Point(away)));
    step_until(&mut t, 80, "the first blast", |t| !hits_on(t, 2).is_empty());
    let thrower = t.sim().object(ObjectId(3)).unwrap().pos;
    assert!(
        thrower.distance(Vec2::new(400.0, 400.0)) > RADIUS,
        "out of the cloud as it lands: {thrower}"
    );
    let share = 100.0 * (576.0 / (std::f32::consts::PI * 40.0 * 40.0));
    let (_, attacker, damage) = hits_on(&t, 2)[0];
    assert_eq!(attacker, 3);
    assert!((damage - share).abs() < 0.01, "in full: {damage} for {share}");
    assert_eq!(buffer(&t, 1), 0.0);
    let before = hp(&t, 2);
    let next = t.sim().tick() + 1;
    order(&mut t, next, P2, &[4], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 80, "the second blast absorbed", |t| buffer(t, 1) > 0.0);
    assert!(launched(&t, 4).is_some());
    assert!(
        (buffer(&t, 1) - share).abs() < 0.01,
        "its share, into the buffer: {}",
        buffer(&t, 1)
    );
    assert_eq!((hp(&t, 2), hits_on(&t, 2).len()), (before, 1));
}

/// Design change 0334 (Q0113): a release's shares take "pre-armor reductions, armor and minimum damage ... but **not
/// directional armor**: a release has no direction". Player 2's Hand of Heaven, firing at the ground on the centre of
/// player 1's Scrap Hangar, which it doesn't see, puts its 600 into the cloud through the hangar (its blast wholly on
/// the hangar, whose centre is under the Disciple's cloud); player 1's
/// Prayer-mobile, a WheeledVehicle with DirectionalArmor, wholly under the cloud, facing east as it spawned with the
/// Disciple and the missile's launcher both behind it, takes its share less its FullArmor of 1, times 1, not the rear's
/// 1.25.
#[test]
fn a_release_has_no_direction() {
    let hangar = 1;
    let mut t = quiet(
        flat(20, 12)
            .spawn(ObjectKind::ScrapHangar, Some(P1), SpawnAt::Tile(TilePos::new(10, 4)))
            .spawn(DISCIPLE, Some(P1), point(852.0, 352.0))
            .spawn(ObjectKind::PrayerMobile, Some(P1), point(952.0, 352.0))
            .spawn(ObjectKind::HandOfHeaven, Some(P2), point(136.0, 352.0))
            .start(),
    );
    order(&mut t, 1, P1, &[2], OrderKind::Shroud);
    order(&mut t, 2, P2, &[4], OrderKind::AttackGround(Vec2::new(736.0, 352.0)));
    step_until(&mut t, 400, "the missile absorbed", |t| buffer(t, 2) > 0.0);
    assert_eq!(hp(&t, hangar), 450.0, "absorbed");
    let absorbed = buffer(&t, 2);
    assert!(
        (absorbed - 600.0).abs() < 0.5,
        "the blast wholly on the hangar: {absorbed}"
    );
    let (points, inside) = areas(&t, 2, 3);
    assert_eq!(inside, 140.0, "the Prayer-mobile's 56x40, wholly under the cloud");
    step_until(&mut t, 9, "the release", |t| !releases(t).is_empty());
    let share = absorbed * (inside / points);
    let expected = ((share - 1.0) * 1.0).max(0.5);
    let (_, attacker, damage) = hits_on(&t, 3)[0];
    assert_eq!((attacker, damage), (2, expected), "no direction");
    assert!(damage > 5.0, "more than the armour and the minimum: {damage}");
    assert_eq!(
        t.sim().object(ObjectId(3)).unwrap().facing,
        0.0,
        "facing east, its back to both"
    );
    assert_eq!(hp(&t, 3), 150.0 - expected);
}

/// Design change 0334 (Q0113): "Syndicate units underground, expansions and lowered turrets under a cloud are protected
/// like ground objects, and take their share of releases, as ground area damage hits them (Q0071)". Player 1's pop-up
/// Laser Turret, lowered in its Tunnel's area, takes a grenade of player 2's without a cloud, its share of the 100 less
/// its underground armour (6 by the part of it in the blast); under one, nothing at the blast, its share whole in the
/// buffer, and the area minimum at the release.
#[test]
fn a_lowered_turret_is_protected_and_takes_its_share() {
    let turret = 3;
    let centre = Vec2::new(9.5 * 64.0, 10.5 * 64.0);
    let game = |shroud: bool| {
        let builder = ScenarioBuilder::new(&vec![".".repeat(40); 20].join("\n"))
            .faction(P1, Faction::Syndicate)
            .faction(P2, Faction::Syndicate)
            .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(4, 8)))
            .spawn(ObjectKind::OpticsLab, Some(P1), SpawnAt::Tile(TilePos::new(2, 6)))
            .spawn(ObjectKind::LaserTurret, Some(P1), SpawnAt::Tile(TilePos::new(9, 10)))
            .spawn(DISCIPLE, Some(P1), point(centre.x + 100.0, centre.y))
            .spawn(ObjectKind::Grenadier, Some(P2), point(centre.x, centre.y + 230.0));
        let mut t = quiet(builder.start());
        if shroud {
            order(&mut t, 1, P1, &[4], OrderKind::Shroud);
        }
        order(&mut t, 2, P2, &[5], OrderKind::AttackGround(centre));
        let landed = |t: &TestSim| (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Blast { .. }));
        step_until(&mut t, 80, "the blast", landed);
        assert_eq!(t.sim().object(ObjectId(turret)).unwrap().popup, Some(PopUp::LOWERED));
        t
    };
    let open = game(false);
    let taken = 300.0 - hp(&open, turret);
    assert!(taken > 50.0, "hit hard: {taken}");
    let mut t = game(true);
    let absorbed = buffer(&t, 4);
    assert_eq!(hp(&t, turret), 300.0, "protected");
    assert!(
        absorbed > taken && absorbed < 100.0,
        "its share, armour and all, into the buffer: {absorbed}"
    );
    step_until(&mut t, 9, "the release", |t| !releases(t).is_empty());
    let (_, attacker, damage) = hits_on(&t, turret)[0];
    assert_eq!(
        (attacker, damage),
        (4, 0.5),
        "its share, the area minimum under its armour"
    );
    assert_eq!(t.sim().object(ObjectId(turret)).unwrap().popup, Some(PopUp::LOWERED));
}
