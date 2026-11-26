//! The Syndicate's Tunnel Space available (slice M4-B; plan decision T33 as design changes 0242 and 0258 revised it;
//! `factions/syndicate/tunnels.md > Tunnel > Tunnel Space`, `factions/syndicate/overview.md > SyndicateResources >
//! Tunnel Space`): "Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4) —
//! rounded down to a whole number — capped at a hard maximum of 200".
//!
//! - **Which Tunnels count** ([`gives_tunnel_space`]): every Tunnel of the player's, at its current tier
//!   (`crate::tunnel::tier_def`; during an upgrade the tier is still the old one, slice M4-G). Slice M4-E adds "not a
//!   site" (`factions/syndicate/units/agent.md > Building > While the Tunnel is under construction`: "no Tunnel
//!   Space"). Another player's Tunnel never counts.
//! - **The region** of each: its Tunnel Area less its footprint (`crate::tunnel::area::region`), every cell counted,
//!   on the map or off it, Drillable or not: 0242, "So every tunnel-region cell is usable" (the off-map and
//!   non-Drillable cases of design changes 0238 and 0241 are void). Only a map or debug spawn can stand a Tunnel by
//!   Water or the edge, and its region counts whole all the same.
//! - **Unused** is a region cell under no structure that occupies tunnel-region ([`occupies_tunnel_region`]): an
//!   underground structure of the player's or a neutral one (design change 0258 point 4: "A neutral expansion partly
//!   inside an own Tunnel Area still blocks its cells and counts as occupied for Tunnel Space"), counted cell by cell,
//!   so that one partly inside takes only its cells inside. And the pop-up Laser Turret, slice M4-S2
//!   (`factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`: it "Occupies 1 grid unit of that Tunnel's
//!   tunnel-region, like an expansion"). Another player's expansion is never counted: areas never overlap (design
//!   change 0237) and expansions lie in their own Tunnel's area, and counting it would leak a hidden structure through
//!   the HUD. An expansion takes its cells from its first frame, construction starting at once (design changes 0254
//!   point 4 and 0255).
//! - **Over capacity** (`factions/syndicate/tunnels.md > Tunnel Space`: "existing units are unaffected — usage is
//!   simply displayed over capacity (e.g. 12/10) and no further units can be produced until usage drops back within
//!   capacity"): being derived, the figure needs no rule of its own, as for Beacon Capacity (`super`).
//!
//! The cells are counted per Tunnel and summed, as the formula reads: two Tunnels whose areas overlap, which only
//! spawns can make (0237), would count their shared cells twice.

use crate::defs::{CELLS_PER_TUNNEL_SPACE, TUNNEL_SPACE_CAP};
use crate::ids::PlayerId;
use crate::nav::TileRect;
use crate::object::Object;
use crate::popup;
use crate::tunnel;

/// Whether `o` gives its owner Tunnel Space: every Tunnel in operation (module docs; `Object::is_operational`: a
/// Tunnel under construction gives none, design change 0234, slice M4-E).
pub(crate) fn gives_tunnel_space(o: &Object) -> bool {
    tunnel::def_of(o.kind).is_some() && o.is_operational()
}

/// Whether `o` occupies tunnel-region for the Tunnels of `player`: an underground structure (`StructureDef::
/// underground`, plan decision T32) of theirs, or a neutral one (design change 0258 point 4); and a pop-up turret of
/// theirs (slice M4-S2; `factions/syndicate/structures/laser_turret.md > Upright vs Pop-up`: it "Occupies 1 grid unit
/// of that Tunnel's tunnel-region, like an expansion, so it reduces Tunnel Space (¼ Tunnel Space, subject to the
/// rounding rule in Tunnel Space)"). The one predicate of what makes a region cell used (module docs).
pub(crate) fn occupies_tunnel_region(o: &Object, player: PlayerId) -> bool {
    let underground = (o.def().structure.as_ref()).is_some_and(|structure| structure.underground);
    // (M4-S2) A turret of the player's with a pop-up form: one in operation, which inside an own Tunnel Area is a
    // pop-up (only cells of a region are counted, so an upright one outside every area takes none), and a Tunnel's
    // turret under construction, underground from its first frame as an expansion is (`popup`). An Agent's site
    // stands on the surface until it completes ("If built inside a Tunnel Area, it is a pop-up turret on completion").
    // Placeholder (Q0091 point 5): so a Tunnel's site takes its ¼ from its first frame, an Agent's at completion.
    let turret =
        popup::def_of(o.kind).is_some() && o.owner == Some(player) && (o.is_operational() || o.popup.is_some());
    turret || underground && (o.owner == Some(player) || o.owner.is_none())
}

