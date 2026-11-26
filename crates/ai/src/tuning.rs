//! The computer players' numbers, in one place (`core/game_rules.md > Computer players (playtest tool)`: "Their
//! behavior (profiles such as standard, rush and passive; build orders) is the Director's placeholder"). Every one is a
//! placeholder: "Placeholder (Q0147 point N)" for a point the Director's question raised (2: how it plays; 3: build
//! orders), "Placeholder (not in the design: why)" for a detail of the computer player's own.

/// The army layer (`crate::army`): the staging point, the waves and the defense.
pub mod army {
    use sc_sim::Tick;

    /// Placeholder (Q0147 point 2: "pulls its army home when its base is attacked"; the brief's "a staging point
    /// between its base and the map's center"): how far the army gathers from its start toward the map's centre, grid
    /// units, and at most halfway there ([`STAGING_SHARE`]).
    pub const STAGING_DISTANCE_GU: f32 = 12.0;
    /// Placeholder (Q0147 point 2): at most this share of the way from the start to the map's centre.
    pub const STAGING_SHARE: f32 = 0.5;
    /// Placeholder (not in the design: how far a unit may stand from the staging point and still wait there), grid
    /// units.
    pub const STAGING_RADIUS_GU: f32 = 6.0;
    /// Placeholder (not in the design: no building of the computer player's own goes this near the staging point, which
    /// its units fill), grid units.
    pub const STAGING_CLEAR_GU: f32 = 4.0;
    /// Placeholder (Q0147 point 2: "attacks in waves of about 8 units"): the units waiting at staging that make the
    /// first wave.
    pub const FIRST_WAVE_SIZE: usize = 8;
    /// Placeholder (Q0147 point 2: "then reinforces"): the units waiting at staging that make each later wave. More
    /// than "about 8": a wave meets the enemy's whole army at its staging point, and waves of 8 trade about evenly
    /// with a passive player's, so that on `dev_skirmish` neither side wins in 20 minutes; waves of 32 beat it within
    /// 15 minutes on every Skirmish Default map, from either slot (`crates/ai/tests/games.rs`, `ai_games`).
    pub const WAVE_SIZE: usize = 32;
    /// Placeholder (not in the design: once units have waited at staging this long, they go as a wave however few they
    /// are, as once the fields run dry the wave fills slowly or never), ticks (2 minutes; two Barracks fill a wave of
    /// 32 in 80 seconds).
    pub const WAVE_PATIENCE_TICKS: Tick = 1920;
    /// Placeholder (not in the design: a wave with nothing known to attack searches the patches this near an enemy's
    /// start, where its last buildings may stand unseen), grid units.
    pub const SEARCH_RADIUS_GU: f32 = 20.0;
    /// Placeholder (Q0147 point 2: "pulls its army home when its base is attacked"): how long the defense lasts after
    /// the last damage to one of its buildings, ticks (10 seconds).
    pub const DEFENSE_TICKS: Tick = 160;
    /// Placeholder (not in the design: a defense moves to the newly attacked building when it is this far from the
    /// last one), grid units.
    pub const DEFENSE_MOVE_GU: f32 = 4.0;
    /// Placeholder (not in the design: a unit at home that stands idle away from the staging point is sent back to it,
    /// at most this often), ticks (4 seconds).
    pub const GATHER_RETRY_TICKS: Tick = 64;
}

/// The build layer (`crate::build`): where a building goes.
pub mod build {
    use sc_sim::Tick;

    /// Placeholder (not in the design: a free ring around each building keeps lanes open for the units), tiles.
    pub const MARGIN: i32 = 1;
    /// Placeholder (not in the design: the most placement checks one search for a spot makes, so that a full base
    /// costs little).
    pub const SPOT_CHECKS: usize = 400;
    /// Placeholder (not in the design: after a search finds no spot, the next waits this long), ticks (1 second).
    pub const SPOT_RETRY_TICKS: Tick = 16;
    /// Placeholder (not in the design: a production building stands this share of the way from its constructor
    /// toward the staging point, so that its units leave toward it).
    pub const PRODUCER_SHARE: f32 = 0.5;
}

/// The GDO's computer player (`crate::factions::gdo`).
pub mod gdo {
    use sc_sim::ObjectKind;

