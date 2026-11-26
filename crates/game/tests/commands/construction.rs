//! Construction from the command panel (slice M2-H2, plan decision T10): the Deployment Center's build menu
//! (`factions/gdo/structures/deployment_center.md > ObjectInterfaceState[DeploymentCenter]`, design change 0104) and
//! the Extraction Facility's plate (`factions/gdo/economy.md > ExtractionFacility >
//! ObjectInterfaceState[ExtractionFacility]`): Construct, Cancel Construction and Cancel Ready Building, then
//! AwaitingPlacement, whose ghost follows the cursor green or red tile by tile (design change 0093) with the letters
//! of its sides (0079), turns with R and F (risk R11), places with a left-click (once with Shift, 0071) and goes back
//! with Esc, a right-click or Back, while the overlay shows the build area as it is. Nothing of it once the game is
//! over (0101). It plays on a field of its own ([`construction_scenario`]).
//!
//! By topic: the build menu, Construct and Cancel Construction (`build_menu`); the ghost, its tiles, its letters
//! turned by R and F, and the overlay (`ghost_and_overlay`); a ready building placed, or not, and placement left
//! (`ready_building`); the Extraction Facility's plate (`extraction_facility`); and nothing once the game is over
//! or the player defeated (`game_over`).

mod build_menu;
mod extraction_facility;
mod game_over;
mod ghost_and_overlay;
mod ready_building;

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
use KeyCode::{ControlLeft, Digit1, Escape, KeyD, KeyE, KeyF, KeyQ, KeyR, KeyW, KeyX, KeyZ, ShiftLeft, Tab};

const P1: PlayerId = PlayerId(1);
const GENERATOR: ObjectKind = ObjectKind::Generator;
const BARRACKS_KIND: ObjectKind = ObjectKind::Barracks;
const EXTRACTION_FACILITY: ObjectKind = ObjectKind::ExtractionFacility;
const PLATE_KIND: ObjectKind = ObjectKind::ExtractionPlate;
/// BuildMenu[Basic]'s D (slice M5-N9): greyed in these games, which have no Barracks, its TechPrerequisite.
const CAREER_CAMPUS: ObjectKind = ObjectKind::CareerCampus;

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
            Command::Order { .. }
            | Command::Ability { .. }
            | Command::Production { .. }
            | Command::Tunnel { .. }
            | Command::Holding { .. }
            | Command::Cast { .. }
            | Command::Debug(_) => None,
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

fn d() -> CommandSlot {
    slot(1, 2)
}

const X: CommandSlot = CommandSlot::CANCEL_PRODUCTION;
