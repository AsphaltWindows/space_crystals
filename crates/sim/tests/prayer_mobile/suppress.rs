//! Suppress (`factions/cults/units/prayer_mobile.md > Placeholder stats`: "**Suppress** (hotkey **F**, instant, no
//! target): 100 energy; an arc of **90° total** in front of the vehicle's current facing, **5 grid units** long; units
//! whose centre is inside can't attack for **4 s**"; `> Abilities`: "**Affects every unit in the arc, enemies and own
//! alike** [...] **Not buildings** [...] **Air units in the arc are affected** too"; "It stops **weapon attacks** and
//! abilities that use the weapon: a Rocketman's **Target Lock can be maintained**, but no missile can be fired while
//! suppressed. **Other abilities are not affected**: a Disciple's Shroud continues"). A suppressed unit's attack under
//! way: its aim is cancelled, a Firing begun lands (`sc_sim::suppress`, **Placeholder (not in the design)**).

use sc_sim::combat::target_lock::TargetLock;
use sc_sim::{OrderKind, SpawnAt, TilePos};

use super::*;

const PEACEKEEPER: ObjectKind = ObjectKind::Peacekeeper;
/// The vehicle of every game below, player 1's, facing east, as it came into the game.
const VEHICLE: Vec2 = Vec2::new(400.0, 640.0);

/// The point `distance` su from [`VEHICLE`], `degrees` off its facing (positive clockwise, toward +y).
fn off_its_facing(degrees: f32, distance: f32) -> Vec2 {
    VEHICLE + distance * sc_sim::math::facing_vector(sc_sim::math::degrees_to_radians(degrees))
}

/// A game of `w` × `h` tiles in which player 1's Prayer-mobile (1) stands at [`VEHICLE`] and player 2 plays `faction`,
/// with `more` in it; auto-targeting on.
fn vehicle(faction: Faction, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    more(versus(faction, 30, 20).spawn(PRAYER_MOBILE, Some(P1), SpawnAt::Point(VEHICLE))).start()
}

/// The ticks in which any of `units` attacked so far, in order.
fn attacks(t: &TestSim, units: &[ObjectId]) -> Vec<Tick> {
    let mut ticks: Vec<Tick> = units.iter().flat_map(|&unit| attacks_by(t, unit)).collect();
    ticks.sort_unstable();
    ticks
}

/// Design change 0365's Verification: "Suppress facing a group of Peacekeepers and your own Militants: neither side
/// fires for 4 s". Player 1's Prayer-mobile (1), facing east, charges its 100 in 100 s; then three of player 1's
/// Militants (2–4) and, 120 su beyond them, three of player 2's Peacekeepers (5–7) come into the game ahead of it, all
/// within 5 grid units and 17° of its facing, and they fight. Once both sides have fired, it casts Suppress: all six
/// can't attack until 4 s on, and the vehicle itself has nothing. No shot of theirs takes effect from 2 ticks after
/// the cast (a Firing begun before it lands, its 2 frames at most) to the end of the 4 s; then both sides fire again
/// within a cycle.
#[test]
fn suppress_facing_peacekeepers_and_your_own_militants_neither_side_fires_for_4_s() {
    let mut t = vehicle(Faction::Gdo, |b| b);
    let pm = id(1);
    charge(&mut t, pm, 100);
    let militants: Vec<ObjectId> = [590.0, 640.0, 690.0]
        .map(|y| spawn(&mut t, MILITANT, P1, Vec2::new(560.0, y)))
        .to_vec();
    let peacekeepers: Vec<ObjectId> = [590.0, 640.0, 690.0]
        .map(|y| spawn(&mut t, PEACEKEEPER, P2, Vec2::new(680.0, y)))
        .to_vec();
    let start = t.sim().tick();
    while attacks(&t, &militants).is_empty() || attacks(&t, &peacekeepers).is_empty() {
        assert!(t.sim().tick() < start + 40, "they fight");
        t.step();
    }
    let c = cast_now(&mut t, pm, SUPPRESS);
    for &unit in militants.iter().chain(&peacekeepers) {
        assert_eq!(buffs(&t, unit), buff(SUPPRESS, c, SUPPRESS_TICKS), "{unit:?}");
    }
    assert_eq!(buffs(&t, pm), [], "not the vehicle");
    t.run_until(c + SUPPRESS_TICKS + 24);
    let silent = c + 2..c + SUPPRESS_TICKS;
    for side in [&militants, &peacekeepers] {
        let fired = attacks(&t, side);
        assert!(fired.iter().all(|tick| !silent.contains(tick)), "{side:?}: {fired:?}");
        let again = fired.iter().find(|&&tick| tick >= c + SUPPRESS_TICKS);
        assert!(again.is_some(), "{side:?} fires again: {fired:?}");
    }
}

