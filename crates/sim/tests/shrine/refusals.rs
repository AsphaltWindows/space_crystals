//! What the gate refuses of a cast, and which Shrine casts (`factions/cults/structures/shrine.md > Abilities`: "Both
//! abilities are cast on an area anywhere the player currently has vision, and spend Shrine energy"; `> Placeholder
//! stats`: "Groupable **false** (user, 2026-11-26; Q0118: each press casts from one Shrine)"; "Targeting:
//! AwaitingTarget, left-click a point in vision"; `sim/casts.rs`), and the Shrine's TechPrerequisite ("player owns at
//! least one Temple").

use super::*;

/// Where [`base`]'s Shrine sees, its centre.
const CENTRE: Vec2 = Vec2::new(448.0, 640.0);

/// Whether player 1 sees `at` now.
fn sees(t: &TestSim, at: Vec2) -> bool {
    t.sim().vision().state(P1, TilePos::containing(at)) == sc_sim::VisibilityState::Visible
}

/// The plan's acceptance: "a point outside vision is refused". With the energy for Q, the gate takes it at the Shrine's
/// centre, which it sees, and refuses it at a point nobody of player 1's sees (`Reject::OutOfSight`) and off the map, a
/// NaN point included (`Reject::OffMap`). A point player 1's Recruit stands on is taken; once the Recruit has gone and
/// the point has left player 1's vision, "currently", it is refused again, though player 1 saw it.
#[test]
fn a_point_out_of_sight_or_off_the_map_is_refused() {
    let (mut t, _) = manned(8);
    charge(&mut t, 75);
    let far = Vec2::new(1800.0, 1100.0);
    assert!(sees(&t, CENTRE) && !sees(&t, far));
    assert_eq!(validate(&t, cast(&[S1], Q, CENTRE)), Ok(()));
    assert_eq!(validate(&t, cast(&[S1], Q, far)), Err(Reject::OutOfSight));
    for off in [
        Vec2::new(-1.0, 640.0),
        Vec2::new(2049.0, 640.0),
        Vec2::new(f32::NAN, 640.0),
    ] {
        assert_eq!(validate(&t, cast(&[S1], Q, off)), Err(Reject::OffMap), "{off}");
    }
    let scout = spawn(&mut t, RECRUIT, P1, far);
    run_for(&mut t, 1);
    assert!(sees(&t, far));
    assert_eq!(validate(&t, cast(&[S1], Q, far)), Ok(()), "{scout:?} sees it");
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, scout);
    while sees(&t, far) {
        assert!(t.sim().tick() < gone + 100, "out of sight");
        step_checked(&mut t);
    }
    assert_eq!(
        validate(&t, cast(&[S1], Q, far)),
        Err(Reject::OutOfSight),
        "seen, not now"
    );
    let now = t.sim().tick() + 1;
    t.at(now, P1, cast(&[S1], Q, far));
    let before = energy(&t, S1);
    run_checked(&mut t, now);
    assert_eq!(casts(&t), [], "refused as it applies too");
    assert_eq!(energy(&t, S1), before + FULL, "not paid");
}

/// The plan's acceptance: "too little energy greys the command" (what the panel greys by, `Sim::can_cast`). A Shrine
/// with less than 75 refuses Q (`Reject::NotEnoughEnergy`) and takes W, at 50; the cast pays its cost and nothing more,
/// and with less than 50 left W is refused too. The energy is checked before the point.
#[test]
fn too_little_energy_refuses_the_cast_and_greys_it() {
    let (mut t, _) = manned(8);
    charge(&mut t, 50);
    let points = energy(&t, S1) / ONE;
    assert!(points < 75, "{points}");
    assert_eq!(t.sim().can_cast(P1, &[S1], Q), Err(Reject::NotEnoughEnergy));
    assert_eq!(t.sim().can_cast(P1, &[S1], W), Ok(S1));
    assert_eq!(validate(&t, cast(&[S1], Q, CENTRE)), Err(Reject::NotEnoughEnergy));
    let off = Vec2::new(-1.0, 640.0);
    assert_eq!(
        validate(&t, cast(&[S1], Q, off)),
        Err(Reject::NotEnoughEnergy),
        "before the point"
    );
    let now = t.sim().tick() + 1;
    t.at(now, P1, cast(&[S1], W, CENTRE));
    let before = energy(&t, S1);
    run_checked(&mut t, now);
    assert_eq!(casts(&t), [(now, S1, W, CENTRE)]);
    assert_eq!(energy(&t, S1), before - 50 * ONE + FULL, "50, and the tick's charge");
    assert_eq!(t.sim().can_cast(P1, &[S1], W), Err(Reject::NotEnoughEnergy));
}

