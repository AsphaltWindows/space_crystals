//! The Grenadier in play (slice M4-U; `factions/syndicate/units/grenadier.md`, design changes 0187 and 0260), with its
//! own numbers, through the gate (the mechanics, with a test thrower of fixed numbers, are `sc_sim::combat`'s tests,
//! slice M4-O): design change 0260's verification, "A Grenadier's grenade visibly arcs about 2 s to a target 4 GU
//! away; a target that keeps walking escapes"; its rate of fire; design change 0264's note, "A Grenadier blast (100)
//! over a hidden 2x2 expansion with n = 6 deals about 93", and what its thrower's owner learns of the expansion:
//! nothing; and the Tunnel Network, which it walks into and is ejected from as infantry. Its training at the Hideout,
//! behind a Chemical Plant, is in `tests/prerequisites/`.
//!
//! Every test runs under the invariants of `TestSim`.

use std::f32::consts::PI;

use sc_sim::combat::flight_ticks;
use sc_sim::defs::TRANSIT_DELAY_TICKS;
use sc_sim::network::{Place, admits};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, Faction, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject, SimEvent, SpawnAt,
    TICKS_PER_SECOND, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const GRENADIER: ObjectKind = ObjectKind::Grenadier;
/// `> AttackAttributes`: "AoE radius: 40 space units (0.625 grid units)".
const RADIUS: f32 = 40.0;
/// `> AttackAttributes`: "one grenade per 40 frames".
const CYCLE: Tick = 40;

/// 16 × 10 open tiles.
fn open() -> String {
    "................\n".repeat(10)
}

fn validate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

fn order(unit: ObjectId, kind: OrderKind) -> Command {
    Command::Order {
        units: vec![unit],
        kind,
        queue: false,
    }
}

/// The launches of the run so far: (tick, shooter, from, to, lands_at).
fn launches(t: &TestSim) -> Vec<(Tick, ObjectId, Vec2, Vec2, Tick)> {
    let launch = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::ProjectileLaunched {
            shooter,
            from,
            to,
            lands_at,
            ..
        } => Some((*tick, shooter, from, to, lands_at)),
        _ => None,
    };
    t.events().iter().filter_map(launch).collect()
}

/// The blasts of the run so far: (tick, centre, radius).
fn blasts(t: &TestSim) -> Vec<(Tick, Vec2, f32)> {
    let blast = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Blast { at, radius, .. } => Some((*tick, at, radius)),
        _ => None,
    };
    t.events().iter().filter_map(blast).collect()
}

/// The HP damage `id` took in each hit of the run so far: (tick, damage).
fn hits_on(t: &TestSim, id: ObjectId) -> Vec<(Tick, f32)> {
    let hit = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::Hit { target, damage, .. } if target == id => Some((*tick, damage)),
        _ => None,
    };
    t.events().iter().filter_map(hit).collect()
}

/// Runs until the first launch, and returns its tick.
fn until_launched(t: &mut TestSim) -> Tick {
    while launches(t).is_empty() {
        assert!(t.sim().tick() < 100, "nothing was thrown\n{}", t.ascii(None));
        t.step();
    }
    t.sim().tick()
}

/// Design change 0260's verification, "A Grenadier's grenade visibly arcs about 2 s to a target 4 GU away; a target
/// that keeps walking escapes" (`> AttackAttributes`: "ProjectileSpeed: 8 space units/frame (a 4 GU throw takes 2 s,
/// so moving units can step out of the blast)"), with the real kind on both sides: player 1's Grenadier attacks player
/// 2's, 4 grid units east. The grenade leaves at the end of the thrower's Firing for where the target stood at the end
/// of its Aiming, and lands 32 ticks, 2 seconds, later. A target that stands takes its share of the blast less its
/// FullArmor of 1, the whole of it inside the 40 su circle ("a 36x36 infantry fully inside takes about 26 before
/// armor"); one that walks off at its 5 su per tick from the launch on is clear of the circle when it lands, and takes
/// nothing.
#[test]
fn a_grenade_takes_two_seconds_over_four_grid_units_and_a_walker_escapes() {
    for walks in [false, true] {
        let from = Vec2::new(200.0, 320.0);
        let stood = from + Vec2::new(256.0, 0.0);
        let (thrower, target) = (ObjectId(1), ObjectId(2));
        let mut t = ScenarioBuilder::new(&open()).start();
        t.without_auto_targeting()
            .spawn_unit_at(1, GRENADIER, P1, from)
            .spawn_unit_at(1, GRENADIER, P2, stood)
            .order_at(2, P1, &[thrower], OrderKind::Attack(target), false);
        let launched = until_launched(&mut t);
        let lands_at = launched + 2 * TICKS_PER_SECOND;
        assert_eq!(launches(&t), [(launched, thrower, from, stood, lands_at)], "2 seconds");
        assert_eq!(lands_at - launched, 32);
        if walks {
            let away = Target::Point(Vec2::new(1000.0, 320.0));
            t.order_at(launched + 1, P2, &[target], OrderKind::Move(away), false);
        }
        t.run_until(lands_at - 1);
        assert_eq!(blasts(&t), [], "in the air");
        t.step();
        assert_eq!(blasts(&t), [(lands_at, stood, RADIUS)]);
        let o = t.sim().object(target).expect("alive");
        if walks {
            assert!(o.pos.x - stood.x > RADIUS + 18.0, "clear of the circle: {}", o.pos);
            assert_eq!((hits_on(&t, target), o.hp), (vec![], Some(70.0)), "untouched");
        } else {
            let share = 100.0 * (36.0 * 36.0 / (PI * RADIUS * RADIUS));
            assert_eq!(share.round(), 26.0, "about 26 before armor");
            let [(tick, taken)] = hits_on(&t, target)[..] else {
                panic!("one hit: {:?}", hits_on(&t, target));
            };
            assert_eq!(tick, lands_at);
            assert!((taken - (share - 1.0)).abs() < 1e-4, "{taken}");
        }
    }
}

