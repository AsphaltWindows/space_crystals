//! Where units go into and come out of a Tunnel (`crate::network`): Side A's two paths ([`Paths`], design changes 0225
//! and 0248), (M6-SD) the two air doors on its roof ([`Paths::air_entry`], [`Paths::air_exit`], [`over_door`]), and the
//! way into any structure that units Enter ([`EntryPath`], slice M5-F1). Split from `network.rs` (plan decision T28).
//!
//! **The air doors** (plan decision T96; `factions/syndicate/tunnels.md > Tunnel > Look by tier`: "**Tier 3**: adds
//! **two horizontal circular sliding doors**, **one for flying units entering and one for flying units exiting**.
//! Aircraft use these doors, not the Side A paths"; `factions/syndicate/tunnel_menus.md > EjectMenu`: "The **air doors
//! are on the Tunnel's roof**: the entry door over the entry path's half of the Tunnel, the exit door over the exit
//! path's half"): two discs of `Tunnel::air_door_radius_su`, each `Tunnel::air_door_offset_su` from the Tunnel's centre
//! towards the side its path's half is nearer, D for the entry and B for the exit, halfway between Sides A and C. They
//! turn and flip with the Tunnel, as its paths do. Every Tunnel has their spots; only a Tier 3 one lets an aircraft
//! through them (`network::admits`).

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::map::Map;
use crate::object::{Object, Placement};
use crate::ruleset::rules;
use crate::sides::{DIRECTIONS, SideLetter, sides};
use crate::units::SU_PER_GU;

/// How far from the Tunnel's wall a unit stands on a path's spot, space units, so that no rounding makes its box
/// overlap the footprint (a technical margin: a sixty-fourth of a space unit, as a doorway's).
pub(crate) const WALL_CLEARANCE: f32 = 1.0 / 64.0;

/// The ways in and out of a placed Tunnel (module docs): the points on Side A's line where units go in and come out,
/// the direction the side faces, and (M6-SD) the centres of its two air doors.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Paths {
    /// The unit vector out of Side A.
    pub outward: Vec2,
    /// The entry path's spot: on Side A's line, `Tunnel::path_offset_su` from its middle towards Side D.
    pub entry: Vec2,
    /// The exit path's spot: on Side A's line, `Tunnel::path_offset_su` from its middle towards Side B.
    pub exit: Vec2,
    /// (M6-SD) The air entry door's centre, "over the entry path's half": `Tunnel::air_door_offset_su` from the
    /// Tunnel's centre towards Side D.
    pub air_entry: Vec2,
    /// (M6-SD) The air exit door's centre, "over the exit path's half": `Tunnel::air_door_offset_su` from the
    /// Tunnel's centre towards Side B.
    pub air_exit: Vec2,
}

impl Paths {
    /// The paths of a `kind` structure placed at `placement`: `None` unless its sides are lettered A, B and D (the
    /// Tunnel's ABCD), wherever they face (`crate::sides::sides`).
    pub fn of(kind: ObjectKind, placement: &Placement) -> Option<Paths> {
        let letters = sides(kind, placement.orientation())?;
        let facing = |letter: SideLetter| {
            let i = letters.iter().position(|&l| l == letter)?;
            let (x, y) = DIRECTIONS[i];
            Some(Vec2::new(x as f32, y as f32))
        };
        let (outward, towards_b, towards_d) = (facing(SideLetter::A)?, facing(SideLetter::B)?, facing(SideLetter::D)?);
        let half = Vec2::new(placement.w as f32, placement.h as f32) * (SU_PER_GU / 2.0);
        let centre = placement.center();
        let middle = centre + outward * outward.abs().dot(half);
        let tuning = &rules().tuning.tunnel;
        let (offset, door) = (tuning.path_offset_su, tuning.air_door_offset_su);
        Some(Paths {
            outward,
            entry: middle + towards_d * offset,
            exit: middle + towards_b * offset,
            air_entry: centre + towards_d * door,
            air_exit: centre + towards_b * door,
        })
    }

    /// The paths of `tunnel`, a placed Tunnel.
    pub fn of_object(tunnel: &Object) -> Option<Paths> {
        tunnel.tunnel.as_ref()?;
        Paths::of(tunnel.kind, tunnel.placement.as_ref()?)
    }

    /// The entry path, as every way in is ([`EntryPath`]), with (M6-SD) the air entry door.
    pub fn entry_path(&self) -> EntryPath {
        EntryPath {
            outward: self.outward,
            spot: self.entry,
            standoff: 0.0,
            air_door: Some(self.air_entry),
        }
    }

    /// Where an entering unit with half extents `half` walks to: on the entry path's spot, against the wall.
    pub fn entry_stand(&self, half: Vec2) -> Vec2 {
        self.entry_path().stand(half)
    }

