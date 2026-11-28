//! (M6-SB) The tank invariant N1 (`Object::tanks`, `Sim::patches`, `crate::patches`): an object has tanks exactly when
//! its type has a tank (`UnitDef::tanks`), and a tank its type doesn't have is full and off; a toggle is on only with a
//! charge left in its tank ("an empty tank switches its toggle off", design change 0425), its trail's newest patch on
//! the ground where it says, and off it names none; a unit held off the map has both off ("Entering the Tunnel Network
//! switches both toggles off"); and every patch on the ground is of a kind a type lays, a disc, and not yet gone (a
//! loaded state's patches name an id below `next_id`: `sim/load.rs`).

use super::Violation;
use crate::defs::PatchKind;
use crate::object::Object;
use crate::patches::{self, Patches};
use crate::sim::Sim;

/// N1 (module docs).
pub(super) fn tanks(sim: &Sim) -> Result<(), Violation> {
    let patches = sim.patches();
    for o in sim.objects() {
        its_tanks(o, patches, false)?;
    }
    for held in sim.held() {
        its_tanks(&held.object, patches, true)?;
    }
    the_patches(sim)
}

/// N1 for one object, `held` off the map or not.
fn its_tanks(o: &Object, on_the_ground: &Patches, held: bool) -> Result<(), Violation> {
    let fail = |problem: String| {
        Err(Violation {
            invariant: "N1",
            message: format!("#{} ({:?}) {problem}", o.id.0, o.kind),
            at: o.pos,
        })
    };
    let has_tanks = o.def().unit.as_ref().is_some_and(|unit| !unit.tanks.is_empty());
    let Some(tanks) = o.tanks else {
        return match has_tanks {
            true => fail("has no tanks, its type a tank".to_string()),
            false => Ok(()),
        };
    };
    if !has_tanks {
        return fail("has tanks, its type none".to_string());
    }
    for kind in PatchKind::ALL {
        let tank = tanks.get(kind);
        let Some(def) = patches::tank_def(o.kind, kind) else {
            if !tank.is_default() {
                return fail(format!("has a {kind:?} tank used or on, {tank:?}, its type none"));
            }
            continue;
        };
        match (tank.on, tank.laid) {
            (true, _) if held => return fail(format!("is held with its {kind:?} toggle on")),
            (true, _) if tank.is_empty(def) => return fail(format!("has its {kind:?} toggle on, its tank empty")),
            (true, None) => return fail(format!("has its {kind:?} toggle on, and has laid nothing")),
            (true, Some(at)) => {
                let newest = (on_the_ground.iter()).any(|p| p.by == o.id && p.kind == kind && p.centre == at);
                if !newest {
                    return fail(format!(
                        "has its {kind:?} toggle on, its newest patch at {at} not on the ground"
                    ));
                }
            }
            (false, Some(at)) => return fail(format!("has its {kind:?} toggle off, its trail at {at}")),
            (false, None) => {}
        }
    }
    Ok(())
}

