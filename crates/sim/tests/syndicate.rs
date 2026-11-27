//! The Syndicate's first kinds (slice M4-A, `factions/syndicate/`), through the gate: the Agent, the Tunnel and the
//! Hideout spawn for either owner; the Tunnel blocks the ground, sees 5 grid units and has its tier's MaxHP (plan
//! decision T39), which a debug tier changes as an upgrade will, keeping the damage (design change 0231 point 4);
//! the Hideout, an underground expansion (plan decision T32), is walked over, may stand on a slope, and is hidden from
//! every player but its owner, unless it is neutral (`factions/syndicate/tunnels.md > Losing Tunnels`); the Agent's
//! pistol shoots every 20 frames at up to 2 grid units. `TestSim` checks the invariants after every tick, the vision's
//! V1 included.

use sc_sim::elevation::Elevation;
use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
const FOG_DELAY_TICKS: sc_sim::Tick = sc_sim::tuning::vision::BUILTIN.fog_delay_ticks;
use sc_sim::tunnel::TunnelState;
use sc_sim::vision::PlayerVision;
use sc_sim::{
    Command, DebugCommand, Faction, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    ScenarioProblem, SimEvent, SpawnAt, SpawnProblem, Target, TargetView, Tick, TilePos, Vec2, VisibilityState, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

/// Both players play the Syndicate.
fn syndicate(map: &str) -> ScenarioBuilder {
    ScenarioBuilder::new(map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn object(t: &TestSim, id: u32) -> &sc_sim::Object {
    t.sim().object(ObjectId(id)).expect("exists")
}

fn vision(t: &TestSim, player: PlayerId) -> &PlayerVision {
    t.sim().vision().player(player).expect("a player")
}

fn state(t: &TestSim, player: PlayerId, id: u32) -> VisibilityState {
    t.sim().vision().object_state(player, object(t, id))
}

fn command(player: PlayerId, command: Command) -> PlayerCommand {
    PlayerCommand { player, command }
}

fn order(player: PlayerId, unit: u32, kind: OrderKind) -> PlayerCommand {
    let units = vec![ObjectId(unit)];
    command(
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    )
}

fn spawn(kind: ObjectKind, owner: PlayerId, at: SpawnAt) -> DebugCommand {
    let orientation = Default::default();
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation,
    }
}

fn set_tier(id: u32, tier: u8) -> DebugCommand {
    DebugCommand::SetTier { id: ObjectId(id), tier }
}

/// Whether tile `(x, y)` is blocked in every ground grid (`NavClass::Ground` and `GroundRugged`, of size class 1, every
/// unit type's), or in none: the game's, or those its first unit of the class would build (slice M5-Z2: a game builds
/// the grids of the units in it).
fn ground_blocked(t: &TestSim, x: i32, y: i32) -> Option<bool> {
    let ground = [NavClass::Ground, NavClass::GroundRugged].map(|class| t.sim().true_grid(class, 1));
    let blocked = ground.map(|grid| grid.is_blocked(TilePos::new(x, y)));
    (blocked.iter().all(|&b| b) || blocked.iter().all(|&b| !b)).then(|| blocked[0])
}

/// The ticks in which unit `attacker`'s hits landed.
fn attacks(t: &TestSim, attacker: u32) -> Vec<Tick> {
    let events = t.events().iter();
    let hits = events.filter(|(_, e)| matches!(e, SimEvent::Attacked { attacker: a, .. } if *a == ObjectId(attacker)));
    hits.map(|&(tick, _)| tick).collect()
}

/// Every kind of the slice spawns for either player, from the map and by a debug command: the Agent with MaxHP 75,
/// the Tunnel at Tier 1 with its MaxHP 600, the Hideout with 400. Only the Tunnel carries a tier.
#[test]
fn each_kind_spawns_for_two_owners() {
    let scenario = syndicate(&flat(24, 16))
        .spawn(AGENT, Some(P1), SpawnAt::Point(centre(2, 2))) // 1
        .spawn(TUNNEL, Some(P1), tile(4, 4)) // 2
        .spawn(HIDEOUT, Some(P1), tile(10, 4)) // 3
        .build();
    assert_eq!(scenario.validate(), []);
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting()
        .spawn_unit_at(1, AGENT, P2, centre(20, 12)) // 4
        .debug_at(1, spawn(TUNNEL, P2, tile(14, 10))) // 5
        .debug_at(1, spawn(HIDEOUT, P2, tile(19, 4))) // 6
        .run_until(2);
    for (id, kind, owner) in [
        (1, AGENT, P1),
        (2, TUNNEL, P1),
        (3, HIDEOUT, P1),
        (4, AGENT, P2),
        (5, TUNNEL, P2),
        (6, HIDEOUT, P2),
    ] {
        let o = object(&t, id);
        let max_hp = [75.0, 600.0, 400.0][(id as usize - 1) % 3];
        let tier = (kind == TUNNEL).then_some(TunnelState::at(1));
        assert_eq!(
            (o.kind, o.owner, o.hp, o.max_hp(), o.tunnel.clone()),
            (kind, Some(owner), Some(max_hp), Some(max_hp), tier),
            "#{id}"
        );
        assert_eq!(o.placement.is_some(), kind != AGENT, "#{id}");
    }
}

/// `factions/syndicate/tunnels.md > Tunnel`: a surface structure, whose 4 × 4 footprint blocks the ground grids, and
/// which gives its owner sight of the tiles whose centres lie within 5 grid units of its centre (`core/vision.md >
/// Vision`).
#[test]
fn the_tunnel_blocks_the_ground_and_sees_5() {
    let mut t = syndicate(&flat(24, 20)).spawn(TUNNEL, Some(P1), tile(9, 8)).start(); // (9, 8) to (12, 11)
    t.run_until(1);
    for y in 7..13 {
        for x in 8..14 {
            let inside = (9..13).contains(&x) && (8..12).contains(&y);
            if inside {
                assert_eq!(ground_blocked(&t, x, y), Some(true), "({x}, {y})");
            }
        }
    }
    assert_eq!(ground_blocked(&t, 8, 8), Some(false));
    assert_eq!(ground_blocked(&t, 13, 11), Some(false));
    let at = object(&t, 1).pos;
    let expected: Vec<TilePos> = (0..20)
        .flat_map(|y| (0..24).map(move |x| TilePos::new(x, y)))
        .filter(|tile| tile.center().distance(at) <= gu(5.0))
        .collect();
    let map = t.sim().map();
    let tiles = (0..map.height() as i32).flat_map(|y| (0..map.width() as i32).map(move |x| TilePos::new(x, y)));
    let seen: Vec<TilePos> = tiles
        .filter(|&tile| t.sim().visibility(P1, tile) == VisibilityState::Visible)
        .collect();
    assert_eq!(seen, expected);
}

/// Plan decision T32 and design change 0240: units walk over an underground expansion. A Hideout adds no blocker to
/// the ground grids, true or known, whoever owns it; a unit may start on one, and both players' Agents walk straight
/// across one, its owner's and an enemy's.
#[test]
fn units_walk_across_a_hideout() {
    let scenario = syndicate(&flat(20, 8))
        .spawn(HIDEOUT, Some(P2), tile(8, 3)) // 1: (8, 3) to (9, 4)
        .spawn(AGENT, Some(P2), SpawnAt::Point(centre(8, 3) + Vec2::splat(32.0))) // 2: on it, from the start
        .build();
    assert_eq!(scenario.validate(), []);
    let mut t = TestSim::new(scenario);
    let (lane_1, lane_2) = (centre(0, 3).y, centre(0, 4).y);
    t.without_auto_targeting()
        .spawn_unit_at(1, AGENT, P1, Vec2::new(centre(3, 0).x, lane_1)) // 3
        .debug_at(
            1,
            DebugCommand::SetPosition {
                id: ObjectId(2),
                pos: Vec2::new(centre(3, 0).x, lane_2),
            },
        )
        .run_until(1);
    for (x, y) in [(8, 3), (9, 3), (8, 4), (9, 4)] {
        assert_eq!(ground_blocked(&t, x, y), Some(false), "({x}, {y})");
        for player in [P1, P2] {
            let known = t
                .sim()
                .known_nav(player)
                .unwrap()
                .grid(NavClass::GroundRugged, 1)
                .unwrap();
            assert!(!known.is_blocked(TilePos::new(x, y)), "{player:?} ({x}, {y})");
        }
    }
    // Spawned onto it, too.
    let onto = spawn(AGENT, P1, SpawnAt::Point(centre(9, 4)));
    assert_eq!(t.sim().validate(&command(P1, Command::Debug(onto))), Ok(()));

    let goal = |lane: f32| Vec2::new(centre(15, 0).x, lane);
    t.order_at(
        2,
        P1,
        &[ObjectId(3)],
        OrderKind::Move(Target::Point(goal(lane_1))),
        false,
    )
    .order_at(
        2,
        P2,
        &[ObjectId(2)],
        OrderKind::Move(Target::Point(goal(lane_2))),
        false,
    );
    let hideout = object(&t, 1).placement.unwrap();
    let (lo, hi) = (hideout.origin.corner(), hideout.origin.corner() + Vec2::splat(128.0));
    let mut over = [false, false];
    for _ in 0..200 {
        t.step();
        for (i, id) in [3, 2].into_iter().enumerate() {
            let at = t.pos(id);
            over[i] |= at.x > lo.x && at.x < hi.x && at.y > lo.y && at.y < hi.y;
            assert!(
                (at.y - [lane_1, lane_2][i]).abs() < 1.0,
                "#{id} keeps to its lane: {at}"
            );
        }
    }
    assert_eq!(over, [true, true], "both crossed it");
    assert_eq!((t.status(3), t.pos(3)), (MoveStatus::Arrived, goal(lane_1)));
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, goal(lane_2)));
}

