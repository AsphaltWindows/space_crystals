//! A Tunnel's expansions where a test needs a type no game has (`defs::test_defs`): an expansion of another cost, and
//! one of another Tier Requirement. `crates/sim/tests/expansions/` plays the Hideout's through the gate.

use crate::command::{Command, DebugCommand, PlayerCommand, TunnelCommand};
use crate::construction::ConstructionCommand;
use crate::construction::site::{Builder, expansion_def};
use crate::defs::test_defs::TestDef;
use crate::defs::{ExpansionDef, ObjectKind};
use crate::events::SimEvent;
use crate::ids::{ObjectId, PlayerId};
use crate::player::Faction;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::sim::Reject;
use crate::testing::{ScenarioBuilder, TestSim};
use crate::tunnel::Operation;
use crate::units::TilePos;

const P1: PlayerId = PlayerId(1);
const TUNNEL: ObjectId = ObjectId(1);
const HIDEOUT: ObjectKind = ObjectKind::Hideout;

/// The Hideout's expansion def as the design has it, with `change` applied, for as long as the guard lives.
fn hideout_with(change: impl FnOnce(&mut ExpansionDef)) -> TestDef {
    let mut def = *expansion_def(HIDEOUT).expect("a Tunnel builds the Hideout");
    change(&mut def);
    TestDef::structure(HIDEOUT, |structure| structure.expansion = Some(def))
}

/// Player 1's Tunnel (1) on tiles (12, 10) to (15, 13) of a 30 × 22 map, its Tier 1 area (10, 8) to (17, 15), and
/// 1000 Space Crystals in its bank from tick 1.
fn base() -> TestSim {
    let mut t = ScenarioBuilder::new(&"..............................\n".repeat(22))
        .faction(P1, Faction::Syndicate)
        .faction(PlayerId(2), Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .start();
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    t.debug_at(1, crystals);
    t.run_until(1);
    t
}

/// An Expand of a Hideout at `(x, y)`, unturned.
fn expand(x: i32, y: i32) -> Command {
    let command = TunnelCommand::Expand {
        kind: HIDEOUT,
        origin: TilePos::new(x, y),
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel {
        tunnel: TUNNEL,
        command,
    }
}

/// Player 1's `command` in the next tick, which runs.
fn next(t: &mut TestSim, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, P1, command);
    t.step();
}

fn crystals(t: &TestSim) -> u32 {
    t.sim().player(P1).expect("in the game").crystals
}

fn operation(t: &TestSim) -> Option<Operation> {
    t.sim().object(TUNNEL)?.tunnel.as_ref()?.operation
}

/// The refunds of the expansions cancelled so far.
fn refunds(t: &TestSim) -> Vec<u32> {
    (t.events().iter())
        .filter_map(|(_, e)| match *e {
            SimEvent::ExpansionCancelled { refund, .. } => Some(refund),
            _ => None,
        })
        .collect()
}

/// Design change 0254's rule for any cost, "refunding **75%** of its cost (rounded down)", and the M4 plan's
/// acceptance: a test kind costing 150 gives back 112, by the Tunnel's X as by the site's own Cancel Construction, and
/// the operation is freed each time.
#[test]
fn a_kind_costing_150_refunds_112() {
    let _cheap = hideout_with(|def| def.cost = 150);
    let mut t = base();
    next(&mut t, expand(16, 10));
    assert_eq!(crystals(&t), 850, "paid at the click");
    let Some(Operation::Building(site)) = operation(&t) else {
        panic!("the Tunnel builds: {:?}", operation(&t));
    };
    let built = t.sim().object(site).and_then(|o| o.construction).map(|c| c.builder);
    assert_eq!(built, Some(Builder::Tunnel(TUNNEL)));
    let cancel = Command::Tunnel {
        tunnel: TUNNEL,
        command: TunnelCommand::Cancel,
    };
    next(&mut t, cancel.clone());
    assert_eq!((crystals(&t), operation(&t), refunds(&t)), (962, None, vec![112]));
    assert!(t.sim().object(site).is_none(), "cancelled");
    let refused = t.sim().validate(&PlayerCommand {
        player: P1,
        command: cancel,
    });
    assert_eq!(refused, Err(Reject::NoOperation(TUNNEL)));
    // Its own Cancel Construction, the site's only command (design change 0254).
    next(&mut t, expand(16, 12));
    let Some(Operation::Building(site)) = operation(&t) else {
        panic!("the Tunnel builds again: {:?}", operation(&t));
    };
    let own = Command::Construction {
        objects: vec![site],
        command: ConstructionCommand::CancelConstruction,
    };
    next(&mut t, own);
    assert_eq!((crystals(&t), operation(&t), refunds(&t)), (924, None, vec![112, 112]));
}

/// `factions/syndicate/tunnels.md > ExpandMenu`: "An expansion is available only if (1) its Tier Requirement is at or
/// below **this** Tunnel's tier": a test kind with Tier Requirement 2 is refused at a Tier 1 Tunnel, by the gate and by
/// the menu's check ([`crate::sim::Sim::can_expand`], which greys its entry), and taken once the Tunnel is at Tier 2.
#[test]
fn a_tier_2_kind_is_refused_at_a_tier_1_tunnel() {
    let _tier_2 = hideout_with(|def| def.tier = 2);
    let mut t = base();
    let refused = Err(Reject::TierRequirement { kind: HIDEOUT, tier: 2 });
    assert_eq!(t.sim().can_expand(P1, TUNNEL, HIDEOUT), refused);
    let command = |t: &TestSim| {
        t.sim().validate(&PlayerCommand {
            player: P1,
            command: expand(16, 10),
        })
    };
    assert_eq!(command(&t), refused);
    next(&mut t, expand(16, 10));
    assert_eq!((operation(&t), crystals(&t)), (None, 1000));
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::SetTier { id: TUNNEL, tier: 2 });
    t.step();
    assert_eq!(t.sim().can_expand(P1, TUNNEL, HIDEOUT), Ok(()));
    assert_eq!(command(&t), Ok(()));
}

/// The gate's other reasons, as the ExpandMenu shows them: a kind no Tunnel builds, a bank too short of its cost, and
/// another player's Tunnel.
#[test]
fn the_menus_reasons_are_the_gates() {
    let t = base();
    let sim = t.sim();
    assert_eq!(
        sim.can_expand(P1, TUNNEL, ObjectKind::Tunnel),
        Err(Reject::CannotConstruct(ObjectKind::Tunnel))
    );
    assert_eq!(
        sim.can_expand(PlayerId(2), TUNNEL, HIDEOUT),
        Err(Reject::NotOwned(TUNNEL))
    );
    assert_eq!(
        sim.can_expand(P1, ObjectId(99), HIDEOUT),
        Err(Reject::UnknownObject(ObjectId(99)))
    );
    let _dear = hideout_with(|def| def.cost = 1001);
    assert_eq!(
        t.sim().can_expand(P1, TUNNEL, HIDEOUT),
        Err(Reject::InsufficientCrystals)
    );
}
