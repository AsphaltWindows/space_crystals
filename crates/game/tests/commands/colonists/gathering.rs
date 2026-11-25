//! The Colonists' gathering UI (slice M3-J2) on a field of its own ([`gathering_game`]): a Drone's right-click
//! (`factions/colonists/units.md > Drone > Right-Click Resolution`), each selected unit resolving its own, with the
//! pointer it shows (`systems/controls/selection_and_hud.md > PointerDisplayType > Resolution Rules`); a load carried,
//! on the map and in the InfoPanel; a portal's rally point on a patch; the portals' units not in the game yet (design
//! change 0124). A Drone's S, Set Drop-off: `drop_off`; the Egress Node's F, Select Returning Drones: `returning`. A
//! right-click on a patch remembered in the fog (design change 0139) is `remembered`'s
//! (`drones_right_clicked_on_a_greyed_patch_walk_there_and_mine`).

mod drop_off;
mod returning;

use sc_sim::orders::state::{Behaviour, Gather, GatherPhase};
use space_crystals::command_ui::indicators::{Indicator, MOVE_COLOR, RallyIndicator, rally_indicators};
use space_crystals::command_ui::interface::templates::Planned;
use space_crystals::command_ui::{PanelCommand, PointerDisplay};
use space_crystals::fog::Sight;
use space_crystals::hud::info_panel::InfoText;
use space_crystals::object_render::LoadMarker;

use super::*;
use crate::indicators::drawn_vertices;

// The objects of [`gathering_game`], by id, beside the Egress Nodes of [`colonists_game`]: player 1's, 1, and player
// 2's, 2, far off screen.
/// Player 1's Egress Nodes: 1, and 8, its second.
const NODE: u32 = 1;
const NODE_2: u32 = 8;
/// A patch by node 1, and one by node 2, more than a Drone's search radius (6 grid units) apart: a Drone sent to
/// patch 3 while another mines it waits there.
const PATCH: u32 = 3;
const FAR_PATCH: u32 = 10;
/// Player 1's Drones: A, B and C under patch 3, D by node 2.
const A: u32 = 4;
const B: u32 = 5;
const C: u32 = 6;
const D: u32 = 9;
/// Player 1's Sabre Legionnaire, to the left of the Drones.
const SABRE: u32 = 7;
/// Player 2's Sabre Legionnaire and Robo Portal, in the sight of player 1's scout.
const ENEMY_SABRE: u32 = 11;
const ENEMY_PORTAL: u32 = 12;
/// Player 1's portals, under the Drones.
const ROBO_PORTAL: u32 = 13;
const BIO_PORTAL: u32 = 14;
/// Player 1's Drone off screen, above the enemies: what lets the sim take orders on them.
const SCOUT: u32 = 15;
/// Bare ground in the world view, away from every object.
const GROUND: Vec2 = Vec2::new(1500.0, 700.0);

/// Where object `id` of [`gathering_game`] is before anything moves, and is drawn (the map is flat): a unit's point,
/// the centre of a structure's or a patch's footprint.
fn home(id: u32) -> Vec2 {
    let tile = |x, y| TilePos::new(x, y).center();
    match id {
        NODE => Vec2::new(832.0, 640.0), // tiles (12, 9) to (13, 10)
        PATCH => tile(18, 9),
        A => tile(14, 12),
        B => tile(16, 12),
        C => tile(18, 12),
        SABRE => tile(12, 12),
        NODE_2 => Vec2::new(1664.0, 1024.0), // tiles (25, 15) to (26, 16)
        D => tile(27, 17),
        FAR_PATCH => tile(28, 15),
        ENEMY_SABRE => tile(26, 10),
        ENEMY_PORTAL => Vec2::new(2112.0, 704.0), // tiles (31, 9) to (34, 12)
        ROBO_PORTAL => Vec2::new(1152.0, 1088.0), // tiles (16, 15) to (19, 18)
        BIO_PORTAL => Vec2::new(1440.0, 1056.0),  // tiles (21, 15) to (23, 17)
        SCOUT => Vec2::new(1800.0, 500.0),
        _ => panic!("no object {id}"),
    }
}