/// Who casts: the Shrine only, built, its player's. A Temple, a Recruit and an object that is gone cast nothing
/// (`Reject::LacksAbility`), and a group of the Shrine and a Temple casts from the Shrine; player 2 can't cast from
/// player 1's Shrine (`Reject::NotOwned`).
#[test]
fn only_its_players_shrine_casts() {
    let scenario = base().spawn(TEMPLE, Some(P1), tile(20, 4)); // 3
    let (mut t, recruits) = manned_with(scenario, &[RecruitKind::Plain; 8]);
    let temple = id(3);
    charge(&mut t, 75);
    let refused = |t: &TestSim, casters: &[ObjectId]| validate(t, cast(casters, Q, CENTRE));
    assert_eq!(refused(&t, &[temple]), Err(Reject::LacksAbility));
    assert_eq!(refused(&t, &[recruits[0]]), Err(Reject::LacksAbility), "held");
    assert_eq!(refused(&t, &[id(500)]), Err(Reject::LacksAbility), "gone");
    assert_eq!(t.sim().can_cast(P1, &[temple, S1], Q), Ok(S1));
    let p2 = PlayerCommand {
        player: P2,
        command: cast(&[S1], Q, CENTRE),
    };
    assert_eq!(t.sim().validate(&p2), Err(Reject::NotOwned(S1)));
}

/// "Groupable **false**": the client sends a press to one Shrine. A command naming several casts from one, the first
/// in id order with the energy (**Placeholder (not in the design)**): of two charged Shrines, the first pays; with
/// the first short of it, the second.
#[test]
fn a_cast_naming_two_shrines_casts_from_one() {
    let s2 = id(3);
    let scenario = base().spawn(SHRINE, Some(P1), tile(14, 9)); // 3
    let (mut t, _) = manned_with(scenario, &[RecruitKind::Plain; 8]);
    let now = t.sim().tick() + 1;
    let more = block(&mut t, now, RECRUIT, 8, Vec2::new(1150.0, 640.0));
    t.at(now + 1, P1, enter(&more, s2));
    run_checked(&mut t, now + 1);
    fill(&mut t, s2, 8);
    while energy(&t, s2) < 75 * ONE {
        assert!(t.sim().tick() < now + 1000, "both charged");
        step_checked(&mut t);
    }
    let c = t.sim().tick() + 1;
    t.at(c, P1, cast(&[s2, S1], Q, CENTRE));
    t.at(c + 1, P1, cast(&[s2, S1], Q, CENTRE));
    let (one, two) = (energy(&t, S1), energy(&t, s2));
    run_checked(&mut t, c + 1);
    assert_eq!(casts(&t), [(c, S1, Q, CENTRE), (c + 1, s2, Q, CENTRE)]);
    assert_eq!(energy(&t, S1), one - 75 * ONE + 2 * FULL, "the first paid once");
    assert_eq!(energy(&t, s2), two - 75 * ONE + 2 * FULL, "then the second");
}

/// `factions/cults/units/recruit.md > Constructs`: "Shrine: 200 Space Crystals, 480 frames (30 seconds)
/// BaseConstructionTime (placeholder). Requires: Temple." A Recruit's Build of a Shrine is refused while its player owns
/// no Temple (`Reject::Requires`); once one stands it is taken, paid 200, and the site, under construction, casts
/// nothing (`Reject::UnderConstruction`). Built, the Shrine starts at 0 energy.
#[test]
fn recruits_build_a_shrine_with_a_temple_and_its_site_casts_nothing() {
    let mut t = base().start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, BANK));
    let recruit = spawn(&mut t, RECRUIT, P1, Vec2::new(1000.0, 300.0));
    let site_at = TilePos::new(20, 4);
    let order = Command::Order {
        units: vec![recruit],
        kind: OrderKind::Build {
            kind: SHRINE,
            origin: site_at,
            orientation: BuildingOrientation::default(),
        },
        queue: false,
    };
    assert_eq!(validate(&t, order.clone()), Err(Reject::Requires(TEMPLE)));
    spawn_structure(&mut t, TEMPLE, P1, tile(24, 14));
    assert_eq!(validate(&t, order.clone()), Ok(()));
    t.at(3, P1, order);
    run_checked(&mut t, 3);
    let bank = |t: &TestSim| t.sim().player(P1).expect("in the game").crystals;
    assert_eq!(bank(&t), BANK - 200, "paid as the order is given");
    let site = |t: &TestSim| {
        (t.sim().objects())
            .find(|o| o.kind == SHRINE && o.id != S1)
            .map(|o| o.id)
    };
    while site(&t).is_none() {
        assert!(t.sim().tick() < 300, "the site is laid");
        step_checked(&mut t);
    }
    let new = site(&t).unwrap();
    assert!(!t.sim().object(new).unwrap().is_operational());
    assert_eq!(
        validate(&t, cast(&[new], W, CENTRE)),
        Err(Reject::UnderConstruction(new))
    );
    while !t.sim().object(new).unwrap().is_operational() {
        assert!(t.sim().tick() < 1200, "built");
        step_checked(&mut t);
    }
    assert_eq!(energy(&t, new), 0, "it starts at 0");
    assert_eq!(t.sim().can_cast(P1, &[new], W), Err(Reject::NotEnoughEnergy));
}
