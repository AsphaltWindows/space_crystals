//! The ObjectInterfaceState templates (`systems/controls/interface.md > ObjectInterfaceState[ObjectEnum]`):
//! which commands a type offers, in which slots of the 4 × 3 grid (`systems/controls/interface.md >
//! CommandPanel > Grid Layout`), and how its clicks resolve (`super::super::resolution`). Each type's template is in
//! its module (`crate::kinds`, [`PanelTemplate`]); a panel of its own, by faction and object: [`gdo`], [`colonists`],
//! [`syndicate`]. A constructor's panel also follows its state and the menu that is open ([`state_grid`]): the
//! Deployment Center's build menus ([`BuildMenu`], design change 0104) and the Extraction Facility's plate
//! (`factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`). The Colonists' Beacon
//! has two Warp In menus built the same way, the Drone its Deploy and its Set Drop-off, the Egress Node its F, and an
//! object that transforms shows only its Cancel ([`colonists`], slices M3-J1 and M3-J2), a portal its C besides
//! ([`unfinished_grid`], slice M5-S6). The Syndicate's Tunnel opens
//! its EjectMenu ([`syndicate`], slice M4-J2), and its Agent builds Tunnels (slice M4-J1). The Cults' pool buildings,
//! the Armory first, open a menu per kind of Recruit ([`cults`], slice M5-F2), and the Recruit two Construct menus
//! built the same way, their entries the group's Builds (slice M5-J1); the Garrison lets everyone out (slice M5-N2).
//! The GDO's Field Engineer repairs, and its Construct menu holds its buildings' slots (slice M5-N9). The Cults' Altar
//! picks a unit type, then a state, then the orders (slice M5-N1b); their Cathedral is a pool building as the Armory
//! is, and their Disciple's panel the BasicCombatUnit's, its spell's slot held in R (slice M5-N4a); their Temple
//! researches each line's next level and lets its staff out (slice M5-N3), and their Shrine casts its two area
//! abilities and lets its participants out (slice M5-N5). The GDO's Drone Operator launches its drone and builds the
//! next, and its drone is flown with Move, Attack and Stop (slice M6-GD); its Radar Station scans, and its Defense
//! System builds Missile Turrets as the Extraction Facility its plate (slice M6-GR). An entry not in the game yet holds
//! its slot,
//! unavailable ([`Planned`]).

pub mod colonists;
pub mod cults;
pub mod gdo;
mod offers;
mod planned;
mod production;
mod site;
mod state;
pub mod syndicate;

use sc_sim::defs::SpellDef;
use sc_sim::{AreaAbility, ObjectKind};
use sc_sim::{conversion, tether};

pub use colonists::TRANSFORMING_GRID;
pub use offers::{commands, offers};
pub use planned::Planned;
pub use site::{SITE_GRID, unfinished_grid};
pub use state::{menu_grid, menu_of, state_grid};

use production::production_grid;
use state::{constructs, place_slot};

use super::{InterfaceState, PanelCommand};
use crate::input::CommandSlot;
use crate::kinds;

