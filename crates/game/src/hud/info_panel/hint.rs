//! The full name of the command button under the pointer, at the foot of the InfoPanel (slices M3-S and M3-K2a): a
//! button's label is short enough to keep its size on its button at 720p, and shortens its command's name ("Raise"
//! for Raise Shield, "PK" for Train Peacekeeper), which this keeps in sight with its hotkey, "R: Raise Shield", for
//! every button of every panel, Back and the greyed-out ones too (`crate::command_ui::Panel::name`: named after the
//! ActiveGroup's type where the design does so, "Q: Warp In (Basic)" on a Beacon's). The client has no tooltips; the
//! design names no place for a command's name (placeholder until the HUD pass).

use bevy::prelude::*;

use super::super::command_panel::CommandPanelView;
use super::super::layout::CommandGrid;
use super::super::{LABEL_VH, set_text, show_node, text};
use crate::ClientMode;
use crate::command_ui::Panel;
use crate::command_ui::interface::slot_index;
use crate::input::{ActionState, CommandSlot};
use crate::platform::SurfaceSize;

/// The colour of the hint, set apart from the InfoPanel's lines (placeholder).
const HINT_COLOR: Color = Color::srgb(0.98, 0.90, 0.55);

/// The InfoPanel's line naming the command button under the pointer; hidden while there is none.
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommandHint;

/// The hint's text node, hidden, for the InfoPanel's content (`super::spawn_content`).
pub(in crate::hud) fn spawn(panel: &mut ChildSpawnerCommands) {
    let hidden = Node {
        display: Display::None,
        ..default()
    };
    panel.spawn((CommandHint, text("", LABEL_VH, HINT_COLOR), hidden));
}

/// What the hint says of `slot` of `panel`, whose hotkeys are `hotkeys` (row by row from the top left): its hotkey
/// and its full name (`Panel::name`), or the name alone for an unbound slot; `None` for an empty slot.
pub fn hint(panel: &Panel, hotkeys: &[String], slot: CommandSlot) -> Option<String> {
    let name = panel.name(slot)?;
    Some(match hotkeys.get(slot_index(slot)).filter(|key| !key.is_empty()) {
        Some(key) => format!("{key}: {name}"),
        None => name,
    })
}

