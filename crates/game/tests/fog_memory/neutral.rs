//! (M4-H) A neutral expansion (`factions/syndicate/tunnels.md > Losing Tunnels`: "Neutral expansions are **no longer
//! hidden** (visible to everyone, no detection needed)"; "They **slowly lose HP** until destroyed"): a Hideout whose
//! Tunnel fell is drawn for every player who sees its tiles, in the neutral colour, with an HP bar that falls as it
//! decays; out of sight, its record shows on the Explored tiles, as any structure's. (M5-Y) A Laser Turret alike
//! (design change 0360), drawn lowered, as is its record.

use sc_sim::testing::ScenarioBuilder;
use space_crystals::combat_effects::hp_bars;
use space_crystals::object_render::{TurretBarrel, player_color};

use super::*;

/// The objects of [`lost_world`], by id.
const TUNNEL: u32 = 1;
const HIDEOUT: u32 = 2;
const WATCHER: u32 = 4;
/// The Laser Turret of [`lost_world_with`], in the Hideout's place.
const TURRET: u32 = 2;

/// A flat 40 × 14 map: player 2 plays the Syndicate, with its Tunnel 1 on tiles (4, 4) to (7, 7), its Hideout 2 on
/// (8, 4) to (9, 5), inside the Tunnel's area, and a second Tunnel 3 far east, which keeps it in the game; player 1's
/// scout (a Peacekeeper, 4) stands on (12, 5), from where it sees the Hideout's tiles. Auto-targeting off.
fn lost_world() -> App {
    lost_world_with(ObjectKind::Hideout, TilePos::new(8, 4))
}

/// [`lost_world`] with an expansion of `kind` at `at`, in the Tunnel's area, in the Hideout's place (object 2).
fn lost_world_with(kind: ObjectKind, at: TilePos) -> App {
    let map = vec![".".repeat(40); 14].join("\n");
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let scenario = ScenarioBuilder::new(&map)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Tunnel, Some(P2), tile(4, 4))
        .spawn(kind, Some(P2), SpawnAt::Tile(at))
        .spawn(ObjectKind::Tunnel, Some(P2), tile(30, 4))
        .build();
    let mut app = headless(AppConfig::new(scenario));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 12, 5);
    app.update();
    app
}

/// The RGB of the first vertex of `id`'s visual's mesh: an underground structure's rim, in its owner's colour.
fn rim(app: &App, id: u32) -> [f32; 3] {
    use bevy::mesh::VertexAttributeValues;
    let world = app.world();
    let entity = world.resource::<ObjectVisuals>().by_id[&ObjectId(id)];
    let handle = &world.get::<Mesh2d>(entity).expect("a mesh").0;
    let mesh = world.resource::<Assets<Mesh>>().get(handle).expect("its mesh");
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
        panic!("a mesh of vertex colours");
    };
    let [r, g, b, _] = colors[0];
    [r, g, b]
}

/// The RGB of `owner`'s colour, as the meshes hold it.
fn rgb(owner: Option<PlayerId>) -> [f32; 3] {
    let [r, g, b, _] = player_color(owner).to_linear().to_f32_array();
    [r, g, b]
}

/// The fractions of the HP bars player 1 sees.
fn bars(app: &App) -> Vec<f32> {
    let sim = app.world().resource::<SimRes>();
    let sight = Sight::new(sim, P1, true);
    hp_bars(sim, &sight, 1.0, &space_crystals::tuning::ClientTuning::BUILTIN, |_| {
        false
    })
    .iter()
    .map(|bar| bar.fraction)
    .collect()
}

