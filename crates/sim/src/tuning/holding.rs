//! (M5-X1) Buildings that hold units (plan decision T60, `crate::holding`): where a unit's walk into one ends. The
//! sides, the capacities and the kinds a building takes are its def's (`defs::HoldsDef`).

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
}