/// An ObjectInterfaceState template: which commands a type offers, in which slots, and how its clicks resolve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InterfaceTemplate {
    /// `systems/controls/interface.md > BasicCombatUnitInterfaceState`: the Peacekeeper's
    /// (`factions/gdo/units/peacekeeper.md`), and its shielded type's, whose A shows only while another selected unit
    /// can attack (design changes 0173 and 0399, [`attack_slot`]), and the Guard's
    /// (`factions/syndicate/units/guard.md`).
    BasicCombatUnit,
    /// A producing structure's panel (plan decision T23), built from its `ProductionDef` ([`production_grid`]): the
    /// Barracks' (`factions/gdo/structures/barracks.md > ObjectInterfaceState[Barracks]`) and the Colonists' portals'
    /// (`factions/colonists/structures/robo_portal.md > Robo Portal ObjectInterfaceState`,
    /// `factions/colonists/structures/bio_portal.md > Bio Portal ObjectInterfaceState`, design change 0124: "Standard
    /// production panel, same conventions as GDO's Barracks"), the Hideout's and the Recruitment Hall's, whose C is its
    /// whole panel (`factions/cults/structures/recruitment_hall.md > ObjectInterfaceState[RecruitmentHall]`, slice
    /// M5-C; design change 0342 removed its X). Its right-click sets the rally point
    /// (`systems/controls/interface.md > CommandPanel > Production Building Default Right-Click`; the Hall's
    /// own: "Right-click Ground: issues SetRallyPoint command to that location").
    Production,
    /// `factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`: its DefaultState
    /// opens the build menus, whose entries construct; once a building is ready, its entry enters AwaitingPlacement.
    /// It produces no units and has no right-click.
    DeploymentCenter,
    /// `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`: its DefaultState
    /// builds the Extraction Plate, then places it. No right-click.
    ExtractionFacility,
    /// `factions/colonists/economy.md > Beacon > Beacon ObjectInterfaceState` (design change 0124): its DefaultState
    /// opens the two Warp In menus, whose entries enter the Warp In's placement on the Beacon. No right-click.
    Beacon,
    /// `factions/colonists/units/drone.md > Drone ObjectInterfaceState`: Q Move, W Deploy, S Set Drop-off, X Stop. Its
    /// right-click (`factions/colonists/units/drone.md > Right-Click Resolution`: Gather, Drop Off, Move, Follow) is
    /// the sim's one rule, as every unit's (`crate::command_ui::resolution::unit_right_click`, slice M3-R).
    Drone,
    /// `factions/colonists/economy.md > Egress Node > Egress Node ObjectInterfaceState`: "F: Select Returning Drones"
    /// (slice M3-J2). No right-click.
    EgressNode,
    /// `factions/syndicate/tunnel_menus.md > Tunnel ObjectInterfaceState` (design change 0257, slice M4-J2): Q Upgrade,
    /// W Expand, E Eject, which opens the EjectMenu, and X Cancel ([`syndicate::tunnel`]). No right-click: the Tunnel
    /// produces no units.
    Tunnel,
    /// `factions/syndicate/units/agent.md > Agent ObjectInterfaceState` (design changes 0257, 0359 and 0398, slice
    /// M4-J1): Q Move, W Build Tunnel, A Attack, S Drop Off, D Repair (slice M5-N8), X Stop ([`syndicate::agent`]). Its
    /// right-click
    /// (`factions/syndicate/units/agent.md > Right-Click Resolution`: Gather, Drop Off, Enter, Repair, Attack, Move) is
    /// the sim's one rule, as every unit's.
    Agent,
    /// A pool building's panel (design change 0337, `factions/cults/structures/README.md > Choosing Recruits, Release
    /// and Eject All`; slice M5-F2), built from its `ProductionDef`: the Armory's ([`cults::armory`]). Its DefaultState
    /// picks a kind of Recruit, each opening its KindMenu, whose entries train or release one of that kind. Its
    /// right-click sets the rally point (`factions/cults/structures/armory.md > ObjectInterfaceState[Armory]`).
    PoolProducer,
    /// (M5-J1) `factions/cults/units/recruit.md > Recruit ObjectInterfaceState` (design changes 0322, 0327 and 0398): Q
    /// Move, W Basic Construct, E Advanced Construct, A Attack, S Assist Construction, D Repair (slice M5-N8), X Stop
    /// ([`cults::recruit`]); its Construct menus' entries enter the group's placement ([`cults::construct_menu`]). Its
    /// right-click (`factions/cults/units/recruit.md > Right-Click Resolution`: Gather, Drop Off, Enter, Assist,
    /// Repair, Attack, Move) is the sim's one rule, as every unit's.
    Recruit,
    /// (M5-N2) `factions/cults/structures/garrison.md > Placeholder stats > Panel`: "**A: Eject All** (everyone inside
    /// leaves); **X: Cancel construction** while being built" ([`cults::garrison`]; X is its site's, [`SITE_GRID`]).
    /// "Right-click resolution: none (it has no rally point)".
    Garrison,
    /// (M5-N9) `factions/gdo/units/field_engineer.md > Placeholder stats` (design changes 0328 and 0398): "**Q** Move;
    /// **W** Construct (a menu: **Q** Bunker, **W** First Aid Tent, both greyed until designed; **Z** back); **D**
    /// Repair (targeted: a damaged own building or vehicle); **X** Stop" ([`gdo::field_engineer`]). Its right-click ("a
    /// damaged own building or vehicle: Repair (Enter wins on a building it can enter, as for Recruits)") is the sim's
    /// one rule, as every unit's.
    FieldEngineer,
    /// (M5-N1b) `factions/cults/augmentation.md > Altar > ObjectInterfaceState[Altar]`: "a three-stage selection: **unit
    /// type → current AugmentationState → Augmentation to apply**" ([`cults::altar`]), its orders those of its
    /// `ProductionDef::augment`, with A Eject All, X Cancel Production and C Set Rally Point. Its right-click sets the
    /// rally point ("Right-click Ground: issues SetRallyPoint command to that location").
    Altar,
    /// (M5-N3) `factions/cults/structures/temple.md > Placeholder stats`: "**Q** Enhanced Constitution (next level),
    /// **W** Improved Metabolism (next level), **A/S/D/F** release one Recruit of a kind [...], **V: Eject All** (staff
    /// leave), **X: Cancel** (last research)" ([`cults::temple`]), its keys showing their line's next level
    /// (`super::Conditions::key`). No right-click: it has no rally point.
    Temple,
    /// (M5-N5) `factions/cults/structures/shrine.md > Placeholder stats`: "**Health regeneration** (hotkey Q)",
    /// "**Energy regeneration** (hotkey W)"; "Targeting: AwaitingTarget, left-click a point in vision. **A/S/D/F**
    /// release one Recruit of a kind, **V: Eject All**" ([`cults::shrine`]). No right-click: it has no rally point.
    Shrine,
    /// (M6-GD) The Drone Operator's drone (plan §GD: "the drone's panel: Move, Attack, Stop";
    /// [`gdo::operator_drone`]). Its right-click is the sim's one rule, as every unit's.
    OperatorDrone,
    /// (M6-GR) `factions/gdo/structures/radar_station.md > Placeholder stats`: "**Scan** (hotkey **Q**, targeted:
    /// left-click any point on the map)" ([`gdo::radar_station`]). No right-click: it has no rally point.
    RadarStation,
    /// (M6-GR) `factions/gdo/structures/defense_system.md > Placeholder stats`: "**Works like the Extraction Facility**
    /// [...] one slot, Q, no build menu": its DefaultState builds a Missile Turret, then places it
    /// ([`gdo::defense_system`]). No right-click.
    DefenseSystem,
}