    /// Where an ejected unit with half extents `half` appears (design change 0250: "Each unit appears at the exit
    /// path's spot just outside Side A"): on the exit path's spot, against the wall.
    pub fn exit_stand(&self, half: Vec2) -> Vec2 {
        let exit = EntryPath {
            spot: self.exit,
            ..self.entry_path()
        };
        exit.stand(half)
    }

    /// Whether a unit with half extents `half` at `pos` has walked into the entry path ([`EntryPath::reached`]).
    pub fn reached_entry(&self, pos: Vec2, half: Vec2) -> bool {
        self.entry_path().reached(pos, half)
    }

    /// Where a unit that appeared at `stand` with no order walks to: `Tunnel::walk_out_su` "straight out of Side A".
    pub fn walk_out(&self, stand: Vec2) -> Vec2 {
        stand + self.outward * rules().tuning.tunnel.walk_out_su
    }
}

/// (M6-SD) Whether an aircraft centred on `pos` is over the air door centred on `door` (module docs): within
/// `Tunnel::air_door_radius_su` of its centre (the plan's D46, which design change 0431 keeps: "an air unit goes in
/// once its centre is over the entry door").
pub fn over_door(door: Vec2, pos: Vec2) -> bool {
    let radius = rules().tuning.tunnel.air_door_radius_su;
    pos.distance_squared(door) <= radius * radius
}

/// (M6-SD) Where at the air door centred on `door` an aircraft whose box has the half extents `half` comes out or goes
/// in: the door's centre, or, where a tuned `Tunnel::air_door_offset_su` puts the box past the map's edge, the point
/// nearest it at which the box lies on the map (`Map::clamp_box`, slice M6-GK; `systems/units.md > Air Collision > Map
/// edge`: "HoverCraft stay inside the map"), where its flight there ends too (`paths::serve`). With the built-in
/// offset a door's centre lies at least 76 su inside its Tunnel's footprint, so on the map for any box up to 152 su
/// across.
pub fn door_spot(map: &Map, door: Vec2, half: Vec2) -> Vec2 {
    map.clamp_box(door, half)
}

/// The way into a structure that units Enter (`systems/controls/commands.md > Enter`: "The unit walks to the
/// structure's entrance side and goes inside"), M4-C's entry path generalized (slice M5-F1): a spot on the line of the
/// side units go in by, and the direction out of that side. A Tunnel's is its entry path on Side A
/// ([`Paths::entry_path`]); a building that holds units has one in the middle of its entrance side
/// (`holding::entry_path`). The Enter order walks to it (`orders::enter`) and the step right after 5a takes in the
/// units that reached it (`Sim::step_entries`), one rule for both. (M6-SD) A Tunnel's has its air entry door too, the
/// way its aircraft go in.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntryPath {
    /// The unit vector out of the side.
    pub outward: Vec2,
    /// The spot, on the side's line.
    pub spot: Vec2,
    /// How far clear of the wall an entering unit's walk ends ([`EntryPath::stand`]), space units: none for a
    /// Tunnel's, whose units walk up against it (M4-C); `Holding::entry_standoff_su` for a building that holds units.
    pub standoff: f32,
    /// (M6-SD) The centre of the air entry door an air unit goes in by ([`Paths::air_entry`]): a Tunnel's; `None` for
    /// a building that holds units, which takes no air unit in.
    pub air_door: Option<Vec2>,
}

impl EntryPath {
    /// The entry path of `target`: a placed Tunnel's ([`Paths::of_object`]), or a placed building's that holds units
    /// (`holding::entry_path`); `None` for anything else.
    pub fn of_object(target: &Object) -> Option<EntryPath> {
        match Paths::of_object(target) {
            Some(paths) => Some(paths.entry_path()),
            None => crate::holding::entry_path(target),
        }
    }

    /// Where an entering unit with half extents `half` walks to: on the spot, "just outside" the side, its box
    /// [`EntryPath::standoff`] clear of the wall (against it, for a Tunnel).
    pub fn stand(&self, half: Vec2) -> Vec2 {
        self.spot + self.outward * (self.outward.abs().dot(half) + WALL_CLEARANCE + self.standoff)
    }

