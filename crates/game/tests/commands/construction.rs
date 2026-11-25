//! Construction from the command panel (slice M2-H2, plan decision T10): the Deployment Center's build menu
//! (`factions/gdo/structures.md > DeploymentCenter > ObjectInterfaceState[DeploymentCenter]`, design change 0104) and
//! the Extraction Facility's plate (`factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility]`): Construct, Cancel Construction and Cancel Ready Building, then
//! AwaitingPlacement, whose ghost follows the cursor green or red tile by tile (design change 0093) with the letters
//! of its sides (0079), turns with R and F (risk R11), places with a left-click (once with Shift, 0071) and goes back
//! with Esc, a right-click or Back, while the overlay shows the build area as it is. Nothing of it once the game is
//! over (0101). It plays on a field of its own ([`construction_scenario`]).

use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use sc_sim::{
    BuildArea, BuildingOrientation, Command, ConstructionCommand, Constructor, CornerPos, DebugCommand, ObjectId,
    ObjectKind, PlacementCheck, PlayerCommand, PlayerId, Rotation, Scenario, SideLetter, SimEvent, SlotId, Spawn,
    SpawnAt, Tile, TilePos, TilePresetId, TileProblem, WholeProblem, scenarios,
};
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::feedback::TargetFeedback;
use space_crystals::command_ui::{BuildMenu, ButtonLook, InterfaceState, PanelButton, PanelCommand, Placing};
use space_crystals::coords::drawn_position;
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::input::{self, CommandSlot};
use space_crystals::placement::ghost::GhostLabel;
use space_crystals::placement::overlay::BuildAreaMesh;
use space_crystals::placement::{BuildAreaOverlay, Ghost, PlacementGhost};
use space_crystals::sim_driver::{PendingCommands, SimRes};

use super::*;
use KeyCode::{ControlLeft, Digit1, Escape, KeyE, KeyF, KeyQ, KeyR, KeyW, KeyX, KeyZ, ShiftLeft, Tab};

const P1: PlayerId = PlayerId(1);
const GENERATOR: ObjectKind = ObjectKind::Generator;
const BARRACKS_KIND: ObjectKind = ObjectKind::Barracks;
const EXTRACTION_FACILITY: ObjectKind = ObjectKind::ExtractionFacility;
const PLATE_KIND: ObjectKind = ObjectKind::ExtractionPlate;

// The objects of [`construction_scenario`], by id.
/// Player 1's Deployment Center, on tiles (20, 11) to (23, 14): its build area is tiles (8, 0) to (35, 26).
const DC: u32 = 1;
/// Player 1's Extraction Facility, on tiles (14, 11) to (16, 13).
const EF: u32 = 2;
/// Two Space Crystals patches in the area, in sight: one stays free, the other takes a plate when a test needs one
/// ([`plate_on`]: a plate mines, which would change player 1's bank as the tests run).
const FREE_PATCH: TilePos = TilePos::new(18, 14);
const PLATED_PATCH: TilePos = TilePos::new(18, 16);
/// A patch out of the build area, in the scout's sight.
const FAR_PATCH: TilePos = TilePos::new(37, 17);
/// Player 1's Peacekeeper, at the centre of tile (26, 13), in the area.
const PK: u32 = 6;
/// Player 2's Generator, far away: their only structure.
const ENEMY_GENERATOR: u32 = 8;

/// A water tile beside the Deployment Center, and the corner raised to height 1 whose four tiles slope.
const WATER: TilePos = TilePos::new(19, 11);
const HILL: CornerPos = CornerPos::new(25, 11);
/// Where a Generator (2 × 2) may go: flat, free, in the area and in the Deployment Center's sight.
const OPEN: TilePos = TilePos::new(21, 16);
/// Where it may go at the area's east edge (x 35), in the scout's sight: it grows the area by a tile to the east.
const EDGE: TilePos = TilePos::new(34, 12);