/// N1 for the patches on the ground (module docs): of a kind a type lays, until a tick to come, a disc of a radius; and
/// the store counts them as they are.
fn the_patches(sim: &Sim) -> Result<(), Violation> {
    let now = sim.tick();
    let patches = sim.patches();
    let fail = |problem: String, at| {
        Err(Violation {
            invariant: "N1",
            message: problem,
            at,
        })
    };
    for p in patches.iter() {
        if patches::def_of(p.kind).is_none() {
            return fail(
                format!("a {:?} patch on the ground, which no type lays: {p:?}", p.kind),
                p.centre,
            );
        }
        if p.until <= now {
            return fail(
                format!("a patch on the ground after its time: {p:?}, in tick {now}"),
                p.centre,
            );
        }
        if !(p.radius.is_finite() && p.radius > 0.0 && p.centre.is_finite()) {
            return fail(format!("a patch of no disc: {p:?}"), p.centre);
        }
    }
    for kind in PatchKind::ALL {
        let counted = patches.iter().filter(|p| p.kind == kind).count();
        if patches.count(kind) != counted {
            let problem = format!(
                "{} {kind:?} patches counted, {counted} on the ground",
                patches.count(kind)
            );
            return fail(problem, glam::Vec2::ZERO);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use glam::Vec2;
    use serde_json::json;

    use super::*;
    use crate::command::TickInputs;
    use crate::defs::ObjectKind;
    use crate::ids::PlayerId;
    use crate::scenario::SpawnAt;
    use crate::testing::ScenarioBuilder;
    use crate::testing::invariants::Invariants;

    const P1: PlayerId = PlayerId(1);

    /// A Shadebringer (1) and a Guard (2) of player 1 on a flat map, as saved before its first tick (tick 0): the
    /// Shadebringer's tanks full and off.
    fn saved() -> serde_json::Value {
        let builder = ScenarioBuilder::new(&"..........\n".repeat(8))
            .spawn(
                ObjectKind::Shadebringer,
                Some(P1),
                SpawnAt::Point(Vec2::new(100.0, 100.0)),
            )
            .spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(Vec2::new(300.0, 100.0)));
        let sim = Sim::new(&builder.build());
        assert_eq!(sim.tick(), 0);
        serde_json::to_value(&sim).expect("a state serializes")
    }

    /// The checker's verdict on `json` loaded, with object `n`'s tanks set to `tanks` and the patches `patches` on the
    /// ground.
    fn check_with(n: u32, tanks: serde_json::Value, patches: serde_json::Value) -> Result<(), Violation> {
        let mut json = saved();
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let object = objects.iter_mut().find(|o| o["id"] == n).expect("in the state");
        object["tanks"] = tanks;
        json["patches"] = patches;
        let sim: Sim = serde_json::from_value(json).expect("the edited state loads");
        Invariants::new(&sim).check(&sim, &TickInputs::default())
    }

    fn patch(until: u32) -> serde_json::Value {
        json!({ "kind": "Sludge", "by": 1, "centre": [72.0, 100.0], "radius": 24.0, "until": until })
    }

    fn broken(result: Result<(), Violation>, says: &str) {
        let violation = result.expect_err(says);
        assert_eq!(violation.invariant, "N1", "{violation:?}");
        assert!(violation.message.contains(says), "{violation:?}");
    }

    /// The negative test: a state edited so that a Shadebringer's toggle is on with an empty tank, or with its trail's
    /// newest patch not on the ground, off with a trail, its Cover's tank used before slice M6-SC, a Guard with tanks,
    /// or a patch of smoke, which no type lays before slice M6-SC, breaks N1, which a toggle on and its patch keep, and
    /// a toggle off and the patch it left. (A state with a patch whose time is over doesn't load.)
    #[test]
    fn a_toggle_on_with_nothing_left_or_no_patch_a_stray_tank_or_smoke_breaks_l1() {
        let trail = json!({ "sludge": { "used": 40960, "on": true, "laid": [72.0, 100.0] } });
        assert_eq!(
            check_with(1, trail.clone(), json!([patch(160)])),
            Ok(()),
            "on, its patch laid"
        );
        assert_eq!(
            check_with(1, json!({}), json!([patch(160)])),
            Ok(()),
            "off, its last patch on the ground"
        );
        let empty = json!({ "sludge": { "used": 6_553_600, "on": true, "laid": [72.0, 100.0] } });
        broken(check_with(1, empty, json!([patch(160)])), "its tank empty");
        broken(check_with(1, trail, json!([])), "not on the ground");
        let off = json!({ "sludge": { "used": 40960, "laid": [72.0, 100.0] } });
        broken(check_with(1, off, json!([patch(160)])), "toggle off, its trail");
        broken(
            check_with(1, json!({ "smoke": { "used": 40960 } }), json!([])),
            "its type none",
        );
        broken(check_with(2, json!({}), json!([])), "has tanks, its type none");
        let smoke = json!([{ "kind": "Smoke", "by": 1, "centre": [72.0, 100.0], "radius": 32.0, "until": 160 }]);
        broken(check_with(1, json!({}), smoke), "which no type lays");
    }
}