/// `> AttackAttributes`: "AimDuration: 6 frames; FiringDuration: 2 frames; CooldownDuration: 4 frames; ReloadDuration:
/// 28 frames (one grenade per 40 frames, 2.5 s)". At a Peacekeeper of player 2's that stands 3 grid units off, player
/// 1's Grenadier throws a grenade every 40 ticks, each landing on it 24 ticks after it leaves (192 su at 8 su a tick).
#[test]
fn it_throws_a_grenade_every_40_ticks_at_a_target_that_stands() {
    let from = Vec2::new(200.0, 320.0);
    let at = from + Vec2::new(192.0, 0.0);
    let (thrower, target) = (ObjectId(1), ObjectId(2));
    let mut t = ScenarioBuilder::new(&open()).start();
    t.without_auto_targeting()
        .spawn_unit_at(1, GRENADIER, P1, from)
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, at)
        .order_at(2, P1, &[thrower], OrderKind::Attack(target), false);
    let first = until_launched(&mut t);
    assert_eq!(flight_ticks(from, at, Some(8.0)), 24);
    t.run_until(first + 3 * CYCLE + 24);
    let thrown: Vec<Tick> = launches(&t).iter().map(|launch| launch.0).collect();
    assert_eq!(thrown, [0, 1, 2, 3].map(|n| first + n * CYCLE));
    let landed: Vec<Tick> = hits_on(&t, target).iter().map(|hit| hit.0).collect();
    assert_eq!(landed, thrown.iter().map(|tick| tick + 24).collect::<Vec<_>>());
    assert_eq!(CYCLE as f32 / TICKS_PER_SECOND as f32, 2.5, "2.5 s");
    assert!(
        t.sim().object(target).is_some(),
        "4 grenades of about 10.5 leave its 50 HP standing"
    );
}