/// A flat 64 × 48 map with player 1's base under the camera ([`VIEW_CENTER`]: x 11 to 39, y 8.4 to 19.75 in tiles),
/// ids in order:
/// 1. player 1's Deployment Center ([`DC`]); 2. their Extraction Facility ([`EF`]);
/// 3. to 5. the patches [`FREE_PATCH`], [`PLATED_PATCH`] and [`FAR_PATCH`];
/// 6. player 1's Peacekeeper [`PK`]; 7. their scout at the centre of tile (33, 16), in sight of the area's east edge
///    and of what lies beyond it;
/// 8. player 2's Generator ([`ENEMY_GENERATOR`]) far off screen.
///
/// The [`WATER`] tile and the [`HILL`] corner are in the Deployment Center's sight; tiles (30, 8) to (31, 9), in the
/// area, are in nobody's.
fn construction_scenario() -> Scenario {
    use ObjectKind::{CrystalPatch, DeploymentCenter, ExtractionFacility, Generator, Peacekeeper};
    let mut scenario = scenarios::empty(64, 48);
    scenario.map.terrain.set_tile(WATER, Tile::new(TilePresetId::WATER));
    scenario.map.terrain.set_corner(HILL, 1);
    let own = |kind, at| Spawn::new(kind, Some(SlotId(1)), at);
    let neutral = |kind, at| Spawn::new(kind, None, at);
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let center = |x, y| SpawnAt::Point(TilePos::new(x, y).center());
    scenario.map.spawns.extend([
        own(DeploymentCenter, tile(20, 11)),
        own(ExtractionFacility, tile(14, 11)),
        neutral(CrystalPatch, SpawnAt::Tile(FREE_PATCH)),
        neutral(CrystalPatch, SpawnAt::Tile(PLATED_PATCH)),
        neutral(CrystalPatch, SpawnAt::Tile(FAR_PATCH)),
        own(Peacekeeper, center(26, 13)),
        own(Peacekeeper, center(33, 16)),
        Spawn::new(Generator, Some(SlotId(2)), tile(56, 40)),
    ]);
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// Player 1's plate on the patch at `patch` (a debug spawn), two frames later.
fn plate_on(app: &mut App, patch: TilePos) {
    let spawn = DebugCommand::Spawn {
        kind: PLATE_KIND,
        owner: Some(P1),
        at: SpawnAt::Tile(patch),
        orientation: BuildingOrientation::default(),
    };
    submit(app, P1, Command::Debug(spawn));
    updates(app, 2);
    assert!(structure_at(app, PLATE_KIND, patch).is_some());
}

/// A game of [`construction_scenario`], as [`game`] sets it up, with `crystals` Space Crystals in player 1's bank.
fn construction_game(crystals: u32) -> App {
    let mut app = game_of(construction_scenario());
    set_crystals(&mut app, crystals);
    app
}

/// The middle of a `w × h` footprint whose top-left tile is `origin`: where the cursor goes to put it there.
fn middle(origin: TilePos, (w, h): (u32, u32)) -> Vec2 {
    origin.corner() + Vec2::new(w as f32, h as f32) * 32.0
}

/// Selects structure `id` (a click on it) alone.
fn select(app: &mut App, id: u32) {
    let at = match id {
        DC => middle(TilePos::new(20, 11), (4, 4)),
        EF => middle(TilePos::new(14, 11), (3, 3)),
        PK => TilePos::new(26, 13).center(),
        _ => panic!("no {id} to select"),
    };
    click(app, at, &[]);
    assert_eq!(selected(app), [id]);
}

/// Constructor `id`'s state, as the sim has it.
fn constructor(app: &App, id: u32) -> Constructor {
    let sim = app.world().resource::<SimRes>();
    sim.object(ObjectId(id))
        .and_then(|o| o.constructor)
        .expect("a constructor")
}

/// Runs until constructor `id` holds its building ready, then a frame more.
fn run_until_ready(app: &mut App, id: u32) {
    for _ in 0..2000 {
        if constructor(app, id).ready().is_some() {
            app.update();
            return;
        }
        app.update();
    }
    panic!("{id} never got its building ready");
}

/// The Deployment Center, selected and its group active, constructs a `kind` with its build menu's `key` (Q, then
/// `key`), and once it is ready enters its placement (Q, then `key` again: "Select completed building").
fn construct_and_place(app: &mut App, key: KeyCode, kind: ObjectKind) {
    tap(app, &[], KeyQ);
    tap(app, &[], key);
    assert_eq!(constructor(app, DC).current(), Some(kind));
    run_until_ready(app, DC);
    tap(app, &[], KeyQ);
    tap(app, &[], key);
    assert_eq!(placing(app), Some(Placing::new(ObjectId(DC), kind)));
}

/// What the local player places now.
fn placing(app: &App) -> Option<Placing> {
    interface(app).placing()
}

/// Every construction command given so far, as the sim received them (like [`orders`]): player, constructors,
/// command.
fn constructions(app: &App) -> Vec<(PlayerId, Vec<u32>, ConstructionCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Construction { objects, command } => {
                Some((c.player, objects.iter().map(|id| id.0).collect(), *command))
            }
            Command::Order { .. } | Command::Ability { .. } | Command::Production { .. } | Command::Debug(_) => None,
        })
        .collect()
}

/// A Place of the building at `origin`, turned by `orientation`.
fn place(origin: TilePos, orientation: BuildingOrientation) -> ConstructionCommand {
    ConstructionCommand::Place { origin, orientation }
}

/// The buttons the HUD shows: slot, button, look, label.
fn buttons(app: &mut App) -> Vec<(CommandSlot, PanelButton, ButtonLook, &'static str)> {
    let mut query = app.world_mut().query::<&CommandButton>();
    let mut shown: Vec<_> = query
        .iter(app.world())
        .map(|b| (b.slot, b.shown.button, b.shown.look, b.shown.label()))
        .collect();
    shown.sort_by_key(|(slot, ..)| *slot);
    shown
}