/// Who the arc takes in, player 2 the Cults': its Sparrow (2), circling ahead of the vehicle, and its Recruits 44° off
/// the facing 250 su out (4) and 30° off it 316 su out (6) are suppressed, as is player 1's own Recruit 40° off it
/// (8); player 2's Storage (3), its centre 144 su straight ahead, isn't ("Not buildings"), nor its Recruits 46° off the
/// facing (5) or 324 su straight ahead (7), outside the 90° and the 5 grid units (320 su).
#[test]
fn a_sparrow_in_the_arc_is_suppressed_and_a_building_or_a_unit_outside_it_isn_t() {
    let mut t = vehicle(Faction::Cults, |b| {
        b.spawn(ObjectKind::Sparrow, Some(P2), SpawnAt::Point(Vec2::new(560.0, 640.0)))
            .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(7, 9)))
            .spawn(RECRUIT, Some(P2), SpawnAt::Point(off_its_facing(-44.0, 250.0)))
            .spawn(RECRUIT, Some(P2), SpawnAt::Point(off_its_facing(46.0, 250.0)))
            .spawn(RECRUIT, Some(P2), SpawnAt::Point(off_its_facing(-30.0, 316.0)))
            .spawn(RECRUIT, Some(P2), SpawnAt::Point(off_its_facing(0.0, 324.0)))
            .spawn(RECRUIT, Some(P1), SpawnAt::Point(off_its_facing(-40.0, 156.0)))
    });
    t.without_auto_targeting();
    let pm = id(1);
    charge(&mut t, pm, 100);
    let o = t.sim().object(pm).expect("the vehicle");
    assert_eq!((o.pos, o.facing), (VEHICLE, 0.0), "as it came in");
    let sparrow = t.sim().object(id(2)).expect("circling").pos;
    assert!(
        sc_sim::suppress::in_arc(VEHICLE, 0.0, std::f32::consts::FRAC_PI_2, 320.0, sparrow),
        "{sparrow}"
    );
    let c = cast_now(&mut t, pm, SUPPRESS);
    for n in [2, 4, 6, 8] {
        assert_eq!(buffs(&t, id(n)), buff(SUPPRESS, c, SUPPRESS_TICKS), "#{n}");
    }
    for n in [3, 5, 7] {
        assert_eq!(buffs(&t, id(n)), [], "#{n}");
    }
}

/// Player 2's Target Lock now, on Rocketman `rocketman`.
fn lock(t: &TestSim, rocketman: ObjectId) -> Option<TargetLock> {
    t.sim().object(rocketman)?.attack.as_ref()?.target_lock
}

/// The ticks of `shooter`'s missiles launched so far.
fn launches(t: &TestSim, shooter: ObjectId) -> Vec<Tick> {
    (t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProjectileLaunched { shooter: by, .. } if by == shooter => Some(*tick),
            _ => None,
        })
        .collect()
}