/// Plan decision T32 (`factions/syndicate/structures/README.md > Tunnel Expansions`: invisible to the other players):
/// an enemy never sees an owned Hideout, even with its units standing on it and the tiles under it Visible; it never
/// records it, can't target it and doesn't shoot it. Its owner sees it. A Tunnel next to it is seen and remembered like
/// any surface structure.
#[test]
fn an_enemy_never_sees_an_owned_hideout() {
    let scenario = syndicate(&flat(30, 10))
        .spawn(HIDEOUT, Some(P1), tile(8, 4)) // 1: (8, 4) to (9, 5)
        .spawn(TUNNEL, Some(P1), tile(11, 3)) // 2: 110 su east of the Agent
        .spawn(AGENT, Some(P2), SpawnAt::Point(centre(8, 4) + Vec2::splat(32.0))) // 3: on it
        .spawn(PK, Some(P2), SpawnAt::Point(centre(6, 5))) // 4: next to it
        .build();
    let mut t = TestSim::new(scenario);
    t.order_at(1, P2, &[ObjectId(3), ObjectId(4)], OrderKind::Hold, false);
    let hidden = |t: &TestSim| {
        let hideout = object(t, 1);
        assert!(t.sim().hidden_from(hideout, P2) && !t.sim().hidden_from(hideout, P1));
        assert_eq!(t.sim().visibility(P2, TilePos::new(8, 4)), VisibilityState::Visible);
        assert_eq!(state(t, P2, 1), VisibilityState::Explored, "on Visible tiles");
        assert!(!t.sim().is_visible(P2, hideout));
        assert_eq!(state(t, P1, 1), VisibilityState::Visible, "its owner's");
        assert!(vision(t, P2).record(ObjectId(1)).is_none());
        assert_eq!(t.sim().target_view(P2, ObjectId(1)), TargetView::Gone);
        assert_eq!(t.sim().default_order(ObjectId(3), Target::Object(ObjectId(1))), None);
        for kind in [
            OrderKind::Attack(ObjectId(1)),
            OrderKind::Move(Target::Object(ObjectId(1))),
        ] {
            assert_eq!(
                t.sim().validate(&order(P2, 3, kind)),
                Err(Reject::NotVisible(ObjectId(1)))
            );
        }
    };
    for _ in 0..40 {
        t.step();
        hidden(&t);
    }
    // The Agent shot the Tunnel in its reach, never the Hideout under it.
    let targets: Vec<ObjectId> = (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::Attacked { target, .. } => Some(*target),
            _ => None,
        })
        .collect();
    assert!(
        targets.contains(&ObjectId(2)) && !targets.contains(&ObjectId(1)),
        "{targets:?}"
    );
    assert_eq!(object(&t, 1).hp, Some(400.0));
    assert!(vision(&t, P2).record(ObjectId(2)).is_some(), "the Tunnel is recorded");
    // Gone from the place, still nothing remembered.
    for (id, to) in [(3, centre(26, 2)), (4, centre(26, 7))] {
        t.debug_at(
            41,
            DebugCommand::SetPosition {
                id: ObjectId(id),
                pos: to,
            },
        );
    }
    t.run_until(41 + FOG_DELAY_TICKS);
    assert_eq!(state(&t, P2, 1), VisibilityState::Explored);
    assert!(vision(&t, P2).records().all(|r| r.id != ObjectId(1)));
    assert!(
        vision(&t, P2).remembered_record(ObjectId(2)).is_some(),
        "the Tunnel, in the fog"
    );
    assert!(!vision(&t, P2).is_record_tile(TilePos::new(8, 4)));
}

