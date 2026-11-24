//! Beacon Deployment (slice M3-F; `factions/colonists/economy.md > Beacon Deployment`, `sc_sim::deploy`): a Drone walks
//! to a site and transforms in place into a Beacon over 48 frames. One test per acceptance point of the plan
//! (`artifacts/director/m3-plan.md` §3.3 F) and per Verification point of design changes 0109 to 0114 that concerns a
//! deployment: the walk and the start on arrival, the timing, the HP, the Shields and the Beacon Capacity, the cancel
//! and its fractions, the load, defeat, a destruction, the footprint claimed, Shift, saves and replays. The gate (what
//! the player knows, the reservations of design change 0125, the commands a deploying Beacon refuses) and the failures
//! on arrival are `tests/deploy_gate.rs`'. The walks are measured by their outcomes, never by their ticks: the Drone's
//! DragMovement (slice M3-G) sets their pace. `TestSim` checks the movement invariants after every tick.

use sc_sim::deploy::{self, DEPLOYMENT_FRAMES};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, Reject, Replay, SU_PER_GU, Sim, SimEvent, SpawnAt, Target, Tick, TickInputs, TilePos, Vec2,
    VisibilityState,
};

use ObjectKind::{Beacon, CrystalPatch, Drone, EgressNode, Peacekeeper};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Where most tests deploy: the Beacon on tiles (8, 9) to (9, 10), in the sight of Drone 2 where it starts.
const SITE: TilePos = TilePos::new(8, 9);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(at(x, y).center())
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("the object exists")
}

fn deploy_at(origin: TilePos) -> OrderKind {
    OrderKind::Deploy { origin }
}

/// Player 1's order `kind` to `units`.
fn order(units: &[u32], kind: OrderKind, queue: bool) -> Command {
    Command::Order {
        units: units.iter().map(|&n| id(n)).collect(),
        kind,
        queue,
    }
}

/// Cancel Deployment (`> Cancel Deployment`): `ConstructionCommand::CancelTransform` to `objects`.
fn cancel(objects: &[u32]) -> Command {
    Command::Construction {
        objects: objects.iter().map(|&n| id(n)).collect(),
        command: ConstructionCommand::CancelTransform,
    }
}

/// Schedules player 1's `command` for the next tick and runs it.
fn next(t: &mut TestSim, command: Command) -> &mut TestSim {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command).step()
}

/// `player`'s Beacon Capacity: used, available.
fn capacity(t: &TestSim, player: PlayerId) -> (u32, u32) {
    let cap = t.sim().army_cap(player).expect("a Colonists player");
    (cap.used, cap.available)
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).unwrap().crystals
}

/// The `KindChanged` events so far: tick, object, from, to.
fn kind_changes(t: &TestSim) -> Vec<(Tick, u32, ObjectKind, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::KindChanged { id, from, to } => Some((*tick, id.0, *from, *to)),
            _ => None,
        })
        .collect()
}

/// The transformations completed so far: tick, object, kind.
fn completions(t: &TestSim) -> Vec<(Tick, u32, ObjectKind)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::TransformCompleted { id, kind } => Some((*tick, id.0, *kind)),
            _ => None,
        })
        .collect()
}

/// The ticks of the events `pick` picks so far.
fn ticks_of(t: &TestSim, pick: impl Fn(&SimEvent) -> bool) -> Vec<Tick> {
    t.events()
        .iter()
        .filter(|(_, e)| pick(e))
        .map(|&(tick, _)| tick)
        .collect()
}

/// Steps until Drone `n` is no longer one: it began deploying. Returns where it stood at the end of the last tick it
/// was a Drone. Panics if its Deploy failed, or if it takes more than 600 ticks.
fn walk(t: &mut TestSim, n: u32) -> Vec2 {
    let mut last = t.pos(n);
    while object(t, n).kind == Drone {
        let failed = t
            .sim()
            .events()
            .iter()
            .any(|e| matches!(e, SimEvent::DeployFailed { drone, .. } if *drone == id(n)));
        assert!(!failed, "Drone {n}'s Deploy failed in tick {}", t.sim().tick());
        assert!(t.sim().tick() < 600, "Drone {n} still walks at {}", t.pos(n));
        last = t.pos(n);
        t.step();
    }
    last
}