/// "A Rocketman's **Target Lock can be maintained**, but no missile can be fired while suppressed". Player 2's
/// Rocketman (2), 200 su ahead of player 1's vehicle, locks onto player 1's Recruit (3) 300 su beyond it, in its
/// LockRange of 7 grid units and in sight; auto-targeting off. Unsuppressed, it is locked LockOnDuration (48 ticks)
/// after the order and fires its first missile soon after. Suppressed 10 ticks into the locking, it locks in the same
/// tick, its lock on the Recruit all the while, and fires nothing until the 4 s are over: its first missile leaves
/// once they are, its aim's 6 frames and its Firing's 2 later.
#[test]
fn a_suppressed_rocketman_keeps_its_lock_and_fires_once_the_4_s_are_over() {
    let game = |suppressed: bool| {
        let mut t = vehicle(Faction::Gdo, |b| {
            b.spawn(ObjectKind::Rocketman, Some(P2), SpawnAt::Point(Vec2::new(600.0, 640.0)))
                .spawn(RECRUIT, Some(P1), SpawnAt::Point(Vec2::new(900.0, 640.0)))
        });
        t.without_auto_targeting();
        let (pm, rocketman, recruit) = (id(1), id(2), id(3));
        charge(&mut t, pm, 100);
        let ordered = t.sim().tick() + 1;
        t.order_at(ordered, P2, &[rocketman], OrderKind::TargetLock(recruit), false);
        t.run_until(ordered + 9);
        let c = suppressed.then(|| cast_now(&mut t, pm, SUPPRESS));
        let mut locked = None;
        while launches(&t, rocketman).is_empty() {
            assert!(t.sim().tick() < ordered + 200, "it fires");
            t.step();
            let now = lock(&t, rocketman).expect("its lock holds");
            assert_eq!(now.target, recruit);
            if now.locked && locked.is_none() {
                locked = Some(t.sim().tick() - ordered);
            }
        }
        let fired = launches(&t, rocketman)[0];
        (c.map(|c| c - ordered), locked.expect("locked"), fired - ordered)
    };
    let (_, locked, fired) = game(false);
    assert!(locked >= 47 && fired > locked, "locked in {locked}, fired {fired}");
    let (c, locked_suppressed, fired_suppressed) = game(true);
    let c = c.expect("cast");
    assert_eq!(c, 10);
    assert_eq!(locked_suppressed, locked, "the locking goes on as it would");
    assert_eq!(
        fired_suppressed,
        c + SUPPRESS_TICKS + 6 + 2 - 1,
        "aimed and fired once the 4 s are over"
    );
}

/// "**Other abilities are not affected**: a Disciple's Shroud continues". Player 1's Disciple (2), 160 su ahead of its
/// vehicle, channels Shroud; the vehicle's Suppress takes it in, and it channels on, its cloud there in every tick of
/// the 4 s and after, its order still Shroud, its energy draining as before (5 a second, its own 0.5 back).
#[test]
fn a_suppressed_disciple_goes_on_channelling_shroud() {
    let mut t = vehicle(Faction::Cults, |b| {
        b.spawn(ObjectKind::Disciple, Some(P1), SpawnAt::Point(Vec2::new(560.0, 640.0)))
    });
    t.without_auto_targeting();
    let (pm, disciple) = (id(1), id(2));
    charge(&mut t, pm, 100);
    let channel = t.sim().tick() + 1;
    t.order_at(channel, P1, &[disciple], OrderKind::Shroud, false);
    t.run_until(channel + 9);
    let before = energy(&t, disciple);
    let c = cast_now(&mut t, pm, SUPPRESS);
    assert_eq!(buffs(&t, disciple), buff(SUPPRESS, c, SUPPRESS_TICKS));
    while t.sim().tick() < c + SUPPRESS_TICKS + 16 {
        let members: Vec<ObjectId> = (t.sim().clouds().list().iter())
            .flat_map(|cloud| cloud.members.iter().map(|m| m.id))
            .collect();
        assert_eq!(members, [disciple], "tick {}", t.sim().tick());
        let order = t
            .sim()
            .object(disciple)
            .and_then(|o| o.orders.as_ref()?.current.as_ref());
        assert_eq!(order.map(|active| active.order.kind), Some(OrderKind::Shroud));
        t.step();
    }
    let ended = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::ShroudEnded { .. }));
    assert!(!ended, "it channels on");
    let ticks = t.sim().tick() - c + 1;
    assert_eq!(
        before - energy(&t, disciple),
        ticks * (5 * ONE / 16 - A_TICK),
        "4.5 a second"
    );
}