/// Player 1's scout sees the Hideout's tiles but not the Hideout, hidden while player 2 owns it. Its Tunnel removed,
/// the Hideout is neutral: player 1 sees it, in the neutral colour, with an HP bar that falls by a quarter of an HP a
/// tick (4 a second, of 400). The scout gone, its record shows instead.
#[test]
fn a_neutral_hideout_shows_to_everyone_in_the_neutral_colour_its_hp_bar_falling() {
    let mut app = lost_world();
    assert!(!drawn(&app, HIDEOUT), "hidden from player 1");
    assert_eq!(rim(&app, HIDEOUT), rgb(Some(P2)), "drawn for its owner in its colour");

    debug(&mut app, DebugCommand::Remove { id: ObjectId(TUNNEL) });
    app.update();
    let fell = tick(&app);
    let owner = app
        .world()
        .resource::<SimRes>()
        .object(ObjectId(HIDEOUT))
        .unwrap()
        .owner;
    assert_eq!(owner, None);
    assert!(drawn(&app, HIDEOUT), "player 1 sees it now");
    assert_eq!(rim(&app, HIDEOUT), rgb(None), "in the neutral colour");
    assert_eq!(bars(&app), [399.75 / 400.0], "a tick of decay");
    run_until(&mut app, fell + 16);
    assert_eq!(bars(&app), [395.75 / 400.0], "4 HP a second");

    move_to(&mut app, ObjectId(WATCHER), 20, 12);
    assert!(!drawn(&app, HIDEOUT));
    assert_eq!(remembered(&app), [HIDEOUT], "its record shows on the Explored tiles");
    assert_eq!(bars(&app), [] as [f32; 0]);
}

/// The mesh of `id`'s visual: its live one (`ObjectVisuals`), or its record's (`RememberedVisuals`).
fn mesh_of(app: &App, id: u32, remembered: bool) -> Handle<Mesh> {
    let world = app.world();
    let entity = match remembered {
        false => world.resource::<ObjectVisuals>().by_id[&ObjectId(id)],
        true => world.resource::<RememberedVisuals>().by_id[&ObjectId(id)],
    };
    world.get::<Mesh2d>(entity).expect("a mesh").0.clone()
}

/// The turrets' barrels drawn.
fn barrels(app: &mut App) -> usize {
    let mut barrels = app.world_mut().query::<&TurretBarrel>();
    barrels.iter(app.world()).count()
}

/// (M5-Y) Design change 0360 (`factions/syndicate/structures/laser_turret.md > When its Tunnel is lost`: "it becomes
/// **neutral**, **visible to everyone**, keeps its **high underground armor** ..., and **slowly decays** (1% of MaxHP
/// per second, placeholder)"): [`lost_world_with`] player 2's Laser Turret on (9, 5), in the Tunnel's area, lowered
/// (auto-targeting is off: it picks no target) and hidden from player 1, whose scout sees its tile. The Tunnel
/// removed, player 1 sees the turret, an underground outline in the neutral colour with no barrel, and an HP bar that
/// falls by 0.1875 HP a tick (3 a second, of 300). The scout gone, its record shows, drawn as the live turret was,
/// lowered: the same mesh (`sc_sim::popup::remembered_lowered`).
#[test]
fn a_neutral_turret_shows_to_everyone_lowered_and_is_remembered_so() {
    let mut app = lost_world_with(ObjectKind::LaserTurret, TilePos::new(9, 5));
    let popup = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        let o = sim.object(ObjectId(TURRET)).expect("the turret");
        (o.owner, o.popup)
    };
    assert_eq!(popup(&app), (Some(P2), Some(sc_sim::popup::PopUp::LOWERED)));
    assert!(!drawn(&app, TURRET), "hidden from player 1");
    assert_eq!(rim(&app, TURRET), rgb(Some(P2)), "an outline, in its owner's colour");

    debug(&mut app, DebugCommand::Remove { id: ObjectId(TUNNEL) });
    app.update();
    let fell = tick(&app);
    assert_eq!(popup(&app), (None, Some(sc_sim::popup::PopUp::LOWERED)));
    assert!(drawn(&app, TURRET), "player 1 sees it now");
    assert_eq!(rim(&app, TURRET), rgb(None), "an outline in the neutral colour");
    assert_eq!(barrels(&mut app), 0, "no barrel: it is down");
    assert_eq!(bars(&app), [299.8125 / 300.0], "a tick of decay");
    run_until(&mut app, fell + 16);
    assert_eq!(bars(&app), [296.8125 / 300.0], "3 HP a second");
    let live = mesh_of(&app, TURRET, false);

    move_to(&mut app, ObjectId(WATCHER), 20, 12);
    assert!(!drawn(&app, TURRET));
    assert_eq!(remembered(&app), [TURRET], "its record shows on the Explored tiles");
    assert_eq!(mesh_of(&app, TURRET, true), live, "drawn lowered, as it was seen");
}