/// `builder` with both players playing the Colonists (plan decision T27), whose army cap is Beacon Capacity.
fn colonists(builder: ScenarioBuilder) -> ScenarioBuilder {
    builder.faction(P1, Faction::Colonists).faction(P2, Faction::Colonists)
}

/// A 32 × 20 flat map for two Colonists players: player 1's Egress Node 1 on (2, 2) to (3, 3) and its Drone 2 at the
/// centre of tile (6, 10); player 2's Egress Node 3 on (28, 16) to (29, 17), far out of player 1's sight.
fn base() -> ScenarioBuilder {
    colonists(ScenarioBuilder::new(&flat(32, 20)))
        .spawn(EgressNode, Some(P1), tile(2, 2))
        .spawn(Drone, Some(P1), point(6, 10))
        .spawn(EgressNode, Some(P2), tile(28, 16))
}

/// [`base`] at its Beacon Capacity, 10/10: player 1's second Egress Node 4 on (2, 6) to (3, 7), and its nine Drones
/// 5 to 13 in a row from (14, 18) to (22, 18).
fn at_capacity() -> ScenarioBuilder {
    (14..23).fold(base().spawn(EgressNode, Some(P1), tile(2, 6)), |builder, x| {
        builder.spawn(Drone, Some(P1), point(x, 18))
    })
}

/// [`base`] with Drone 2 ordered in tick 2, the site in its sight, to deploy at [`SITE`] (checked by the gate first),
/// run until it began. Returns the tick it began.
fn deployed(t: &mut TestSim) -> Tick {
    t.step();
    let tiles = deploy::site(SITE);
    assert!(
        tiles
            .tiles()
            .all(|tile| t.sim().visibility(P1, tile) == VisibilityState::Visible),
        "the site is in sight"
    );
    let command = order(&[2], deploy_at(SITE), false);
    let cmd = PlayerCommand {
        player: P1,
        command: command.clone(),
    };
    assert_eq!(t.sim().validate(&cmd), Ok(()));
    next(t, command);
    walk(t, 2);
    t.sim().tick()
}

/// The plan's acceptance, "Deploy in sight", at Beacon Capacity 10/10:
/// - `> Deploy Command`: "The Drone walks to the location, then begins deploying in place"; design change 0117: "the
///   Drone walks to the centre of the Beacon's footprint and starts deploying when it gets there". Drone 2 stood on
///   the centre of its site the tick before it began.
/// - Design change 0109: "The moment deployment begins, the Drone becomes a Beacon under deployment", "the same
///   object", "a structure on its 2x2 footprint": the object is the Beacon on the site, transforming from the Drone
///   (no footprint to go back to, nothing paid), with neither orders nor a move nor a worker.
/// - Design change 0112: "The Drone's 1 Beacon Capacity is freed as soon as deployment starts", "The Beacon's 5 is
///   provided only on completion": 9/10 at once, 9/15 at completion.
/// - `> Deployment Duration` (48 frames) and `ARCHITECTURE.md > Economy > Recipe timing`: the first frame is the
///   tick it began, and it is complete at the end of that tick + 47.
/// - `> HP/Shields Interpolation`: "HP = Drone MaxHP + (Beacon MaxHP - Drone MaxHP) x progress", 20 to 120, and an
///   undamaged one at its maximum; its maximum Shields 10 throughout ("Shields stay at 10", 10 to 10).
#[test]
fn a_drone_walks_to_the_centre_of_its_site_and_deploys_on_arrival() {
    let mut t = at_capacity().start();
    assert_eq!(capacity(&t, P1), (10, 10));
    t.step();
    next(&mut t, order(&[2], deploy_at(SITE), false));
    assert_eq!(object(&t, 2).kind, Drone, "it walks there first");
    let centre = deploy::site(SITE).center();
    let stood = walk(&mut t, 2);
    let start = t.sim().tick();
    assert_eq!(stood, centre, "it got to the centre of its site");
    assert_eq!(kind_changes(&t), [(start, 2, Drone, Beacon)]);
    let o = object(&t, 2);
    assert_eq!(
        (o.owner, o.placement, o.pos),
        (Some(P1), Some(deploy::site(SITE)), centre)
    );
    assert!(o.orders.is_none() && o.mover.is_none() && o.worker.is_none());
    assert!(o.is_transforming() && o.is_deploying());
    let deploying = o.transform.as_ref().unwrap();
    assert_eq!(
        (
            deploying.from,
            deploying.from_placement,
            deploying.total,
            deploying.paid,
            deploying.load
        ),
        (Drone, None, DEPLOYMENT_FRAMES, 0, 0)
    );
    assert_eq!(DEPLOYMENT_FRAMES, 48);
    assert_eq!(capacity(&t, P1), (9, 10), "the Drone's 1 freed, the Beacon's 5 not yet");
    assert_eq!(
        t.sim().pending_deploys(P1),
        [],
        "the site is built on, no longer pending"
    );
    for frames in 1..DEPLOYMENT_FRAMES {
        t.run_until(start + frames - 1);
        let o = object(&t, 2);
        assert_eq!(o.transform.as_ref().unwrap().frames_done(), frames);
        let max = 20.0 + 100.0 * frames as f32 / 48.0;
        assert_eq!(
            (o.max_hp(), o.hp, o.max_shields()),
            (Some(max), Some(max), Some(10.0)),
            "{frames} frames"
        );
        assert_eq!(capacity(&t, P1), (9, 10));
    }
    assert_eq!(completions(&t), []);
    t.step();
    assert_eq!(completions(&t), [(start + 47, 2, Beacon)]);
    let o = object(&t, 2);
    assert!(!o.is_transforming() && !o.is_deploying());
    assert_eq!((o.kind, o.hp, o.max_hp()), (Beacon, Some(120.0), Some(120.0)));
    assert_eq!(capacity(&t, P1), (9, 15));
    assert_eq!(kind_changes(&t).len(), 1, "one object throughout");
}

