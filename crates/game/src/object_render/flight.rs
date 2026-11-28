//! (M5-H1) Air units in flight (plan decision T81's slice, H1: "air units are drawn raised, with a ground shadow, at
//! their true position, even past the edge"; placeholders until the art direction, Q0001, settles them). An air unit's
//! body is drawn in the air layer (`DrawLayer::Air`: over every ground unit and the Shroud's clouds),
//! `LookTuning::flight_lift_levels` above the ground under it, and its shadow, a dark ellipse as wide as its silhouette
//! ([`FlightShadow`]), on that ground, under the ground units, where their selection rings are. Both are drawn at the
//! sim's `Object::pos`, past the map's edge too, where the sim counts the unit as being at the nearest edge tile
//! (`Object::map_pos`, `systems/units.md > Air Collision`): the player sees where it flies. What is drawn on a unit
//! follows its body ([`body_height`]): its picking box and selection ring (`crate::selection::pick`), its HP bar
//! (`crate::combat_effects`), the marker of an order on it (`crate::command_ui::indicators`) and its shots' ends.
//!
//! (M5-N11) An aircraft on its takeoff run (`sc_sim::mover::Takeoff`: a Sparrow out of the Airstrip) rolls into the air
//! along the runway (design change 0428's Verification: "A Sparrow finishes at the Airstrip: it appears on the runway's
//! B end and rolls into the air toward C"; `factions/cults/units/sparrow.md > Placeholder stats`: "speeding up along
//! the runway and lifting off"): drawn on the runway where it appears, it stays on it until `LookTuning::lift_off_su`
//! before the run's end and rises evenly from there to its flight's height at the end ([`object_lift`]), its shadow
//! under it all the way. Placeholder (Q0001): where on the run it lifts off. The shots at it and the bursts on it are
//! drawn at its flight's height meanwhile, as before (`crate::combat_effects`, [`flight_lift`]).
//!
//! (M6-SD) An aircraft rising out of a Tier 3 Tunnel's air exit door, or descending into its air entry door
//! (`sc_sim::network::door_lift`; design change 0431: "it **descends** into the entry door and **rises** out of the
//! exit door over **16 frames (1 s) each**"), is drawn on the door at the bottom of it and at its flight's height at
//! the top, evenly by the sim's whole ticks in between ([`object_lift`]), its shadow under it all the way.
//!
//! (M6-GH2) An aircraft landing on an Airfield's slot (`sc_sim::landing::lift`;
//! `factions/gdo/structures/airfield.md > Landing slots`: "Landing and take-off: **16 frames (1 s)** each") is drawn
//! sinking from its flight's height to the ground, evenly by the sim's whole ticks, its shadow under it; down on its
//! slot it is drawn on the ground with no shadow ([`shadow_shown`]: plan §GH2's landed look, "no air shadow"). Its
//! take-off is a rise (`Mover::rise`), drawn as one out of an air door is.

use bevy::prelude::*;
use sc_sim::defs::Domain;
use sc_sim::{Map, Object, ObjectId, ObjectKind, landing, network};

use super::{ObjectVisual, object_height};
use crate::coords::{DrawLayer, lift};
use crate::shapes::ColorMesh;
use crate::sim_driver::{RenderAlpha, SimRes};
use crate::tuning::ClientTuning;
use crate::tuning::look::LookTuning;

/// A shadow's darkness: black at this alpha. Placeholder (Q0001).
const SHADOW_ALPHA: f32 = 0.35;

/// A shadow's height over its width: an ellipse, the sun high in the south (placeholder, Q0001).
const SHADOW_FLATTEN: f32 = 0.6;

/// How far under the ground units' selection rings a shadow is drawn, z within their layer.
const SHADOW_UNDER_RINGS_Z: f32 = 0.005;

/// Whether a `kind` unit flies: an air unit (`systems/units.md > Air Collision`, its base's Domain), the Sparrow's
/// Glider among them.
pub fn flies(kind: ObjectKind) -> bool {
    (kind.def().unit.as_ref()).is_some_and(|unit| unit.base_def().domain == Domain::Air)
}

/// How far above the ground an object of `kind` is drawn, in terrain levels: `look`'s
/// `LookTuning::flight_lift_levels` for an air unit, nothing for anything else.
pub fn flight_lift(kind: ObjectKind, look: &LookTuning) -> f32 {
    if flies(kind) { look.flight_lift_levels } else { 0.0 }
}

