//! (M6-GA) The Artillery Rig's two panels as plain data, read from a game (`factions/gdo/units/artillery_rig.md >
//! Placeholder stats`: "Panel: BasicCombatUnitInterfaceState plus **R: Deploy** (mobile) / **R: Undeploy** (deployed);
//! the mobile Rig has no attack commands"; `> Modes`): what a deploy under way and the deployed Rig's standstill grey,
//! R's bar, and the type a deploying Rig is judged as. In a game: `crates/game/tests/commands/artillery_rig_ui.rs`.

use bevy::math::Vec2;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Command, Faction, PlayerId, SpawnAt, vehicle_deploy};

use super::*;
use crate::command_ui::feedback::{CANNOT_MOVE_TEXT, DEPLOYING_TEXT, unavailable_text};
use crate::command_ui::resolution::UnitCapabilities;

const RIG: ObjectKind = ObjectKind::ArtilleryRig;
const DEPLOYED: ObjectKind = ObjectKind::ArtilleryRigDeployed;
const P1: PlayerId = PlayerId(1);
const DEPLOY: PanelCommand = PanelCommand::VehicleDeploy(DEPLOYED);
const UNDEPLOY: PanelCommand = PanelCommand::VehicleDeploy(RIG);

/// Player 1's Artillery Rig (1), Artillery Rig (Deployed) (2) and Peacekeeper (3) on a flat map; tick 1 has run.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(24); 24].join("\n"))
        .faction(P1, Faction::Gdo)
        .spawn(RIG, Some(P1), SpawnAt::Point(Vec2::new(400.0, 600.0)))
        .spawn(DEPLOYED, Some(P1), SpawnAt::Point(Vec2::new(900.0, 600.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(600.0, 1000.0)))
        .start();
    t.run_until(1);
    t
}

/// Unit `n`'s deploy into `into`, unqueued, given in the next tick, which runs.
fn deploy(t: &mut TestSim, n: u32, into: ObjectKind) {
    let tick = t.sim().tick() + 1;
    let command = Command::Order {
        units: vec![id(n)],
        kind: OrderKind::VehicleDeploy { into },
        queue: false,
    };
    t.at(tick, P1, command);
    t.step();
}

/// `selection`'s DefaultState panel in `t`.
fn panel_of(t: &TestSim, selection: &Selection) -> Panel {
    let conditions = Conditions::of(t.sim(), P1, selection);
    panel(selection, InterfaceState::DefaultState, &conditions)
}

/// What the greyed button of `slot` says.
fn why(panel: &Panel, slot: CommandSlot) -> Option<&'static str> {
    panel.get(slot).and_then(|shown| shown.why).map(unavailable_text)
}

