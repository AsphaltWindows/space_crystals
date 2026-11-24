//! The Colonists' panels and placements (slice M3-J1, design change 0124), on a Colonists field of their own
//! ([`colonists_game`]): a Drone's Deploy, its ghost and the lingering ghosts of the Deploys ordered (`deploy`); a
//! Beacon's Warp In menus and its ghost on the Beacon, the Cancel of whatever transforms, and what pressing a
//! greyed-out button says (`warp_in`); the group of a type's unfinished buildings, apart from its finished ones
//! (design change 0185, `unfinished`).

mod deploy;
mod unfinished;
mod warp_in;

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Command, ConstructionCommand, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, Placement, PlayerId,
    SpawnAt, TilePos,
};
use space_crystals::command_ui::cursor::PointerLook;
use space_crystals::command_ui::feedback::TargetFeedback;
use space_crystals::command_ui::{ButtonLook, PanelButton, Placing};
use space_crystals::coords::drawn_position;
use space_crystals::hud::command_panel::CommandButton;
use space_crystals::hud::layout::CommandGrid;
use space_crystals::input;
use space_crystals::placement::lingering::LingeringMesh;
use space_crystals::placement::{Ghost, LingeringGhosts, PlacementGhost};
use space_crystals::platform::SurfaceSize;

use super::*;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The top-left tile of player 1's Egress Node 1, on tiles (12, 9) to (13, 10) in the world view; player 2's, 2, is
/// far off screen: what keeps each of them in the game.
const OWN_NODE_AT: TilePos = TilePos::new(12, 9);

/// A game of player 1, the Colonists, against player 2, the Colonists too, on a flat 64 × 48 map: player 1's Egress
/// Node at [`OWN_NODE_AT`], player 2's far off screen, then what `add` spawns (ids from 3); `crystals` Space Crystals in
/// player 1's bank. Set up as [`game`] does: no fog, no auto-targeting, the camera at [`VIEW_CENTER`] (tiles x 11 to
/// 39, y 8.4 to 19.75 in the world view).
fn colonists_game(crystals: u32, add: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> App {
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Colonists)
        .faction(P2, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(P1), SpawnAt::Tile(OWN_NODE_AT))
        .spawn(ObjectKind::EgressNode, Some(P2), SpawnAt::Tile(TilePos::new(56, 40)));
    let mut app = game_of(add(builder).build());
    set_crystals(&mut app, crystals);
    app
}

/// Object `id`, as the sim has it.
fn object(app: &App, id: u32) -> &Object {
    app.world()
        .resource::<SimRes>()
        .object(ObjectId(id))
        .expect("in the game")
}

/// Unit `id`'s current order, if it has one.
fn current(app: &App, id: u32) -> Option<OrderKind> {
    let orders = object(app, id).orders.as_ref()?;
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The middle of a `w × h` footprint whose top-left tile is `origin`: where the cursor goes to put it there.
fn middle(origin: TilePos, (w, h): (u32, u32)) -> Vec2 {
    origin.corner() + Vec2::new(w as f32, h as f32) * 32.0
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

/// Points the cursor at the command panel's button of `slot`, over the HUD, and lets a frame pass.
fn hover_panel(app: &mut App, slot: CommandSlot) {
    let surface = app.world().resource::<SurfaceSize>().logical();
    input(app).cursor = Some(CommandGrid::on(surface).slot_rect(slot).center());
    app.update();
}

/// The placement ghost as it shows now.
fn ghost(app: &App) -> Option<Ghost> {
    app.world().resource::<PlacementGhost>().shown.clone()
}

/// The footprint the ghost shows: its top-left tile and size.
fn ghost_at(app: &App) -> Option<(TilePos, (u32, u32))> {
    ghost(app).map(|g| (g.origin, g.size))
}

/// What the local player places now.
fn placing(app: &App) -> Option<Placing> {
    interface(app).placing()
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

/// The lingering ghosts shown: their footprints, while their mesh shows.
fn lingering(app: &mut App) -> Vec<Placement> {
    let shown = app.world().resource::<LingeringGhosts>().shown.clone();
    let mut query = app.world_mut().query_filtered::<&Visibility, With<LingeringMesh>>();
    let visible = query.single(app.world()).is_ok_and(|v| *v != Visibility::Hidden);
    assert_eq!(
        visible,
        !shown.is_empty(),
        "the mesh shows exactly while there are ghosts"
    );
    shown
}

/// `player`'s pending Deploys, as the sim has them: Drone and site.
fn pending(app: &App, player: PlayerId) -> Vec<(u32, Placement)> {
    let sim = app.world().resource::<SimRes>();
    sim.pending_deploys(player)
        .into_iter()
        .map(|(drone, site)| (drone.0, site))
        .collect()
}

/// Every construction command given so far, as the sim received them (like [`orders`]): player, objects, command.
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

/// Runs until `done` holds, at most `ticks` frames; panics if it never does.
fn run_until(app: &mut App, ticks: usize, what: &str, done: impl Fn(&App) -> bool) {
    for _ in 0..ticks {
        if done(app) {
            return;
        }
        app.update();
    }
    assert!(done(app), "{what}: not within {ticks} ticks");
}

/// Removes object `id` (a debug command), two frames later.
fn remove(app: &mut App, id: u32) {
    let remove = DebugCommand::Remove { id: ObjectId(id) };
    submit(app, P1, Command::Debug(remove));
    updates(app, 2);
}