/// (M5-N11) How far above the ground under it `o`, drawn at `pos`, is drawn, in terrain levels: an air unit's flight
/// ([`flight_lift`]); on its takeoff run, none until `LookTuning::lift_off_su` before the run's end, then rising
/// evenly to all of it at the end (module docs); (M6-SD) at an air door, the share of its rise or of what is left of
/// its descent (`network::door_lift`), and so (M6-GH2) taking off from an Airfield's slot; landing on one, the share of
/// its landing left (`landing::lift`), none down on it; nothing for anything else.
pub fn object_lift(o: &Object, pos: Vec2, look: &LookTuning) -> f32 {
    let flight = flight_lift(o.kind, look);
    if let Some(share) = network::door_lift(o).or_else(|| landing::lift(o))
        && flight > 0.0
    {
        return flight * share;
    }
    match o.mover.as_ref().and_then(|mover| mover.takeoff) {
        Some(takeoff) if flight > 0.0 => {
            let left = (takeoff.end - pos).dot(takeoff.along);
            flight * (1.0 - left / look.lift_off_su).clamp(0.0, 1.0)
        }
        _ => flight,
    }
}

/// The height `o`, drawn at `pos`, is drawn at, in levels: the ground's (`super::object_height`), an air unit's lifted
/// by its flight as it flies now ([`object_lift`]: (M5-N11) on the runway at the start of its takeoff run), as `look`
/// lifts it.
pub fn body_height(map: &Map, o: &Object, pos: Vec2, look: &LookTuning) -> f32 {
    object_height(map, pos, o.placement.as_ref()) + object_lift(o, pos, look)
}

/// The shadow of an air unit's visual: a child of the visual, which the fog hides with it, kept on the ground under
/// it ([`place_shadows`]).
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct FlightShadow(pub ObjectId);

/// A `kind` unit's shadow: a black ellipse at [`SHADOW_ALPHA`], as wide as its silhouette, [`SHADOW_FLATTEN`] as tall.
pub(super) fn shadow_mesh(kind: ObjectKind) -> Mesh {
    let half = kind.def().half_extents().unwrap_or(Vec2::splat(12.0));
    let outline: Vec<Vec2> = (0..20)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 20.0;
            Vec2::new(half.x * a.cos(), half.x * SHADOW_FLATTEN * a.sin())
        })
        .collect();
    let mut mesh = ColorMesh::triangles();
    mesh.polygon(&outline, 0.0, Color::srgba(0.0, 0.0, 0.0, SHADOW_ALPHA));
    mesh.into_mesh()
}

/// A shadow's transform under a visual turned by `rotation`, `levels` above the ground ([`object_lift`]): down the
/// screen by that lift, back to the ground it flies over, and down into the layer of the ground units' selection rings
/// (`DrawLayer::UnitDecals`, y-sorted as its unit is); its own rotation undoes the visual's, so that it never turns
/// round its body.
pub(super) fn shadow_transform(rotation: Quat, levels: f32) -> Transform {
    let back = rotation.inverse();
    let below = DrawLayer::UnitDecals.base_z() - DrawLayer::Air.base_z() - SHADOW_UNDER_RINGS_Z;
    Transform::from_translation(back * Vec3::new(0.0, -lift(levels), below)).with_rotation(back)
}

/// (M6-GH2) Whether an air unit's shadow is shown: but while it is down on an Airfield's slot (`landing::down`; module
/// docs), on the ground.
pub fn shadow_shown(o: &Object) -> bool {
    !landing::down(o)
}