/// The Deployment Center's build menus (`factions/gdo/structures/deployment_center.md >
/// ObjectInterfaceState[DeploymentCenter]`, design change 0104: "the DC's buildings don't fit one 12-slot panel, so
/// they are split"). Their idle, constructing and ready states work the same ([`state_grid`]). The Colonists' Beacon
/// has two Warp In menus "like the Deployment Center's two build menus" (`factions/colonists/economy.md > Beacon >
/// Beacon ObjectInterfaceState`): Warp In (Basic) and Warp In (Advanced), whose entries are its own ([`menu_grid`]).
/// So has the Cults' Recruit: BuildMenu\[Basic\] and BuildMenu\[Advanced\] (`factions/cults/units/recruit.md > Recruit
/// ObjectInterfaceState`, design change 0327), the Construct menus ([`cults::construct_menu`]). The GDO's Field
/// Engineer has one, its Construct menu ([`gdo::field_engineer`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuildMenu {
    /// BuildMenu\[Basic\], "Q: Basic Construction".
    Basic,
    /// BuildMenu\[Advanced\], "W: Advanced Construction": the Factory and everything after it.
    Advanced,
    /// (M5-N9) The Field Engineer's one build menu, its W: "**W** Construct (a menu: **Q** Bunker, **W** First Aid
    /// Tent, both greyed until designed; **Z** back)" (`factions/gdo/units/field_engineer.md > Placeholder stats`). A
    /// menu of its own, named Construct, since a panel with one build menu has no Basic or Advanced one. No other type
    /// has it.
    Construct,
}

