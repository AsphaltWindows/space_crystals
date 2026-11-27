//! The controls doc, `docs/playtest/controls.md` (slice M5-Ma), from the code, so that it can't drift from it:
//!
//! - **The keys** of the game ([`keys`]): the bindings table's (`Bindings::default`), each with what it does, by
//!   group, the command panel's grid (`systems/controls/interface.md > CommandPanel > Grid Layout`), and the cursor's
//!   two modes, aiming and placing, with the pointer's looks (`systems/controls/hud.md > PointerDisplayType`).
//! - **Every faction's panels** ([`faction`]): for each type, the panel of one selected object of it, as the game
//!   builds it (`space_crystals::command_ui::interface::panel`, from the type's template:
//!   `space_crystals::command_ui::interface::templates`), every button with its key, its label, its full name (the
//!   hint's, `space_crystals::hud::info_panel::hint`) and notes: how it is used, why it is greyed, and what the player
//!   must own first (the sim's prerequisites, `sc_sim::tech`, in the game's words, "Requires a Temple"). Every menu a
//!   button opens follows it, its keys after the button's ("Q → W"), and a menu that is the same as one before it
//!   says so. Then a building's right-click, a constructor's panel while it builds, and the panel of a building not
//!   finished yet.
//!
//! The panels are those of [`reference`] conditions: everything the panel counts is there (free units, a load,
//! returning Drones), so that a button is greyed only for good, by its type or as not in the game yet.

use std::collections::BTreeSet;

use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use sc_sim::construction::site::{expansion_def, site_def};
use sc_sim::construction::{self, Constructor};
use sc_sim::{Faction, ObjectId, ObjectKind, Vec2, deploy, tech, transform};
use space_crystals::command_ui::feedback::{requires_text, unavailable_text};
use space_crystals::command_ui::interface::{grid, panel, slot_index};
use space_crystals::command_ui::resolution::object_right_click;
use space_crystals::command_ui::{
    ButtonLook, Conditions, InterfaceState, MapTarget, Modes, Panel, PanelButton, PanelCommand, PanelSlot,
    PointerDisplay, Resolved, UnitCapabilities,
};
use space_crystals::hud::command_panel::hotkey_label;
use space_crystals::input::{
    Binding, Bindings, Button, Chord, CommandSlot, EDGE_PAN_MARGIN_PX, EditorInput, InputAction, InputContext,
    Modifiers,
};
use space_crystals::kinds;
use space_crystals::selection::{GroupType, Selection};

use super::keep_current;

/// The doc, from the repository's root.
const DOC: &str = "docs/playtest/controls.md";
/// Its generated part's name ([`super::markers`]).
const PART: &str = "controls";
/// The command that regenerates it, which the test names when the doc is stale.
const REGENERATE: &str = "UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls";

/// The object whose panel is shown, and another one (a returning Drone).
const OBJECT: ObjectId = ObjectId(1);
const OTHER: ObjectId = ObjectId(2);
/// How deep menus nest: an Altar's state menus are the deepest, the third stage.
const MENU_DEPTH: usize = 3;

/// What gives a panel: the game's ([`game_panel`]), or, in a test, one with a changed template.
type Panels<'a> = &'a dyn Fn(GroupType, InterfaceState, &Conditions) -> Panel;

/// The doc is what the bindings and the templates give: else the test fails, naming the line and [`REGENERATE`].
#[test]
fn the_controls_doc_is_current() {
    keep_current(DOC, PART, &render(&game_panel), REGENERATE);
}

/// A changed template changes the doc, and the doc's test then fails, naming [`REGENERATE`]: here the Chop Shop's
/// kind menus become the Armory's, as if its template trained what the Armory does.
#[test]
fn a_changed_template_makes_the_doc_stale() {
    let changed = |ty: GroupType, state: InterfaceState, conditions: &Conditions| {
        let kind = match (ty.kind, state) {
            (ObjectKind::ChopShop, InterfaceState::KindMenu(_)) => ObjectKind::Armory,
            (kind, _) => kind,
        };
        game_panel(GroupType { kind, ..ty }, state, conditions)
    };
    let (rendered, doc) = (render(&changed), render(&game_panel));
    assert_ne!(rendered, doc);
    let (begin, end) = super::markers(PART);
    let current = format!("{begin}\n\n{doc}\n{end}\n");
    let problem = super::stale(DOC, &current, PART, &rendered, REGENERATE).expect("stale");
    assert!(problem.contains(REGENERATE), "{problem}");
    assert!(problem.contains("Train Militant"), "names the changed line: {problem}");
}