/// Keeps every shadow on the ground under its visual, after the visuals are placed (`super::place_object_visuals`), as
/// far under it as its body is lifted at the render `alpha` ([`object_lift`]), and (M6-GH2) hides it while its unit is
/// down on a slot ([`shadow_shown`]).
pub(super) fn place_shadows(
    sim: Res<SimRes>,
    alpha: Res<RenderAlpha>,
    tuning: Res<ClientTuning>,
    visuals: Query<&Transform, (With<ObjectVisual>, Without<FlightShadow>)>,
    mut shadows: Query<(&FlightShadow, &ChildOf, &mut Transform, Option<&mut Visibility>)>,
) {
    let look = &tuning.look;
    for (shadow, parent, mut transform, visibility) in &mut shadows {
        let Ok(visual) = visuals.get(parent.parent()) else {
            continue;
        };
        let lift_at = |o: &Object| object_lift(o, o.prev_pos.lerp(o.pos, alpha.0), look);
        let object = sim.object(shadow.0);
        let levels = object.map_or(look.flight_lift_levels, lift_at);
        transform.set_if_neq(shadow_transform(visual.rotation, levels));
        // A mesh's Visibility comes with the render plugins (`Mesh2d`'s required components), which a headless app
        // may lack.
        if let Some(mut visibility) = visibility {
            let shown = object.is_none_or(shadow_shown);
            visibility.set_if_neq(if shown {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::{drawn_at_height, ground_translation};

    const LOOK: LookTuning = crate::tuning::look::BUILTIN;

    /// A Sparrow flies and nothing else of its faction does; its body is drawn 48 su above the ground, and its shadow,
    /// turned or not, lands where the ground under it is drawn, in the rings' layer.
    #[test]
    fn a_flying_unit_s_shadow_lies_on_the_ground_under_its_raised_body() {
        assert!(flies(ObjectKind::Sparrow));
        assert!(!flies(ObjectKind::Militant) && !flies(ObjectKind::HandOfHeaven) && !flies(ObjectKind::Airstrip));
        assert_eq!(lift(flight_lift(ObjectKind::Sparrow, &LOOK)), 48.0);
        let (ground, height) = (Vec2::new(300.0, 200.0), 2.0);
        let body = ground_translation(DrawLayer::Air, ground, height + LOOK.flight_lift_levels);
        let under = ground_translation(DrawLayer::UnitDecals, ground, height) - Vec3::Z * SHADOW_UNDER_RINGS_Z;
        for facing in [0.0, 1.0, -2.5] {
            let rotation = Quat::from_rotation_z(facing);
            let visual = Transform::from_translation(body).with_rotation(rotation);
            let shadow = visual * shadow_transform(rotation, LOOK.flight_lift_levels);
            assert!(
                shadow.translation.distance(under) < 1e-3,
                "{} vs {under}",
                shadow.translation
            );
            assert!(shadow.rotation.angle_between(Quat::IDENTITY) < 1e-5);
        }
        assert_eq!(drawn_at_height(ground, height).x, body.x);
    }

    /// (M5-N11) Design change 0428's Verification: "A Sparrow finishes at the Airstrip: it appears on the runway's B end
    /// and rolls into the air toward C". On its takeoff run (here toward the west, its run ending at x 100) a Sparrow is
    /// drawn on the ground until 128 su before the run's end, at half its flight's height 64 su before it, and at all
    /// of it at the end and past it; with no takeoff run, at its flight; a Militant on the ground whatever.
    #[test]
    fn a_sparrow_on_its_takeoff_run_rolls_on_the_runway_then_rises_to_its_flight() {
        use sc_sim::mover::Takeoff;
        use sc_sim::testing::ScenarioBuilder;
        use sc_sim::{Faction, PlayerId, SpawnAt};
        let p1 = PlayerId(1);
        let t = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
            .faction(p1, Faction::Cults)
            .spawn(ObjectKind::Sparrow, Some(p1), SpawnAt::Point(Vec2::new(300.0, 300.0)))
            .spawn(ObjectKind::Militant, Some(p1), SpawnAt::Point(Vec2::new(300.0, 500.0)))
            .start();
        let mut sparrow = t.sim().object(ObjectId(1)).expect("a Sparrow").clone();
        assert_eq!(
            object_lift(&sparrow, sparrow.pos, &LOOK),
            LOOK.flight_lift_levels,
            "in flight"
        );
        let takeoff = Takeoff {
            end: Vec2::new(100.0, 300.0),
            along: Vec2::NEG_X,
        };
        sparrow.mover.as_mut().expect("a mover").takeoff = Some(takeoff);
        let at = |x| object_lift(&sparrow, Vec2::new(x, 300.0), &LOOK);
        assert_eq!(
            [at(330.0), at(228.0), at(164.0), at(100.0), at(90.0)],
            [0.0, 0.0, 1.5, 3.0, 3.0]
        );
        let mut militant = t.sim().object(ObjectId(2)).expect("a Militant").clone();
        militant.mover.as_mut().expect("a mover").takeoff = Some(takeoff);
        assert_eq!(object_lift(&militant, Vec2::new(100.0, 300.0), &LOOK), 0.0);
    }

    /// (M6-SD) Design change 0431: a Phantom rising out of an air door over its 16 frames is drawn on the door with
    /// none of them done, at half its flight's height with 8 done, and at all of it once risen; descending into the
    /// entry door, at all of it with none done and at a quarter with 12 done.
    #[test]
    fn a_phantom_rising_out_of_an_air_door_and_descending_into_one_is_drawn_rising_and_sinking() {
        use sc_sim::economy::RATIO_ONE;
        use sc_sim::orders::state::Enter;
        use sc_sim::orders::{ActiveOrder, Behaviour, Order, OrderGroup, OrderKind};
        use sc_sim::progress::Progress;
        use sc_sim::testing::ScenarioBuilder;
        use sc_sim::{Faction, PlayerId, SpawnAt};
        let p1 = PlayerId(1);
        let t = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
            .faction(p1, Faction::Syndicate)
            .spawn(ObjectKind::Phantom, Some(p1), SpawnAt::Point(Vec2::new(300.0, 300.0)))
            .start();
        let phantom = t.sim().object(ObjectId(1)).expect("a Phantom");
        let flight = LOOK.flight_lift_levels;
        assert_eq!(object_lift(phantom, phantom.pos, &LOOK), flight, "in flight");
        let done = |ticks: u32| Progress {
            elapsed: ticks * RATIO_ONE,
            total: 16,
        };
        let lift_rising = |ticks| {
            let mut rising = phantom.clone();
            rising.mover.as_mut().expect("a mover").rise = Some(done(ticks));
            object_lift(&rising, rising.pos, &LOOK)
        };
        assert_eq!(
            [lift_rising(0), lift_rising(8), lift_rising(15)],
            [0.0, flight / 2.0, flight * 15.0 / 16.0]
        );
        let lift_descending = |ticks| {
            let mut descending = phantom.clone();
            let orders = descending.orders.as_mut().expect("orders");
            orders.current = Some(ActiveOrder {
                order: Order {
                    kind: OrderKind::Enter(ObjectId(9)),
                    group: OrderGroup(0),
                    slot: None,
                },
                since: 1,
                state: Behaviour::Enter(Enter {
                    walked: false,
                    descent: Some(done(ticks)),
                }),
            });
            object_lift(&descending, descending.pos, &LOOK)
        };
        assert_eq!([lift_descending(0), lift_descending(12)], [flight, flight / 4.0]);
    }

    /// (M6-GH2) A Hornet landing on an Airfield's slot over its 16 frames is drawn at its flight's height with none of
    /// them done, at a quarter of it with 12 done, and on the ground, its shadow hidden, once down; taking off, it
    /// rises as from an air door, its shadow shown.
    #[test]
    fn a_hornet_landing_on_its_slot_is_drawn_sinking_to_the_ground_and_down_without_its_shadow() {
        use sc_sim::economy::RATIO_ONE;
        use sc_sim::landing::Landing;
        use sc_sim::progress::Progress;
        use sc_sim::testing::ScenarioBuilder;
        use sc_sim::{PlayerId, SpawnAt};
        let t = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
            .spawn(
                ObjectKind::Hornet,
                Some(PlayerId(1)),
                SpawnAt::Point(Vec2::new(300.0, 300.0)),
            )
            .start();
        let hornet = t.sim().object(ObjectId(1)).expect("a Hornet");
        let flight = LOOK.flight_lift_levels;
        assert_eq!(object_lift(hornet, hornet.pos, &LOOK), flight, "in flight");
        assert!(shadow_shown(hornet));
        let landing = |ticks: u32| {
            let mut landing = hornet.clone();
            let state = landing.aircraft.as_mut().expect("an aircraft");
            state.landing = Some(Landing {
                at: ObjectId(9),
                descent: Progress {
                    elapsed: ticks * RATIO_ONE,
                    total: 16,
                },
                rearm: None,
            });
            landing
        };
        let lift_landing = |ticks| {
            let landing = landing(ticks);
            object_lift(&landing, landing.pos, &LOOK)
        };
        assert_eq!(
            [lift_landing(0), lift_landing(12), lift_landing(16)],
            [flight, flight / 4.0, 0.0]
        );
        assert!(shadow_shown(&landing(15)), "landing, in the air");
        assert!(!shadow_shown(&landing(16)), "down, on the ground");
        let mut rising = hornet.clone();
        rising.mover.as_mut().expect("a mover").rise = Some(Progress::new(16));
        assert_eq!(
            object_lift(&rising, rising.pos, &LOOK),
            0.0,
            "taking off from the ground"
        );
        assert!(shadow_shown(&rising));
    }
}