impl BuildMenu {
    pub const ALL: [BuildMenu; 3] = [BuildMenu::Basic, BuildMenu::Advanced, BuildMenu::Construct];

    /// The Deployment Center's entries, slot by slot (`CommandSlot::all` order): the buildings it constructs, and
    /// (M6-GK) those it will, greyed ([`gdo::deployment_center::menu`]).
    pub fn entries(self) -> &'static [Option<PanelCommand>; 12] {
        gdo::deployment_center::menu(self)
    }
}

/// A type's panel template, from its module (`crate::kinds::ClientKind::template`): its ObjectInterfaceState
/// template, and what its own panel adds to it. A type with a panel of its own has its template in [`gdo`] or
/// [`colonists`], by object.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PanelTemplate {
    /// Its ObjectInterfaceState template ([`InterfaceTemplate::of`]); none for a type without commands.
    pub interface: Option<InterfaceTemplate>,
    /// The units of a producing structure's panel that aren't in the game yet ([`Planned`]), each with its slot
    /// (`CommandSlot::all` order), which [`production_grid`] adds to its recipes: a portal's (design change 0124); and
    /// the commands of a combat unit's panel that aren't in the game yet, which the BasicCombatUnit grid adds to its
    /// own ([`basic_combat_unit_grid`]): none now, the Disciple's Shroud having held R from slice M5-N4a until slice
    /// M5-N4b brought it ([`SHROUD_SLOT`]).
    pub planned: &'static [(usize, Planned)],
    /// (M5-N9) The units of a producing structure's panel that the design puts in a slot of their own, each with its
    /// slot (`CommandSlot::all` order), where [`production_grid`] puts their Train, the others going in order into the
    /// slots left: the Barracks' Rocketman in W and Field Engineer in E (slice M5-K, [`gdo::barracks`]).
    pub trains: &'static [(usize, ObjectKind)],
    /// The commands the design names after the object whose panel shows them, with their names
    /// (`PanelCommand::name_on`).
    pub names: &'static [(PanelCommand, &'static str)],
    /// (M5-N11) Whether its design leaves its template's attack commands out: the Prayer-mobile's,
    /// "BasicCombatUnitInterfaceState without attack commands" (`factions/cults/units/prayer_mobile.md > Placeholder
    /// stats`), which shows no D Attack Ground, not even unavailable as the Disciple's is
    /// ([`basic_combat_unit_grid`]). Its A is the selection's, as every unit's ([`attack_slot`], design change 0399: "A
    /// is shown whenever at least one selected group can attack"): not shown while no selected unit can attack, and
    /// beside one that can, the whole selection's Attack, which sends it along as a move (**reading**: the newer rule
    /// names every unit).
    pub without_attacks: bool,
}

impl PanelTemplate {
    /// No panel: a type without commands.
    pub const NONE: PanelTemplate = PanelTemplate {
        interface: None,
        planned: &[],
        trains: &[],
        names: &[],
        without_attacks: false,
    };

    /// The panel of `interface`, with nothing of its own.
    pub const fn of(interface: InterfaceTemplate) -> PanelTemplate {
        PanelTemplate {
            interface: Some(interface),
            ..PanelTemplate::NONE
        }
    }
}

/// `systems/controls/interface.md > BasicCombatUnitInterfaceState > DefaultState commands`, row by row from the
/// top left (`CommandSlot::all`).
#[rustfmt::skip]
const BASIC_COMBAT_UNIT_GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Attack, AttackGround, HoldPosition, Move, Patrol, Reverse, Stop};
    [
        Some(Move),   Some(Reverse), Some(HoldPosition), None,
        Some(Attack), Some(Patrol),  Some(AttackGround), None,
        None,         Some(Stop),    None,               None,
    ]
};