/// Points the cursor at the ground point `ground`, where it is drawn, and lets a frame pass.
fn hover(app: &mut App, ground: Vec2) {
    let view = drawn_position(app.world().resource::<SimRes>().map(), ground);
    let screen = screen_at(app, view);
    input(app).cursor = Some(screen);
    app.update();
}

/// The placement ghost as it shows now.
fn ghost(app: &App) -> Option<Ghost> {
    app.world().resource::<PlacementGhost>().shown.clone()
}

/// The ghost's labels that show (their text isn't empty), by their place (north, east, south and west side, then the
/// middle). None is ever hidden or culled: Bevy would drop a change of its text (`placement::ghost`).
fn ghost_labels(app: &mut App) -> Vec<(usize, String)> {
    let mut query = app
        .world_mut()
        .query::<(&GhostLabel, &Text2d, &Visibility, Has<NoFrustumCulling>)>();
    let mut shown = Vec::new();
    for (label, text, visibility, unculled) in query.iter(app.world()) {
        assert_eq!(
            (*visibility, unculled),
            (Visibility::Inherited, true),
            "label {}",
            label.0
        );
        if !text.0.is_empty() {
            shown.push((label.0, text.0.clone()));
        }
    }
    shown.sort();
    shown
}

fn labels(texts: &[&str]) -> Vec<(usize, String)> {
    texts.iter().enumerate().map(|(i, t)| (i, t.to_string())).collect()
}

/// The ghost's side letters, north, east, south and west.
fn letters(app: &App) -> Option<[SideLetter; 4]> {
    ghost(app).and_then(|g| g.sides)
}

/// What the build area overlay shows: the area it was built from, and whether it shows.
fn overlay(app: &mut App) -> (Option<BuildArea>, bool) {
    let shown = app.world().resource::<BuildAreaOverlay>().shown.clone();
    let mut query = app.world_mut().query_filtered::<&Visibility, With<BuildAreaMesh>>();
    let visible = query.single(app.world()).is_ok_and(|v| *v != Visibility::Hidden);
    assert_eq!(visible, app.world().resource::<BuildAreaOverlay>().visible);
    (shown, visible)
}

/// How many times the overlay's mesh was built.
fn overlay_builds(app: &App) -> u32 {
    app.world().resource::<BuildAreaOverlay>().builds
}

fn build_area(app: &App) -> BuildArea {
    app.world().resource::<SimRes>().build_area(P1)
}

/// Whether the input layer takes R and F as the placement keys (`input::Placing`).
fn placement_keys(app: &App) -> bool {
    app.world().resource::<input::Placing>().0
}

/// Whether no pointer shows over the map: the ghost is the cursor.
fn ghost_is_cursor(app: &App) -> bool {
    app.world().resource::<PointerLook>().placing
}

/// The feedback message that shows.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// The InfoPanel's activity area, while it shows.
fn activity(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    texts
        .iter(app.world())
        .find(|(which, _, node)| **which == InfoText::Activity && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
}

/// The structure of `kind` that player 1 owns at `origin`, turned by `orientation`, if there is one.
fn structure_at(app: &App, kind: ObjectKind, origin: TilePos) -> Option<(u32, BuildingOrientation)> {
    let sim = app.world().resource::<SimRes>();
    sim.objects().find_map(|o| {
        let placement = o.placement.as_ref()?;
        let orientation = BuildingOrientation {
            rotation: placement.rotation,
            flip_h: placement.flip_h,
            flip_v: placement.flip_v,
        };
        (o.kind == kind && o.owner == Some(P1) && placement.origin == origin).then_some((o.id.0, orientation))
    })
}

fn q() -> CommandSlot {
    slot(0, 0)
}

fn w() -> CommandSlot {
    slot(0, 1)
}

fn e() -> CommandSlot {
    slot(0, 2)
}

const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;

/// `> ObjectInterfaceState[DeploymentCenter]` (design change 0104): a selected Deployment Center shows Q Basic
/// Construction, and W Advanced Construction unavailable while none of its buildings is in the game. Q, or a click on
/// its button, opens BuildMenu[Basic]: Q Generator, W Barracks, E Extraction Facility, the other slots hidden, and Z
/// Back. Back, Esc and a right-click return to DefaultState (`> BuildMenu (instance idle)`: "Escape/right-click:
/// returns to DefaultState"), and a left-click on the map selects as ever. Beside an Extraction Facility, recalled
/// together from a control group (design change 0048's verification: "Select two own Ungroupable buildings (e.g. via
/// control group): Tab switches the command panel between them"), Tab makes that the active group: the menu closes,
/// and Q builds a plate there, at the Extraction Facility only.
#[test]
fn the_deployment_center_opens_its_basic_build_menu() {
    use ButtonLook::{Back, Common, Group, Unavailable};
    use PanelCommand::{BuildMenu as Menu, Construct};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (w(), c(Menu(BuildMenu::Advanced)), Unavailable, "Adv."),
        ]
    );
    tap(&mut app, &[], KeyW);
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "nothing to build in it yet"
    );

    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), menu);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Common, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Common, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Common, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
        ]
    );
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    click_button(&mut app, q(), &[]);
    assert_eq!(interface(&app), menu);
    click_button(&mut app, CommandSlot::BACK, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], Escape);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    right_click(&mut app, middle(OPEN, (2, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(constructions(&app), []);
    assert_eq!(orders(&app), []);

    // The menu leaves the map's left-clicks to the selection: a click on the Peacekeeper selects it.
    tap(&mut app, &[], KeyQ);
    select(&mut app, PK);
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    // Beside the Extraction Facility, in control group 1: the Deployment Center's group is active first (`ObjectKind`
    // order).
    select(&mut app, DC);
    add_box(&mut app, middle(TilePos::new(14, 11), (3, 3)));
    tap(&mut app, &[ControlLeft], Digit1);
    select(&mut app, PK);
    tap(&mut app, &[], Digit1);
    assert_eq!(selected(&app), [DC, EF]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), menu);
    tap(&mut app, &[], Tab);
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "the Extraction Facility has no build menu"
    );
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Group, "EP")]);
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        constructions(&app),
        [(P1, vec![EF], ConstructionCommand::Construct(PLATE_KIND))]
    );
    updates(&mut app, 1);
    assert_eq!(constructor(&app, DC), Constructor::Idle);
    assert_eq!(constructor(&app, EF).current(), Some(PLATE_KIND));
}