/// Design change 0109 ("On cancel, it turns back into the Drone, standing at the centre of the footprint") and 0111's
/// Verification ("Cancel a deploy at 90/120 → Drone 15/20"; "Cancelling never kills: the result is at least 1 HP"),
/// with `> Cancel Deployment`: "the unit reverts to being a normal, fully-functional Drone". 90/120 is 75%, which a
/// deploying Beacon has at 52.5 of its 70 halfway (24 frames: 20 + 100 × 0.5); its HP is set by editing a save, as no
/// command deals an exact amount. Cancelled there it is a Drone at 15/20; cancelled with 0.5 of 70 it is a Drone at
/// 1; cancelled undamaged it is a Drone at 20/20, "no HP/Shields loss". Each time it is the same object, at the
/// centre of the footprint, idle, its 1 Beacon Capacity back, the footprint free, and nothing is refunded (the
/// deployment cost nothing).
#[test]
fn a_cancel_turns_it_back_into_the_drone_with_the_fraction_of_its_hp() {
    let mut t = base().start();
    let start = deployed(&mut t);
    t.run_until(start + 23);
    let o = object(&t, 2);
    assert_eq!((o.max_hp(), o.hp), (Some(70.0), Some(70.0)), "halfway");
    assert_eq!(capacity(&t, P1), (0, 5));
    let centre = deploy::site(SITE).center();
    for (hp, drone) in [(52.5, 15.0), (0.5, 1.0), (70.0, 20.0)] {
        let mut sim = edited(t.sim(), 2, "hp", serde_json::json!(hp));
        let cmd = PlayerCommand {
            player: P1,
            command: cancel(&[2]),
        };
        assert_eq!(sim.validate(&cmd), Ok(()));
        sim.step(&TickInputs { commands: vec![cmd] });
        assert_eq!(
            sim.events()[..2],
            [
                SimEvent::TransformCancelled {
                    id: id(2),
                    kind: Beacon,
                    refund: 0
                },
                SimEvent::KindChanged {
                    id: id(2),
                    from: Beacon,
                    to: Drone
                },
            ]
        );
        let o = sim.object(id(2)).unwrap();
        assert_eq!(
            (o.kind, o.hp, o.max_hp()),
            (Drone, Some(drone), Some(20.0)),
            "from {hp}"
        );
        assert_eq!((o.pos, o.placement, o.transform.as_ref()), (centre, None, None));
        assert!(o.orders.as_ref().is_some_and(|orders| orders.is_idle()));
        assert!(o.mover.is_some() && o.worker.is_some());
        let cap = sim.army_cap(P1).unwrap();
        assert_eq!((cap.used, cap.available), (1, 5), "the Drone's 1 is back");
        // The footprint's tiles are free again: a unit can be put on each of them.
        for tile in deploy::site(SITE).tiles() {
            let spawn = PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Spawn {
                    kind: Peacekeeper,
                    owner: Some(P2),
                    at: SpawnAt::Point(tile.center()),
                    orientation: Default::default(),
                }),
            };
            assert_ne!(sim.validate(&spawn), Err(Reject::Blocked), "{tile:?}");
        }
    }
}