/// Every type's section names it as the game does, and every panel row has its key; the reference conditions grey
/// no button that the game would light once the player owns what it requires.
#[test]
fn every_panel_is_in_the_doc() {
    let doc = render(&game_panel);
    for &kind in ObjectKind::ALL.iter().filter(|kind| kind.def().faction.is_some()) {
        assert!(
            doc.contains(&format!("{} ({})", kind.def().name, kinds::short_label(kind))),
            "{kind:?}"
        );
    }
    let chop_shop = section(&doc, "Chop Shop (CS)");
    assert!(chop_shop.contains("| Q → Q | SKR | Build Seeker |"), "{chop_shop}");
    assert!(chop_shop.contains("| W | Const. |"), "{chop_shop}");
    let barracks = section(&doc, "Barracks (BR)");
    assert!(
        barracks.contains("| W | RN | Train Rocketman | Requires an Arsenal |"),
        "{barracks}"
    );
}

/// The part of `doc` under the heading `title`, to the next heading.
fn section<'a>(doc: &'a str, title: &str) -> &'a str {
    let start = doc
        .find(&format!("#### {title}\n"))
        .unwrap_or_else(|| panic!("{title}"));
    let rest = &doc[start + 5..];
    &rest[..rest.find("\n#").map_or(rest.len(), |end| end + 1)]
}

/// The panel the game shows for one selected object of type `ty`, in `state`, under `conditions`.
fn game_panel(ty: GroupType, state: InterfaceState, conditions: &Conditions) -> Panel {
    panel(&Selection::new([(OBJECT, ty)]), state, conditions)
}

/// The doc's generated part: the keys, then each faction's panels.
fn render(panels: Panels) -> String {
    let bindings = Bindings::default();
    let hotkeys: Vec<String> = CommandSlot::all().map(|slot| hotkey_label(&bindings, slot)).collect();
    let mut doc = keys(&bindings, &hotkeys);
    for &faction in Faction::ALL {
        doc.push('\n');
        doc.push_str(&self::faction(faction, panels, &hotkeys));
    }
    doc
}

/// A faction's name, as the HUD says it.
fn faction_name(faction: Faction) -> &'static str {
    match faction {
        Faction::Gdo => "GDO",
        Faction::Syndicate => "Syndicate",
        Faction::Cults => "Cults",
        Faction::Colonists => "Colonists",
    }
}

/// A type's name and its label, as its heading says them: "Chop Shop (CS)".
fn titled(kind: ObjectKind) -> String {
    format!("{} ({})", kind.def().name, kinds::short_label(kind))
}

/// A faction's section: its units, then its buildings, in the registry's order, each with its panels; then the types
/// without a command.
fn faction(faction: Faction, panels: Panels, hotkeys: &[String]) -> String {
    let kinds: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
        .filter(|kind| kind.def().faction == Some(faction))
        .collect();
    let mut out = format!("## {}\n", faction_name(faction));
    let mut without = Vec::new();
    for (heading, units) in [("Units", true), ("Buildings", false)] {
        out.push_str(&format!("\n### {heading}\n"));
        for &kind in kinds.iter().filter(|kind| kind.def().unit.is_some() == units) {
            match kind_section(kind, panels, hotkeys) {
                Some(section) => {
                    out.push('\n');
                    out.push_str(&section);
                }
                None => without.push(titled(kind)),
            }
        }
    }
    if !without.is_empty() {
        out.push_str(&format!("\nNo commands: {}.\n", without.join(", ")));
    }
    out
}

/// A type's section, or `None` for a type without commands: its panel and menus as a table, then its right-click, a
/// constructor's panel while it builds, and the panel of one not finished yet.
fn kind_section(kind: ObjectKind, panels: Panels, hotkeys: &[String]) -> Option<String> {
    let ty = GroupType::finished(kind);
    let rows = tree(
        ty,
        InterfaceState::DefaultState,
        &reference(kind, false),
        panels,
        hotkeys,
        0,
    );
    if rows.is_empty() {
        return None;
    }
    let mut out = format!("#### {}\n\n", titled(kind));
    out.push_str(&table(&rows));
    let lines = (right_click(kind).into_iter())
        .chain(constructing(kind, panels, hotkeys))
        .chain(unfinished(kind, panels, hotkeys));
    for line in lines {
        out.push('\n');
        out.push_str(&line);
        out.push('\n');
    }
    Some(out)
}