/// `> BuildMenu (instance idle)`: Q pays 100 Space Crystals for a Generator and returns to DefaultState, which then
/// shows X, Cancel Construction ("Only visible when CurrentConstruction is set"). In the build menu the entries are
/// unavailable while it is constructed (`> BuildMenu (instance constructing)`), and X cancels from there too, by
/// hotkey or by button: a full refund (`> Cancellation`), and back to DefaultState. The InfoPanel shows the
/// construction and its progress (design change 0105).
#[test]
fn a_construction_is_cancelled_with_a_full_refund() {
    use ButtonLook::{Back, Common, Disabled, Unavailable};
    use PanelCommand::{BuildMenu as Menu, CancelConstruction, Construct};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(GENERATOR);
    assert_eq!(constructions(&app), [(P1, vec![DC], construct)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(bank(&app), 900);
    assert_eq!(constructor(&app, DC).current(), Some(GENERATOR));
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Menu(BuildMenu::Basic)), Common, "Basic"),
            (w(), c(Menu(BuildMenu::Advanced)), Unavailable, "Adv."),
            (X, c(CancelConstruction), Common, "Cancel"),
        ]
    );
    updates(&mut app, 24);
    let progress = activity(&mut app).expect("an activity");
    assert!(progress.starts_with("Constructing Generator: "), "{progress}");
    assert_ne!(progress, "Constructing Generator: 0%");

    tap(&mut app, &[], KeyX);
    let cancel = ConstructionCommand::CancelConstruction;
    assert_eq!(constructions(&app), [(P1, vec![DC], construct), (P1, vec![DC], cancel)]);
    assert_eq!((bank(&app), constructor(&app, DC)), (1000, Constructor::Idle));
    assert_eq!(activity(&mut app), None);

    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyW);
    assert_eq!(bank(&app), 800, "a Barracks");
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(Construct(GENERATOR)), Disabled, "GN"),
            (w(), c(Construct(BARRACKS_KIND)), Disabled, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Disabled, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
            (X, c(CancelConstruction), Common, "Cancel"),
        ]
    );
    tap(&mut app, &[], KeyE);
    assert_eq!(constructions(&app).len(), 3, "unavailable");
    click_button(&mut app, X, &[]);
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], cancel)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!((bank(&app), constructor(&app, DC)), (1000, Constructor::Idle));
}

