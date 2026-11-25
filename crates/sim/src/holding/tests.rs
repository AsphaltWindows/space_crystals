//! The rules of a building that holds units, on an Armory as a game has one: its entry path, and whom it takes in.

use super::*;
use crate::command::DebugCommand;
use crate::construction::site::{Builder, Construction};
use crate::ids::{ObjectId, PlayerId};
use crate::object::Rotation;
use crate::production::Doorway;
use crate::progress::Progress;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::testing::ScenarioBuilder;
use crate::units::TilePos;

/// An Armory of player 1's, its top-left tile at (4, 4), turned by `rotation` and flipped by `flip_h`, as a game has
/// one (3 × 2 tiles, ABAC: B on the right and C on the left, unturned).
fn armory(rotation: Rotation, flip_h: bool) -> Object {
    let mut t = ScenarioBuilder::new(&"..........\n".repeat(10)).start();
    let orientation = BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    };
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::Armory,
            owner: Some(PlayerId(1)),
            at: SpawnAt::Tile(TilePos::new(4, 4)),
            orientation,
        },
    )
    .run(1);
    let armory = t.sim().objects().find(|o| o.kind == ObjectKind::Armory);
    armory.expect("the Armory").clone()
}

/// Placeholder (Q0104 point 1): "The middle of side B, as the Tunnel's Side A". Unturned, the Armory covers tiles 4..7
/// across and 4..6 down, so its B, the right side, is the line x = 448, its middle at y = 320.
#[test]
fn the_entry_path_is_the_middle_of_side_b() {
    let path = entry_path(&armory(Rotation::R0, false)).unwrap();
    assert_eq!(path.outward, Vec2::new(1.0, 0.0));
    assert_eq!(path.spot, Vec2::new(448.0, 320.0));
    // A Recruit (24 x 24) walks to it with its box 4 su clear of the wall, and is in once it is there, or nearer.
    let half = Vec2::splat(12.0);
    let stand = path.stand(half);
    let against = 448.0 + 12.0 + crate::network::WALL_CLEARANCE;
    assert_eq!(stand, Vec2::new(against + 4.0, 320.0));
    assert!(path.reached(stand, half));
    assert!(path.reached(Vec2::new(against, 320.0), half));
    assert!(!path.reached(stand + Vec2::new(8.0, 0.0), half), "8 su at most");
}

/// In each orientation the entry path is the middle of the side lettered B, which faces away from the exit doorway's
/// side, C (`factions/cults/structures/armory.md > EntranceSide - one short side (B)`,
/// `factions/cults/structures/armory.md > ExitSide - opposite short side (C)`).
#[test]
fn the_entrance_turns_and_flips_with_the_armory() {
    for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
        for flip_h in [false, true] {
            let armory = armory(rotation, flip_h);
            let placement = armory.placement.unwrap();
            let letters = sides(ObjectKind::Armory, placement.orientation()).unwrap();
            let b = DIRECTIONS[letters.iter().position(|&l| l == SideLetter::B).unwrap()];
            let path = entry_path(&armory).unwrap();
            assert_eq!(path.outward, Vec2::new(b.0 as f32, b.1 as f32), "{rotation:?} {flip_h}");
            let door = Doorway::of(&armory).expect("its exit doorway").door();
            assert_eq!(door.outward(), -path.outward, "{rotation:?} {flip_h}");
            assert_eq!(path.spot + door.middle(), 2.0 * placement.center(), "opposite middles");
        }
    }
}

/// `systems/controls/commands.md > Enter`: "Cults — Armory, Cathedral: Recruits only (to be trained)"; Placeholder
/// (Q0104 point 4): one under construction takes none; and nothing but a building that holds units is one.
#[test]
fn an_armory_in_operation_takes_recruits_only() {
    let mut armory = armory(Rotation::R0, false);
    assert_eq!(admits(&armory, ObjectKind::Recruit), Ok(()));
    assert_eq!(admits(&armory, ObjectKind::Guard), Err(NoTransit::NotAccepted));
    assert_eq!(admits(&armory, ObjectKind::Storage), Err(NoTransit::NotAUnit));
    assert_eq!(
        crate::network::admits(&armory, ObjectKind::Recruit),
        Ok(()),
        "the one rule"
    );
    armory.construction = Some(Construction {
        progress: Progress::new(480),
        builder: Builder::Held(ObjectId(99)),
    });
    assert_eq!(admits(&armory, ObjectKind::Recruit), Err(NoTransit::NotATunnel));
    assert_eq!(entry_path(&armory).map(|path| path.spot), Some(Vec2::new(448.0, 320.0)));
    armory.construction = None;
    armory.owner = None;
    assert_eq!(admits(&armory, ObjectKind::Recruit), Err(NoTransit::NotATunnel));
    let mut storage = armory.clone();
    storage.kind = ObjectKind::Storage;
    assert_eq!(holds(&storage), None);
    assert_eq!(entry_path(&storage), None);
}
