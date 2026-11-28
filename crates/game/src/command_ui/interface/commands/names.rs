//! The panel commands' labels and names ([`PanelCommand::label`], [`PanelCommand::name`], [`PanelCommand::name_on`]):
//! what a button shows, and what the InfoPanel's hint for it says (`crate::hud::info_panel::hint`).

use sc_sim::defs::PatchKind;
use sc_sim::{AreaAbility, AugmentOrder, ObjectKind, RecruitKind, Tech, vehicle_deploy};

use super::super::BuildMenu;
use super::PanelCommand;
use crate::kinds::{self, short_label};

impl PanelCommand {
    /// The button's label: a placeholder for its icon (Q0001), short enough to keep the label size on its button at
    /// 720p (slice M3-K2a: six characters at most, `crate::hud::command_panel`), its full name being
    /// [`Self::name_on`]'s, in the InfoPanel's hint for the button under the pointer (`crate::hud::info_panel::hint`).
    /// What a button makes, a unit to train or a building to construct, warp in or build on its site (one that isn't in
    /// the game yet too), it shows as an icon would: by its type's tag, as the map and the portraits show it
    /// ([`short_label`]: an Agent's Build Tunnel "T1", the tier a Tunnel starts at), and so does the Deployment
    /// Center's E: Place, "with the building's icon" (design change 0388). A ready building's slot is "Place", and
    /// every Cancel "Cancel". The others are the command's name or its first word ("Hold", "Rally", "Raise", "Lower",
    /// "Drop" for Set Drop-off Location and for the Agent's Drop Off, "Drones" for Select Returning Drones), or a
    /// shortening of it ("Rev.", "AtkGnd", "Conv.", "Adv.", the Tunnel's "Upgr." for Upgrade Tunnel). An EjectMenu tile
    /// shows its unit type's tag, as a unit trained does, and its count (`super::super::PanelSlot::count`); an
    /// ExpandMenu entry its expansion's, as a building constructed does. A pool building's kind buttons show their kind
    /// for the icon of its augments, "Plain", "Const.", "Metab.", "Both", with their counts; its KindMenu's training
    /// the unit's tag, as a Train does, its Release "Rel." and its Eject All "Eject" (slice M5-F2). An Altar's unit
    /// type tiles show their type's tag, as an EjectMenu tile does, its state tiles their kind as the kind buttons do,
    /// and its orders the Augmentation they apply, "Const.", "Metab." and "Both", for their icons (slice M5-N1b). A
    /// Temple's keys show their technology, its line's initials and its level, "EC L1", "IM L2", a key in progress
    /// "InProg" ("its key shows "In progress" meanwhile"), and its releases their kind as the kind buttons do, with
    /// their counts (slice M5-N3). A Disciple's spell is its name, "Shroud", whole (6 letters, as "AtkGnd"; slice
    /// M5-N4b). A Shrine's casts are their names' first words, "Health" and "Energy" (slice M5-N5; the names are TBD,
    /// `factions/cults/structures/shrine.md > Abilities`). A Rocketman's Target Lock is "Lock", its name's second word,
    /// which says what it does (slice M5-K). A Prayer-mobile's casts shorten their names, "Insp." and "Suppr." (slice
    /// M5-N11). A Drone Operator's Launch Drone is its name's first word, "Launch", and its Build Drone the drone's tag,
    /// "DRN", what it makes, as a Train shows its unit's (slice M6-GD). An aircraft's Return to Base is its name's last
    /// word, "Base", where it goes (slice M6-GH1). A Radar Station's Scan is its name, whole, and so is (M6-CM) a
    /// Disciple's Mirage, "Mirage" (6 letters, as "Shroud")
    /// (slice M6-GR). A vehicle's R is "Deploy", its name, into a type that can't move, and "Undep.", a shortening of
    /// Undeploy, into one that can (`sc_sim::vehicle_deploy::immobile`; slice M6-GA). (M6-SB) A tank's toggle says what
    /// a press does, as the Riot Shield's "Raise" and "Lower": its name, "Sludge", switching it on, and a shortening of
    /// "Sludge Off", "Sl.Off", switching it off; (slice M6-SC) a Cover's alike, "Cover" and "Cv.Off". (M6-ST1) A
    /// carrier's Unload All is its name's first word, "Unload" (6 letters, as "Shroud").
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
            | PanelCommand::TrainFrom(kind, _)
            | PanelCommand::AugmentMenu(kind)
            | PanelCommand::Construct(kind)
            | PanelCommand::WarpIn(kind)
            | PanelCommand::Eject(kind)
            | PanelCommand::Build(kind)
            | PanelCommand::Expand(kind)
            | PanelCommand::Place(kind)
            | PanelCommand::Buy(kind) => short_label(kind),
            PanelCommand::Planned(planned) => planned.label(),
            PanelCommand::CancelProduction
            | PanelCommand::CancelConstruction
            | PanelCommand::CancelReady
            | PanelCommand::CancelTransform
            | PanelCommand::CancelOperation => "Cancel",
            PanelCommand::SetRallyPoint => "Rally",
            PanelCommand::BuildMenu(BuildMenu::Basic) => "Basic",
            PanelCommand::BuildMenu(BuildMenu::Advanced) => "Adv.",
            PanelCommand::BuildMenu(BuildMenu::Construct) => "Constr",
            PanelCommand::PlaceReady(_) => "Place",
            PanelCommand::Deploy => "Deploy",
            PanelCommand::SetDropOff | PanelCommand::DropOff => "Drop",
            PanelCommand::SelectReturningDrones => "Drones",
            PanelCommand::EjectMenu => "Eject",
            PanelCommand::ExpandMenu => "Expand",
            PanelCommand::UpgradeTunnel => "Upgr.",
            PanelCommand::KindMenu(kind) | PanelCommand::AugmentState(_, kind) => kind_label(kind),
            PanelCommand::Augment(_, _, order) => order_label(order),
            PanelCommand::Release(_) | PanelCommand::ReleaseOf(..) => "Rel.",
            PanelCommand::EjectAll => "Eject",
            PanelCommand::Assist => "Assist",
            PanelCommand::Repair => "Repair",
            PanelCommand::Research(tech) => tech_label(tech),
            PanelCommand::Researching(_) => "InProg",
            PanelCommand::ReleaseOne(kind) => kind_label(kind),
            PanelCommand::Shroud => "Shroud",
            PanelCommand::Cast(AreaAbility::HealthRegeneration) => "Health",
            PanelCommand::Cast(AreaAbility::EnergyRegeneration) => "Energy",
            PanelCommand::Cast(AreaAbility::Inspire) => "Insp.",
            PanelCommand::Cast(AreaAbility::Suppress) => "Suppr.",
            PanelCommand::Cast(AreaAbility::Scan) => "Scan",
            PanelCommand::TargetLock => "Lock",
            PanelCommand::Launch => "Launch",
            PanelCommand::ReturnToBase => "Base",
            PanelCommand::VehicleDeploy(into) if vehicle_deploy::immobile(into) => "Deploy",
            PanelCommand::VehicleDeploy(_) => "Undep.",
            PanelCommand::Mirage => "Mirage",
            PanelCommand::Toggle { lays, on: true } => toggle_name(lays),
            PanelCommand::Toggle {
                lays: PatchKind::Sludge,
                on: false,
            } => "Sl.Off",
            PanelCommand::Toggle {
                lays: PatchKind::Smoke,
                on: false,
            } => "Cv.Off",
            PanelCommand::UnloadAll => "Unload",
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
            PanelCommand::BuildMenu(BuildMenu::Construct) => "Construct".into(),
            PanelCommand::Construct(kind) => named("Build", kind),
            PanelCommand::CancelConstruction => "Cancel Construction".into(),
            PanelCommand::CancelReady => "Cancel Ready Building".into(),
            PanelCommand::PlaceReady(kind) | PanelCommand::Place(kind) => named("Place", kind),
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
            PanelCommand::KindMenu(kind) => format!("{} Recruits", kind.name()),
            PanelCommand::TrainFrom(kind, _) => named("Train", kind),
            PanelCommand::Release(_) | PanelCommand::ReleaseOf(..) => "Release".into(),
            PanelCommand::EjectAll => "Eject All".into(),
            PanelCommand::AugmentMenu(kind) => kind.def().name.into(),
            PanelCommand::AugmentState(_, kind) => state_name(kind).into(),
            PanelCommand::Augment(_, _, order) => order_name(order).into(),
            PanelCommand::Assist => "Assist Construction".into(),
            PanelCommand::Research(tech) => format!("Research {}", tech.def().name),
            PanelCommand::Researching(tech) => format!("{}: In progress", tech.def().name),
            PanelCommand::ReleaseOne(kind) => format!("Release one {} Recruit", kind.name()),
            PanelCommand::Cast(ability) => ability.def().name.into(),
            PanelCommand::TargetLock => "Target Lock".into(),
            PanelCommand::Launch => "Launch Drone".into(),
            PanelCommand::ReturnToBase => "Return to Base".into(),
            PanelCommand::Buy(kind) => named("Build", kind),
            PanelCommand::VehicleDeploy(into) if vehicle_deploy::immobile(into) => "Deploy".into(),
            PanelCommand::VehicleDeploy(_) => "Undeploy".into(),
            PanelCommand::Toggle { lays, on: true } => format!("{} On", toggle_name(lays)),
            PanelCommand::Toggle { lays, on: false } => format!("{} Off", toggle_name(lays)),
            PanelCommand::UnloadAll => "Unload All".into(),
            PanelCommand::Move
            | PanelCommand::Attack
            | PanelCommand::Patrol
            | PanelCommand::Stop
            | PanelCommand::Deploy
            | PanelCommand::EjectMenu
            | PanelCommand::Repair
            | PanelCommand::Shroud
            | PanelCommand::Mirage => self.label().into(),
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
    ///   ObjectInterfaceState[ExtractionFacility]`), and (M6-GR) the Defense System's "X: Cancel Ready Turret"
    ///   (`factions/gdo/structures/defense_system.md > Placeholder stats`).
    pub fn name_on(self, kind: ObjectKind) -> String {
        let names = kinds::of(kind).template.names;
        match names.iter().find(|&&(command, _)| command == self) {
            Some(&(_, name)) => name.into(),
            None if self == PanelCommand::CancelTransform => "Cancel Warp In".into(),
            None => self.name(),
        }
    }
}