/// `factions/gdo/units/peacekeeper.md > Riot Shield > Hotkey`: "**Hotkey: R** (Raise Shield on Peacekeeper, Lower
/// Shield on Peacekeeper (Shielded)), placeholder" (design change 0174): slot R, the first slot the BasicCombatUnit
/// grid leaves free, where the Colonists' Legionnaires have their own command ([`CONVERT_SLOT`]).
const RIOT_SHIELD_SLOT: usize = 3;

/// `systems/controls/interface.md > BasicCombatUnitInterfaceState > DefaultState commands`: Attack's slot, A, every
/// unit's (`systems/controls/interface.md > CommandPanel > Standard Slot Assignments`: "Attack is always in slot
/// **A**"), which the panel fills as the selection has it ([`attack_slot`]).
const ATTACK_SLOT: usize = 4;

/// (M5-N11) `systems/controls/interface.md > BasicCombatUnitInterfaceState > DefaultState commands`: "[D]
/// AttackGround*", which a panel without attack commands leaves empty ([`PanelTemplate::without_attacks`]).
const ATTACK_GROUND_SLOT: usize = 6;

/// `factions/colonists/units/sabre_legionnaire.md > ObjectInterfaceState`: "BasicCombatUnitInterfaceState, plus:
/// R: Convert to Rifle Legionnaire", and the Rifle Legionnaire's "R: Convert to Sabre Legionnaire".
const CONVERT_SLOT: usize = 3;

/// (M5-N4b) `factions/cults/units/disciple.md > Placeholder stats`: "ObjectInterfaceState:
/// BasicCombatUnitInterfaceState, plus **R: Shroud**" ("Q0112: R, as the other units that add an ability to the combat
/// panel; Q stays Move").
const SHROUD_SLOT: usize = 3;

/// (M5-K) `factions/gdo/units/rocketman.md > Placeholder stats`: "Panel: BasicCombatUnitInterfaceState, plus **R:
/// Target Lock** (targeted: left-click an enemy unit or structure)".
const TARGET_LOCK_SLOT: usize = 3;

/// (M5-N11) The slot of a unit's cast of `ability` (`sc_sim::defs::SpellDef::Cast`):
/// `factions/cults/units/prayer_mobile.md > Placeholder stats`: "ObjectInterfaceState: BasicCombatUnitInterfaceState
/// without attack commands, plus **R: Inspire** and **F: Suppress**". None for the Shrine's and (M6-GR) the Radar
/// Station's, a structure's, in its own grid (`cults::shrine::GRID`, `gdo::radar_station::GRID`).
fn cast_slot(ability: AreaAbility) -> Option<usize> {
    match ability {
        AreaAbility::Inspire => Some(3),
        AreaAbility::Suppress => Some(7),
        AreaAbility::HealthRegeneration | AreaAbility::EnergyRegeneration | AreaAbility::Scan => None,
    }
}

impl InterfaceTemplate {
    /// The template of `kind`'s ObjectInterfaceState, its module's (`crate::kinds`); none for types without commands:
    /// the map resources, and the structures that have none (the Generator, the Extraction Plate). The Legionnaires'
    /// is "BasicCombatUnitInterfaceState, plus" their R (`factions/colonists/units/sabre_legionnaire.md >
    /// ObjectInterfaceState`: Convert, [`grid`]). Every type that produces units has the Production template; the
    /// Egress Node has its own, its F (slice M3-J2). Whatever its type, an object that transforms shows only its
    /// Cancel ([`TRANSFORMING_GRID`], `super::panel`) and resolves no right-click
    /// (`super::super::resolution::UnitCapabilities::of`), but a building that will produce or release units, which
    /// adds its C and sets its rally point (slice M5-S6, design change 0396: [`unfinished_grid`]).
    pub fn of(kind: ObjectKind) -> Option<Self> {
        kinds::of(kind).template.interface
    }