/// The conditions the doc shows a type's panels in: one object of it, selected, with everything its panel counts
/// there (free units of every kind, the types its Altar tiles show, a load, a Drone returning), and nothing the gate
/// refuses, so that a button is greyed only by its type (`WhyUnavailable::NotForThisType`) or as not in the game yet
/// (`WhyUnavailable::NotYet`). What the player must own first is in the notes instead ([`requires`]). A `site` is under
/// construction.
fn reference(kind: ObjectKind, site: bool) -> Conditions {
    let mut conditions = Conditions::with_modes(Modes::default().with_loaded([OBJECT]))
        .with_returning(vec![OTHER])
        .with_pool([2; 4]);
    for command in grid(kind).into_iter().flatten() {
        if let PanelCommand::AugmentMenu(unit) = command {
            conditions = conditions.with_stored(unit, [1; 4]);
        }
    }
    if site { conditions.on_site() } else { conditions }
}

/// A button of a panel, in the doc.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    /// The keys to press in turn, the menus' before it.
    keys: Vec<String>,
    /// Its label, on the button.
    label: String,
    /// Its full name, the hint's.
    name: String,
    /// How it is used and why it is greyed.
    notes: Vec<String>,
}

/// The buttons of `ty`'s panel in `state`, each with the menu it opens, if any. Back, which every menu has in Z, is the
/// keys' (`keys`).
fn buttons(
    ty: GroupType,
    state: InterfaceState,
    conditions: &Conditions,
    panels: Panels,
    hotkeys: &[String],
) -> Vec<(Row, Option<InterfaceState>)> {
    let shown = panels(ty, state, conditions);
    let rows = shown.slots().filter_map(|(slot, button)| {
        let button = button?;
        let PanelButton::Command(command) = button.button else {
            return None;
        };
        let row = Row {
            keys: vec![hotkeys[slot_index(slot)].clone()],
            label: button.label().to_owned(),
            name: shown.name(slot).unwrap_or_default(),
            notes: notes(ty.kind, command, &button),
        };
        Some((row, menu(command)))
    });
    rows.collect()
}

/// The buttons of `ty`'s panel in `state` with the menus they open, each menu's after its button, their keys after
/// the button's; a menu that is the same as one before it in this panel only says so.
fn tree(
    ty: GroupType,
    state: InterfaceState,
    conditions: &Conditions,
    panels: Panels,
    hotkeys: &[String],
    depth: usize,
) -> Vec<Row> {
    let mut rows = Vec::new();
    let mut menus: Vec<(String, Vec<Row>)> = Vec::new();
    for (mut row, opens) in buttons(ty, state, conditions, panels, hotkeys) {
        let inside = opens
            .filter(|_| depth < MENU_DEPTH)
            .map(|menu| tree(ty, menu, conditions, panels, hotkeys, depth + 1))
            .filter(|inside| !inside.is_empty());
        let Some(inside) = inside else {
            rows.push(row);
            continue;
        };
        let key = row.keys[0].clone();
        if let Some((same, _)) = menus.iter().find(|(_, rows)| *rows == inside) {
            row.notes.insert(0, format!("opens a menu, the same as {same}'s"));
            rows.push(row);
            continue;
        }
        row.notes.insert(0, "opens a menu".to_owned());
        rows.push(row);
        rows.extend(inside.iter().cloned().map(|mut row| {
            row.keys.insert(0, key.clone());
            row
        }));
        menus.push((key, inside));
    }
    rows
}

/// The menu a command opens, as the state its press leads to.
fn menu(command: PanelCommand) -> Option<InterfaceState> {
    Some(match command {
        PanelCommand::BuildMenu(menu) => InterfaceState::BuildMenu(menu),
        PanelCommand::EjectMenu => InterfaceState::EjectMenu,
        PanelCommand::ExpandMenu => InterfaceState::ExpandMenu,
        PanelCommand::KindMenu(kind) => InterfaceState::KindMenu(kind),
        PanelCommand::AugmentMenu(unit) => InterfaceState::AugmentMenu(unit),
        PanelCommand::AugmentState(unit, state) => InterfaceState::AugmentStateMenu(unit, state),
        _ => return None,
    })
}