/// What a player's Tunnel Space is worked out from, gathered object by object ([`TunnelSpace::see`]): their Tunnels,
/// and the footprints that occupy tunnel-region.
#[derive(Default)]
pub(super) struct TunnelSpace {
    /// Each Tunnel's footprint, and its Tunnel Area radius at its current tier.
    tunnels: Vec<(TileRect, u32)>,
    occupied: Vec<TileRect>,
}

impl TunnelSpace {
    /// Takes `o` into account, for `player`'s Tunnel Space.
    pub(super) fn see(&mut self, o: &Object, player: PlayerId) {
        let Some(placement) = o.placement else {
            return;
        };
        let footprint = TileRect::new(placement.origin, placement.w, placement.h);
        if o.owner == Some(player)
            && gives_tunnel_space(o)
            && let Some(tier) = tunnel::tier_def(o)
        {
            self.tunnels.push((footprint, tier.area_radius));
        }
        if occupies_tunnel_region(o, player) {
            self.occupied.push(footprint);
        }
    }

    /// The unused tunnel-region cells of all the Tunnels seen, Tunnel by Tunnel.
    pub(super) fn unused(&self) -> u32 {
        let mut unused = 0u32;
        for &(footprint, radius) in &self.tunnels {
            let area = tunnel::area::area(footprint, radius);
            // Only the structures that reach into the area.
            let near: Vec<TileRect> = (self.occupied.iter().copied())
                .filter(|&rect| tunnel::area::overlaps(area, rect))
                .collect();
            let free = tunnel::area::region(footprint, radius)
                .filter(|&cell| !near.iter().any(|&rect| tunnel::area::contains(rect, cell)))
                .count();
            unused = unused.saturating_add(u32::try_from(free).unwrap_or(u32::MAX));
        }
        unused
    }

