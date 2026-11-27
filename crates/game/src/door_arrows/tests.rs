use sc_sim::{BuildingOrientation, Footprint, Rotation, TilePos};

use super::*;

const PATH_OFFSET_SU: f32 = sc_sim::tuning::tunnel::BUILTIN.path_offset_su;

/// The doorways of a `kind` building placed with its top-left tile at (3, 2), turned by `orientation`: each arrow's
/// way, the letter of its side as turned, and where it stands.
fn doors(kind: ObjectKind, orientation: BuildingOrientation) -> Vec<(Way, SideLetter, Vec2)> {
    let placement = placed(kind, orientation);
    let letters = sides(kind, orientation).expect("a structure");
    door_arrows(kind, &placement)
        .into_iter()
        .map(|arrow| {
            let i = (DIRECTIONS.iter())
                .position(|&(dx, dy)| Vec2::new(dx as f32, dy as f32) == arrow.outward)
                .expect("out of a side");
            (arrow.way, letters[i], arrow.at)
        })
        .collect()
}

fn placed(kind: ObjectKind, orientation: BuildingOrientation) -> Placement {
    let Footprint::Tiles { w, h } = kind.def().footprint else {
        panic!("{kind:?} has no tiles");
    };
    let (w, h) = if orientation.swaps_axes() { (h, w) } else { (w, h) };
    Placement {
        origin: TilePos::new(3, 2),
        w,
        h,
        rotation: orientation.rotation,
        flip_h: orientation.flip_h,
        flip_v: orientation.flip_v,
    }
}

fn turned(rotation: Rotation) -> BuildingOrientation {
    BuildingOrientation {
        rotation,
        ..Default::default()
    }
}

/// Design change 0390's Verification: "Place a Cults Armory: an inward arrow shows on side B and an outward arrow on
/// side C. Rotating the ghost turns the arrows with it." Unturned (3 × 2, ABAC), B faces east and C west, each arrow
/// at its side's middle (`factions/cults/structures/armory.md > EntranceSide - one short side (B)`,
/// `factions/cults/structures/armory.md > ExitSide - opposite short side (C)`); turned clockwise, B faces south and C
/// north; flipped, B west and C east.
#[test]
fn an_armory_goes_in_at_b_and_out_at_c_however_it_turns() {
    use SideLetter::{B, C};
    let armory = ObjectKind::Armory;
    // Tiles (3, 2) to (5, 3): x 192 to 384, y 128 to 256.
    assert_eq!(
        doors(armory, BuildingOrientation::default()),
        [
            (Way::In, B, Vec2::new(384.0, 192.0)),
            (Way::Out, C, Vec2::new(192.0, 192.0))
        ]
    );
    // Tiles (3, 2) to (4, 4): x 192 to 320, y 128 to 320.
    assert_eq!(
        doors(armory, turned(Rotation::R90)),
        [
            (Way::Out, C, Vec2::new(256.0, 128.0)),
            (Way::In, B, Vec2::new(256.0, 320.0))
        ]
    );
    let flipped = BuildingOrientation {
        flip_h: true,
        ..Default::default()
    };
    assert_eq!(
        doors(armory, flipped),
        [
            (Way::Out, C, Vec2::new(384.0, 192.0)),
            (Way::In, B, Vec2::new(192.0, 192.0))
        ]
    );
}

/// Design change 0390's Verification: "A Garrison ghost shows one two-way arrow", at its one door, side B
/// (`factions/cults/structures/garrison.md`: "the door on side B (one door, entrance and exit)"), facing west unturned
/// (AAAB) and north turned clockwise.
#[test]
fn a_garrison_has_one_two_way_arrow() {
    let garrison = ObjectKind::Garrison;
    // Tiles (3, 2) to (4, 3): x 192 to 320, y 128 to 256.
    assert_eq!(
        doors(garrison, BuildingOrientation::default()),
        [(Way::Both, SideLetter::B, Vec2::new(192.0, 192.0))]
    );
    assert_eq!(
        doors(garrison, turned(Rotation::R90)),
        [(Way::Both, SideLetter::B, Vec2::new(256.0, 128.0))]
    );
}

/// Another faction's building with an exit (`core/entities.md > Entrance and exit (production and research
/// buildings)`: "including those that release units through an exit side, e.g. the GDO Barracks"): the Barracks lets
/// its units out at side B ("Units exit from one of the short sides (B side ...)", `factions/gdo/structures/
/// barracks.md`) and takes none in, so it has one arrow, out; so does the Recruitment Hall. Each Cults building that
/// holds units is entered at B and left at C.
#[test]
fn every_exit_side_has_an_arrow_out_and_every_entrance_one_in() {
    use SideLetter::{B, C};
    let unturned = BuildingOrientation::default;
    let ways = |kind| -> Vec<(Way, SideLetter)> { (doors(kind, unturned()).iter()).map(|&(w, l, _)| (w, l)).collect() };
    // Tiles (3, 2) to (5, 3): side B, east, is x 384.
    assert_eq!(
        doors(ObjectKind::Barracks, unturned()),
        [(Way::Out, B, Vec2::new(384.0, 192.0))]
    );
    assert_eq!(ways(ObjectKind::RecruitmentHall), [(Way::Out, B)]);
    for kind in [
        ObjectKind::Cathedral,
        ObjectKind::Altar,
        ObjectKind::Temple,
        ObjectKind::Shrine,
        ObjectKind::ChopShop,
        ObjectKind::Airstrip,
    ] {
        let mut found = ways(kind);
        found.sort_by_key(|&(_, letter)| letter);
        assert_eq!(found, [(Way::In, B), (Way::Out, C)], "{kind:?}");
    }
}