/// `factions/syndicate/tunnels.md > Losing Tunnels`: hiding is for owned expansions only. A neutral Hideout is seen
/// and recorded like any neutral structure, from the start (design change 0159), by every player.
#[test]
fn a_neutral_hideout_is_visible() {
    let mut t = syndicate(&flat(24, 10))
        .spawn(HIDEOUT, None, tile(8, 4)) // 1
        .spawn(AGENT, Some(P2), SpawnAt::Point(centre(6, 4))) // 2: in sight of it
        .start();
    t.run_until(1);
    let hideout = object(&t, 1);
    assert!(!t.sim().hidden_from(hideout, P1) && !t.sim().hidden_from(hideout, P2));
    assert_eq!(state(&t, P2, 1), VisibilityState::Visible);
    assert!(t.sim().is_visible(P2, hideout));
    assert_eq!(t.sim().target_view(P2, ObjectId(1)), TargetView::Visible);
    for player in [P1, P2] {
        let record = vision(&t, player).record(ObjectId(1)).expect("known from the start");
        assert_eq!((record.kind, record.owner), (HIDEOUT, None));
    }
    assert_eq!(
        state(&t, P1, 1),
        VisibilityState::Unexplored,
        "player 1 has never looked there"
    );
}

/// `factions/syndicate/units/agent.md > Attack`: the pistol, "one shot every 20 frames", Range 2 grid units,
/// edge to edge (design change 0031). On Hold Position, an Agent's first hit on an enemy Peacekeeper lands after its
/// Aim of 3 frames, in tick 5, and then every 20 ticks, at a gap of 128 su; 1 su further, never.
#[test]
fn an_agent_shoots_an_enemy_peacekeeper_every_20_ticks_up_to_2_gu() {
    let run = |gap: f32| {
        let mut t = syndicate(&flat(12, 3)).start();
        t.spawn_unit_at(1, AGENT, P1, Vec2::new(100.0, 96.0))
            .spawn_unit_at(1, PK, P2, Vec2::new(100.0 + 18.0 + gap + 12.0, 96.0))
            .order_at(2, P1, &[ObjectId(1)], OrderKind::Hold, false)
            .order_at(2, P2, &[ObjectId(2)], OrderKind::Hold, false)
            .run_until(90);
        t
    };
    let t = run(gu(2.0));
    assert_eq!(attacks(&t, 1), [5, 25, 45, 65, 85]);
    // 5 Damage less the Peacekeeper's PointArmor 1, five times.
    assert_eq!(object(&t, 2).hp, Some(50.0 - 5.0 * 4.0));
    assert_eq!(attacks(&run(gu(2.0) + 1.0), 1), [] as [Tick; 0]);
}