/// A [`colonists_game`] with the objects above, no Space Crystals in the bank.
fn gathering_game() -> App {
    use ObjectKind::{BioPortal, CrystalPatch, Drone, EgressNode, RoboPortal, SabreLegionnaire};
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let point = |id| SpawnAt::Point(home(id));
    let spawns = [
        (PATCH, CrystalPatch, None, tile(18, 9)),
        (A, Drone, Some(P1), point(A)),
        (B, Drone, Some(P1), point(B)),
        (C, Drone, Some(P1), point(C)),
        (SABRE, SabreLegionnaire, Some(P1), point(SABRE)),
        (NODE_2, EgressNode, Some(P1), tile(25, 15)),
        (D, Drone, Some(P1), point(D)),
        (FAR_PATCH, CrystalPatch, None, tile(28, 15)),
        (ENEMY_SABRE, SabreLegionnaire, Some(P2), point(ENEMY_SABRE)),
        (ENEMY_PORTAL, RoboPortal, Some(P2), tile(31, 9)),
        (ROBO_PORTAL, RoboPortal, Some(P1), tile(16, 15)),
        (BIO_PORTAL, BioPortal, Some(P1), tile(21, 15)),
        (SCOUT, Drone, Some(P1), point(SCOUT)),
    ];
    let app = colonists_game(0, |builder| {
        (spawns.iter()).fold(builder, |builder, &(_, kind, owner, at)| builder.spawn(kind, owner, at))
    });
    for (id, kind, owner, _) in spawns {
        let o = object(&app, id);
        assert_eq!((o.kind, o.owner), (kind, owner), "object {id}");
    }
    app
}

/// Drone `id`'s Gather, while it carries one out.
fn gather(app: &App, id: u32) -> Option<Gather> {
    let active = object(app, id).orders.as_ref()?.current.as_ref()?;
    match active.state {
        Behaviour::Gather(gather) => Some(gather),
        _ => None,
    }
}

/// Which phases of a Gather some check is about.
type Phases = fn(GatherPhase) -> bool;

/// Whether Drone `id` carries out a Gather in one of the `phases`.
fn in_phase(app: &App, id: u32, phases: Phases) -> bool {
    gather(app, id).is_some_and(|gather| phases(gather.phase))
}

/// The node Drone `id` unloads at, while it does.
fn unloading_at(app: &App, id: u32) -> Option<u32> {
    let gather = gather(app, id).filter(|g| matches!(g.phase, GatherPhase::DroppingOff { .. }))?;
    gather.node.map(|node| node.0)
}

/// The Space Crystals Drone `id` carries.
fn load(app: &App, id: u32) -> u32 {
    object(app, id).worker.map_or(0, |worker| worker.load)
}

/// Drone `id`'s AssignedDropOff.
fn drop_off(app: &App, id: u32) -> Option<u32> {
    object(app, id)
        .worker
        .and_then(|worker| worker.drop_off)
        .map(|node| node.0)
}

/// Whether the load marker of Drone `id` shows.
fn load_shown(app: &mut App, id: u32) -> bool {
    let mut markers = app.world_mut().query::<(&LoadMarker, &Visibility)>();
    let shown: Vec<bool> = (markers.iter(app.world()))
        .filter(|(marker, _)| marker.0 == ObjectId(id))
        .map(|(_, visibility)| *visibility != Visibility::Hidden)
        .collect();
    assert_eq!(shown.len(), 1, "Drone {id} has one load marker");
    shown[0]
}

/// The InfoPanel's lines, while it shows them.
fn info_lines(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    (texts.iter(app.world()))
        .find(|(which, _, node)| **which == InfoText::Body && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
}

/// Whether the command panel shows the Egress Node's F.
fn shows_f(app: &mut App) -> bool {
    let f = PanelButton::Command(PanelCommand::SelectReturningDrones);
    buttons(app).iter().any(|&(_, button, ..)| button == f)
}

/// What the feedback line says after `press`: its message, if `press` gave a new one.
fn says(app: &mut App, press: impl FnOnce(&mut App)) -> Option<&'static str> {
    let before = app.world().resource::<TargetFeedback>().since;
    press(app);
    let feedback = app.world().resource::<TargetFeedback>();
    feedback
        .since
        .filter(|&since| Some(since) != before)
        .map(|_| feedback.text)
}

/// A Move to object `id`: "Move to that object's location", or Follow for a unit.
fn to(id: u32) -> OrderKind {
    OrderKind::Move(Target::Object(ObjectId(id)))
}

