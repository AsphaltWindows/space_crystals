//! The air doors at the map's edge (the brief: "A Tunnel by the map's edge may put its exit door's Phantom partly off
//! the map. Test a Tier 3 Tunnel at the edge. The Phantom comes out inside the map, or waits; it never ends up placed
//! off it"; `systems/units.md > Air Collision > Map edge`: "HoverCraft stay inside the map", slice M6-GK). A door's
//! centre lies 76 su inside its Tunnel's footprint with the built-in offset, so a Phantom's 44 su box over it is on the
//! map whatever the Tunnel's place; a tuned offset that puts the door past the edge is `tests/rules.rs`'s, as it
//! changes the process's rules.

use sc_sim::Rotation;

use super::*;

/// A Tunnel against the map's edge that its Side B faces, set to Tier 3 by a debug command (no upgrade reaches Tier 3
/// there: the Tier 3 area would leave the map, design change 0283), turned by `rotation`: Side A faces north unturned
/// and Side B east, then a quarter turn clockwise at a time. Its tile, for the 40 × 24 map, against the east, south,
/// west and north edges in turn.
fn at_the_edge(rotation: Rotation) -> TestSim {
    let origin = match rotation {
        Rotation::R0 => tile(36, 10),
        Rotation::R90 => tile(18, 20),
        Rotation::R180 => tile(0, 10),
        Rotation::R270 => tile(18, 0),
    };
    let mut scenario = ScenarioBuilder::new(&flat(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), origin)
        .build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation,
        flip_h: false,
        flip_v: false,
    };
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    t.debug_at(1, DebugCommand::SetTier { id: T1, tier: 3 });
    run_checked(&mut t, 1);
    t
}

/// Whether unit `id`'s box lies wholly on the map.
fn on_the_map(t: &TestSim, id: ObjectId) -> bool {
    let o = object(t, id);
    let half = o.def().half_extents().expect("a Silhouette");
    let size = Vec2::new(40.0, 24.0) * sc_sim::SU_PER_GU;
    o.pos.cmpge(half).all() && o.pos.cmple(size - half).all()
}

/// A Tier 3 Tunnel against each edge of the map, its exit door's side facing it: a Phantom from the middle of the map
/// right-clicks it, flies to its air entry door and goes in there, its box on the map throughout; ejected once the
/// transit delay is over, it comes out over the air exit door, 76 su from the edge, its box on the map, and rises
/// there. Side B faced each edge in turn.
#[test]
fn a_tier_3_tunnel_at_the_map_s_edge_lets_its_phantom_in_and_out_inside_the_map() {
    let mut faced = Vec::new();
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        let mut t = at_the_edge(rotation);
        let doors = paths(&t, T1);
        let towards_b = (doors.air_exit - object(&t, T1).pos).normalize();
        faced.push((towards_b.x.round() as i32, towards_b.y.round() as i32));
        let phantom = id(next_id(&t));
        t.debug_at(2, spawn(PHANTOM, P1, SpawnAt::Point(Vec2::new(1280.0, 768.0))));
        run_checked(&mut t, 2);
        assert_eq!(
            t.sim().default_order(phantom, Target::Object(T1)),
            Some(OrderKind::Enter(T1))
        );
        t.at(3, P1, order(&[phantom], OrderKind::Enter(T1)));
        while entries(&t, T1).is_empty() {
            assert!(t.sim().tick() < 600, "{rotation:?}: the Phantom goes in");
            step_checked(&mut t);
            if t.sim().object(phantom).is_some() {
                assert!(on_the_map(&t, phantom), "{rotation:?}, tick {}", t.sim().tick());
            }
        }
        assert_eq!(inside(&t), [phantom], "{rotation:?}");
        let out = t.sim().tick() + network::rules().transit_delay + 1;
        t.at(out, P1, eject(T1, PHANTOM));
        run_checked(&mut t, out);
        assert_eq!(exits(&t, T1), [(out, phantom)], "{rotation:?}");
        assert_eq!(object(&t, phantom).pos, doors.air_exit, "{rotation:?}: over the door");
        assert!(on_the_map(&t, phantom), "{rotation:?}");
        assert_eq!(rising(&t, phantom), Some(0), "{rotation:?}");
        let edge = match rotation {
            Rotation::R0 => 2560.0 - doors.air_exit.x,
            Rotation::R90 => 1536.0 - doors.air_exit.y,
            Rotation::R180 => doors.air_exit.x,
            Rotation::R270 => doors.air_exit.y,
        };
        assert_eq!(edge, 76.0, "{rotation:?}: the door's centre from the edge");
    }
    faced.sort_unstable();
    assert_eq!(faced, [(-1, 0), (0, -1), (0, 1), (1, 0)], "Side B faced each edge");
}