/// Plan decision T39 (`factions/syndicate/tunnels.md > Tiers`): a Tunnel has its tier's MaxHP, 600 at Tier 1; set to
/// Tier 2 (by the debug command here; upgrades come with slice M4-H) it has 800, and keeps its damage (design change
/// 0231 point 4). There is no tier 4, and nothing but a Tunnel has a tier. A player who sees the Tunnel records the
/// tier it sees (design change 0237), and keeps it once out of sight.
#[test]
fn a_tunnels_tier_sets_its_max_hp() {
    let mut t = syndicate(&flat(28, 10))
        .spawn(TUNNEL, Some(P1), tile(4, 3)) // 1: (4, 3) to (7, 6)
        .spawn(HIDEOUT, Some(P1), tile(12, 2)) // 2
        .start();
    // An enemy Agent east of the Tunnel, on Hold: its first hit lands in tick 5, its next in tick 25.
    t.spawn_unit_at(1, AGENT, P2, Vec2::new(8.0 * 64.0 + 18.0 + 32.0, 5.0 * 64.0)) // 3
        .order_at(2, P2, &[ObjectId(3)], OrderKind::Hold, false)
        .run_until(5);
    let hp = |t: &TestSim| (object(t, 1).hp, object(t, 1).max_hp(), object(t, 1).tunnel.clone());
    let tier = |tier| Some(TunnelState::at(tier));
    assert_eq!(hp(&t), (Some(596.0), Some(600.0), tier(1)));
    let record = |t: &TestSim| {
        vision(t, P2)
            .record(ObjectId(1))
            .map(|r| (r.tunnel.clone(), r.max_hp()))
    };
    assert_eq!(record(&t), Some((tier(1), Some(600.0))));

    t.debug_at(6, set_tier(1, 2)).run_until(6);
    assert_eq!(hp(&t), (Some(796.0), Some(800.0), tier(2)));
    assert_eq!(record(&t), Some((tier(2), Some(800.0))), "seen at Tier 2");
    let rejected = |t: &TestSim, id, tier| t.sim().validate(&command(P1, Command::Debug(set_tier(id, tier))));
    for bad in [0, 4] {
        assert_eq!(
            rejected(&t, 1, bad),
            Err(Reject::NoSuchTier {
                id: ObjectId(1),
                tier: bad
            })
        );
    }
    assert_eq!(
        rejected(&t, 2, 2),
        Err(Reject::NoSuchTier {
            id: ObjectId(2),
            tier: 2
        })
    );
    assert_eq!(rejected(&t, 9, 2), Err(Reject::UnknownObject(ObjectId(9))));

    // Out of sight, then Tier 3 in the fog: player 2 remembers Tier 2.
    let away = DebugCommand::SetPosition {
        id: ObjectId(3),
        pos: centre(26, 8),
    };
    t.debug_at(7, away).run_until(7 + FOG_DELAY_TICKS);
    t.debug_at(8 + FOG_DELAY_TICKS, set_tier(1, 3))
        .run_until(8 + FOG_DELAY_TICKS);
    assert_eq!(hp(&t), (Some(996.0), Some(1000.0), tier(3)));
    let remembered = vision(&t, P2)
        .remembered_record(ObjectId(1))
        .map(|r| (r.tunnel.clone(), r.max_hp()));
    assert_eq!(remembered, Some((tier(2), Some(800.0))));
}