/// Names the button under the pointer, in a game mode: the slot the cursor is in (the geometry the clicks use,
/// [`CommandGrid`]) of the panel shown.
pub(in crate::hud) fn update_command_hint(
    actions: Res<ActionState>,
    mode: Res<ClientMode>,
    size: Res<SurfaceSize>,
    view: Res<CommandPanelView>,
    mut hints: Query<(&mut Text, &mut Node), With<CommandHint>>,
) {
    let slot = (mode.is_game() && actions.pointer().over_ui)
        .then(|| actions.cursor())
        .flatten()
        .and_then(|cursor| CommandGrid::on(size.logical()).slot_at(cursor));
    let shown = slot.and_then(|slot| hint(&view.panel, &view.hotkeys, slot));
    for (mut text, mut node) in &mut hints {
        set_text(&mut text, shown.as_deref().unwrap_or(""));
        show_node(&mut node, shown.is_some());
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use sc_sim::{Constructor, ObjectId, ObjectKind, construction, transform};

    use super::*;
    use crate::command_ui::interface::{BuildMenu, commands, grid, panel};
    use crate::command_ui::{Conditions, InterfaceState, PanelButton, PanelCommand};
    use crate::hud::command_panel::hotkey_label;
    use crate::input::Bindings;
    use crate::selection::{GroupType, Selection};

    /// The panels of a selection of one object of type `ty`, in every state it may be in: DefaultState, for a
    /// constructor idle, constructing and holding ready each building it constructs; in each of its menus; awaiting
    /// the target of each of its commands that needs one. An Egress Node has Drones to select.
    fn panels_of(ty: GroupType) -> Vec<Panel> {
        let selection = Selection::new([(ObjectId(1), ty)]);
        let base = Conditions::default().with_returning(vec![ObjectId(2)]);
        let mut conditions = vec![base.clone()];
        for recipe in construction::recipes(ty.kind) {
            let states = [
                Constructor::Idle,
                Constructor::Constructing {
                    kind: recipe.kind,
                    progress: 0,
                },
                Constructor::Ready { kind: recipe.kind },
            ];
            conditions.extend(states.map(|state| base.clone().with_constructor(state)));
        }
        let commands = grid(ty.kind).into_iter().flatten();
        let mut states = vec![InterfaceState::DefaultState];
        states.extend(BuildMenu::ALL.map(InterfaceState::BuildMenu));
        states.extend(commands.filter(|c| c.needs_target()).map(InterfaceState::awaiting));
        let panels = conditions
            .iter()
            .flat_map(|c| states.iter().map(|&state| panel(&selection, state, c)));
        panels.collect()
    }

    /// What the hint says of `slot` on the first panel of `ty` in `state` that has a button there.
    fn hint_on(ty: GroupType, state: InterfaceState, slot: CommandSlot, constructor: Option<Constructor>) -> String {
        let hotkeys: Vec<String> = CommandSlot::all()
            .map(|s| hotkey_label(&Bindings::default(), s))
            .collect();
        let selection = Selection::new([(ObjectId(1), ty)]);
        let mut conditions = Conditions::default();
        if let Some(constructor) = constructor {
            conditions = conditions.with_constructor(constructor);
        }
        hint(&panel(&selection, state, &conditions), &hotkeys, slot).expect("a button")
    }

    /// Slice M3-K2a: the hint names every button of every panel in full, after its hotkey: every command each type
    /// may show, in every state, greyed out or not, and Back. A label that is its command's name in full ("Move",
    /// "Back") is the only one the hint repeats; a button that makes something names it in full ("PK": "Train
    /// Peacekeeper").
    #[test]
    fn the_hint_names_every_button_in_full() {
        let bindings = Bindings::default();
        let hotkeys: Vec<String> = CommandSlot::all().map(|slot| hotkey_label(&bindings, slot)).collect();
        let whole = ["Move", "Attack", "Patrol", "Stop", "Deploy", "Back"];
        let mut types: Vec<GroupType> = ObjectKind::ALL.iter().map(|&kind| GroupType::finished(kind)).collect();
        let warped_in = transform::warp_in_recipes(ObjectKind::Beacon).iter().map(|r| r.kind);
        types.extend(warped_in.chain([ObjectKind::Beacon]).map(GroupType::unfinished));
        let mut backs = 0;
        for ty in types {
            let mut seen = BTreeSet::new();
            for panel in panels_of(ty) {
                for (slot, shown) in panel.slots() {
                    let Some(shown) = shown else {
                        continue;
                    };
                    let name = panel.name(slot).unwrap();
                    let key = &hotkeys[slot_index(slot)];
                    assert_eq!(hint(&panel, &hotkeys, slot), Some(format!("{key}: {name}")));
                    let label = shown.label();
                    assert!(
                        name.len() > label.len() || whole.contains(&label),
                        "{ty:?}: {label} named {name}"
                    );
                    let PanelButton::Command(command) = shown.button else {
                        backs += 1;
                        continue;
                    };
                    match command {
                        PanelCommand::Train(kind)
                        | PanelCommand::Construct(kind)
                        | PanelCommand::PlaceReady(kind)
                        | PanelCommand::WarpIn(kind)
                        | PanelCommand::Convert(kind) => assert!(name.ends_with(kind.def().name), "{name}"),
                        PanelCommand::Planned(planned) => assert!(name.ends_with(planned.name()), "{name}"),
                        _ => {}
                    }
                    seen.insert(command);
                }
            }
            // All of them; but the Peacekeeper (Shielded)'s panel shows "no Attack" (design change 0173).
            let hidden = (ty.kind == ObjectKind::PeacekeeperShielded).then_some(PanelCommand::Attack);
            for command in commands(ty).into_iter().filter(|&c| Some(c) != hidden) {
                assert!(seen.contains(&command), "{ty:?}'s {command:?} is shown");
            }
        }
        assert!(backs > 0);
    }

    /// The names the design gives some buttons after the object whose panel shows them (`PanelCommand::name_on`), and
    /// the other kinds of names.
    #[test]
    fn the_hint_names_a_button_as_the_design_does_on_its_panel() {
        use ObjectKind::{
            Barracks, Beacon, DeploymentCenter, Drone, EgressNode, ExtractionFacility, ExtractionPlate, Generator,
            Peacekeeper, RifleLegionnaire, RoboPortal, SabreLegionnaire,
        };
        let slot = |row, column| CommandSlot::new(row, column).unwrap();
        let (q, w, r, s, x) = (slot(0, 0), slot(0, 1), slot(0, 3), slot(1, 1), slot(2, 1));
        let finished = GroupType::finished;
        let default = InterfaceState::DefaultState;
        let menu = InterfaceState::BuildMenu;
        let cases = [
            (hint_on(finished(Beacon), default, q, None), "Q: Warp In (Basic)"),
            (hint_on(finished(Beacon), default, w, None), "W: Warp In (Advanced)"),
            (
                hint_on(finished(DeploymentCenter), default, q, None),
                "Q: Basic Construction",
            ),
            (
                hint_on(finished(SabreLegionnaire), default, r, None),
                "R: Convert to Rifle Legionnaire",
            ),
            (
                hint_on(finished(RifleLegionnaire), default, r, None),
                "R: Convert to Sabre Legionnaire",
            ),
            (
                hint_on(GroupType::unfinished(Beacon), default, x, None),
                "X: Cancel Deployment",
            ),
            (
                hint_on(GroupType::unfinished(EgressNode), default, x, None),
                "X: Cancel Warp In",
            ),
            (
                hint_on(
                    finished(ExtractionFacility),
                    default,
                    x,
                    Some(Constructor::Ready { kind: ExtractionPlate }),
                ),
                "X: Cancel Ready Plate",
            ),
            (
                hint_on(
                    finished(DeploymentCenter),
                    default,
                    x,
                    Some(Constructor::Ready { kind: Generator }),
                ),
                "X: Cancel Ready Building",
            ),
            (
                hint_on(
                    finished(DeploymentCenter),
                    menu(BuildMenu::Basic),
                    q,
                    Some(Constructor::Ready { kind: Generator }),
                ),
                "Q: Place Generator",
            ),
            (
                hint_on(finished(DeploymentCenter), menu(BuildMenu::Basic), w, None),
                "W: Build Barracks",
            ),
            (
                hint_on(
                    finished(DeploymentCenter),
                    menu(BuildMenu::Basic),
                    CommandSlot::BACK,
                    None,
                ),
                "Z: Back",
            ),
            (hint_on(finished(Barracks), default, q, None), "Q: Train Peacekeeper"),
            (hint_on(finished(RoboPortal), default, w, None), "W: Train Scout"),
            (
                hint_on(finished(Beacon), menu(BuildMenu::Basic), w, None),
                "W: Warp In Robo Portal",
            ),
            (
                hint_on(finished(Beacon), menu(BuildMenu::Advanced), q, None),
                "Q: Warp In Exophysics Engine",
            ),
            (hint_on(finished(Drone), default, s, None), "S: Set Drop-off Location"),
            (hint_on(finished(Peacekeeper), default, slot(0, 1), None), "W: Reverse"),
            (hint_on(finished(Peacekeeper), default, r, None), "R: Raise Shield"),
        ];
        for (said, expected) in cases {
            assert_eq!(said, expected);
        }
    }
}