/// Whether a command places a building: its ghost follows the cursor until a left-click places it
/// (`systems/controls/interface.md > ObjectInterfaceState[ObjectEnum]`'s AwaitingPlacement).
fn places(command: PanelCommand) -> bool {
    matches!(
        command,
        PanelCommand::PlaceReady(_)
            | PanelCommand::Place(_)
            | PanelCommand::WarpIn(_)
            | PanelCommand::Deploy
            | PanelCommand::Build(_)
            | PanelCommand::Expand(_)
    )
}

/// A button's notes: how it is used (it awaits a target, it places a building, Shift queues it), why it is greyed,
/// what the player must own first, and a research's later levels.
fn notes(kind: ObjectKind, command: PanelCommand, shown: &PanelSlot) -> Vec<String> {
    let mut notes = Vec::new();
    if command.needs_target() {
        notes.push("click a target".to_owned());
    } else if places(command) {
        notes.push("place it".to_owned());
    }
    if command.queues() {
        notes.push("Shift queues".to_owned());
    }
    if command == PanelCommand::SelectReturningDrones {
        notes.push("shown while Drones return to it".to_owned());
    }
    if let (ButtonLook::Unavailable | ButtonLook::Disabled, Some(why)) = (shown.look, shown.why) {
        notes.push(format!("greyed: \"{}\"", unavailable_text(why)));
    }
    notes.extend(
        requires(kind, command)
            .into_iter()
            .map(|required| requires_text(required).to_owned()),
    );
    if let PanelCommand::Expand(building) = command
        && let Some(def) = expansion_def(building).filter(|def| def.tier > 1)
    {
        notes.push(format!("needs a Tier {} Tunnel", def.tier));
    }
    if let PanelCommand::Research(tech) = command {
        let line = tech.def().line.techs();
        for later in line.into_iter().skip_while(|&t| t != tech).skip(1) {
            let mut then = format!("then {}", later.def().name);
            for &required in later.def().requires_structures {
                then.push_str(&format!(", {}", requires_text(required)));
            }
            notes.push(then);
        }
    }
    notes
}

/// What the player must own before the command is taken, as the gate checks it (`sc_sim::tech`): a training's entry's
/// Unit Prerequisite, a construction's or a Warp In's entry's and its building's TechPrerequisite, a worker's build's
/// or an expansion's building's, a research's structures. One missing greys the button, with the first of them.
fn requires(kind: ObjectKind, command: PanelCommand) -> Vec<ObjectKind> {
    match command {
        PanelCommand::Train(unit) | PanelCommand::TrainFrom(unit, _) => {
            let production = kind.def().structure.as_ref().and_then(|s| s.production.as_ref());
            let recipe = production.and_then(|production| production.recipe(unit));
            recipe.map_or_else(Vec::new, |recipe| recipe.requires.to_vec())
        }
        PanelCommand::Construct(building) => {
            construction::recipe(kind, building).map_or_else(Vec::new, |recipe| tech::recipe_requires(recipe).collect())
        }
        PanelCommand::WarpIn(building) => (transform::warp_in_recipe(kind, building))
            .map_or_else(Vec::new, |recipe| tech::recipe_requires(recipe).collect()),
        PanelCommand::Build(building) | PanelCommand::Expand(building) => tech::structure_requires(building).to_vec(),
        PanelCommand::Research(tech) => tech.def().requires_structures.to_vec(),
        _ => Vec::new(),
    }
}

/// The table of a type's buttons.
fn table(rows: &[Row]) -> String {
    let mut out = String::from("| Keys | Button | Command | Notes |\n|---|---|---|---|\n");
    for row in rows {
        let line = [
            row.keys.join(" → "),
            row.label.clone(),
            row.name.clone(),
            row.notes.join("; "),
        ];
        out.push_str(&format!("| {} |\n", line.join(" | ")));
    }
    out
}