    /// The Tunnel Space available: min([`TUNNEL_SPACE_CAP`], ⌊unused / [`CELLS_PER_TUNNEL_SPACE`]⌋).
    pub(super) fn available(&self) -> u32 {
        (self.unused() / CELLS_PER_TUNNEL_SPACE).min(TUNNEL_SPACE_CAP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::ObjectKind;
    use crate::economy::army::{ArmyCap, ArmyResource, cap};
    use crate::map::Map;
    use crate::player::Faction;
    use crate::scenario::SpawnAt;
    use crate::sim::Sim;
    use crate::testing::ScenarioBuilder;
    use crate::units::TilePos;

    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);
    const TUNNEL: ObjectKind = ObjectKind::Tunnel;
    const HIDEOUT: ObjectKind = ObjectKind::Hideout;
    const AGENT: ObjectKind = ObjectKind::Agent;

    fn tile(x: i32, y: i32) -> SpawnAt {
        SpawnAt::Tile(TilePos::new(x, y))
    }

    /// Both players play the Syndicate, on a flat `w` × `h` map.
    fn syndicate(w: usize, h: usize) -> ScenarioBuilder {
        ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
            .faction(P1, Faction::Syndicate)
            .faction(P2, Faction::Syndicate)
    }

    /// The objects the map of `builder` starts with, by spawns: exempt from the placement rules, design change 0242's
    /// included.
    fn objects(builder: ScenarioBuilder) -> Vec<Object> {
        Sim::new(&builder.build()).objects().cloned().collect()
    }

    /// The unused tunnel-region cells of `player`'s Tunnels among `objects`.
    fn unused(objects: &[Object], player: PlayerId) -> u32 {
        let mut space = TunnelSpace::default();
        for o in objects {
            space.see(o, player);
        }
        space.unused()
    }

    /// `player`'s Tunnel Space among `objects`, on a map it doesn't look at.
    fn tunnel_space(objects: &[Object], player: PlayerId) -> ArmyCap {
        cap(objects, &Map::new(1, 1), player, Faction::Syndicate)
    }

    /// `factions/syndicate/tunnels.md > Tunnel Space > Tunnel-region` at each tier (`factions/syndicate/tunnels.md >
    /// Tiers`: radius 2, 3, 4): 48, 84 and 128 unused cells, so 12, 21 and 32 Tunnel Space for a Tunnel alone. The tier
    /// is the Tunnel's current one.
    #[test]
    fn each_tier_gives_its_region() {
        let mut objects = objects(syndicate(24, 24).spawn(TUNNEL, Some(P1), tile(10, 10)));
        let mut found = Vec::new();
        for tier in 1..=3 {
            assert!(crate::tunnel::set_tier(&mut objects[0], tier));
            found.push((unused(&objects, P1), tunnel_space(&objects, P1).available));
        }
        assert_eq!(found, [(48, 12), (84, 21), (128, 32)]);
        let none = tunnel_space(&objects, P2);
        assert_eq!(
            (none.resource, none.used, none.available),
            (ArmyResource::TunnelSpace, 0, 0)
        );
    }

    /// An own Hideout inside the area takes its 4 cells (`factions/syndicate/tunnels.md > Tunnel Space`: "Building an
    /// expansion permanently converts part of a Tunnel's unused tunnel-region into occupied space"): 44 cells, 11
    /// Tunnel Space, as the Starting Conditions have it. Another player's expansion is never counted (slice M4-B: it
    /// can't lie in an own area, and would leak through the HUD), and nor is an own structure that isn't underground.
    #[test]
    fn an_own_expansion_takes_its_cells_and_another_players_none() {
        let base = || syndicate(16, 16).spawn(TUNNEL, Some(P1), tile(4, 4));
        let own = objects(base().spawn(HIDEOUT, Some(P1), tile(8, 8)));
        assert_eq!((unused(&own, P1), tunnel_space(&own, P1).available), (44, 11));
        let enemy = objects(base().spawn(HIDEOUT, Some(P2), tile(8, 8)));
        assert_eq!((unused(&enemy, P1), tunnel_space(&enemy, P1).available), (48, 12));
        let surface = objects(base().spawn(ObjectKind::Generator, Some(P1), tile(8, 8)));
        assert_eq!(unused(&surface, P1), 48);
    }

    /// Design change 0258 point 4: "A neutral expansion partly inside an own Tunnel Area still blocks its cells and
    /// counts as occupied for Tunnel Space": a neutral Hideout with 2 of its 4 cells in the area (x 2 to 9) takes 2.
    /// With 1 cell in, 47 are left: rounded down, 11 (`factions/syndicate/tunnels.md > Tunnel Space`: "rounded down to
    /// a whole number").
    #[test]
    fn a_neutral_expansion_takes_its_cells_inside_the_area() {
        let base = || syndicate(16, 16).spawn(TUNNEL, Some(P1), tile(4, 4));
        let half = objects(base().spawn(HIDEOUT, None, tile(1, 4))); // (1, 4) to (2, 5)
        assert_eq!((unused(&half, P1), tunnel_space(&half, P1).available), (46, 11));
        let corner = objects(base().spawn(HIDEOUT, None, tile(1, 1))); // (1, 1) to (2, 2)
        assert_eq!((unused(&corner, P1), tunnel_space(&corner, P1).available), (47, 11));
        let out = objects(base().spawn(HIDEOUT, None, tile(0, 0)));
        assert_eq!(unused(&out, P1), 48);
    }

    /// `factions/syndicate/tunnels.md > Tunnel Space`: "sum of unused tunnel-region grid units across all owned
    /// Tunnels": two Tunnels give 96 cells, 24 Tunnel Space; another player's Tunnel gives nothing.
    #[test]
    fn the_owners_tunnels_add_up() {
        let two = objects(
            syndicate(24, 16)
                .spawn(TUNNEL, Some(P1), tile(2, 4))
                .spawn(TUNNEL, Some(P1), tile(10, 4))
                .spawn(TUNNEL, Some(P2), tile(18, 4)),
        );
        assert_eq!((unused(&two, P1), tunnel_space(&two, P1).available), (96, 24));
        assert_eq!(tunnel_space(&two, P2).available, 12);
    }

    /// "Capped at a hard maximum of 200": 16 Tier 1 Tunnels give 768 cells, 192; 17 give 816, past 800, and 200.
    #[test]
    fn tunnel_space_caps_at_200() {
        let tunnels = |n: i32| {
            let mut builder = syndicate(48, 32);
            for i in 0..n {
                builder = builder.spawn(TUNNEL, Some(P1), tile(2 + 8 * (i % 6), 2 + 8 * (i / 6)));
            }
            objects(builder)
        };
        let sixteen = tunnels(16);
        assert_eq!((unused(&sixteen, P1), tunnel_space(&sixteen, P1).available), (768, 192));
        let seventeen = tunnels(17);
        assert_eq!(
            (unused(&seventeen, P1), tunnel_space(&seventeen, P1).available),
            (816, 200)
        );
    }

    /// Design change 0242: "So every tunnel-region cell is usable": a Tunnel that only a spawn can stand by the map's
    /// edge or by Water counts its whole region, 48 cells.
    #[test]
    fn a_spawned_tunnel_by_the_edge_or_water_counts_its_whole_region() {
        let edge = objects(syndicate(16, 16).spawn(TUNNEL, Some(P1), tile(0, 0)));
        assert_eq!((unused(&edge, P1), tunnel_space(&edge, P1).available), (48, 12));
        let lake = ScenarioBuilder::new(&vec!["~~~.............".to_string(); 16].join("\n"))
            .faction(P1, Faction::Syndicate)
            .spawn(TUNNEL, Some(P1), tile(4, 4));
        let lake = objects(lake);
        assert_eq!((unused(&lake, P1), tunnel_space(&lake, P1).available), (48, 12));
    }

    /// Used (`factions/syndicate/tunnels.md > Production and Tunnel Space`: "the unit's Tunnel Space counts as used
    /// from that moment — through production and for as long as the unit lives"): the player's units by their
    /// TunnelSpaceCost, and their producers' units in production (design change 0143), not those only queued; a unit of
    /// another faction costs none. No Syndicate building produces yet (slice M4-C), so a Barracks of the player's
    /// stands in for one.
    #[test]
    fn used_counts_the_units_and_the_units_in_production() {
        let mut objects = objects(
            syndicate(24, 16)
                .spawn(TUNNEL, Some(P1), tile(2, 2))
                .spawn(ObjectKind::Barracks, Some(P1), tile(14, 2))
                .spawn(AGENT, Some(P1), SpawnAt::Point(TilePos::new(12, 10).center()))
                .spawn(
                    ObjectKind::Guard,
                    Some(P1),
                    SpawnAt::Point(TilePos::new(14, 10).center()),
                )
                .spawn(
                    ObjectKind::Peacekeeper,
                    Some(P1),
                    SpawnAt::Point(TilePos::new(16, 10).center()),
                )
                .spawn(AGENT, Some(P2), SpawnAt::Point(TilePos::new(18, 10).center())),
        );
        assert_eq!(tunnel_space(&objects, P1).used, 2);
        let production = objects[1].production.as_mut().expect("a producer");
        production.queue = vec![AGENT, AGENT];
        assert_eq!(tunnel_space(&objects, P1).used, 2, "only queued");
        objects[1].production.as_mut().unwrap().progress = 1;
        assert_eq!(tunnel_space(&objects, P1).used, 3, "the first in production");
    }
}