    /// Whether a unit with half extents `half` at `pos` has walked into the path: its box is within
    /// `Tunnel::entry_reach_su` of the spot.
    pub fn reached(&self, pos: Vec2, half: Vec2) -> bool {
        let apart = (pos - self.spot).abs() - half;
        apart.max(Vec2::ZERO).length() <= rules().tuning.tunnel.entry_reach_su
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::Rotation;
    use crate::scenario::BuildingOrientation;
    use crate::units::TilePos;
    const PATH_OFFSET_SU: f32 = crate::tuning::tunnel::BUILTIN.path_offset_su;
    const DOOR_OFFSET_SU: f32 = crate::tuning::tunnel::BUILTIN.air_door_offset_su;

    fn tunnel_at(rotation: Rotation, flip_h: bool) -> Placement {
        let turned = BuildingOrientation {
            rotation,
            flip_h,
            flip_v: false,
        };
        Placement {
            rotation: turned.rotation,
            flip_h: turned.flip_h,
            ..Placement::new(TilePos::new(4, 4), ObjectKind::Tunnel.def().footprint)
        }
    }

    /// Design change 0248: unturned, Side A is the top; the exit path is its half nearer Side B (east), the entry
    /// path the half nearer Side D (west). The footprint covers tiles 4..8, so the side's line is y = 256 and its
    /// middle x = 384. (M6-SD) The air doors lie on the line across the middle of the roof, y = 384, the exit's 52 su
    /// east of the centre and the entry's 52 su west, each its 24 su about it.
    #[test]
    fn the_exit_is_nearer_b_and_the_entry_nearer_d() {
        let paths = Paths::of(ObjectKind::Tunnel, &tunnel_at(Rotation::R0, false)).unwrap();
        assert_eq!(paths.outward, Vec2::new(0.0, -1.0));
        assert_eq!(paths.exit, Vec2::new(384.0 + PATH_OFFSET_SU, 256.0));
        assert_eq!(paths.entry, Vec2::new(384.0 - PATH_OFFSET_SU, 256.0));
        assert_eq!(
            (paths.air_exit, paths.air_entry),
            (Vec2::new(436.0, 384.0), Vec2::new(332.0, 384.0))
        );
        assert!(over_door(paths.air_entry, Vec2::new(332.0 + 24.0, 384.0)));
        assert!(!over_door(paths.air_entry, Vec2::new(332.0 + 24.1, 384.0)));
        assert!(over_door(paths.air_exit, Vec2::new(436.0 - 16.0, 384.0 + 16.0)));
        assert!(
            !over_door(paths.air_entry, paths.air_exit),
            "the exit door is no way in"
        );
        // A Guard (36 x 36) stands with its box against the wall.
        let half = Vec2::splat(18.0);
        assert_eq!(paths.exit_stand(half), Vec2::new(414.0, 256.0 - 18.0 - WALL_CLEARANCE));
        assert_eq!(paths.entry_stand(half), Vec2::new(354.0, 256.0 - 18.0 - WALL_CLEARANCE));
        assert!(paths.reached_entry(paths.entry_stand(half), half));
        assert!(
            !paths.reached_entry(paths.exit_stand(half), half),
            "the exit is no way in"
        );
        assert_eq!(
            paths.walk_out(paths.exit_stand(half)).y,
            256.0 - 18.0 - WALL_CLEARANCE - 64.0
        );
        assert_eq!(paths.entry_path().air_door, Some(paths.air_entry));
    }

    /// "Both turn and flip with the Tunnel": in each of the 8 orientations the two spots lie on the side A faces,
    /// the exit towards B and the entry towards D; (M6-SD) and the two air doors on the roof's middle line across from
    /// D to B, the exit's towards B and the entry's towards D.
    #[test]
    fn the_paths_turn_and_flip_with_the_tunnel() {
        let centre = Vec2::new(384.0, 384.0);
        for rotation in [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270] {
            for flip_h in [false, true] {
                let placement = tunnel_at(rotation, flip_h);
                let letters = sides(ObjectKind::Tunnel, placement.orientation()).unwrap();
                let facing = |letter| {
                    let (x, y) = DIRECTIONS[letters.iter().position(|&l| l == letter).unwrap()];
                    Vec2::new(x as f32, y as f32)
                };
                let paths = Paths::of(ObjectKind::Tunnel, &placement).unwrap();
                assert_eq!(paths.outward, facing(SideLetter::A), "{rotation:?} {flip_h}");
                for (spot, towards) in [(paths.exit, SideLetter::B), (paths.entry, SideLetter::D)] {
                    let from_centre = spot - centre;
                    assert_eq!(from_centre.dot(paths.outward), 128.0, "on Side A's line");
                    assert_eq!(
                        from_centre.dot(facing(towards)),
                        PATH_OFFSET_SU,
                        "{rotation:?} {flip_h}"
                    );
                }
                for (door, towards) in [(paths.air_exit, SideLetter::B), (paths.air_entry, SideLetter::D)] {
                    assert_eq!(door - centre, facing(towards) * DOOR_OFFSET_SU, "{rotation:?} {flip_h}");
                }
            }
        }
        // Nothing but a Tunnel has the two paths.
        let hideout = Placement::new(TilePos::new(0, 0), ObjectKind::Hideout.def().footprint);
        assert_eq!(Paths::of(ObjectKind::Hideout, &hideout), None);
    }
}