/// A building's right-click (`systems/controls/interface.md > CommandPanel > Production Building Default
/// Right-Click`), as the game resolves it: its rally point, if it has one. A unit's is the keys'.
fn right_click(kind: ObjectKind) -> Option<String> {
    kind.def().structure.as_ref()?;
    let capabilities = UnitCapabilities::of(OBJECT, kind, &Modes::default());
    match object_right_click(capabilities, &MapTarget::ground(Vec2::ZERO))? {
        Resolved::Rally(_) => Some(
            "**Right-click** sets its rally point, on the ground or on an object; on the building itself, it clears it."
                .to_owned(),
        ),
        _ => None,
    }
}

/// A constructor's panel while it builds its first building and once that is ready (`Constructor`): the buttons it
/// doesn't show while idle, the building named generically.
fn constructing(kind: ObjectKind, panels: Panels, hotkeys: &[String]) -> Vec<String> {
    let Some(recipe) = construction::recipes(kind).first() else {
        return Vec::new();
    };
    let building = recipe.kind;
    let ty = GroupType::finished(kind);
    let rows = |state| {
        let conditions = reference(kind, false).with_constructor(state);
        buttons(ty, InterfaceState::DefaultState, &conditions, panels, hotkeys)
    };
    let idle: Vec<Row> = rows(Constructor::Idle).into_iter().map(|(row, _)| row).collect();
    let states = [
        (
            "While it builds",
            Constructor::Constructing {
                kind: building,
                progress: 0,
            },
        ),
        ("Once the building is ready", Constructor::Ready { kind: building }),
    ];
    let mut lines = Vec::new();
    for (when, state) in states {
        let new: Vec<String> = (rows(state).into_iter())
            .filter(|(row, _)| !idle.contains(row))
            .map(|(row, _)| {
                let named = row.name.replace(building.def().name, "the building");
                let greyed = row.notes.iter().find(|note| note.starts_with("greyed"));
                format!(
                    "{} {named}{}",
                    row.keys[0],
                    greyed.map_or_else(String::new, |note| format!(" ({note})"))
                )
            })
            .collect();
        if !new.is_empty() {
            lines.push(format!("**{when}:** {}.", new.join(", ")));
        }
    }
    lines
}

/// The kinds that transform before they are finished: what a Beacon warps in, and what a Drone deploys into.
fn transforming() -> BTreeSet<ObjectKind> {
    let warped_in = (ObjectKind::ALL.iter()).flat_map(|&kind| transform::warp_in_recipes(kind).iter().map(|r| r.kind));
    let deployed = (ObjectKind::ALL.iter()).filter_map(|&kind| deploy::deployment(kind).map(|recipe| recipe.kind));
    warped_in.chain(deployed).collect()
}

/// The panel of a building not finished yet: a worker's site or an expansion under construction, or a building that
/// transforms (`space_crystals::command_ui::interface::unfinished_grid`).
fn unfinished(kind: ObjectKind, panels: Panels, hotkeys: &[String]) -> Option<String> {
    let site = site_def(kind).is_some() || expansion_def(kind).is_some();
    if !site && !transforming().contains(&kind) {
        return None;
    }
    let conditions = reference(kind, site);
    let rows = buttons(
        GroupType::unfinished(kind),
        InterfaceState::DefaultState,
        &conditions,
        panels,
        hotkeys,
    );
    let named: Vec<String> = rows
        .iter()
        .map(|(row, _)| format!("{} {}", row.keys[0], row.name))
        .collect();
    (!named.is_empty()).then(|| format!("**Before it is finished:** {}.", named.join(", ")))
}

/// The keys: every binding of the game by group, each with what it does, then the command panel's grid, the cursor's
/// modes and the pointer's looks.
fn keys(bindings: &Bindings, hotkeys: &[String]) -> String {
    let mut groups: Vec<Vec<(String, Vec<String>)>> = vec![Vec::new(); Group::ALL.len()];
    for &(action, binding) in bindings.entries() {
        if !action.works_in(InputContext::Game) || matches!(action, InputAction::CommandSlot(_)) {
            continue;
        }
        let (group, does) = describe(action);
        let rows = &mut groups[group as usize];
        let label = binding_label(binding);
        match rows.iter_mut().find(|(said, _)| *said == does) {
            Some((_, keys)) if !keys.contains(&label) => keys.push(label),
            Some(_) => {}
            None => rows.push((does, vec![label])),
        }
    }
    groups[Group::Camera as usize].push((
        "Pan the camera that way".to_owned(),
        vec![format!(
            "The cursor at the screen's edge (within {EDGE_PAN_MARGIN_PX} pixels)"
        )],
    ));
    let mut out = String::from("## Keys\n");
    for group in Group::ALL {
        out.push_str(&format!("\n### {}\n\n| Keys | Does |\n|---|---|\n", group.title()));
        for (does, keys) in &groups[group as usize] {
            out.push_str(&format!("| {} | {does} |\n", key_list(keys)));
        }
        if group == Group::Commands {
            out.push_str(&panel_keys(bindings, hotkeys));
        }
    }
    out.push_str(&cursor(bindings, hotkeys));
    out
}