/// `core/map.md > Elevation` (design change 0005): an underground expansion may stand on sloped tiles, a surface
/// structure such as the Tunnel may not. For the ElevationModifier the Hideout stands at the level of the terrain
/// under its centre (plan decision T32), here 1 on a footprint rising from 0 to 2.
#[test]
fn a_hideout_may_stand_on_a_slope() {
    // 8 × 4 tiles: level 0 in the west, tiles 2 to 4 rising one level each, level 3 in the east.
    let map = format!("{}\n\n{}", flat(8, 4), ["000123333"; 5].join("\n"));
    let scenario = syndicate(&map).spawn(HIDEOUT, Some(P1), tile(2, 1)).build();
    assert_eq!(scenario.validate(), []);
    let t = TestSim::new(scenario);
    let hideout = object(&t, 1);
    assert_eq!(hideout.pos, Vec2::new(192.0, 128.0));
    assert_eq!(Elevation::of_object(t.sim().map(), hideout), Elevation::Level(1));
    let tunnel = syndicate(&map).spawn(TUNNEL, Some(P1), tile(1, 0)).build();
    assert!(
        matches!(
            tunnel.validate()[..],
            [ScenarioProblem::InvalidSpawn {
                problem: SpawnProblem::NotFlat(_),
                ..
            }]
        ),
        "{:?}",
        tunnel.validate()
    );
}