/// `> BuildMenu (building ready)`: once the Generator is ready its slot becomes "Select completed building", and X
/// Cancel Ready Building. It enters AwaitingPlacement: the ghost follows the cursor in place of the pointer, green
/// where it may go, the build area overlay shows the area, and R and F become the placement keys. A left-click there
/// places it at the ghost's tiles and orientation (one Place to the Deployment Center) and returns to DefaultState;
/// the Generator stands there from the next tick, and the ghost and the overlay are gone.
#[test]
fn a_ready_generator_is_placed_where_the_ghost_is_green() {
    use ButtonLook::{Back, Common, Disabled, Pending};
    use PanelCommand::{CancelReady, Construct, PlaceReady};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    run_until_ready(&mut app, DC);
    assert_eq!(activity(&mut app).as_deref(), Some("Generator ready to place"));
    tap(&mut app, &[], KeyQ);
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(PlaceReady(GENERATOR)), Common, "Place"),
            (w(), c(Construct(BARRACKS_KIND)), Disabled, "BR"),
            (e(), c(Construct(EXTRACTION_FACILITY)), Disabled, "EF"),
            (CommandSlot::BACK, PanelButton::Back, Back, "Back"),
            (X, c(CancelReady), Common, "Cancel"),
        ]
    );
    assert!(!placement_keys(&app));
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
    assert_eq!(buttons(&mut app)[0], (q(), c(PlaceReady(GENERATOR)), Pending, "Place"));
    assert!(placement_keys(&app));

    hover(&mut app, middle(OPEN, (2, 2)));
    let shown = ghost(&app).expect("the ghost");
    assert_eq!((shown.kind, shown.origin, shown.size), (GENERATOR, OPEN, (2, 2)));
    assert!(shown.is_valid());
    assert!(shown.tiles().all(|t| shown.tile_is_green(t)));
    assert_eq!(ghost_labels(&mut app), labels(&["A", "A", "A", "A", "GN"]));
    assert!(ghost_is_cursor(&app));
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    click(&mut app, middle(OPEN, (2, 2)), &[]);
    let construct = ConstructionCommand::Construct(GENERATOR);
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![DC], construct), (P1, vec![DC], placed)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, GENERATOR, OPEN).is_some());
    assert_eq!(constructor(&app, DC), Constructor::Idle);
    assert_eq!(selected(&app), [DC], "the click selected nothing");
    assert_eq!(ghost(&app), None);
    assert_eq!(ghost_labels(&mut app), []);
    assert!(!overlay(&mut app).1);
    assert!(!placement_keys(&app) && !ghost_is_cursor(&app));
    assert_eq!(activity(&mut app), None);
}

/// Design change 0093: the overlay shows the build area only, and the ghost shows each of its tiles green or red as
/// the placement check has it now: water (not Buildable), a slope, a unit, a structure, a resource, a tile nobody
/// sees, each red where it is and the other tiles green; a footprint with no tile in the build area is red
/// throughout. A click on a red ghost places nothing and says why; placement goes on.
#[test]
fn the_ghost_is_green_or_red_tile_by_tile_by_the_placement_rules() {
    use TileProblem::{NotBuildable, NotFlat, NotVisible, Resource, Structure, Unit};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let at = TilePos::new;
    let tiles = |tiles: Vec<(TilePos, TileProblem)>| PlacementCheck { tiles, whole: vec![] };
    let every = |origin: TilePos, problem| {
        let all = [(0, 0), (1, 0), (0, 1), (1, 1)];
        tiles(all.map(|(dx, dy)| (at(origin.x + dx, origin.y + dy), problem)).to_vec())
    };
    let outside = PlacementCheck {
        tiles: vec![],
        whole: vec![WholeProblem::OutsideBuildArea],
    };
    let cases = [
        (OPEN, PlacementCheck::default()),
        (at(18, 10), tiles(vec![(WATER, NotBuildable)])),
        (at(24, 10), every(at(24, 10), NotFlat)),
        (at(25, 12), tiles(vec![(at(26, 13), Unit)])),
        (at(23, 14), tiles(vec![(at(23, 14), Structure)])),
        (FREE_PATCH, tiles(vec![(FREE_PATCH, Resource)])),
        (at(30, 8), every(at(30, 8), NotVisible)),
        (at(36, 14), outside),
    ];
    for (origin, check) in cases {
        hover(&mut app, middle(origin, (2, 2)));
        let shown = ghost(&app).expect("the ghost");
        assert_eq!((shown.origin, &shown.check), (origin, &check));
        for t in shown.tiles() {
            let red = !check.whole.is_empty() || check.tiles.iter().any(|(bad, _)| *bad == t);
            assert_eq!(shown.tile_is_green(t), !red, "{t:?} of {origin:?}");
        }
        assert_eq!(shown.is_valid(), check == PlacementCheck::default());
    }
    // The overlay is the area, whatever the ghost is over.
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    // A click on the red ghost (still outside the area) sends nothing, and the player hears why.
    click(&mut app, middle(at(36, 14), (2, 2)), &[]);
    assert_eq!(constructions(&app).len(), 1, "the Construct only");
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
    assert_eq!(feedback(&app), Some("Outside the build area"));
    click(&mut app, middle(at(25, 12), (2, 2)), &[]);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
    assert_eq!(constructions(&app).len(), 1);
    assert_eq!(selected(&app), [DC], "the click on the Peacekeeper selected nothing");
}