/// `> Right-Click Resolution`, "issued to all selected Drones": a patch gives a Gather, under the GatherResources
/// pointer (`> PointerDisplayType > Resolution Rules`: "Cursor over resource (resource gatherer selected)"); an own
/// node to empty Drones (design change 0140: there is no empty Drop Off), an enemy structure and an enemy unit "Move
/// to that object's location", the unit followed; the ground a Move. The last four under the Move pointer.
#[test]
fn drones_right_clicked_gather_move_or_follow() {
    use PointerDisplay::{GatherResources, Move};
    let mut app = gathering_game();
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    let targets = [home(PATCH), home(NODE), home(ENEMY_SABRE), home(ENEMY_PORTAL), GROUND];
    let pointers = targets.map(|at| pointer_over(&mut app, at));
    assert_eq!(pointers, [GatherResources, Move, Move, Move, Move].map(Some));
    for at in targets {
        right_click(&mut app, at, &[]);
    }
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 5, "{given_orders:?}");
    assert_eq!(
        given_orders[..4],
        [
            given(&[A, B], OrderKind::Gather(ObjectId(PATCH)), false),
            given(&[A, B], to(NODE), false),
            given(&[A, B], to(ENEMY_SABRE), false),
            given(&[A, B], to(ENEMY_PORTAL), false),
        ]
    );
    let ground = &given_orders[4];
    assert!(
        ground.units == [A, B] && near(point_of(ground.kind), GROUND),
        "{ground:?}"
    );
}

/// Each selected unit resolves the right-click for itself (`> Right-Click Resolution`, design changes 0062 and 0063,
/// as for the M2 units): with two Drones and a Sabre Legionnaire selected, on a patch the Drones gather and the Sabre
/// moves there; on an enemy unit or structure the Sabre attacks it and the Drones follow the unit or go to the
/// structure; on an own node (the Drones empty) and on the ground all three move. The pointer shows Mixed where they
/// differ (`> Mixed Right-Click Preview`).
#[test]
fn drones_and_a_sabre_each_resolve_their_own_right_click() {
    use PointerDisplay::{Mixed, Move};
    let mut app = gathering_game();
    select_around(&mut app, &[home(SABRE), home(A), home(B)], &[A, B, SABRE]);
    let targets = [home(PATCH), home(NODE), home(ENEMY_SABRE), home(ENEMY_PORTAL), GROUND];
    let pointers = targets.map(|at| pointer_over(&mut app, at));
    assert_eq!(pointers, [Mixed, Move, Mixed, Mixed, Move].map(Some));
    for at in targets {
        right_click(&mut app, at, &[]);
    }
    let attack = |id| OrderKind::Attack(ObjectId(id));
    let given_orders = orders(&app);
    assert_eq!(given_orders.len(), 8, "{given_orders:?}");
    assert_eq!(
        given_orders[..7],
        [
            given(&[A, B], OrderKind::Gather(ObjectId(PATCH)), false),
            given(&[SABRE], to(PATCH), false),
            given(&[A, B, SABRE], to(NODE), false),
            given(&[A, B], to(ENEMY_SABRE), false),
            given(&[SABRE], attack(ENEMY_SABRE), false),
            given(&[A, B], to(ENEMY_PORTAL), false),
            given(&[SABRE], attack(ENEMY_PORTAL), false),
        ]
    );
    let ground = &given_orders[7];
    assert!(
        ground.units == [A, B, SABRE] && near(point_of(ground.kind), GROUND),
        "{ground:?}"
    );
}

/// `> Right-Click Resolution`: "Own Egress Node while carrying → Drop Off", and an empty Drone "gets a Move there"
/// (design change 0140). With a Drone carrying its load back and an empty one selected, the pointer over the node is
/// Mixed (ReturnResources for the one, "Cursor over drop-off point (resource gatherer carrying resources)", Move for
/// the other), and one right-click gives each its own. Once the load is in the bank, a right-click there moves both,
/// under the Move pointer.
#[test]
fn a_right_click_on_an_own_node_drops_a_load_off_and_moves_an_empty_drone() {
    let mut app = gathering_game();
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    right_click(&mut app, home(PATCH), &[]);
    run_until(&mut app, 300, "a Drone carries a load", |app| {
        load(app, A) + load(app, B) > 0
    });
    let carrier = if load(&app, A) > 0 { A } else { B };
    assert_eq!(pointer_over(&mut app, home(NODE)), Some(PointerDisplay::Mixed));
    let from = orders(&app).len();
    right_click(&mut app, home(NODE), &[]);
    let order = |id| {
        if id == carrier {
            OrderKind::DropOff(ObjectId(NODE))
        } else {
            to(NODE)
        }
    };
    assert_eq!(
        orders(&app)[from..],
        [given(&[A], order(A), false), given(&[B], order(B), false)]
    );

    run_until(&mut app, 300, "the load is delivered", |app| bank(app) == 8);
    assert_eq!(load(&app, A) + load(&app, B), 0);
    assert_eq!(pointer_over(&mut app, home(NODE)), Some(PointerDisplay::Move));
    let from = orders(&app).len();
    right_click(&mut app, home(NODE), &[]);
    assert_eq!(orders(&app)[from..], [given(&[A, B], to(NODE), false)]);
}