/// Keys that do the same, in a row's cell: all of them, or the first and the last of a long run ("Ctrl+1 …
/// Ctrl+0").
fn key_list(keys: &[String]) -> String {
    match keys {
        [first, .., last] if keys.len() > 4 => format!("{first} … {last}"),
        _ => keys.join(", "),
    }
}

/// The groups of the keys, in the doc's order.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Group {
    Camera,
    Selecting,
    ControlGroups,
    Commands,
    Placing,
    Game,
    Debug,
    Replays,
}

impl Group {
    const ALL: [Group; 8] = [
        Group::Camera,
        Group::Selecting,
        Group::ControlGroups,
        Group::Commands,
        Group::Placing,
        Group::Game,
        Group::Debug,
        Group::Replays,
    ];

    fn title(self) -> &'static str {
        match self {
            Group::Camera => "The camera",
            Group::Selecting => "Selecting",
            Group::ControlGroups => "Control groups",
            Group::Commands => "Commands",
            Group::Placing => "Placing a building",
            Group::Game => "The game",
            Group::Debug => "Debug keys (with debug commands on, as they are unless `--no-debug`)",
            Group::Replays => "Replays (`--replay`)",
        }
    }
}

/// What a key of the game does, and its group. A new action gets its row's text here; until then the doc names it
/// by its variant.
fn describe(action: InputAction) -> (Group, String) {
    use InputAction as A;
    let (group, does) = match action {
        A::PanLeft => (Group::Camera, "Pan west (held)"),
        A::PanRight => (Group::Camera, "Pan east (held)"),
        A::PanUp => (Group::Camera, "Pan north (held)"),
        A::PanDown => (Group::Camera, "Pan south (held)"),
        A::GrabPan => (Group::Camera, "Drag the map (held)"),
        A::Select => (
            Group::Selecting,
            "Select: click a unit or a building, or drag a box; while a command awaits its target, click the target",
        ),
        A::ToggleSelection => (
            Group::Selecting,
            "With a click or a box: add to the selection, or take a unit out of it; on a portrait, take it out",
        ),
        A::SelectSameType => (Group::Selecting, "With a click: every unit of that type on screen"),
        A::CenterOnObject => (
            Group::Selecting,
            "With a click on a portrait: centre the camera on that unit",
        ),
        A::NextGroup => (Group::Selecting, "The next type in the selection, whose panel shows"),
        A::PreviousGroup => (Group::Selecting, "The previous type in the selection"),
        A::AssignGroup(_) => (Group::ControlGroups, "Make the selection that control group"),
        A::AddToGroup(_) => (Group::ControlGroups, "Add the selection to that control group"),
        A::RecallGroup(_) => (Group::ControlGroups, "Select that control group"),
        A::RecallAndCenterGroup(_) => (
            Group::ControlGroups,
            "Select that control group and centre the camera on it",
        ),
        A::ContextCommand => (
            Group::Commands,
            "The selection's right-click on what is under the cursor, which the pointer previews (below): move, \
             attack, gather, drop off, enter, assist, repair, or a building's rally point; while a command awaits its \
             target or a building is placed, cancel it",
        ),
        A::Queue => (Group::Commands, "With an order: queue it after the unit's orders"),
        A::Cancel => (
            Group::Commands,
            "Cancel the command that awaits its target; in a menu or while placing, go back one step",
        ),
        A::JumpToAlert => (Group::Game, "Centre the camera on the latest minimap alert"),
        A::PlaytestMenu => (
            Group::Game,
            "The playtest menu, which pauses the game: Resume, Restart, Quit",
        ),
        A::ToggleFullscreen => (Group::Game, "Full screen or a window"),
        A::DebugSwitchPlayer => (Group::Debug, "Switch to the next player: hot-seat"),
        A::DebugToggleFog => (Group::Debug, "The fog of war off and on"),
        A::ToggleGrid => (Group::Debug, "The tile grid"),
        A::TogglePaths => (Group::Debug, "The units' paths"),
        A::ReplayPause => (Group::Replays, "Pause and resume"),
        A::ReplayStep => (Group::Replays, "Play one tick"),
        A::ReplaySlower => (Group::Replays, "Slower"),
        A::ReplayFaster => (Group::Replays, "Faster"),
        A::Editor(EditorInput::RotateClockwise) => (Group::Placing, "Turn the building clockwise"),
        A::Editor(EditorInput::RotateCounterClockwise) => (Group::Placing, "Turn it the other way"),
        A::Editor(EditorInput::FlipHorizontal) => (Group::Placing, "Flip it left to right"),
        A::Editor(EditorInput::FlipVertical) => (Group::Placing, "Flip it top to bottom"),
        A::Editor(EditorInput::PlayTest) => (Group::Game, "In a play-test from the map editor: back to the editor"),
        other => return (Group::Game, format!("{other:?}")),
    };
    (group, does.to_owned())
}