    /// Its DefaultState's commands for a `kind` object, slot by slot (`CommandSlot::all` order): the Production
    /// template's come from the type's `ProductionDef` ([`production_grid`]), an object's own from its module.
    pub fn grid(self, kind: ObjectKind) -> [Option<PanelCommand>; 12] {
        match self {
            InterfaceTemplate::BasicCombatUnit => basic_combat_unit_grid(kind),
            InterfaceTemplate::Production => production_grid(kind),
            InterfaceTemplate::DeploymentCenter => gdo::deployment_center::GRID,
            InterfaceTemplate::ExtractionFacility => gdo::extraction_facility::GRID,
            InterfaceTemplate::Beacon => colonists::beacon::GRID,
            InterfaceTemplate::Drone => colonists::drone::GRID,
            InterfaceTemplate::EgressNode => colonists::egress_node::GRID,
            InterfaceTemplate::Tunnel => syndicate::tunnel::GRID,
            InterfaceTemplate::Agent => syndicate::agent::GRID,
            InterfaceTemplate::PoolProducer => cults::armory::GRID,
            InterfaceTemplate::Recruit => cults::recruit::GRID,
            InterfaceTemplate::Garrison => cults::garrison::GRID,
            InterfaceTemplate::FieldEngineer => gdo::field_engineer::GRID,
            InterfaceTemplate::Altar => cults::altar::GRID,
            InterfaceTemplate::Temple => cults::temple::GRID,
            InterfaceTemplate::Shrine => cults::shrine::GRID,
            InterfaceTemplate::OperatorDrone => gdo::operator_drone::GRID,
            InterfaceTemplate::RadarStation => gdo::radar_station::GRID,
            InterfaceTemplate::DefenseSystem => gdo::defense_system::GRID,
        }
    }
}

/// A `kind` unit's BasicCombatUnit DefaultState ([`BASIC_COMBAT_UNIT_GRID`]), with its commands not in the game yet in
/// their slots ([`PanelTemplate::planned`]), none now (the Disciple's Shroud held R in slice M5-N4a); (M5-N11) without
/// its D, Attack Ground, for a type whose design leaves its attack commands out ([`PanelTemplate::without_attacks`]:
/// the Prayer-mobile).
fn basic_combat_unit_grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = BASIC_COMBAT_UNIT_GRID;
    let template = kinds::of(kind).template;
    for &(slot, planned) in template.planned {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    if template.without_attacks {
        grid[ATTACK_GROUND_SLOT] = None;
    }
    grid
}

/// `systems/combat.md > AttackType > CanTargetGround`, derived from the AttackType: the sim's rule, which its gate
/// takes an Attack Ground by (slice M4-O).
pub use sc_sim::combat::can_target_ground;

