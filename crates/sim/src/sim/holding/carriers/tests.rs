//! The unload's spots where a test needs the sim's inside (`Sim::unload`, `crate::carry`, *Unload All*): in a pocket of
//! cliffs, the passengers come out on the rings that have room for them, and one for whom no ring has a free spot stays
//! aboard (`systems/mechanics/carrying.md > Unloading`: "**Crowded**: passengers with no free spot **stay aboard**").
//! The rings in the open are `tests/transport/unloading.rs`'s, through the gate.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::network::Place;
use crate::player::Faction;
use crate::scenario::SpawnAt;
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const P1: PlayerId = PlayerId(1);
const TRANSPORT: ObjectId = ObjectId(1);

/// A 12 × 8 map of cliffs with a pocket of open ground on tiles (4, 2) to (6, 3), x 256..448 and y 128..256, and open
/// ground east of column 7 from row 0 to row 7, but for column 7's cliffs: the pocket is closed.
const POCKET: &str = "
    ccccccccc...
    ccccccccc...
    cccc...cc...
    cccc...cc...
    ccccccccc...
    ccccccccc...
    ccccccccc...
    ccccccccc...
";

/// Player 1's Subterranean Transport ([`TRANSPORT`]), facing east, in [`POCKET`] at (410, 192), its front 2 su clear of
/// the pocket's east end and its sides 40 su clear of its north and south ends, and 4 of player 1's Guards in the open
/// ground east of it, taken aboard (ids 2 to 5, boarded in that order): a game with tick 1 run.
fn loaded_in_the_pocket() -> Sim {
    let mut builder = ScenarioBuilder::new(POCKET).faction(P1, Faction::Syndicate).spawn(
        ObjectKind::SubterraneanTransport,
        Some(P1),
        SpawnAt::Point(Vec2::new(410.0, 192.0)),
    );
    for k in 0..4 {
        let at = Vec2::new(624.0 + 64.0 * (k % 2) as f32, 64.0 + 128.0 * (k / 2) as f32);
        builder = builder.spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(at));
    }
    let mut sim = Sim::new(&builder.build());
    sim.step(&Default::default());
    for n in 2..6 {
        assert!(
            sim.take_off_map(ObjectId(n), Place::Carrier(TRANSPORT), true),
            "#{n} aboard"
        );
    }
    sim
}

/// In the pocket, a Guard fits nowhere on the nearest ring, whose spots are each 53 su off the Transport's centreline,
/// nor ahead of it, nor beside it on any ring: only the second ring's two spots behind it, 120 su back and 36 su off
/// its centreline, have room, the third's behind them being beyond the pocket's west end. The two passengers longest
/// aboard come out there, facing as the Transport does; the other two stay aboard, and the Transport's room stays 2.
#[test]
fn in_a_pocket_two_come_out_behind_it_and_two_with_no_free_spot_stay_aboard() {
    let mut sim = loaded_in_the_pocket();
    sim.events.clear();
    sim.unload(TRANSPORT);
    let out: Vec<(ObjectId, Vec2, f32)> = (sim.objects())
        .filter(|o| o.kind == ObjectKind::Guard)
        .map(|o| (o.id, o.pos, o.facing))
        .collect();
    let behind = 410.0 - (36.0 + 84.367_53);
    assert_eq!(out.len(), 2, "two out: {out:?}");
    for &(id, pos, facing) in &out {
        assert!((pos.x - behind).abs() < 0.01, "{id:?} at {pos}: behind it");
        assert!(
            ((pos.y - 192.0).abs() - 36.124).abs() < 0.01,
            "{id:?} at {pos}: off its centreline"
        );
        assert_eq!(facing, 0.0, "{id:?} faces as the Transport does");
    }
    assert_eq!(
        out.iter().map(|&(id, ..)| id).collect::<Vec<_>>(),
        [ObjectId(2), ObjectId(3)],
        "the longest aboard"
    );
    let aboard: Vec<ObjectId> = sim.passengers(TRANSPORT).iter().map(|held| held.object.id).collect();
    assert_eq!(aboard, [ObjectId(4), ObjectId(5)], "no free spot: aboard still");
    assert_eq!(sim.room(TRANSPORT), 2);
    assert_eq!(sim.events.len(), 2, "an Unloaded each: {:?}", sim.events);
}