/// `> AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise.
/// Non-square buildings swap footprint dimensions when rotated to 90 or 270. F flips the ghost horizontally, Shift+F
/// flips vertically. Side labels (A/B/C/D per SymmetryType) are displayed on the ghost and update with
/// rotation/flipping" (design change 0079): the Barracks, ABAC (top A, right B, bottom A, left C). R and F are the
/// placement keys here, not command slots, and Shift with them turns the other way: it doesn't queue anything (risk
/// R11). The Barracks goes on the map as the ghost was turned.
#[test]
fn r_and_f_turn_the_barracks_ghost_and_its_side_letters() {
    use SideLetter::{A, B, C};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyW, BARRACKS_KIND);
    hover(&mut app, middle(OPEN, (3, 2)));
    let shown = ghost(&app).expect("the ghost");
    assert_eq!(
        (shown.origin, shown.size, shown.sides),
        (OPEN, (3, 2), Some([A, B, A, C]))
    );
    assert_eq!(ghost_labels(&mut app), labels(&["A", "B", "A", "C", "BR"]));

    let turned = |app: &App| placing(app).map(|p| p.orientation);
    let size = |app: &App| ghost(app).map(|g| g.size);
    tap(&mut app, &[], KeyR);
    assert_eq!(turned(&app).map(|o| o.rotation), Some(Rotation::R90));
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([C, A, B, A])));
    assert_eq!(ghost_labels(&mut app), labels(&["C", "A", "B", "A", "BR"]));
    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(turned(&app), Some(BuildingOrientation::default()));
    assert_eq!((size(&app), letters(&app)), (Some((3, 2)), Some([A, B, A, C])));
    tap(&mut app, &[], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((3, 2)), Some([A, C, A, B])));
    tap(&mut app, &[], KeyF);
    assert_eq!(turned(&app), Some(BuildingOrientation::default()));
    tap(&mut app, &[], KeyR);
    tap(&mut app, &[ShiftLeft], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([B, A, C, A])));
    tap(&mut app, &[ShiftLeft], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([C, A, B, A])));
    assert_eq!(
        constructions(&app).len(),
        1,
        "the Construct only: nothing queued, no slot pressed"
    );
    assert_eq!(placing(&app).map(|p| p.kind), Some(BARRACKS_KIND));

    // Placed as turned: its exit side, B, faces south.
    let shown = ghost(&app).expect("the ghost");
    let orientation = turned(&app).unwrap();
    click(&mut app, middle(OPEN, (3, 2)), &[]);
    assert_eq!(
        constructions(&app).last(),
        Some(&(P1, vec![DC], place(shown.origin, orientation)))
    );
    assert_eq!(
        structure_at(&app, BARRACKS_KIND, shown.origin).map(|(_, o)| o),
        Some(orientation)
    );
    assert!(!placement_keys(&app));
}

/// `> AwaitingPlacement`: "Escape/right-click: returns to BuildMenu (StateOnlyTransition), building remains ready to
/// place", and so does Back (Z); the ghost, the overlay and the placement keys go with the placement. From the build
/// menu, X Cancel Ready Building refunds 75% of the cost, rounded down (`> BuildMenu (building ready)`), and returns
/// to DefaultState. Then R is a command slot again: the Peacekeeper's Riot Shield.
#[test]
fn escape_right_click_and_back_leave_placement_for_the_build_menu() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let menu = InterfaceState::BuildMenu(BuildMenu::Basic);
    let ready = Constructor::Ready { kind: GENERATOR };
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app) && ghost_is_cursor(&app));
    tap(&mut app, &[], Escape);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1 && !placement_keys(&app) && !ghost_is_cursor(&app));

    tap(&mut app, &[], KeyQ);
    assert!(placing(&app).is_some());
    right_click(&mut app, middle(OPEN, (2, 2)), &[]);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));
    assert_eq!(orders(&app), [], "the right-click ordered nothing");
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyZ);
    assert_eq!((interface(&app), constructor(&app, DC)), (menu, ready));

    tap(&mut app, &[], KeyX);
    let cancel = ConstructionCommand::CancelReady;
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], cancel)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!((bank(&app), constructor(&app, DC)), (900 + 75, Constructor::Idle));

    select(&mut app, PK);
    tap(&mut app, &[], KeyR);
    assert_eq!(
        abilities(&app),
        [(P1, vec![PK], sc_sim::Ability::RiotShield { raise: true })]
    );
}

/// `> AwaitingTarget > Shift while placing` (design change 0071): a GDO constructor has one building ready at most,
/// so a Shift-click places it once and ends placement like a plain click; the next Shift-click places nothing.
#[test]
fn a_shift_click_places_once_and_ends_placement() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    click(&mut app, middle(OPEN, (2, 2)), &[ShiftLeft]);
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], placed)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, GENERATOR, OPEN).is_some());
    assert!(!placement_keys(&app));
    click(&mut app, middle(TilePos::new(21, 18), (2, 2)), &[ShiftLeft]);
    assert_eq!(constructions(&app).len(), 2, "one Construct, one Place");
}

