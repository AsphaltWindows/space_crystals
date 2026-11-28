use sc_sim::{BuildingOrientation, DebugCommand, ProductionCommand, TilePos, TunnelCommand};

use super::*;

/// Player 1's Tunnel, at Tier 3, on tiles (4, 8) to (7, 11).
const T1: ObjectId = ObjectId(1);
/// Player 1's Aerospace Assembly, a debug spawn in the Tunnel's Tier 3 area, which trains the Prism.
const AA: ObjectId = ObjectId(2);

fn spawn(kind: ObjectKind, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: BuildingOrientation::default(),
    }
}

/// A Prism trained at the Aerospace Assembly, behind an Orbital Command (`factions/syndicate/units/prism.md >
/// TechPrerequisite`), ejected through the Tier 3 Tunnel's air exit door: its id, in the first tick it is on the map,
/// launching (`Mover::launching`).
fn launched() -> (TestSim, ObjectId) {
    let mut t = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(4, 8)))
        .start();
    t.without_auto_targeting();
    t.debug_at(1, DebugCommand::SetTier { id: T1, tier: 3 });
    let bank = DebugCommand::SetResources {
        player: P1,
        crystals: 10_000,
        supplies: 100,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, bank);
    t.debug_at(1, spawn(ObjectKind::AerospaceAssembly, 0, 6));
    t.debug_at(1, spawn(ObjectKind::OrbitalCommand, 6, 4));
    t.run_until(1);
    let train = Command::Production {
        objects: vec![AA],
        command: ProductionCommand::Train(PRISM),
    };
    assert_eq!(gate(&t, train.clone()), Ok(()));
    t.at(2, P1, train);
    step_until(&mut t, 600, "the Prism is trained", |t| {
        t.sim().network(P1).any(|held| held.object.kind == PRISM)
    });
    let eject = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Eject { kind: PRISM },
    };
    let next = t.sim().tick() + 1;
    t.at(next, P1, eject);
    step_until(&mut t, 60, "the Prism is out", |t| {
        t.sim().objects().any(|o| o.kind == PRISM)
    });
    let prism = t.sim().objects().find(|o| o.kind == PRISM).unwrap().id;
    assert!(t.mover(prism.0).launching());
    (t, prism)
}

/// "Orders given during the 4 s launch/unfold are **held** (last one wins)" (design change 0434 point 3): a thrust
/// given while the Prism rises and unfolds waits in its queue, the Prism holding still and its charges unspent, and a
/// second one without Shift replaces it; it starts in the tick after the unfold's last, and the burst moves the Prism
/// its 3 grid units toward the point.
#[test]
fn a_thrust_given_while_it_launches_is_held_and_runs_once_it_has_unfolded() {
    let (mut t, prism) = launched();
    let from = t.pos(prism.0);
    let tick = t.sim().tick();
    t.at(tick + 1, P1, thrust(&[prism], Vec2::new(from.x, from.y - 320.0), false));
    t.at(tick + 2, P1, thrust(&[prism], Vec2::new(from.x + 640.0, from.y), false));
    t.run_until(tick + 2);
    let orders = object(&t, prism).orders.as_ref().unwrap();
    assert!(orders.current.is_none(), "held");
    let held: Vec<OrderKind> = orders.queue.iter().map(|order| order.kind).collect();
    assert_eq!(
        held,
        [OrderKind::Thrust(Vec2::new(from.x + 640.0, from.y))],
        "the last one wins"
    );
    step_until(&mut t, 80, "the Prism has unfolded", |t| !t.mover(prism.0).launching());
    assert_eq!(
        (t.pos(prism.0), charges(&t, prism)),
        (from, CHARGES),
        "still, its charges unspent"
    );
    let unfolded = t.sim().tick();
    t.run_until(unfolded + 1);
    let current = object(&t, prism)
        .orders
        .as_ref()
        .unwrap()
        .current
        .as_ref()
        .map(|active| active.order.kind);
    assert_eq!(current, Some(OrderKind::Thrust(Vec2::new(from.x + 640.0, from.y))));
    assert_eq!(charges(&t, prism), CHARGES - 1);
    t.run_until(unfolded + THRUST_TICKS);
    assert_eq!(t.pos(prism.0), from + Vec2::new(THRUST_SU, 0.0));
}