/// A binding as the doc says it: "Ctrl+1", "1 twice", "Shift (held)".
fn binding_label(binding: Binding) -> String {
    match binding {
        Binding::Press(chord) => chord_label(chord),
        Binding::DoubleTap(chord) => format!("{} twice", chord_label(chord)),
        Binding::Modifiers(held) => format!("{} (held)", modifiers_label(held).trim_end_matches('+')),
        Binding::Wheel(held) => format!("{}Mouse wheel", modifiers_label(held)),
    }
}

/// A chord: its modifiers, then its key or button.
fn chord_label(chord: Chord) -> String {
    let button = match chord.button {
        Button::Key(key) => key_label(key),
        Button::Mouse(MouseButton::Left) => "Left click".to_owned(),
        Button::Mouse(MouseButton::Right) => "Right click".to_owned(),
        Button::Mouse(MouseButton::Middle) => "Middle button".to_owned(),
        Button::Mouse(other) => format!("{other:?} mouse button"),
    };
    format!("{}{button}", modifiers_label(chord.modifiers))
}

/// Modifiers, each with its "+", as the panel's hotkeys say them (`hotkey_label`).
fn modifiers_label(modifiers: Modifiers) -> String {
    let held = [
        (modifiers.ctrl, "Ctrl+"),
        (modifiers.alt, "Alt+"),
        (modifiers.shift, "Shift+"),
    ];
    held.iter().filter(|(on, _)| *on).map(|(_, label)| *label).collect()
}

/// A key's name: as the panel's hotkeys say it, without "Key" or "Digit" ("Q", "1"), and the others' as on the
/// keyboard.
fn key_label(key: KeyCode) -> String {
    let named = match key {
        KeyCode::ArrowLeft => "Left arrow",
        KeyCode::ArrowRight => "Right arrow",
        KeyCode::ArrowUp => "Up arrow",
        KeyCode::ArrowDown => "Down arrow",
        KeyCode::Escape => "Esc",
        KeyCode::Period => ".",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        _ => {
            let name = format!("{key:?}");
            let short = name.strip_prefix("Key").or_else(|| name.strip_prefix("Digit"));
            return short.unwrap_or(&name).to_owned();
        }
    };
    named.to_owned()
}

/// The command panel's keys, its grid row by row (`systems/controls/interface.md > CommandPanel > Grid Layout`), and
/// the slots every panel keeps (`systems/controls/interface.md > CommandPanel > Standard Slot Assignments`).
fn panel_keys(bindings: &Bindings, hotkeys: &[String]) -> String {
    let columns = usize::from(CommandSlot::COLUMNS);
    let mut out = String::from(
        "\nThe command panel, at the bottom right, has 12 buttons; a key presses each, or a click. Its keys, as on the \
         keyboard:\n\n",
    );
    for (row, keys) in hotkeys.chunks(columns).enumerate() {
        out.push_str(&format!("| {} |\n", keys.join(" | ")));
        if row == 0 {
            out.push_str(&format!("|{}\n", "---|".repeat(columns)));
        }
    }
    let key = |slot: CommandSlot| hotkeys[slot_index(slot)].clone();
    let cancel = bindings
        .of(InputAction::Cancel)
        .map(binding_label)
        .collect::<Vec<_>>()
        .join(", ");
    let attack = CommandSlot::new(1, 0).map_or_else(String::new, key);
    out.push_str(&format!(
        "\n{back} is Back in every menu, as {cancel} is, and while a command awaits its target. On every unit, {attack} \
         is Attack (shown while one of the selected units can attack) and {cancel_key} is Stop; on a building, \
         {cancel_key} cancels, and {rally} sets the rally point.\n",
        back = key(CommandSlot::BACK),
        cancel_key = key(CommandSlot::CANCEL_PRODUCTION),
        rally = key(CommandSlot::RALLY_POINT),
    ));
    out
}

