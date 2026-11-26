//! The gathering markers (design change 0201 point 4, answering Q0052; `systems/controls/hud.md >
//! CommandIndicators > Gathering markers`): "Gather, Drop Off and Set Drop-off each have their own distinct marker on
//! the target object, not the Move marker", shown while the unit is selected. Placeholder looks, each with a colour of
//! its own (`command_ui::indicators::gathering`), found here in the indicators' mesh by their colours.

use bevy::mesh::VertexAttributeValues;
use space_crystals::command_ui::indicators::{
    GATHER_COLOR, GatheringMarker, IndicatorMesh, RETURN_COLOR, SET_DROP_OFF_COLOR, drop_off_indicators,
    selection_indicators,
};

use super::*;
use GatheringMarker::{Gather, ReturnResources, SetDropOff};
use KeyCode::KeyS;

/// The local player's markers now (no fog): its selected units' orders' ([`selection_indicators`]) and their
/// drop-offs' ([`drop_off_indicators`]).
fn markers(app: &App) -> Vec<Indicator> {
    let world = app.world();
    let (sim, states) = (world.resource::<SimRes>(), world.resource::<ControlStates>());
    let sight = Sight::new(sim, P1, false);
    let mut markers = selection_indicators(sim, &sight, states, false);
    markers.extend(drop_off_indicators(sim, &sight, states));
    markers
}

fn marker(id: u32, marker: GatheringMarker) -> Indicator {
    Indicator::Gathering {
        id: ObjectId(id),
        marker,
    }
}

/// Where the indicators' mesh draws `color`, at any opacity: the view point in the middle of its vertices of that
/// colour (every marker is symmetric about its centre), if it draws it at all. One marker of that colour at a time.
fn drawn_at(app: &mut App, color: Color) -> Option<Vec2> {
    let mut query = (app.world_mut()).query_filtered::<(&Mesh2d, &Visibility), With<IndicatorMesh>>();
    let (mesh, visibility) = query.single(app.world()).unwrap();
    if *visibility == Visibility::Hidden {
        return None;
    }
    let mesh = app.world().resource::<Assets<Mesh>>().get(&mesh.0).unwrap();
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        panic!("positions");
    };
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
        panic!("colours");
    };
    let rgb = color.to_linear().to_f32_array();
    let points: Vec<Vec2> = (positions.iter().zip(colors))
        .filter(|(_, c)| c[..3] == rgb[..3])
        .map(|(p, _)| Vec2::new(p[0], -p[1]))
        .collect();
    (!points.is_empty()).then(|| points.iter().sum::<Vec2>() / points.len() as f32)
}

/// Whether marker `color` is drawn around object `id`'s home (the map is flat).
fn drawn_on(app: &mut App, color: Color, id: u32) -> bool {
    drawn_at(app, color).is_some_and(|at| at.distance(home(id)) < 0.5)
}

/// `> Indicator assignments`: "Gather (sets the source: crystal patch, refinery's output) | Object | Gather marker
/// (distinct look, TBD)", "Drop Off (return resources to an Egress Node or refinery) | Object | Return Resources
/// marker (distinct look, TBD)". A Drone right-clicked onto a patch marks the patch with the Gather marker, not the
/// green Move marker (0201's verification: "Gather: a distinct marker appears on the patch, not the green Move
/// marker"), and its drop-off, the node its Gather assigned, with the Set Drop-off marker. With a load, right-clicked
/// onto the node, it marks the node with the Return Resources marker ("Drop Off: its own marker appears on the Node").
/// Deselected, it marks nothing.
#[test]
fn a_gather_and_a_drop_off_mark_their_targets_with_markers_of_their_own() {
    let mut app = gathering_game();
    click(&mut app, home(A), &[]);
    assert_eq!(selected(&app), [A]);
    right_click(&mut app, home(PATCH), &[]);
    assert_eq!(current(&app, A), Some(OrderKind::Gather(ObjectId(PATCH))));
    assert_eq!(markers(&app), [marker(PATCH, Gather), marker(NODE, SetDropOff)]);
    assert!(drawn_on(&mut app, GATHER_COLOR, PATCH));
    assert!(drawn_on(&mut app, SET_DROP_OFF_COLOR, NODE));
    assert_eq!(drawn_at(&mut app, MOVE_COLOR), None, "no Move marker");
    assert_eq!(drawn_at(&mut app, RETURN_COLOR), None);

    run_until(&mut app, 300, "A carries a load", |app| load(app, A) > 0);
    right_click(&mut app, home(NODE), &[]);
    assert_eq!(current(&app, A), Some(OrderKind::DropOff(ObjectId(NODE))));
    assert_eq!(markers(&app), [marker(NODE, ReturnResources), marker(NODE, SetDropOff)]);
    assert!(drawn_on(&mut app, RETURN_COLOR, NODE));
    assert!(drawn_on(&mut app, SET_DROP_OFF_COLOR, NODE));
    assert_eq!(drawn_at(&mut app, MOVE_COLOR), None, "no Move marker");
    assert_eq!(drawn_at(&mut app, GATHER_COLOR), None, "the Gather is over");

    click(&mut app, GROUND, &[]);
    assert!(selected(&app).is_empty());
    assert_eq!(markers(&app), []);
    assert_eq!(drawn_at(&mut app, RETURN_COLOR), None, "deselected");
    assert_eq!(drawn_at(&mut app, SET_DROP_OFF_COLOR), None);
}

/// `> Indicator assignments`: "Set Drop-off | Object | Set Drop-off marker (distinct look, TBD)"; 0201's verification:
/// "Set Drop-off: a third distinct marker appears, and the ReturnResources pointer shows while choosing the drop-off".
/// Drones A and B gathering patch 3 for node 1: S, the ReturnResources pointer over node 2, and the click there moves
/// their Set Drop-off marker from node 1 to node 2, one marker for both; their Gather marker stays on the patch.
#[test]
fn set_drop_off_marks_the_new_drop_off_with_a_marker_of_its_own() {
    let mut app = gathering_game();
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    right_click(&mut app, home(PATCH), &[]);
    assert_eq!(markers(&app), [marker(PATCH, Gather), marker(NODE, SetDropOff)]);
    assert!(drawn_on(&mut app, SET_DROP_OFF_COLOR, NODE));

    tap(&mut app, &[], KeyS);
    assert_eq!(interface(&app), InterfaceState::awaiting(PanelCommand::SetDropOff));
    assert_eq!(
        pointer_over(&mut app, home(NODE_2)),
        Some(PointerDisplay::ReturnResources)
    );
    click(&mut app, home(NODE_2), &[]);
    assert_eq!([drop_off(&app, A), drop_off(&app, B)], [Some(NODE_2); 2]);
    assert_eq!(markers(&app), [marker(PATCH, Gather), marker(NODE_2, SetDropOff)]);
    assert!(drawn_on(&mut app, SET_DROP_OFF_COLOR, NODE_2));
    assert!(drawn_on(&mut app, GATHER_COLOR, PATCH));
    assert_eq!(drawn_at(&mut app, MOVE_COLOR), None, "no Move marker");
}