/// `kind`'s DefaultState commands, slot by slot (`CommandSlot::all` order): its template's grid, plus its own
/// commands in slots the template leaves free: a unit that carries a Riot Shield the command that turns it, in R
/// (`factions/gdo/units/peacekeeper.md > Riot Shield > Two unit types`, design change 0173: "The Peacekeeper's
/// panel shows **Raise Shield**; Peacekeeper (Shielded)'s shows **Lower Shield**": [`PanelCommand::RaiseShield`] or
/// [`PanelCommand::LowerShield`]); a unit with another form, a Legionnaire, the Convert to that form in R
/// ([`PanelCommand::Convert`]: "R: Convert to Rifle Legionnaire" on the Sabre's panel); (M5-N4b) a unit that casts
/// Shroud, the Disciple, the spell in R ([`PanelCommand::Shroud`], `sc_sim::defs::UnitDef::shroud`); (M5-K) a unit
/// with a Target Lock, the Rocketman, its Target Lock in R ([`PanelCommand::TargetLock`],
/// `sc_sim::defs::UnitDef::target_lock`); (M5-N11) a unit that casts area abilities, the Prayer-mobile, Inspire in R and
/// Suppress in F ([`PanelCommand::Cast`], [`cast_slot`]); (M6-GD) a unit whose stock holds drones, the Drone Operator,
/// Launch Drone in R and Build Drone in F ([`PanelCommand::Launch`], [`PanelCommand::Buy`]: `sc_sim::defs::UnitDef::stock`
/// whose item is on a leash, `sc_sim::defs::UnitDef::tether`; [`gdo::drone_operator`]). These are the commands the type
/// offers ([`offers`]); its panel's A is the selection's ([`attack_slot`]), and a constructor's panel follows its state
/// ([`state_grid`]).
pub fn grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let Some(template) = InterfaceTemplate::of(kind) else {
        return [None; 12];
    };
    let mut grid = template.grid(kind);
    if let Some(shield) = kind.def().unit.as_ref().and_then(|unit| unit.riot_shield) {
        grid[RIOT_SHIELD_SLOT] = Some(match kind == shield.raised {
            true => PanelCommand::LowerShield,
            false => PanelCommand::RaiseShield,
        });
    }
    if let Some(other) = conversion::converts_to(kind) {
        grid[CONVERT_SLOT] = Some(PanelCommand::Convert(other));
    }
    if kind.def().unit.as_ref().and_then(|unit| unit.shroud()).is_some() {
        grid[SHROUD_SLOT] = Some(PanelCommand::Shroud);
    }
    if kind.def().unit.as_ref().and_then(|unit| unit.target_lock).is_some() {
        grid[TARGET_LOCK_SLOT] = Some(PanelCommand::TargetLock);
    }
    let spells = kind.def().unit.as_ref().map_or(&[][..], |unit| unit.spells);
    for ability in spells.iter().filter_map(SpellDef::cast) {
        if let Some(slot) = cast_slot(ability) {
            grid[slot] = Some(PanelCommand::Cast(ability));
        }
    }
    let stock = kind.def().unit.as_ref().and_then(|unit| unit.stock);
    if let Some(stock) = stock.filter(|stock| tether::def_of(stock.item).is_some()) {
        grid[gdo::drone_operator::LAUNCH_SLOT] = Some(PanelCommand::Launch);
        grid[gdo::drone_operator::BUY_SLOT] = Some(PanelCommand::Buy(stock.item));
    }
    grid
}

/// `systems/controls/interface.md > CommandPanel > Standard Slot Assignments` (design changes 0397 and 0399): "**A:
/// Attack, on every unit** (user, 2026-11-27): Attack is always in slot **A**, whichever group is in focus, so the
/// key never changes. No unit puts another command on A [...]. **A is shown whenever at least one selected group can
/// attack** (user, 2026-11-27), whichever group is in focus, and issues the Attack to the whole selection. Only when
/// **no** selected unit can attack is A not shown." A DefaultState grid's A, `commands`'s, as the selection has it:
/// Attack while one of the selected units can attack (`attackers`: `super::Conditions::attacks`), on an unarmed unit's
/// panel too (a Drone's, a Field Engineer's), and nothing while none can (a Peacekeeper (Shielded)'s panel, which
/// "shows Lower Shield and no Attack" alone, design change 0173; a Disciple's, which has no weapon). On a building's
/// panel its A stays its own where it has one (a pool building's Eject All), and is the selection's Attack where it is
/// free: **reading**, the rule says "whichever group is in focus" and names the units' panels only ("No unit puts
/// another command on A").
pub fn attack_slot(commands: &mut [Option<PanelCommand>; 12], attackers: bool) {
    if matches!(commands[ATTACK_SLOT], None | Some(PanelCommand::Attack)) {
        commands[ATTACK_SLOT] = attackers.then_some(PanelCommand::Attack);
    }
}

/// A slot's position in `CommandSlot::all` order.
pub fn slot_index(slot: CommandSlot) -> usize {
    usize::from(slot.row()) * usize::from(CommandSlot::COLUMNS) + usize::from(slot.column())
}