/// The cursor's modes, aiming and placing (`systems/controls/hud.md > AwaitingTarget[CommandType]`,
/// `systems/controls/hud.md > AwaitingPlacement`), and the pointer's looks (`systems/controls/hud.md >
/// PointerDisplayType`), as `space_crystals::command_ui::cursor` draws them.
fn cursor(bindings: &Bindings, hotkeys: &[String]) -> String {
    let said = |action| bindings.of(action).map(binding_label).collect::<Vec<_>>().join(", ");
    let (right, cancel, queue) = (
        said(InputAction::ContextCommand),
        said(InputAction::Cancel),
        said(InputAction::Queue),
    );
    let back = &hotkeys[slot_index(CommandSlot::BACK)];
    let mut out = format!(
        "\n### Aiming and placing\n\n\
         - **Aiming** (\"click a target\" in the panels' notes): the pointer is a crosshair. Left-click the target; \
         with {queue} the order is queued and the aim goes on until Shift is let go. {right}, {cancel} or {back} \
         cancels.\n\
         - **Placing** (\"place it\"): the building's ghost follows the cursor, green where it can stand and red \
         where it can't. Left-click places it; the placing keys above turn it; {cancel} or {back} goes back. \
         {right} stops placing (a building the Deployment Center has ready stays ready).\n\
         - **The pointer** shows what a right-click would do there, or the awaited click; over an object, a dot in its \
         owner's colour (yours green, an enemy's red, neutral yellow). Its colours are placeholders:\n\n\
         | Pointer | Means |\n|---|---|\n"
    );
    for display in POINTERS {
        let (look, means) = pointer(display);
        out.push_str(&format!("| {look} | {means} |\n"));
    }
    out
}

/// The pointer's looks, in the doc's order.
const POINTERS: [PointerDisplay; 12] = [
    PointerDisplay::Move,
    PointerDisplay::Attack,
    PointerDisplay::AttackGround,
    PointerDisplay::Patrol,
    PointerDisplay::GatherResources,
    PointerDisplay::ReturnResources,
    PointerDisplay::Enter,
    PointerDisplay::Assist,
    PointerDisplay::Repair,
    PointerDisplay::Cast,
    PointerDisplay::Mixed,
    PointerDisplay::Inactive,
];

/// A pointer's look and what it means (`space_crystals::command_ui::cursor`). A new PointerDisplay needs its row here
/// and in [`POINTERS`], then the doc regenerated ([`REGENERATE`]).
fn pointer(display: PointerDisplay) -> (&'static str, &'static str) {
    match display {
        PointerDisplay::Move => ("Green", "Move there, or follow the unit; a building's rally point"),
        PointerDisplay::Attack => ("Red", "Attack it; aiming an Attack at the ground, Attack Move there"),
        PointerDisplay::AttackGround => ("Crimson", "Attack Ground there"),
        PointerDisplay::Patrol => ("Orange", "Patrol there"),
        PointerDisplay::GatherResources => ("Cyan", "Gather from it"),
        PointerDisplay::ReturnResources => ("Gold", "Drop off the load there, or make it the drop-off"),
        PointerDisplay::Enter => ("Violet", "Enter it"),
        PointerDisplay::Assist => ("Yellow-green", "Help build it"),
        PointerDisplay::Repair => ("Tan", "Repair it"),
        PointerDisplay::Cast => ("Pink", "Cast there, its circle on the ground"),
        PointerDisplay::Mixed => ("Red and green stripes", "The selected units would do different things"),
        PointerDisplay::Inactive => ("Grey", "Nothing"),
    }
}
