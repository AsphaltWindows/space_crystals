//! The production field of these tests (slice M1-P on): player 1's three Barracks beside the mixed field of
//! [`scenario`], and what reads and drives their production. `main.rs` takes all of it in, so every module of the
//! tests has it as it has the rest.

use super::*;

/// Player 1's Barracks of [`production_scenario`], by id.
pub(crate) const BARRACKS: [u32; 3] = [15, 16, 17];

/// The centre of Barracks `id`'s footprint ([`BARRACKS`]), where it is drawn (the map is flat).
pub(crate) fn barracks_at(id: u32) -> Vec2 {
    match id {
        15 => Vec2::new(1120.0, 960.0),  // tiles (16, 14) to (18, 15)
        16 => Vec2::new(1376.0, 960.0),  // tiles (20, 14) to (22, 15)
        17 => Vec2::new(1120.0, 1152.0), // tiles (16, 17) to (18, 18)
        _ => panic!("no Barracks {id}"),
    }
}

/// [`scenario`] with player 1's three Barracks ([`BARRACKS`], their exit sides facing east), then `extra` more of
/// player 1's Peacekeepers far off screen (ids from 18), which take up its Unit Control.
pub(crate) fn production_scenario(extra: usize) -> Scenario {
    let mut scenario = scenario();
    for (x, y) in [(16, 14), (20, 14), (16, 17)] {
        let at = SpawnAt::Tile(TilePos::new(x, y));
        scenario
            .map
            .spawns
            .push(Spawn::new(ObjectKind::Barracks, Some(SlotId(1)), at));
    }
    for i in 0..extra {
        let at = Vec2::new(200.0 + 40.0 * (i % 25) as f32, 1700.0 + 40.0 * (i / 25) as f32);
        scenario
            .map
            .spawns
            .push(Spawn::new(ObjectKind::Peacekeeper, Some(SlotId(1)), SpawnAt::Point(at)));
    }
    assert_eq!(scenario.validate(), vec![]);
    scenario
}

/// A game of [`production_scenario`], as [`game`] sets it up, with `crystals` Space Crystals in player 1's bank.
pub(crate) fn production_game(extra: usize, crystals: u32) -> App {
    let mut app = game_of(production_scenario(extra));
    set_crystals(&mut app, crystals);
    app
}

/// Player 1's bank of Space Crystals becomes `crystals` (a debug command), a frame later.
pub(crate) fn set_crystals(app: &mut App, crystals: u32) {
    let player = PlayerId(1);
    let set = DebugCommand::SetResources {
        player,
        crystals,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(app, player, Command::Debug(set));
    app.update();
}

/// Player 1's Space Crystals.
pub(crate) fn bank(app: &App) -> u32 {
    let sim = app.world().resource::<SimRes>();
    sim.player(PlayerId(1)).expect("in the game").crystals
}

/// Selects player 1's Barracks `ids`: a click on the first, and a Shift + box around each of the others, which adds
/// it (`systems/controls/selection.md > BoxSelection > Shift + box (add)`; a box selects one structure at a
/// time).
pub(crate) fn select_barracks(app: &mut App, ids: &[u32]) {
    click(app, barracks_at(ids[0]), &[]);
    for &id in &ids[1..] {
        add_box(app, barracks_at(id));
    }
    assert_eq!(selected(app), ids);
}

/// A Shift + box around the view point `around`, which adds what it selects to the selection.
pub(crate) fn add_box(app: &mut App, around: Vec2) {
    press_keys(app, &[KeyCode::ShiftLeft]);
    drag(app, around - 40.0, around + 40.0);
    release_keys(app, &[KeyCode::ShiftLeft]);
}

/// Every production command given so far, as the sim received them (like [`orders`]): player, producers, command.
pub(crate) fn productions(app: &App) -> Vec<(PlayerId, Vec<u32>, ProductionCommand)> {
    let recorded = app.world().resource::<RecordedInputs>();
    let pending = app.world().resource::<PendingCommands>();
    let commands = recorded.ticks.iter().flat_map(|t| t.commands.iter()).chain(&pending.0);
    commands
        .filter_map(|c| match &c.command {
            Command::Production { objects, command } => {
                Some((c.player, objects.iter().map(|id| id.0).collect(), *command))
            }
            Command::Order { .. }
            | Command::Ability { .. }
            | Command::Construction { .. }
            | Command::Tunnel { .. }
            | Command::Holding { .. }
            | Command::Cast { .. }
            | Command::Debug(_) => None,
        })
        .collect()
}

/// Producer `id`'s production state, as the sim has it.
pub(crate) fn production_of(app: &App, id: u32) -> sc_sim::Production {
    let sim = app.world().resource::<SimRes>();
    let object = sim.object(ObjectId(id)).expect("in the game");
    object.production.clone().expect("a producer")
}