/// (M6-SB) The name of the toggle of a tank that lays `lays`' patches (`PanelCommand::Toggle`), the design's:
/// `factions/syndicate/units/shadebringer.md`, "Has two **toggleable abilities**, Sludge and Cover".
fn toggle_name(lays: PatchKind) -> &'static str {
    match lays {
        PatchKind::Sludge => "Sludge",
        PatchKind::Smoke => "Cover",
    }
}

/// The label of a kind's button (`PanelCommand::KindMenu`), and of an Altar's state tile (`PanelCommand::AugmentState`):
/// a placeholder for the icon of its augments (Q0001, design change 0337: "Each kind's button shows the unit with an
/// **icon for its augments**").
fn kind_label(kind: RecruitKind) -> &'static str {
    match kind {
        RecruitKind::Plain => "Plain",
        RecruitKind::Constitution => "Const.",
        RecruitKind::Metabolism => "Metab.",
        RecruitKind::Both => "Both",
    }
}

/// (M5-N3) The label of a Temple's key (`PanelCommand::Research`): a placeholder for the icon of its technology
/// (Q0001), its line's initials and its level.
fn tech_label(tech: Tech) -> &'static str {
    match tech {
        Tech::EnhancedConstitutionL1 => "EC L1",
        Tech::EnhancedConstitutionL2 => "EC L2",
        Tech::ImprovedMetabolismL1 => "IM L1",
        Tech::ImprovedMetabolismL2 => "IM L2",
    }
}