/// A load carried (slice M3-J2's placeholders): a Drone that carries one shows a crystal shard on its body
/// ([`LoadMarker`]), and its InfoPanel, which its owner sees, says what it carries and where it takes it (`> Drone >
/// Assigned Drop-off`): "Carrying 8 Space Crystals", "Drop-off: Egress Node". Neither shows while it is empty: before
/// its first load, its drop-off not assigned yet, and once the load is delivered.
#[test]
fn a_drone_shows_the_load_it_carries() {
    let mut app = gathering_game();
    click(&mut app, home(A), &[]);
    assert_eq!(selected(&app), [A]);
    let idle = "Player 1\nHP 20 / 20\nShields 10 / 10";
    assert_eq!(info_lines(&mut app).as_deref(), Some(idle));
    assert!(!load_shown(&mut app, A));

    right_click(&mut app, home(PATCH), &[]);
    run_until(&mut app, 300, "A carries a load", |app| load(app, A) > 0);
    app.update();
    assert_eq!(load(&app, A), 8);
    assert!(load_shown(&mut app, A) && !load_shown(&mut app, B));
    assert_eq!(
        info_lines(&mut app),
        Some(format!("{idle}\nCarrying 8 Space Crystals\nDrop-off: Egress Node"))
    );

    run_until(&mut app, 300, "A delivers its load", |app| load(app, A) == 0);
    app.update();
    assert!(!load_shown(&mut app, A));
    assert_eq!(info_lines(&mut app), Some(format!("{idle}\nDrop-off: Egress Node")));
}

/// `> CommandIndicators > Indicator assignments`: "Rally point (own or neutral object) | Object | Green". A Robo
/// Portal right-clicked onto a patch rallies there, under the Move pointer ("Selection is a production building: Move
/// (rally point)"), and while it is selected a green ring marks the patch, with its line from the portal. Its Drones
/// gather there (design change 0136): `remembered`.
#[test]
fn a_portal_rallied_on_a_patch_marks_the_patch() {
    let mut app = gathering_game();
    click(&mut app, home(ROBO_PORTAL), &[]);
    assert_eq!(selected(&app), [ROBO_PORTAL]);
    assert_eq!(pointer_over(&mut app, home(PATCH)), Some(PointerDisplay::Move));
    right_click(&mut app, home(PATCH), &[]);
    let world = app.world();
    let sim = world.resource::<SimRes>();
    let rallies = rally_indicators(sim, &Sight::new(sim, P1, false), world.resource::<ControlStates>());
    let marker = Indicator::Object {
        id: ObjectId(PATCH),
        color: MOVE_COLOR,
    };
    assert_eq!(
        rallies,
        [RallyIndicator {
            producer: ObjectId(ROBO_PORTAL),
            marker
        }]
    );
    assert!(drawn_vertices(&mut app) > 0);
}

/// Design change 0124: "Buildings and units whose costs are TBD show in their slot, unavailable": the Robo Portal's W
/// Scout, the Bio Portal's W Lancer and E psionic unit (slice M3-J2). Pressing one, by its hotkey or on the panel,
/// trains nothing and says why (design change 0162: "Not available yet").
#[test]
fn a_portals_units_not_in_the_game_yet_are_unavailable() {
    use KeyCode::{KeyE, KeyW};
    use Planned::{Lancer, PsionicUnit, Scout};
    let mut app = gathering_game();
    set_crystals(&mut app, 1000);
    let robo = [(slot(0, 1), Scout, KeyW, "SC")];
    let bio = [(slot(0, 1), Lancer, KeyW, "LA"), (slot(0, 2), PsionicUnit, KeyE, "PU")];
    for (portal, planned) in [(ROBO_PORTAL, &robo[..]), (BIO_PORTAL, &bio[..])] {
        click(&mut app, home(portal), &[]);
        assert_eq!(selected(&app), [portal]);
        let shown = buttons(&mut app);
        for &(at, unit, key, label) in planned {
            let button = PanelButton::Command(PanelCommand::Planned(unit));
            assert!(
                shown.contains(&(at, button, ButtonLook::Unavailable, label)),
                "{shown:?}"
            );
            assert_eq!(says(&mut app, |app| tap(app, &[], key)), Some("Not available yet"));
            assert_eq!(
                says(&mut app, |app| click_button(app, at, &[])),
                Some("Not available yet")
            );
        }
    }
    assert_eq!(productions(&app), []);
    assert_eq!(bank(&app), 1000);
}
