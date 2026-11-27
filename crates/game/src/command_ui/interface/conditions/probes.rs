//! What the gate says of each command the ActiveGroup's panel may send (`super::Conditions::of`; plan decision T10:
//! "button availability uses the same `sc_sim` predicates as the gate"): the command each button would send, to the
//! objects it would go to ([`refusal`]), and the Build order that stands for a placement not yet made
//! ([`build_probe`]).

use sc_sim::{
    BuildingOrientation, Command, ConstructionCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    Reject, Sim, TilePos,
};

use super::super::{PanelCommand, recipients};
use crate::selection::Selection;

/// Why the gate would refuse `player`'s `command` to its recipients in `selection` now ([`recipients`]), if it would:
/// `Sim::validate`'s answer to the `Command::Production`, `Command::Construction`, `Command::Tunnel`,
/// `Command::Holding`, `Command::Order` or `Command::Ability` it would send (module docs of `super`). An ExpandMenu
/// entry asks `Sim::can_expand`, the gate's Expand short of its placement; a Warp In menu entry asks for its Warp In
/// unturned, and an Agent's or a Recruits' Build for its Build order ([`build_probe`]), each leaving out the
/// placement's refusal (`Reject::Unplaceable`, the gate's last check): the ghost judges each spot as it is placed.
/// (M5-N5) A Shrine's cast asks `Sim::can_cast`, the gate's checks of a cast but its point's: where it is cast is the
/// click's to judge (`crate::command_ui::resolution::target_resolution`). (M5-N4b) A Disciple's Shroud, (M6-GD) a
/// Drone Operator's Launch Drone, (M6-GH1) an aircraft's Return to Base and (M6-GA) a vehicle's Deploy or Undeploy ask
/// for their order, unqueued: R is greyed while every one of its units deploys already (`Reject::Deploying`). `None`
/// for a command the panel sends no sim command for here (a menu, a target command).
pub(super) fn refusal(sim: &Sim, player: PlayerId, selection: &Selection, command: PanelCommand) -> Option<Reject> {
    let objects = recipients(selection, command);
    if let PanelCommand::Expand(kind) = command {
        return sim.can_expand(player, *objects.first()?, kind).err();
    }
    if let PanelCommand::Cast(ability) = command {
        return sim.can_cast(player, &objects, ability).err();
    }
    let sent = if let Some(production) = command.production() {
        Command::Production {
            objects,
            command: production,
        }
    } else if let Some(construction) = command.construction() {
        Command::Construction {
            objects,
            command: construction,
        }
    } else if let PanelCommand::WarpIn(kind) = command {
        let orientation = BuildingOrientation::default();
        Command::Construction {
            objects,
            command: ConstructionCommand::WarpIn { kind, orientation },
        }
    } else if let Some(tunnel) = command.tunnel() {
        Command::Tunnel {
            tunnel: *objects.first()?,
            command: tunnel,
        }
    } else if let Some(holding) = command.holding() {
        Command::Holding {
            objects,
            command: holding,
        }
    } else if let PanelCommand::Build(kind) = command {
        build_probe(objects, kind)
    } else if matches!(
        command,
        PanelCommand::Shroud | PanelCommand::Launch | PanelCommand::ReturnToBase | PanelCommand::VehicleDeploy(_)
    ) {
        Command::Order {
            units: objects,
            kind: command.immediate_order()?,
            queue: false,
        }
    } else {
        Command::Ability {
            units: objects,
            ability: command.ability()?,
        }
    };
    let sent = PlayerCommand { player, command: sent };
    let reject = sim.validate(&sent).err()?;
    let placed =
        matches!(command, PanelCommand::WarpIn(_) | PanelCommand::Build(_)) && matches!(reject, Reject::Unplaceable(_));
    (!placed).then_some(reject)
}

/// The Build order an Agent's W or a Recruits' Construct menu entry would send (`PanelCommand::Build`, to the command's
/// recipients), to ask the gate whether the button is available: unqueued, unturned, at the map's north-west corner.
/// Where it stands matters only to the placement's check, the gate's last (`Reject::Unplaceable`), which the panel
/// leaves out: the ghost judges each spot as it is placed.
fn build_probe(units: Vec<ObjectId>, kind: ObjectKind) -> Command {
    let kind = OrderKind::Build {
        kind,
        origin: TilePos::new(0, 0),
        orientation: BuildingOrientation::default(),
    };
    Command::Order {
        units,
        kind,
        queue: false,
    }
}