/// Plan decision T4: the sim checks a Place again as it applies it. When the game changed after the click (here a
/// unit arrives on the footprint in the same tick, before the Place), the sim refuses it, the Generator stays ready,
/// and the player hears why (`SimEvent::PlacementRefused`); they are in DefaultState, where the valid click left
/// them (`> AwaitingPlacement`), and the ready building's slot starts placement again.
#[test]
fn a_place_the_game_no_longer_allows_leaves_the_building_ready() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let placed = place(OPEN, BuildingOrientation::default());
    assert_eq!(constructions(&app).last(), Some(&(P1, vec![DC], placed)));
    assert_eq!(interface(&app), InterfaceState::DefaultState);

    let arrives = Command::Debug(DebugCommand::Spawn {
        kind: ObjectKind::Peacekeeper,
        owner: Some(P1),
        at: SpawnAt::Point(OPEN.center()),
        orientation: BuildingOrientation::default(),
    });
    let pending = &mut app.world_mut().resource_mut::<PendingCommands>().0;
    pending.insert(
        0,
        PlayerCommand {
            player: P1,
            command: arrives,
        },
    );
    let events = run_collecting(&mut app, 1);
    let refused = events.iter().any(|event| {
        matches!(event, SimEvent::PlacementRefused { constructor, kind, origin, .. }
            if *constructor == ObjectId(DC) && *kind == GENERATOR && *origin == OPEN)
    });
    assert!(refused, "{events:?}");
    assert_eq!(constructor(&app, DC), Constructor::Ready { kind: GENERATOR });
    assert_eq!(structure_at(&app, GENERATOR, OPEN), None);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
}

/// `> ObjectInterfaceState[ExtractionFacility]`: Q builds the plate (75 Space Crystals); while it is built only X
/// (Cancel Construction) shows; once it is ready Q places it and X cancels it. Its ghost is green only on a patch in
/// the build area without a plate; Esc and a right-click return to DefaultState, the plate still ready; a click on the
/// free patch places it there.
#[test]
fn the_extraction_facility_places_its_plate_on_a_free_patch() {
    use ButtonLook::Common;
    use PanelCommand::{CancelConstruction, CancelReady, Construct, PlaceReady};
    let mut app = construction_game(1000);
    select(&mut app, EF);
    let c = PanelButton::Command;
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Common, "EP")]);
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(PLATE_KIND);
    assert_eq!(constructions(&app), [(P1, vec![EF], construct)]);
    assert_eq!(bank(&app), 925);
    assert_eq!(buttons(&mut app), [(X, c(CancelConstruction), Common, "Cancel")]);
    plate_on(&mut app, PLATED_PATCH);
    run_until_ready(&mut app, EF);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(PlaceReady(PLATE_KIND)), Common, "Place"),
            (X, c(CancelReady), Common, "Cancel"),
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(EF), PLATE_KIND)));
    assert_eq!(buttons(&mut app)[0].2, ButtonLook::Pending);

    let bare = TilePos::new(17, 15);
    let cases = [
        (bare, vec![], vec![WholeProblem::NotOnPatch]),
        (PLATED_PATCH, vec![(PLATED_PATCH, TileProblem::Structure)], vec![]),
        (FAR_PATCH, vec![], vec![WholeProblem::OutsideBuildArea]),
        (FREE_PATCH, vec![], vec![]),
    ];
    for (origin, tiles, whole) in cases {
        hover(&mut app, middle(origin, (1, 1)));
        let shown = ghost(&app).expect("the ghost");
        assert_eq!((shown.origin, shown.size), (origin, (1, 1)));
        assert_eq!(
            (&shown.check.tiles, &shown.check.whole),
            (&tiles, &whole),
            "at {origin:?}"
        );
        assert_eq!(shown.tile_is_green(origin), tiles.is_empty() && whole.is_empty());
    }
    assert_eq!(
        ghost_labels(&mut app),
        [(4, "EP".to_string())],
        "no side letters on one tile"
    );
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    let ready = Constructor::Ready { kind: PLATE_KIND };
    tap(&mut app, &[], Escape);
    assert_eq!(
        (interface(&app), constructor(&app, EF)),
        (InterfaceState::DefaultState, ready)
    );
    tap(&mut app, &[], KeyQ);
    right_click(&mut app, middle(FREE_PATCH, (1, 1)), &[]);
    assert_eq!(
        (interface(&app), constructor(&app, EF)),
        (InterfaceState::DefaultState, ready)
    );
    tap(&mut app, &[], KeyQ);
    hover(&mut app, middle(FREE_PATCH, (1, 1)));
    click(&mut app, middle(FREE_PATCH, (1, 1)), &[]);
    let placed = place(FREE_PATCH, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![EF], construct), (P1, vec![EF], placed)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, PLATE_KIND, FREE_PATCH).is_some());
    assert_eq!(constructor(&app, EF), Constructor::Idle);
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Common, "EP")]);
}