/// The Shields of a deployment (`crate::shields`, slice M3-B; `> HP/Shields Interpolation`: "Shields stay at 10", the
/// Drone's MaxShields and the Beacon's). Drone 2, its Shields set to 0 a tile from its site's centre (edited into a
/// save, as no command deals an exact amount), begins deploying with what it had, not the Beacon's full 10, and they go
/// on regenerating, 0.25 a frame up to the maximum now (`systems/combat.md > Shields > ShieldRegenRate`, design changes
/// 0110 and 0146). Design change 0111, "Shields by the same fraction (no minimum needed)": cancelled halfway with 4 of
/// its 10, it is a Drone with 4 of its 10, and with none, a Drone with none, unlike the HP's 1. Each is 0.25 more at
/// the end of the cancel's tick, as Shields regenerate after the commands.
#[test]
fn a_deployment_keeps_the_shields_and_a_cancel_their_fraction() {
    let shields = |sim: &Sim| sim.object(id(2)).unwrap().shields.unwrap();
    let mut t = base().start();
    t.step();
    next(&mut t, order(&[2], deploy_at(SITE), false));
    let centre = deploy::site(SITE).center();
    while object(&t, 2).pos.distance(centre) > SU_PER_GU {
        assert!(t.sim().tick() < 600, "it gets there");
        t.step();
    }
    assert_eq!(object(&t, 2).kind, Drone, "still walking");
    let mut sim = edited(t.sim(), 2, "shields", serde_json::json!(0.0));
    let mut before = shields(&sim);
    while sim.object(id(2)).unwrap().kind == Drone {
        assert!(sim.tick() < 600, "it deploys");
        before = shields(&sim);
        sim.step(&TickInputs::default());
    }
    assert!(
        before < 9.75,
        "the walk is shorter than the Shields take to come back: {before}"
    );
    assert_eq!(shields(&sim), before + 0.25, "kept, and regenerating");
    sim.step(&TickInputs::default());
    assert_eq!(shields(&sim), before + 0.5);
    assert_eq!(sim.object(id(2)).unwrap().max_shields(), Some(10.0));

    let mut t = base().start();
    let start = deployed(&mut t);
    t.run_until(start + 23);
    let o = object(&t, 2);
    assert_eq!((o.max_shields(), o.shields), (Some(10.0), Some(10.0)), "halfway");
    for (beacon, drone) in [(4.0, 4.0), (0.0, 0.0)] {
        let mut sim = edited(t.sim(), 2, "shields", serde_json::json!(beacon));
        let cmd = PlayerCommand {
            player: P1,
            command: cancel(&[2]),
        };
        sim.step(&TickInputs { commands: vec![cmd] });
        let o = sim.object(id(2)).unwrap();
        assert_eq!(
            (o.kind, o.shields, o.max_shields()),
            (Drone, Some(drone + 0.25), Some(10.0)),
            "from {beacon}"
        );
    }
}

