//! The reference layouts of the Syndicate's economy check (`factions/syndicate/units/agent.md > Gathering > Space
//! Crystals > Syndicate economy check > Reference layout`, design changes 0259 and 0294): one Tunnel facing a patch at
//! a distance, the Syndicate's start on it, and two Tunnels stacked on one field.

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Rotation;
use crate::player::Faction;
use crate::scenario::{BuildingOrientation, Scenario, SpawnAt, SpawnSettings};
use crate::testing::ScenarioBuilder;
use crate::units::{SU_PER_GU, TilePos};

/// The player of the reference layouts.
pub const FIELD_PLAYER: PlayerId = PlayerId(1);
/// The Tunnel of [`reference_field`] and [`opening_field`]; the western one of [`stacked_field`].
pub const FIELD_TUNNEL: ObjectId = ObjectId(1);
/// The patch of [`reference_field`] and [`opening_field`]; the western one of [`stacked_field`].
pub const FIELD_PATCH: ObjectId = ObjectId(2);
/// [`opening_field`]'s Hideout.
pub const FIELD_HIDEOUT: ObjectId = ObjectId(3);
/// [`stacked_field`]'s eastern patch.
pub const STACKED_PATCH: ObjectId = ObjectId(3);
/// [`stacked_field`]'s eastern Tunnel.
pub const STACKED_TUNNEL: ObjectId = ObjectId(4);
/// The patches of the reference layouts hold so much that none runs out.
pub const FIELD_RESERVE: u32 = 1_000_000;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n")).faction(FIELD_PLAYER, Faction::Syndicate)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn patch() -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: FIELD_RESERVE,
    }
}

/// `factions/syndicate/units/agent.md > Gathering > Space Crystals > Syndicate economy check > Reference layout`
/// (design change 0259): "one Tunnel with its crystal hatch facing a single patch at each distance", here `distance`
/// grid units from the hatch square, edge to edge (0259: "measured edge to edge, from the patch's edge to the Tunnel's
/// crystal hatch"), on a flat 30 × 20 map. The Tunnel ([`FIELD_TUNNEL`]) is unturned at tiles (6, 6) to (9, 9), Side A
/// to the north and Side B to the east, so its crystal hatch is tile (9, 8) (`tunnel::hatch_square`); the patch
/// ([`FIELD_PATCH`], [`FIELD_RESERVE`]) lies east of it in the same row, on tile (10 + `distance`, 8). The `agents`
/// (ids 3 on) start in a row south-east of the Tunnel, on the centres of tiles (12, 12), (13, 12) and on, as
/// `tests/agent_gathering/` stands them. Player 1 has the Syndicate's starting banks. "The reference layout is only a
/// benchmark" (design change 0108): nothing in the game assumes it.
pub fn reference_field(distance: i32, agents: u32) -> Scenario {
    let mut builder = flat(30, 20)
        .spawn(ObjectKind::Tunnel, Some(FIELD_PLAYER), tile(6, 6))
        .spawn_with(patch(), tile(10 + distance, 8));
    for i in 0..agents as i32 {
        builder = builder.spawn(
            ObjectKind::Agent,
            Some(FIELD_PLAYER),
            SpawnAt::Point(TilePos::new(12 + i, 12).center()),
        );
    }
    builder.build()
}

/// [`reference_field`]'s Agents, when it has `agents`.
pub fn field_agents(agents: u32) -> Vec<ObjectId> {
    (0..agents).map(|i| ObjectId(3 + i)).collect()
}

/// The Syndicate's start on the reference layout (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`:
/// "One pre-built Tier 1 Tunnel containing one pre-built Hideout", "One Agent", 1 Supply and 100 Space Crystals, as
/// `ScenarioBuilder::faction` gives them): [`reference_field`]'s Tunnel and patch, `distance` grid units from the
/// hatch; the Hideout ([`FIELD_HIDEOUT`]) on tiles (4, 10) to (5, 11), the south-west corner of the Tunnel's Tier 1
/// area ((4, 4) to (11, 11)), off the units' ways; and the Agent (id 4) 1.5 grid units north of Side A's middle, at
/// (8, 4.5) grid units, where `dev_skirmish`'s Starting Layout stands its Agent off its Tunnel's Side A.
pub fn opening_field(distance: i32) -> Scenario {
    flat(30, 20)
        .spawn(ObjectKind::Tunnel, Some(FIELD_PLAYER), tile(6, 6))
        .spawn_with(patch(), tile(10 + distance, 8))
        .spawn(ObjectKind::Hideout, Some(FIELD_PLAYER), tile(4, 10))
        .spawn(
            ObjectKind::Agent,
            Some(FIELD_PLAYER),
            SpawnAt::Point(Vec2::new(8.0, 4.5) * SU_PER_GU),
        )
        .build()
}

/// The stacked check's layout (0259: "the two Tunnels stand on opposite sides of the field, each hatch facing it, so
/// every Agent's nearest Tunnel is its own"; `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule > Crystal
/// intake and stacked Tunnels`): [`reference_field`]'s Tunnel and patch, `distance` grid units apart; a second patch
/// ([`STACKED_PATCH`]) on tile (12 + `distance`, 8), east of the first across a free column; and a second Tunnel
/// ([`STACKED_TUNNEL`]) flipped left and right on tiles (13 + 2 × `distance`, 6) on, its crystal hatch on its west
/// side, `distance` grid units from the second patch. Each patch lies wholly on its own Tunnel's side of the line
/// halfway between the hatches, so an Agent at any of its faces has its own Tunnel nearest; with patches side by side,
/// the ends of their north and south faces would lie as near to either hatch. With `distance` 1 or more the Tier 1
/// areas don't overlap. `agents` Agents for each Tunnel (ids 5 on, the western Tunnel's first) start in a row south
/// of the field, the western Tunnel's west of the free column and the eastern's east of it.
pub fn stacked_field(distance: i32, agents: u32) -> Scenario {
    let east = 13 + 2 * distance;
    let mut builder = flat(east as usize + 10, 20)
        .spawn(ObjectKind::Tunnel, Some(FIELD_PLAYER), tile(6, 6))
        .spawn_with(patch(), tile(10 + distance, 8))
        .spawn_with(patch(), tile(12 + distance, 8))
        .spawn(ObjectKind::Tunnel, Some(FIELD_PLAYER), tile(east, 6));
    for (from, step) in [(10 + distance, -1), (12 + distance, 1)] {
        for i in 0..agents as i32 {
            let at = SpawnAt::Point(TilePos::new(from + step * i, 13).center());
            builder = builder.spawn(ObjectKind::Agent, Some(FIELD_PLAYER), at);
        }
    }
    let mut scenario = builder.build();
    scenario.map.spawns[3].orientation = BuildingOrientation {
        rotation: Rotation::R0,
        flip_h: true,
        flip_v: false,
    };
    scenario
}

/// [`stacked_field`]'s Agents of each Tunnel, when it has `agents` for each: the western Tunnel's, the eastern's.
pub fn stacked_agents(agents: u32) -> (Vec<ObjectId>, Vec<ObjectId>) {
    let west = (0..agents).map(|i| ObjectId(5 + i)).collect();
    let east = (0..agents).map(|i| ObjectId(5 + agents + i)).collect();
    (west, east)
}