/// Design change 0264's note, "A Grenadier blast (100) over a hidden 2x2 expansion with n = 6 deals about 93"
/// (`factions/syndicate/structures/README.md > Tunnel Expansions > High armor underground`: a 2x2 expansion's
/// FullArmor is 24), and hidden information (plan decision T32; `factions/syndicate/structures/README.md > Tunnel
/// Expansions`: "Expansions are invisible to enemies without detection"). Player 2's Grenadier throws at the ground
/// over player 1's Hideout, which player 2 can't see; it may (`systems/combat.md > AttackType > CanTargetGround`). The
/// blast, its whole circle inside the Hideout, does 100 less 24 × 5026.5 / 16384, about 92.6, and reveals the thrower
/// to player 1 when it lands (design change 0293). Player 2 learns nothing of the Hideout: before the throw, after the
/// blast and through the next grenade, player 2 neither sees nor remembers it, nor can name it in an order.
#[test]
fn a_blast_over_a_hidden_hideout_does_about_93_and_tells_its_thrower_nothing() {
    let mut t = ScenarioBuilder::new(&open())
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(TilePos::new(8, 4)))
        .start();
    let (hideout, thrower) = (ObjectId(1), ObjectId(2));
    let centre = t.sim().object(hideout).expect("the Hideout").pos;
    let from = centre - Vec2::new(64.0 + 180.0, 0.0);
    t.without_auto_targeting().spawn_unit_at(1, GRENADIER, P2, from);
    t.run_until(1);
    let unknown = |t: &TestSim| {
        let o = t.sim().object(hideout).expect("standing");
        assert!(t.sim().hidden_from(o, P2) && !t.sim().is_visible(P2, o));
        let vision = t.sim().vision().player(P2).expect("in the game");
        assert!(vision.record(hideout).is_none(), "unremembered");
        let attack = order(thrower, OrderKind::Attack(hideout));
        assert_eq!(validate(t, P2, attack), Err(Reject::NotVisible(hideout)));
    };
    unknown(&t);
    let throw = order(thrower, OrderKind::AttackGround(centre));
    assert_eq!(validate(&t, P2, throw.clone()), Ok(()));
    t.at(2, P2, throw);
    let launched = until_launched(&mut t);
    let lands_at = launched + flight_ticks(from, centre, Some(8.0));
    assert_eq!(launches(&t), [(launched, thrower, from, centre, lands_at)]);
    let revealed = |t: &TestSim| t.sim().vision().player(P1).unwrap().is_revealed(thrower);
    t.run_until(lands_at - 1);
    assert!(!revealed(&t), "not before the blast");
    unknown(&t);
    t.step();
    let [(tick, taken)] = hits_on(&t, hideout)[..] else {
        panic!("one hit on the Hideout: {:?}", hits_on(&t, hideout));
    };
    let expected = 100.0 - 24.0 * (PI * RADIUS * RADIUS) / (128.0 * 128.0);
    assert_eq!(tick, lands_at);
    assert!((taken - expected).abs() < 1e-3 && taken.round() == 93.0, "{taken}");
    let hp = t.sim().object(hideout).and_then(|o| o.hp).expect("standing");
    assert!((hp - (400.0 - taken)).abs() < 1e-3, "{hp}");
    assert!(revealed(&t), "the thrower, to the Hideout's owner");
    unknown(&t);
    t.run_until(launched + CYCLE + 1);
    assert_eq!(launches(&t).len(), 2, "an Attack Ground keeps throwing");
    unknown(&t);
}

/// `factions/syndicate/tunnels.md > Tunnel Network > Transit Tier Requirements`: "Tier 1+: Infantry", which the
/// Grenadier is ("UnitBase - HeavyInfantry (assumed, like all Syndicate infantry)"). On a 30 × 22 map, player 1's Tier
/// 1 Tunnel on tiles (12, 10) to (15, 13), unturned, its Side A north (y = 640), its entry path's spot at (866, 640),
/// its exit path's at (926, 640). The Grenadier, just north of Side A, takes an Enter, walks in and is held in the
/// Tunnel Network, off the map; an Eject is refused for the transit delay (design change 0245, `Reject::InTransit`),
/// and then brings it out on the exit path's spot, its box against the wall (design change 0250), from where it walks
/// 1 grid unit straight out (design change 0288).
#[test]
fn it_walks_into_a_tier_1_tunnel_and_is_ejected_as_infantry() {
    let map = vec![".".repeat(30); 22].join("\n");
    let mut t = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .start();
    let (tunnel, grenadier) = (ObjectId(1), ObjectId(2));
    t.without_auto_targeting()
        .spawn_unit_at(1, GRENADIER, P1, Vec2::new(866.0, 580.0));
    t.run_until(1);
    assert_eq!(admits(t.sim().object(tunnel).unwrap(), GRENADIER), Ok(()));
    let enter = order(grenadier, OrderKind::Enter(tunnel));
    assert_eq!(validate(&t, P1, enter.clone()), Ok(()));
    t.at(2, P1, enter);
    while t.sim().held_unit(grenadier).is_none() {
        assert!(t.sim().tick() < 100, "it walks in\n{}", t.ascii(None));
        t.step();
    }
    let entered = t.sim().tick();
    let held = t.sim().held_unit(grenadier).unwrap();
    assert_eq!(
        (held.place, held.since, held.object.kind),
        (Place::Network, entered, GRENADIER)
    );
    assert!(t.sim().object(grenadier).is_none(), "off the map");
    let eject = Command::Tunnel {
        tunnel,
        command: TunnelCommand::Eject { kind: GRENADIER },
    };
    t.run_until(entered + TRANSIT_DELAY_TICKS - 1);
    assert_eq!(validate(&t, P1, eject.clone()), Err(Reject::InTransit(GRENADIER)));
    t.at(entered + TRANSIT_DELAY_TICKS, P1, eject)
        .run_until(entered + TRANSIT_DELAY_TICKS);
    let exit = Vec2::new(926.0, 640.0 - 18.0 - 1.0 / 64.0);
    assert_eq!(t.pos(grenadier.0), exit, "out on the exit path's spot");
    assert!(t.sim().held_unit(grenadier).is_none());
    t.run(30);
    assert_eq!(
        t.pos(grenadier.0),
        exit - Vec2::new(0.0, 64.0),
        "1 grid unit straight out"
    );
}
