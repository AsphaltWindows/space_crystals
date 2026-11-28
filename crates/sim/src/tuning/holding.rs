//! (M5-X1) Buildings that hold units (plan decision T60, `crate::holding`): where a unit's walk into one ends; and
//! (M6-ST1) where a carrier's passengers come out round it (plan decision T94, `crate::carry`). The sides, the
//! capacities and the kinds a building takes are its def's (`defs::HoldsDef`), a carrier's places its own
//! (`defs::CarrierDef`).

tunables! {
    /// The tunables of holding (module docs).
    Holding;

    /// (M5-F1) How far clear of the wall a unit's walk into a building that holds units ends (`EntryPath::stand`),
    /// space units: half of the Tunnel's entry reach (`Tunnel::entry_reach_su`, 8 su), within which it goes in all the
    /// same. Placeholder (not in the design: where a unit on its way in stops is the implementation's). Against the
    /// wall, as at a Tunnel (M4-C), the spot is one that a unit pressed along the wall by a crowd coming round a corner
    /// of the entrance side can't get round the corner to: it slid towards it ever more slowly and gave up (slice
    /// M5-F1's QA run, `tests/holding/entering.rs`: two of eleven Recruits walking round an Armory to side B).
    entry_standoff_su: f32 = 8.0 / 2.0,
    /// (M6-ST1) How far clear of its carrier's shape, and of each other, the passengers an Unload All puts on the map
    /// come out (`carry::ring_spots`), space units: the gap between a ring of spots round the carrier and its shape,
    /// between one ring and the next, and between two spots of a ring, at the least. Placeholder (not in the design:
    /// `factions/syndicate/units/subterranean_transport.md > Placeholder stats` has "they come out around it"): half
    /// the Proximity (`Orders::proximity_su`, 8 su), within which a unit boards.
    unload_gap_su: f32 = 8.0 / 2.0,
    /// (M6-ST1) How many rings of spots round its carrier a passenger an Unload All puts on the map is tried at, the
    /// nearest first (`carry::ring_spots`): one for whom none has a free spot stays aboard
    /// (`systems/mechanics/carrying.md > Unloading`: "the **nearest free spots** around the carrier"; "passengers with
    /// no free spot **stay aboard**"). Placeholder (not in the design: how far round the carrier a free spot is
    /// sought).
    unload_rings: u32 = 3,
}