/// Design change 0112's Verification, as 0143 restates it ("0112's example now reads 'a new Drone starts
/// production'"; the production side is slice M3-P's): at 10/10 a Drone deploys (9/10), a new Drone comes (spawned
/// here: 10/10), then the deploy is cancelled → 11/10. "Cancelling takes the 1 back, even if that puts the player over
/// capacity": the cancel is never refused, and nothing is removed for being over.
#[test]
fn a_cancel_takes_the_drones_capacity_back_even_over_the_cap() {
    let mut t = at_capacity().start();
    let start = deployed(&mut t);
    assert_eq!(capacity(&t, P1), (9, 10));
    t.spawn_unit_at(start + 1, Drone, P1, at(14, 16).center()).step();
    assert_eq!(capacity(&t, P1), (10, 10), "a new Drone");
    let objects = t.sim().objects().count();
    let cmd = PlayerCommand {
        player: P1,
        command: cancel(&[2]),
    };
    assert_eq!(t.sim().validate(&cmd), Ok(()), "never refused for the capacity");
    next(&mut t, cmd.command);
    assert_eq!(object(&t, 2).kind, Drone);
    assert_eq!(capacity(&t, P1), (11, 10), "over capacity");
    t.run(20);
    assert_eq!(t.sim().objects().count(), objects, "nothing removed");
    assert_eq!(capacity(&t, P1), (11, 10));
}

/// [`base`] with a Space Crystals patch 4 at (6, 13), which Drone 2 is sent to gather in tick 1, run until it holds
/// its first load, 8 (`factions/colonists/units.md > Drone > Gathering`: CarryCapacity 8), on its way to the Egress
/// Node. Then it is ordered in the next tick to deploy at [`SITE`], replacing its Gather: "A Drone carrying a load
/// deploys without delivering it first" (`> Deploying with a load`). Run until it began; returns that tick.
fn deployed_with_a_load(t: &mut TestSim) -> Tick {
    t.order_at(1, P1, &[id(2)], OrderKind::Gather(id(4)), false);
    while object(t, 2).worker.is_some_and(|worker| worker.load == 0) {
        assert!(t.sim().tick() < 400, "it mines");
        t.step();
    }
    assert_eq!(object(t, 2).worker.unwrap().load, 8);
    next(t, order(&[2], deploy_at(SITE), false));
    walk(t, 2);
    t.sim().tick()
}

/// Design change 0114's Verification ("Deploy a carrying Drone: the bank doesn't change, the load is gone at
/// completion"): the load stays with the deploying object (`Transform::load`) and is lost when the deployment
/// completes; nothing was delivered, and nothing is.
#[test]
fn a_load_deployed_with_is_lost_at_completion() {
    let mut t = base().spawn(CrystalPatch, None, tile(6, 13)).start();
    let start = deployed_with_a_load(&mut t);
    let crystals = bank(&t, P1);
    assert_eq!(
        object(&t, 2).transform.as_ref().unwrap().load,
        8,
        "kept while it deploys"
    );
    t.run_until(start + 47);
    assert_eq!(completions(&t), [(start + 47, 2, Beacon)]);
    assert!(
        object(&t, 2).transform.is_none() && object(&t, 2).worker.is_none(),
        "the load is gone"
    );
    t.run(100);
    assert_eq!(bank(&t, P1), crystals, "never delivered");
    let delivered = ticks_of(&t, |e| matches!(e, SimEvent::ResourcesDelivered { .. }));
    assert_eq!(delivered, [] as [Tick; 0]);
}

/// Design change 0114 ("on cancel the Drone returns still carrying it"): the Drone cancelled back carries its 8, and
/// delivers them when sent to Egress Node 1 (a Drop Off order): the bank rises by 8.
#[test]
fn a_load_comes_back_with_the_drone_and_can_be_delivered() {
    let mut t = base().spawn(CrystalPatch, None, tile(6, 13)).start();
    let start = deployed_with_a_load(&mut t);
    let crystals = bank(&t, P1);
    t.run_until(start + 10);
    next(&mut t, cancel(&[2]));
    let o = object(&t, 2);
    assert_eq!((o.kind, o.worker.map(|worker| worker.load)), (Drone, Some(8)));
    next(&mut t, order(&[2], OrderKind::DropOff(id(1)), false));
    while bank(&t, P1) == crystals {
        assert!(t.sim().tick() < start + 400, "it delivers");
        t.step();
    }
    assert_eq!(bank(&t, P1), crystals + 8);
    assert!(t.events().iter().any(|(_, e)| *e
        == SimEvent::ResourcesDelivered {
            worker: id(2),
            drop_off: id(1),
            amount: 8
        }));
    assert_eq!(object(&t, 2).worker.unwrap().load, 0);
}

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

