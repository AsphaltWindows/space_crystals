use sc_sim::buffs::BuffKind;
use sc_sim::{BuildingOrientation, DebugCommand, ProductionCommand, Target, TilePos, TunnelCommand};

use super::*;

/// Player 1's Tunnel, set to Tier 3, whose Transit lists air units, on tiles (4, 8) to (7, 11): Side A faces north,
/// its entry path's spot at (354, 512).
const T1: ObjectId = ObjectId(1);

/// [`field`] with player 1's Tunnel ([`T1`]) and the objects `more` spawns, started with auto-targeting off, the Tunnel
/// set to Tier 3 and 10 000 Space Crystals and 100 Supplies in player 1's bank, in tick 1, which has run.
fn at_tier_3(more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let mut t = more(field().spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(4, 8)))).start();
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
    t.run_until(1);
    t
}

fn enter(units: &[ObjectId]) -> Command {
    Command::Order {
        units: units.to_vec(),
        kind: OrderKind::Enter(T1),
        queue: false,
    }
}

fn spawn(kind: ObjectKind, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(P1),
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation: BuildingOrientation::default(),
    }
}

/// `factions/syndicate/units/prism.md > One way only`: "the only unit that can't get back into the Tunnel Network",
/// even by a Tier 3 Tunnel, which lets air units through. Ordered in alone, it is refused (`Reject::OneWay`), and its
/// right-click on the Tunnel is a Move to it, as for a unit the tier doesn't let through, where a Guard's is an Enter.
/// Ordered in with the Guard, the Guard goes in and the Prism ignores the order: the Move it was on goes on.
#[test]
fn a_prism_is_not_let_back_in() {
    let mut t = at_tier_3(|scenario| {
        scenario
            .spawn(PRISM, Some(P1), point(354.0, 400.0)) // 2
            .spawn(ObjectKind::Guard, Some(P1), point(300.0, 420.0)) // 3
    });
    let (prism, guard) = (id(2), id(3));
    assert_eq!(gate(&t, enter(&[prism])), Err(Reject::OneWay(T1)));
    let right_click = |unit| t.sim().default_order(unit, Target::Object(T1));
    assert_eq!(right_click(prism), Some(OrderKind::Move(Target::Object(T1))));
    assert_eq!(right_click(guard), Some(OrderKind::Enter(T1)));
    let away = OrderKind::Move(Target::Point(Vec2::new(2000.0, 400.0)));
    let order = Command::Order {
        units: vec![prism],
        kind: away,
        queue: false,
    };
    t.at(2, P1, order);
    assert_eq!(gate(&t, enter(&[prism, guard])), Ok(()));
    t.at(3, P1, enter(&[prism, guard]));
    step_until(&mut t, 200, "the Guard is in", |t| t.sim().held_unit(guard).is_some());
    let orders = object(&t, prism).orders.as_ref().expect("a unit");
    assert_eq!(orders.current.as_ref().map(|active| active.order.kind), Some(away));
}

/// `factions/syndicate/units/prism.md > Manned - no (no Null Chamber buff)`: trained into the network of a player
/// who owns a Null Chamber, the Prism takes no turn there, where a Guard that walked in took one, and it leaves the
/// network unbuffed.
#[test]
fn a_prism_trained_with_a_null_chamber_stays_unmanned() {
    let mut t = at_tier_3(|scenario| {
        scenario
            .spawn(ObjectKind::NullChamber, Some(P1), SpawnAt::Tile(TilePos::new(8, 12))) // 2
            .spawn(ObjectKind::Guard, Some(P1), point(300.0, 420.0)) // 3
    });
    let (guard, assembly) = (id(3), id(4));
    t.debug_at(2, spawn(ObjectKind::AerospaceAssembly, 0, 6)); // 4
    t.debug_at(2, spawn(ObjectKind::OrbitalCommand, 6, 4)); // 5
    t.at(2, P1, enter(&[guard]));
    let train = Command::Production {
        objects: vec![assembly],
        command: ProductionCommand::Train(PRISM),
    };
    t.at(3, P1, train);
    step_until(&mut t, 600, "the Prism is trained", |t| {
        t.sim().network(P1).any(|held| held.object.kind == PRISM)
    });
    let prism = t
        .sim()
        .network(P1)
        .find(|held| held.object.kind == PRISM)
        .unwrap()
        .object
        .id;
    // Longer than a turn, 2 s (`factions/syndicate/structures/null_chamber.md > Placeholder stats`).
    for _ in 0..40 {
        t.step();
        assert!(t.sim().held_unit(prism).expect("inside").service.is_none(), "no turn");
    }
    assert!(
        t.sim().held_unit(guard).expect("inside").service.chamber.is_some(),
        "the Guard's turn"
    );
    let eject = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Eject { kind: PRISM },
    };
    let next = t.sim().tick() + 1;
    t.at(next, P1, eject);
    step_until(&mut t, 60, "the Prism is out", |t| t.sim().object(prism).is_some());
    let buffs = &object(&t, prism).buffs;
    assert!(
        buffs.iter().all(|buff| buff.ability != BuffKind::NullChamber),
        "unbuffed"
    );
}