    /// Placeholder (Q0147 point 3: "the GDO's 'Plate, Generator, Generator, Barracks'", `factions/gdo/economy.md >
    /// Starting Conditions > Opening Economy Baseline`: "Fastest clean Barracks (Plate, Generator, Generator,
    /// Barracks)"): the Deployment Center's opening, while the Extraction Facility builds its plates.
    pub const OPENING: [ObjectKind; 3] = [ObjectKind::Generator, ObjectKind::Generator, ObjectKind::Barracks];
    /// Placeholder (Q0147 point 2: "one or two production buildings"): the Barracks it keeps.
    pub const BARRACKS: usize = 2;
    /// Placeholder (Q0147 point 2; the brief: "a second Barracks once its income allows"): the plates it owns before
    /// the second Barracks, or every patch of its field plated if it has fewer. Six plates mine 20 Space Crystals a
    /// second, what two Barracks spend on Peacekeepers.
    pub const SECOND_BARRACKS_PLATES: usize = 6;
    /// Placeholder (Q0147 point 2: "training its basic units without pause"): the Peacekeepers each Barracks keeps
    /// queued.
    pub const TRAIN_QUEUE: usize = 2;
}

/// The Syndicate's computer player (`crate::factions::syndicate`).
pub mod syndicate {
    use sc_sim::{ObjectKind, Tick};

    /// Placeholder (Q0147 point 2: "workers up to what its field seats"; design change 0294: "3 Agents saturate out to
    /// about 3 GU", the distance from a starting Tunnel's crystal hatch to its nearest patches on every Skirmish
    /// Default map): the Agents that gather Space Crystals for each Tunnel on a field ([`FIELD_REACH_GU`]).
    pub const AGENTS_PER_HATCH: usize = 3;
    /// Placeholder (not in the design: a Tunnel is on a field while a patch it knows, with crystals left, lies this
    /// near its crystal hatch, centre to centre), grid units. A Skirmish Default map's starting field lies within 6.
    pub const FIELD_REACH_GU: f32 = 8.0;
    /// Placeholder (not in the design: the Agents that gather Supplies at a Supply Delivery Station and build the
    /// Tunnels, `factions/syndicate/overview.md > SyndicateResources > Supplies`: "Spent on Tunnel construction").
    pub const RUNNERS: usize = 1;
    /// Placeholder (not in the design: the Tunnels it owns at most; `factions/syndicate/overview.md > TheSyndicate`:
    /// "building more Tunnels/upgrades grows" Tunnel Space, its army cap).
    pub const TUNNELS: usize = 6;
    /// Placeholder (Q0147 point 3: the Syndicate's line, `testing::syndicate::skirmish`: a Hideout's Guards behind "an
    /// Optics Lab"): the expansions it keeps, one of each, in order (`factions/syndicate/structures/hideout.md >
    /// Produces`: the Guard's "Unit Prerequisite: player owns at least one Optics Lab"). No Chemical Plant, so no
    /// Grenadiers: with one fighter in 3, 4 or 5 a Grenadier, standard beat passive later in the mirror than with
    /// Guards alone, or not within 15 minutes (`crates/ai/tests/games.rs`, `ai_games`).
    pub const EXPANSIONS: [ObjectKind; 2] = [ObjectKind::Hideout, ObjectKind::OpticsLab];
    /// Placeholder (Q0147 point 2: "training its basic units without pause"): the units each Hideout keeps queued.
    pub const TRAIN_QUEUE: usize = 2;
    /// Placeholder (Q0147 point 2: "attacks in waves of about 8 units"): the Guards waiting at staging that make each of
    /// its waves, the first too (`FactionAi::waves`). More than 8: a wave meets the enemy's whole army at its staging
    /// point, beside the Tunnel its Guards come out of. The army layer's waves (`crate::tuning::army`: 8, then 32, or
    /// after 2 minutes) beat a passive player on `maps/dev_skirmish.json` only after 16 minutes from slot 1 and 18 from
    /// slot 2; waves of 20 beat it within 11 minutes on every Skirmish Default map, from either slot
    /// (`crates/ai/tests/games.rs`, `ai_games`).
    pub const WAVE_SIZE: usize = 20;
    /// Placeholder (not in the design: as `crate::tuning::army::WAVE_PATIENCE_TICKS`, once Guards have waited at
    /// staging this long, they go as a wave however few they are), ticks (4 minutes; at a crystal hatch's 12.5 Space
    /// Crystals a second, design change 0294, a wave of 20 Guards takes about 3 and a half).
    pub const WAVE_PATIENCE_TICKS: Tick = 3840;
    /// Placeholder (not in the design: an Agent standing idle is given a job again at most this often), ticks (2
    /// seconds).
    pub const JOB_RETRY_TICKS: Tick = 32;
    /// Placeholder (not in the design: a new Tunnel's site lies within this distance of where it is wanted), grid
    /// units.
    pub const SITE_REACH_GU: f32 = 16.0;
    /// Placeholder (not in the design: the most build checks one search for a Tunnel's site makes, so that it costs
    /// little).
    pub const SITE_CHECKS: usize = 400;
    /// Placeholder (not in the design: after a search for a Tunnel's site finds none, the next waits this long), ticks
    /// (10 seconds).
    pub const SITE_RETRY_TICKS: Tick = 160;
}