/// (M5-N1b) The name of an Altar's state tile (`factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar] >
/// AugmentMenu[UnitType]`): "**Q: Unaugmented**", "**W: Enhanced Constitution only**", "**E: Improved Metabolism
/// only**", "**R: Fully augmented**".
fn state_name(kind: RecruitKind) -> &'static str {
    match kind {
        RecruitKind::Plain => "Unaugmented",
        RecruitKind::Constitution => "Enhanced Constitution only",
        RecruitKind::Metabolism => "Improved Metabolism only",
        RecruitKind::Both => "Fully augmented",
    }
}

/// (M5-N1b) The label of an Altar's order (`PanelCommand::Augment`): a placeholder for the icon of the Augmentation it
/// applies (Q0001), as a kind's button shows its augments.
fn order_label(order: AugmentOrder) -> &'static str {
    match order {
        AugmentOrder::EnhancedConstitution => "Const.",
        AugmentOrder::ImprovedMetabolism => "Metab.",
        AugmentOrder::Both => "Both",
    }
}

/// (M5-N1b) The name of an Altar's order (`factions/cults/augmentation.md > Altar > Augmentation Orders`: "**Enhanced
/// Constitution**, **Improved Metabolism**, or **Both** (a single order that applies both)"): the Augmentation's
/// (`AugmentOrder::name`), Both's said in full, "Both Augmentations" (placeholder wording), as the hint names a
/// button more fully than its label does.
fn order_name(order: AugmentOrder) -> &'static str {
    match order {
        AugmentOrder::Both => "Both Augmentations",
        _ => order.name(),
    }
}
