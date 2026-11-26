//! The Tunnel expansions' regeneration (`factions/syndicate/structures/README.md > Tunnel Expansions`: "a completed
//! expansion **regenerates HP gradually** while it is part of a Tunnel system (its Tunnel is alive). Placeholder rate
//! (designer): **1% of MaxHP per second**, always on, **including during combat**").

use super::*;

/// What a Hideout regenerates in a tick: 1% of its 400 HP a second, of 16 ticks.
const HIDEOUT_REGEN: f32 = 0.25;

/// Design change 0329's verification: "A damaged Hideout climbs back to full in about 100 s". At half its HP it is
/// back to full in 800 ticks, 50 seconds, 0.25 HP every tick, so from nothing in 100; and stays there.
#[test]
fn a_damaged_hideout_climbs_back_to_full_in_100_seconds() {
    let mut sim = damaged(syndicate().sim(), HIDEOUT, 200.0);
    assert_eq!(restored_to(&mut sim, HIDEOUT, 400.0, HIDEOUT_REGEN, |_| {}), 800);
    run(&mut sim, 16);
    assert_eq!(hp(&sim, HIDEOUT), 400.0);
}

/// "... but not once its Tunnel is gone" (design change 0329's verification): its Tunnel removed, the Hideout is
/// nobody's and decays (`factions/syndicate/tunnels.md > Losing Tunnels`), regaining nothing.
#[test]
fn a_hideout_regenerates_no_more_once_its_tunnel_is_gone() {
    let mut sim = damaged(syndicate().sim(), HIDEOUT, 200.0);
    step(&mut sim, vec![Command::Debug(DebugCommand::Remove { id: id(T1) })]);
    assert_eq!(sim.object(id(HIDEOUT)).expect("on the map").owner, None, "neutral");
    let mut last = hp(&sim, HIDEOUT);
    for _ in 0..64 {
        run(&mut sim, 1);
        let now = hp(&sim, HIDEOUT);
        assert!(now < last, "it decays and regains nothing: {last} to {now}");
        last = now;
    }
}

/// "always on, **including during combat**": expansions "are hit by area damage over them (systems/combat.md >
/// AoEDamage)", so player 2's Grenadier, north of the Hideout within its Range of 4 and beyond the turret's of 5,
/// throws its grenades at the ground over it (an AttackGround: the Hideout is hidden from player 2, and nobody fights
/// by themselves here). Until the first blast the Hideout is full; from then on it regains its 0.25 HP in every tick
/// without a blast.
#[test]
fn a_hideout_regenerates_under_fire() {
    let mut t = syndicate();
    t.debug_at(2, DebugCommand::SetAutoTargeting { on: false });
    t.debug_at(
        2,
        DebugCommand::Spawn {
            kind: ObjectKind::Grenadier,
            owner: Some(P2),
            at: SpawnAt::Point(Vec2::new(576.0, 360.0)),
            orientation: Default::default(),
        },
    );
    t.run(1);
    let thrower = t.sim().objects().last().expect("spawned").id;
    let throw = Command::Order {
        units: vec![thrower],
        kind: OrderKind::AttackGround(Vec2::new(576.0, 576.0)),
        queue: false,
    };
    t.at(3, P2, throw);
    let (mut blasts, mut regenerated) = (0, 0);
    let mut last = hp(t.sim(), HIDEOUT);
    for _ in 0..160 {
        t.step();
        let now = hp(t.sim(), HIDEOUT);
        if now < last {
            blasts += 1;
        } else if blasts == 0 {
            assert_eq!(now, 400.0, "full until hit");
        } else {
            assert!(about(now - last, HIDEOUT_REGEN), "{last} to {now}");
            regenerated += 1;
        }
        last = now;
    }
    assert!(
        blasts >= 2 && regenerated >= 60,
        "{blasts} blasts, {regenerated} ticks regenerating"
    );
}

/// The Laser Turret is repaired, by the Agents (design change 0360), and regenerates nothing (**Placeholder (not in the
/// design: whether a turret, built from the ExpandMenu, regenerates as the expansions do)**: `factions/syndicate/units/
/// agent.md > Repair` sets it apart from them). Damaged, with no one to repair it, it stays as it is.
#[test]
fn a_laser_turret_regenerates_nothing() {
    let mut sim = damaged(syndicate().sim(), TURRET, 150.0);
    run(&mut sim, 64);
    assert_eq!(hp(&sim, TURRET), 150.0);
}