/// (M5-N7) The Airstrip's arrows lie along its runway (design changes 0390 and 0394; `factions/cults/structures/
/// airstrip.md > Placeholder stats`: "EntranceSide B (Recruits walk in); [...] takes off along it toward C"): in at
/// the middle of B, its entrance, and out at the middle of C, the end its Sparrows take off toward (the module's
/// placeholder reading), both short sides of its 4 × 2; turned clockwise, B faces south and C north.
#[test]
fn an_airstrip_goes_in_at_b_and_out_at_c_along_its_runway() {
    use SideLetter::{B, C};
    let airstrip = ObjectKind::Airstrip;
    // Tiles (3, 2) to (6, 3): x 192 to 448, y 128 to 256.
    assert_eq!(
        doors(airstrip, BuildingOrientation::default()),
        [
            (Way::In, B, Vec2::new(448.0, 192.0)),
            (Way::Out, C, Vec2::new(192.0, 192.0))
        ]
    );
    // Tiles (3, 2) to (4, 5): x 192 to 320, y 128 to 384.
    assert_eq!(
        doors(airstrip, turned(Rotation::R90)),
        [
            (Way::Out, C, Vec2::new(256.0, 128.0)),
            (Way::In, B, Vec2::new(256.0, 384.0))
        ]
    );
}

/// A portal's units leave its centre through an open side (`factions/colonists/structures/README.md > Portal Exits
/// (energy doorway)`): an arrow out at the middle of each, the Robo Portal's four (AAAA) and the Bio Portal's two, its
/// top and right unturned (AABB: "top and right open in the default orientation, rotating with the building"), its
/// right and bottom turned clockwise.
#[test]
fn a_portal_has_an_arrow_out_of_each_open_side() {
    let outs = |kind, orientation| -> Vec<(Way, SideLetter)> {
        (doors(kind, orientation).iter()).map(|&(w, l, _)| (w, l)).collect()
    };
    let out_a = (Way::Out, SideLetter::A);
    assert_eq!(outs(ObjectKind::RoboPortal, BuildingOrientation::default()), [out_a; 4]);
    assert_eq!(outs(ObjectKind::BioPortal, BuildingOrientation::default()), [out_a; 2]);
    // Tiles (3, 2) to (5, 4): x 192 to 384, y 128 to 320; turned clockwise, the open sides face east and south.
    let bio: Vec<Vec2> = (doors(ObjectKind::BioPortal, turned(Rotation::R90)).iter())
        .map(|&(_, _, at)| at)
        .collect();
    assert_eq!(bio, [Vec2::new(384.0, 224.0), Vec2::new(288.0, 320.0)]);
}

/// The Tunnel's Side A, "two paths: one for entering, one for exiting" (`factions/syndicate/tunnels.md`): an arrow in
/// at the entry path, nearer Side D, and one out at the exit path, nearer Side B, where the sim takes units in and
/// lets them out (`sc_sim::network::Paths`). Unturned (ABCD), A is the top: tiles (3, 2) to (6, 5), its middle
/// (320, 128), D to the west and B to the east.
#[test]
fn a_tunnel_is_entered_and_left_by_its_two_paths() {
    use SideLetter::A;
    assert_eq!(
        doors(ObjectKind::Tunnel, BuildingOrientation::default()),
        [
            (Way::In, A, Vec2::new(320.0 - PATH_OFFSET_SU, 128.0)),
            (Way::Out, A, Vec2::new(320.0 + PATH_OFFSET_SU, 128.0))
        ]
    );
}

/// No arrow where no unit goes in or out: a Hideout's units stay in the Tunnel Network (`core/entities.md > Exit
/// Doorway`: "units released underground (e.g. Syndicate Tunnels) are not covered by this rule"); a Generator, a
/// Storage and a Gunworks (`factions/cults/structures/gunworks.md`: "Until its research is designed [...] Recruits
/// can't enter it") take none in.
#[test]
fn a_building_without_doorways_has_no_arrows() {
    for kind in [
        ObjectKind::Hideout,
        ObjectKind::Generator,
        ObjectKind::Storage,
        ObjectKind::Gunworks,
    ] {
        assert_eq!(
            door_arrows(kind, &placed(kind, BuildingOrientation::default())),
            [],
            "{kind:?}"
        );
    }
}

/// An arrow is a head and a shaft, a two-way one two heads and a shaft (`draw_arrows`): the Armory's two, 7 vertices
/// each; the Garrison's one, 10.
#[test]
fn each_arrow_is_a_head_and_a_shaft() {
    let map = Map::from_ascii(&vec![".".repeat(12); 10].join("\n")).unwrap();
    let vertices = |kind| {
        let mut mesh = ColorMesh::triangles();
        let arrows = door_arrows(kind, &placed(kind, BuildingOrientation::default()));
        draw_arrows(&mut mesh, &map, &arrows, 0.0);
        mesh.vertex_count()
    };
    assert_eq!(vertices(ObjectKind::Armory), 2 * (3 + 4));
    assert_eq!(vertices(ObjectKind::Garrison), 3 + 3 + 4);
}
