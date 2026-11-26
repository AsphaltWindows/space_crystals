//! The commands' labels and full names ([`PanelCommand::label`], [`PanelCommand::name`], [`PanelCommand::name_on`]),
//! which the command panel's buttons and the InfoPanel's hint show: split from `super`, the commands and what they
//! give the sim.

use sc_sim::ObjectKind;

use super::super::BuildMenu;
use super::PanelCommand;
use crate::kinds::{self, short_label};

impl PanelCommand {
    /// The button's label: a placeholder for its icon (Q0001), short enough to keep the label size on its button
    /// at 720p (slice M3-K2a: six characters at most, `crate::hud::command_panel`), its full name being
    /// [`Self::name_on`]'s, in the InfoPanel's hint for the button under the pointer (`crate::hud::info_panel::hint`).
    /// What a button makes, a unit to train or a building to construct, warp in or build on its site (one that isn't
    /// in the game yet too), it shows as an icon would: by its type's tag, as the map and the portraits show it
    /// ([`short_label`]: an Agent's Build Tunnel "T1", the tier a Tunnel starts at). A ready building's slot is
    /// "Place", and every Cancel "Cancel". The others are the command's name or its first word ("Hold", "Rally",
    /// "Raise", "Lower", "Drop" for Set Drop-off Location and for the Agent's Drop Off, "Drones" for Select Returning
    /// Drones), or a shortening of it ("Rev.", "AtkGnd", "Conv.", "Adv.", the Tunnel's "Upgr." for Upgrade Tunnel). An
    /// EjectMenu tile shows its unit type's tag, as a unit trained does, and its count
    /// (`super::super::PanelSlot::count`); an ExpandMenu entry its expansion's, as a building constructed does.
    pub fn label(self) -> &'static str {
        match self {
            PanelCommand::Move => "Move",
            PanelCommand::Reverse => "Rev.",
            PanelCommand::HoldPosition => "Hold",
            PanelCommand::Attack => "Attack",
            PanelCommand::Patrol => "Patrol",
            PanelCommand::AttackGround => "AtkGnd",
            PanelCommand::Stop => "Stop",
            PanelCommand::RaiseShield => "Raise",
            PanelCommand::LowerShield => "Lower",
            PanelCommand::Convert(_) => "Conv.",
            PanelCommand::Train(kind)
            | PanelCommand::Construct(kind)
            | PanelCommand::WarpIn(kind)
            | PanelCommand::Eject(kind)
            | PanelCommand::Build(kind)
            | PanelCommand::Expand(kind) => short_label(kind),
            PanelCommand::Planned(planned) => planned.label(),
            PanelCommand::CancelProduction
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::CancelTransform
            | PanelCommand::CancelOperation => "Cancel",
            PanelCommand::SetRallyPoint => "Rally",
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic",
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Adv.",
            PanelCommand::PlaceReady(_) => "Place",
            PanelCommand::Deploy => "Deploy",
            PanelCommand::SetDropOff | PanelCommand::DropOff => "Drop",
            PanelCommand::SelectReturningDrones => "Drones",
            PanelCommand::EjectMenu => "Eject",
            PanelCommand::ExpandMenu => "Expand",
            PanelCommand::UpgradeTunnel => "Upgr.",
            PanelCommand::Assist => "Assist",
        }
    }

    /// The command's full name, which its button's label shortens ([`Self::label`]), as the design names it. Where
    /// the design names a command after the object whose panel shows it, the panel names it so ([`Self::name_on`]):
    /// this is the name of the others, and the generic one of those.
    pub fn name(self) -> String {
        let named = |verb: &str, kind: ObjectKind| format!("{verb} {}", kind.def().name);
        match self {
            PanelCommand::Reverse => "Reverse".into(),
            PanelCommand::HoldPosition => "Hold Position".into(),
            PanelCommand::AttackGround => "Attack Ground".into(),
            PanelCommand::RaiseShield => "Raise Shield".into(),
            PanelCommand::LowerShield => "Lower Shield".into(),
            PanelCommand::Convert(kind) => named("Convert to", kind),
            PanelCommand::Train(kind) => named("Train", kind),
            PanelCommand::CancelProduction => "Cancel Production".into(),
            PanelCommand::SetRallyPoint => "Set Rally Point".into(),
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic Construction".into(),
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Advanced Construction".into(),
            PanelCommand::Construct(kind) => named("Build", kind),
            PanelCommand::CancelConstruction => "Cancel Construction".into(),
            PanelCommand::CancelReady => "Cancel Ready Building".into(),
            PanelCommand::PlaceReady(kind) => named("Place", kind),
            PanelCommand::WarpIn(kind) => named("Warp In", kind),
            PanelCommand::Planned(planned) => planned.command_name(),
            PanelCommand::CancelTransform => "Cancel".into(),
            PanelCommand::SetDropOff => "Set Drop-off Location".into(),
            PanelCommand::SelectReturningDrones => "Select Returning Drones".into(),
            PanelCommand::Eject(kind) => named("Eject", kind),
            PanelCommand::Build(kind) => named("Build", kind),
            PanelCommand::DropOff => "Drop Off Resources".into(),
            PanelCommand::ExpandMenu => "Expand Tunnel".into(),
            PanelCommand::Expand(kind) => named("Build", kind),
            PanelCommand::CancelOperation => "Cancel Operation".into(),
            PanelCommand::UpgradeTunnel => "Upgrade Tunnel".into(),
            PanelCommand::Assist => "Assist Construction".into(),
            PanelCommand::Move
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::Stop
            | PanelCommand::Deploy
            | PanelCommand::EjectMenu => self.label().into(),
        }
    }

    /// The command's full name on the panel of a `kind` ActiveGroup, which the InfoPanel's hint shows
    /// (`super::super::Panel::name`): [`Self::name`], but for the commands the design names after the object whose
    /// panel shows them, from its panel template (`super::super::templates::PanelTemplate::names`):
    /// - a Beacon's menus: "Q: Warp In (Basic)", "W: Warp In (Advanced)" (`factions/colonists/economy.md > Beacon >
    ///   Beacon ObjectInterfaceState`), the Deployment Center's being "Basic Construction" and "Advanced
    ///   Construction";
    /// - the Cancel of an object that transforms (`factions/colonists/economy.md > Beacon Deployment > While
    ///   transforming: only Cancel`: "Cancel Deployment / Cancel Warp In"): a Beacon's under deployment, any other
    ///   building's under Warp In (the object is the building from the first frame, `sc_sim::transform`);
    /// - the Extraction Facility's "X: Cancel Ready Plate" (`factions/gdo/economy.md > ExtractionFacility >
    ///   ObjectInterfaceState[ExtractionFacility]`).
    pub fn name_on(self, kind: ObjectKind) -> String {
        let names = kinds::of(kind).template.names;
        match names.iter().find(|&&(command, _)| command == self) {
            Some(&(_, name)) => name.into(),
            None if self == PanelCommand::CancelTransform => "Cancel Warp In".into(),
            None => self.name(),
        }
    }
}