/// The plan's acceptance, "Shift" (`systems/controls/commands.md > Unit Command`: "the new command is appended to the
/// end of the queue"): a Deploy queued behind a Move runs after it. Drone 2 is sent to (6, 15), and the Deploy is
/// queued in the same tick: it is pending from then on (design change 0125), the Drone stands on (6, 15) before it
/// sets off for its site, and it deploys there.
#[test]
fn a_deploy_queued_behind_a_move_runs_after_it() {
    let mut t = base().start();
    let first = at(6, 15).center();
    t.order_at(1, P1, &[id(2)], OrderKind::Move(Target::Point(first)), false)
        .order_at(1, P1, &[id(2)], deploy_at(SITE), true)
        .step();
    assert_eq!(t.sim().pending_deploys(P1), [(id(2), deploy::site(SITE))]);
    let current = |t: &TestSim| {
        object(t, 2)
            .orders
            .as_ref()
            .and_then(|orders| orders.current.as_ref())
            .map(|active| active.order.kind)
    };
    let mut stood_there = false;
    while current(&t) != Some(deploy_at(SITE)) {
        assert!(t.sim().tick() < 300, "the Move ends");
        stood_there |= t.pos(2) == first;
        t.step();
    }
    assert!(stood_there, "the Move first");
    walk(&mut t, 2);
    assert_eq!(object(&t, 2).placement, Some(deploy::site(SITE)));
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

/// `sim` saved, with object `n`'s `field` set to `value`, and loaded: a state no command makes on demand.
fn edited(sim: &Sim, n: u32, field: &str, value: serde_json::Value) -> Sim {
    let mut state = serde_json::to_value(sim).unwrap();
    let objects = state["objects"].as_array_mut().unwrap();
    let o = objects.iter_mut().find(|o| o["id"] == n).expect("the object is saved");
    o[field] = value;
    serde_json::from_value(state).unwrap()
}

/// Saves and replays (`ARCHITECTURE.md > Replays and golden hashes`): a game where Drone 2 deploys with a load, Drone
/// 6 fails on arrival (its site, on player 2's Egress Node 3, never seen), and Drone 5, sent once Drone 2 deploys,
/// deploys and is cancelled goes on alike after a save in the middle of Drone 2's deployment, tick by tick, and its
/// replay file plays back to the same state.
#[test]
fn deployments_go_on_alike_after_a_save_and_in_a_replay() {
    let mut t = at_capacity().spawn(CrystalPatch, None, tile(6, 13)).start();
    // The patch is object 14.
    t.order_at(1, P1, &[id(2)], OrderKind::Gather(id(14)), false).order_at(
        1,
        P1,
        &[id(6)],
        deploy_at(at(27, 15)),
        false,
    );
    while object(&t, 2).worker.is_some_and(|worker| worker.load == 0) {
        assert!(t.sim().tick() < 400, "it mines");
        t.step();
    }
    next(&mut t, order(&[2], deploy_at(SITE), false));
    walk(&mut t, 2);
    let tick = t.sim().tick();
    t.order_at(tick + 1, P1, &[id(5)], deploy_at(at(14, 14)), false);
    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    assert_eq!(object(&t, 2).transform.as_ref().unwrap().load, 8);
    let mut cancelled = false;
    while t.sim().tick() < tick + 500 {
        if !cancelled && object(&t, 5).is_deploying() {
            t.at(t.sim().tick() + 1, P1, cancel(&[5]));
            cancelled = true;
        }
        t.step();
        loaded.step(t.replay().inputs.last().unwrap());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert!(cancelled);
    assert_eq!(object(&t, 2).kind, Beacon);
    assert_eq!(object(&t, 5).kind, Drone, "cancelled");
    let failed: Vec<ObjectId> = t
        .events()
        .iter()
        .filter_map(|(_, e)| match e {
            SimEvent::DeployFailed { drone, .. } => Some(*drone),
            _ => None,
        })
        .collect();
    assert_eq!(failed, [id(6)]);
    let replay: Replay = serde_json::from_str(&serde_json::to_string(t.replay()).unwrap()).unwrap();
    assert_eq!(replay.run().state_hash(), t.sim().state_hash());
}