/// The mobile Rig's DefaultState is the BasicCombatUnit's without its attack commands (no A while no selected unit can
/// attack, no D), its W Reverse (a TrackedVehicle), with R Deploy, "Deploy": a vehicle deploy into the deployed type,
/// at once, an order that Shift queues, to every Rig of the ActiveGroup, the Rig being Groupable. The deployed Rig's is
/// the BasicCombatUnit's with its A and its D, Attack Ground, and R Undeploy, "Undep.", into the mobile type; its Move,
/// Reverse and Patrol are greyed, "Cannot move while deployed", as the gate would refuse them, its Hold Position and
/// Stop not. Beside a Peacekeeper, Move goes on: the Peacekeeper takes it.
#[test]
fn the_mobile_rigs_r_deploys_it_and_the_deployed_ones_undeploys_it() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    let t = field();
    let c = PanelButton::Command;
    let (q, w, r, s, d) = (slot(0, 0), slot(0, 1), slot(0, 3), slot(1, 1), slot(1, 2));
    let mobile = Selection::new([(id(1), RIG)]);
    let start = panel_of(&t, &mobile);
    assert_eq!(
        shown(&start),
        [
            (q, c(Move), Common),
            (w, c(Reverse), Common),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(DEPLOY), Common),
            (s, c(Patrol), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    assert_eq!(start.get(r).map(|shown| shown.label()), Some("Deploy"));
    assert_eq!(start.name(r), Some("Deploy".into()));
    let order = OrderKind::VehicleDeploy { into: DEPLOYED };
    assert_eq!(press(&start, r), SlotPress::Issue(DEPLOY, order));
    assert!(DEPLOY.queues() && !DEPLOY.needs_target() && !DEPLOY.instance_only());
    let two = Selection::new([(id(1), RIG), (id(4), RIG)]);
    assert_eq!(two.groups().len(), 1, "Groupable");
    assert_eq!(recipients(&two, DEPLOY), [id(1), id(4)]);

    let deployed = Selection::new([(id(2), DEPLOYED)]);
    let standing = panel_of(&t, &deployed);
    assert_eq!(
        shown(&standing),
        [
            (q, c(Move), Disabled),
            (w, c(Reverse), Disabled),
            (slot(0, 2), c(HoldPosition), Common),
            (r, c(UNDEPLOY), Common),
            (slot(1, 0), c(Attack), Common),
            (s, c(Patrol), Disabled),
            (d, c(AttackGround), Common),
            (slot(2, 1), c(Stop), Common),
        ]
    );
    for at in [q, w, s] {
        assert_eq!(why(&standing, at), Some(CANNOT_MOVE_TEXT), "{at:?}");
    }
    assert_eq!(press(&standing, q), SlotPress::Nothing, "Move greyed");
    assert_eq!(standing.get(r).map(|shown| shown.label()), Some("Undep."));
    assert_eq!(standing.name(r), Some("Undeploy".into()));
    let order = OrderKind::VehicleDeploy { into: RIG };
    assert_eq!(press(&standing, r), SlotPress::Issue(UNDEPLOY, order));
    let with_pk = Selection::new([(id(2), DEPLOYED), (id(3), PK)]);
    let look = |panel: &Panel, at| panel.get(at).map(|shown| shown.look);
    assert_eq!(look(&panel_of(&t, &with_pk), q), Some(Common), "the Peacekeeper moves");
}

/// While the mobile Rig deploys, its R is greyed ("**Undeploy is only available once deployment is complete**"), "Still
/// switching modes", with the deploy's progress on its bar, the sim's; its Move stays (the order is held), and it is
/// judged as the type it becomes: its panel shows A, Attack, and it may attack. Once deployed, its type's panel is the
/// deployed one's, R Undeploy lit, no bar. Undeploying, the deployed Rig's R is greyed in turn, its Move lit (held for
/// the mobile type), and no A shows: the mobile Rig has no attack.
#[test]
fn while_it_deploys_its_r_is_greyed_with_the_progress_on_its_bar_and_it_is_judged_as_its_new_type() {
    use ButtonLook::{Common, Disabled};
    use PanelCommand::Attack;
    let mut t = field();
    let (q, r, a) = (slot(0, 0), slot(0, 3), slot(1, 0));
    let look = |panel: &Panel, at| panel.get(at).map(|shown| shown.look);
    deploy(&mut t, 1, DEPLOYED);
    t.run(20);
    let progress = vehicle_deploy::under_way(t.sim().object(id(1)).unwrap()).map(|(_, progress)| progress);
    assert_eq!(progress.map(|p| p.ticks_done()), Some(20), "20 of its 48 frames");
    let mobile = Selection::new([(id(1), RIG)]);
    let deploying = panel_of(&t, &mobile);
    assert_eq!(look(&deploying, r), Some(Disabled));
    assert_eq!(why(&deploying, r), Some(DEPLOYING_TEXT));
    assert_eq!(deploying.get(r).and_then(|shown| shown.progress), progress, "the sim's");
    assert_eq!(look(&deploying, q), Some(Common), "held");
    assert_eq!(
        deploying.get(a).map(|shown| (shown.button, shown.look)),
        Some((PanelButton::Command(Attack), Common))
    );
    let conditions = Conditions::of(t.sim(), P1, &mobile);
    assert!(UnitCapabilities::of(id(1), RIG, &conditions.modes).may_attack());

    t.run(28);
    assert_eq!(t.sim().object(id(1)).map(|o| o.kind), Some(DEPLOYED), "deployed");
    let deployed = Selection::new([(id(1), DEPLOYED)]);
    let done = panel_of(&t, &deployed);
    assert_eq!(
        done.get(r).map(|shown| (shown.button, shown.look, shown.progress)),
        Some((PanelButton::Command(UNDEPLOY), Common, None))
    );

    deploy(&mut t, 1, RIG);
    let undeploying = panel_of(&t, &deployed);
    assert_eq!(look(&undeploying, r), Some(Disabled));
    assert_eq!(why(&undeploying, r), Some(DEPLOYING_TEXT));
    assert_eq!(look(&undeploying, q), Some(Common), "held for the mobile type");
    assert_eq!(undeploying.get(a), None, "the mobile Rig has no attack");
    let conditions = Conditions::of(t.sim(), P1, &deployed);
    assert!(!UnitCapabilities::of(id(1), DEPLOYED, &conditions.modes).may_attack());
}