/// `factions/gdo/structures.md > GDOBuildArea`: "After placement, the build area expands outward from the placed
/// building by that building's BuildRadiusExtension", and "it shrinks when a building is destroyed". The overlay shows
/// the area as it is each time a building is placed, and follows it while placement goes on; its mesh is built only
/// when the area changed (design change 0093: "one clean shape, which doesn't flicker").
#[test]
fn the_overlay_follows_the_build_area_as_generators_come_and_go() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let first = build_area(&app);
    assert_eq!(overlay(&mut app), (Some(first.clone()), true));
    let built = overlay_builds(&app);
    updates(&mut app, 10);
    assert_eq!(overlay_builds(&app), built, "nothing changed: not built again");
    let east = TilePos::new(36, 12);
    assert!(!first.contains(east));

    hover(&mut app, middle(EDGE, (2, 2)));
    assert!(ghost(&app).is_some_and(|g| g.is_valid()));
    click(&mut app, middle(EDGE, (2, 2)), &[]);
    let (generator, _) = structure_at(&app, GENERATOR, EDGE).expect("placed");
    assert!(!overlay(&mut app).1, "no overlay out of placement");
    let grown = build_area(&app);
    assert!(grown.contains(east));

    construct_and_place(&mut app, KeyQ, GENERATOR);
    assert_eq!(overlay(&mut app), (Some(grown), true));
    assert_eq!(overlay_builds(&app), built + 1);

    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove {
            id: ObjectId(generator),
        }),
    );
    updates(&mut app, 2);
    assert_eq!(build_area(&app), first, "it shrinks back");
    assert_eq!(overlay(&mut app), (Some(first), true));
    assert_eq!(overlay_builds(&app), built + 2);
}

/// Design change 0101 (`core/game_rules.md > End of Game`): once the game is over nobody gives commands. Player 2
/// loses their last structure while player 1 places a Generator: the placement ends with its ghost, its overlay and
/// its keys, the command panel goes, and neither the keys nor clicks construct or place anything.
#[test]
fn once_the_game_is_over_nothing_is_constructed_or_placed() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app));
    let given = constructions(&app).len();

    let remove = DebugCommand::Remove {
        id: ObjectId(ENEMY_GENERATOR),
    };
    submit(&mut app, P1, Command::Debug(remove));
    updates(&mut app, 2);
    let outcome = app.world().resource::<SimRes>().outcome().cloned();
    assert!(outcome.is_some_and(|o| o.is_winner(P1)));
    assert_eq!(interface(&app), InterfaceState::DefaultState, "placement ends");
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1, "no overlay");
    assert!(!placement_keys(&app), "R and F are no placement keys");
    assert!(!ghost_is_cursor(&app), "the pointer is back");
    assert_eq!(buttons(&mut app), [], "no command panel");

    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyQ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    click(&mut app, middle(OPEN, (2, 2)), &[]);
    tap(&mut app, &[], KeyX);
    assert_eq!(constructions(&app).len(), given);
    assert_eq!(constructor(&app, DC), Constructor::Ready { kind: GENERATOR });
    assert_eq!(structure_at(&app, GENERATOR, OPEN), None);
}

/// Design change 0101 (`core/game_rules.md > On Defeat`): a player defeated while the game goes on gives no commands
/// either (slice M2-J2's check of the placement keys, which the game reports outside the command input). Player 1
/// loses their structures while placing a Generator, with players 2 and 3 still in the game: the placement ends with
/// its ghost, its overlay, its keys and its pointer, and R, F and the build menu's keys do nothing.
#[test]
fn a_player_defeated_while_placing_places_nothing() {
    let mut scenario = construction_scenario();
    scenario.map.slots.push(sc_sim::Slot::new(TilePos::new(56, 4)));
    scenario.map.spawns.push(Spawn::new(
        GENERATOR,
        Some(SlotId(3)),
        SpawnAt::Tile(TilePos::new(56, 2)),
    ));
    scenario.setup = sc_sim::GameSetup::default_for(&scenario.map);
    let mut app = game_of(scenario);
    set_crystals(&mut app, 1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    hover(&mut app, middle(OPEN, (2, 2)));
    assert!(ghost(&app).is_some() && overlay(&mut app).1 && placement_keys(&app) && ghost_is_cursor(&app));
    let given = constructions(&app).len();

    for id in [DC, EF] {
        submit(&mut app, P1, Command::Debug(DebugCommand::Remove { id: ObjectId(id) }));
    }
    updates(&mut app, 2);
    let sim = app.world().resource::<SimRes>();
    assert!(sim.player(P1).unwrap().is_defeated() && sim.outcome().is_none());
    assert_eq!(interface(&app), InterfaceState::DefaultState, "placement ends");
    assert_eq!(ghost(&app), None);
    assert!(!overlay(&mut app).1, "no overlay");
    assert!(!placement_keys(&app), "R and F are no placement keys");
    assert!(!ghost_is_cursor(&app), "the pointer is back");
    assert_eq!(buttons(&mut app), [], "no command panel");
    for key in [KeyR, KeyF, KeyQ, KeyQ] {
        tap(&mut app, &[], key);
        assert!(!placement_keys(&app));
    }
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(constructions(&app).len(), given);
}
