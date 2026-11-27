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
    /// Placeholder (not in the design: once those patches are searched, a wave searches the ground around the start, on
    /// a square grid of points this far apart, within [`SEARCH_RADIUS_GU`] of it; slice A2a, for the buildings a Cults
    /// player spreads over its Recruitable land, and since slice A2d in every game, for a Colonists' last Beacons and a
    /// Syndicate's last Tunnels: scoped to games with a Cults player, the Colonists' standard against standard on
    /// `maps/playtest.json` ended in no defeat, the loser's last two Beacons never found), grid units. Each of a wave's
    /// units sees 5 grid units around it (`factions/gdo/units/peacekeeper.md > SightRange - 5 grid units`,
    /// `factions/cults/units/gunner.md > SightRange - 5 grid units`), so a wave at each point and on its way between
    /// them sees nearly all the ground there.
    pub const SEARCH_STEP_GU: f32 = 8.0;
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
    /// Placeholder (Q0147 point 2's rush: "a small early army and an early first attack"; point 3: the build order;
    /// slice A2d): the rush's opening, a Barracks before the second Generator. The Generators after it come as the
    /// Power planned needs them, as standard's do.
    pub const RUSH_OPENING: [ObjectKind; 2] = [ObjectKind::Generator, ObjectKind::Barracks];
    /// Placeholder (Q0147 point 2's rush: "a small early army and an early first attack"; slice A2d): the Peacekeepers
    /// of the rush's first wave, which its Barracks trains before the plates and the Generators, and which leave as
    /// soon as they wait at staging (`FactionAi::waves`). Smaller than standard's 8 ([`super::army::FIRST_WAVE_SIZE`]),
    /// so that it goes sooner.
    pub const RUSH_WAVE: usize = 6;
}

/// The Cults' computer player (`crate::factions::cults`).
pub mod cults {
    use sc_sim::Tick;

    /// Placeholder (not in the design: the field is the patches the start sees as its own): its starting field is the
    /// Space Crystals patches whose centres lie this near its start location, grid units. Every Skirmish Default map's
    /// 8-patch field lies within 7 grid units of its start.
    pub const FIELD_RADIUS_GU: f32 = 10.0;
    /// `factions/cults/economy.md > Economy check`: "full efficiency at about **2 Recruits per tile of face length**":
    /// the Recruits a patch seats for each Storage straight across a 1-tile gap from it, the patch's tile of face.
    pub const SEATS_ACROSS: u32 = 2;
    /// Placeholder (not in the design: a patch at a Storage's corner, a tile from it, has no face across from it): the
    /// Recruits it seats for that Storage, as `sc_sim::testing::cults::skirmish`'s seats give such a patch one.
    pub const SEATS_AT_CORNER: u32 = 1;
    /// Placeholder (not in the design: a Storage worth its 100 Space Crystals): the fewest seats a new Storage must add.
    pub const STORAGE_SEATS: u32 = 3;
    /// Placeholder (not in the design: no building but a Storage goes this near the field, so that the Storages' sites
    /// and the Recruits' ways stay free), tiles from a patch of the field: a 1-tile gap and a Storage's 3 tiles.
    pub const FIELD_ZONE: i32 = 4;
    /// Placeholder (Q0147 point 2: "one or two production buildings", for the Cults the Recruitment Halls whose
    /// Recruits become the army; `factions/cults/overview.md > TheCults`: "Army size scales with territorial control of
    /// Recruitable tiles, with no hard cap"): the most Halls it keeps, the starting one included. Fewer sites fit
    /// ([`HALL_EFFECTIVENESS`], [`HALL_SEARCH_GU`]): four on `dev_skirmish`, five on `playtest.json`.
    pub const HALLS: usize = 8;
    /// Placeholder (not in the design; slice M5-S4: "A Hall at the map's edge recruits less"): a new Hall goes only
    /// where it would claim at least this share of its area, percent, as far as the player knows the Halls there.
    pub const HALL_EFFECTIVENESS: u32 = 90;
    /// Placeholder (not in the design: a Hall's site is sought this near the start, grid units): the ground the army's
    /// waves search around an enemy's start ([`super::army::SEARCH_RADIUS_GU`]), so that a game against it ends.
    pub const HALL_SEARCH_GU: f32 = super::army::SEARCH_RADIUS_GU;
    /// Placeholder (Q0147 point 3: the build order; the brief: "Gunners from a Gunworks, if they fit simply"): the
    /// complete Halls it owns before it builds a Gunworks.
    pub const GUNWORKS_AFTER_HALLS: usize = 2;
    /// Placeholder (not in the design): once a Gunworks stands, one in this many units the Armory trains is a Gunner,
    /// the rest Militants. 1, all Gunners: its Recruits' Unit Control bounds the Cults' army while Space Crystals pile up,
    /// and a Gunner does more with its Recruit (`factions/cults/units/gunner.md > AttackAttributes`: "Damage: 16")
    /// than a Militant (`factions/cults/units/militant.md > AttackAttributes`: "Damage: 4"). With 2, its standard
    /// player took over 16 minutes to beat the GDO's passive one on `dev_skirmish` (`crates/ai/tests/games.rs`,
    /// `ai_games`).
    pub const GUNNER_EVERY: u32 = 1;
    /// Placeholder (not in the design: who builds, `factions/cults/economy.md > BaseConstructionTime Scaling`: "2
    /// Recruits halve it, 3 Recruits reduce it to a third"): the Recruits that build a Storage.
    pub const STORAGE_BUILDERS: usize = 2;
    /// Placeholder (not in the design; `sc_sim::testing::cults`' lines build their Armory and Halls with 3): the
    /// Recruits that build the Armory and the Gunworks.
    pub const ARMORY_BUILDERS: usize = 3;
    /// Placeholder (not in the design): the Recruits that build a Recruitment Hall, a fourth of its 90 seconds each.
    pub const HALL_BUILDERS: usize = 4;
    /// Placeholder (not in the design: after a search finds no site for a building, the next waits this long), ticks
    /// (10 seconds): a field whose Storages all stand stays so until one is lost.
    pub const SITE_RETRY_TICKS: Tick = 160;
    /// Placeholder (not in the design: a Build whose site hasn't started this long after it was ordered is given up,
    /// its builders sent back to work, so that nothing waits for good, M3-G3), ticks (40 seconds).
    pub const BUILD_PATIENCE_TICKS: Tick = 640;
    /// Placeholder (not in the design; the A2d brief: "Stop growing the gathering crew once income outruns what the
    /// army can spend"): once the bank holds more than this, Space Crystals, the crew stops growing and Recruits leave
    /// the field for the Armory ([`PULL_TICKS`]), until it holds less than [`BANK_LOW`]. Without it the bank piled up to
    /// 4,000 to 6,000 by 7 to 9 minutes on `maps/playtest.json` (slice A2a).
    pub const BANK_HIGH: u32 = 600;
    /// Placeholder (not in the design): see [`BANK_HIGH`], Space Crystals.
    pub const BANK_LOW: u32 = 300;
    /// Placeholder (not in the design: while the bank outruns the army, [`BANK_HIGH`]), ticks (1 second): a seated
    /// Recruit leaves the field for the Armory at most this often, which trains one every 2 seconds
    /// (`factions/cults/structures/armory.md > Training`: "TrainingTime: 32 frames").
    pub const PULL_TICKS: Tick = 16;
    /// Placeholder (not in the design): the fewest Recruits left mining while the bank outruns the army
    /// ([`BANK_HIGH`]).
    pub const MIN_CREW: usize = 8;
    /// Placeholder (Q0147 point 2's rush: "a small early army and an early first attack"; the A2d brief: "every
    /// Recruit beyond a minimal gathering crew becomes a Militant"): the Recruits that mine while a rush plays its
    /// opening, the Starting Conditions' six (`factions/cults/economy.md > Starting Conditions`).
    pub const RUSH_CREW: usize = 6;
    /// Placeholder (Q0147 point 2's rush; slice A2d): the Militants of the rush's first wave, which its Armory trains
    /// before any other building goes up, and which leave as soon as they wait at staging (`FactionAi::waves`).
    /// Smaller than standard's 8 ([`super::army::FIRST_WAVE_SIZE`]), so that it goes sooner.
    pub const RUSH_WAVE: usize = 6;
}

/// The Colonists' computer player (`crate::factions::colonists`, slice A2c).
pub mod colonists {
    use sc_sim::{ObjectKind, Tick};

    /// Placeholder (not in the design: the field it saturates is the patches this near its Egress Node, where its
    /// Drones deliver), grid units: a Skirmish Default start's 8 patches lie within 6 of it.
    pub const FIELD_RADIUS_GU: f32 = 8.0;
    /// Placeholder (Q0147 point 2: "workers up to what its field seats"; `factions/colonists/units/drone.md > Patch
    /// Occupancy`: 8 patches 2 grid units from the Egress Node "saturate at about **24 working Drones** (3 per
    /// patch, 32 Space Crystals per second)"): the gathering Drones it keeps for each patch of its field with crystals
    /// left.
    pub const DRONES_PER_PATCH: usize = 3;
    /// Placeholder (Q0147 point 2: "training its basic units without pause"; `factions/gdo/economy.md > Economy
    /// Targets (GDO vs Colonists)`: "one Robo Portal producing Drones nonstop"): the Drones each Robo Portal keeps
    /// queued while its field wants more or a Beacon is wanted.
    pub const DRONE_QUEUE: usize = 1;
    /// Placeholder (Q0147 point 2: "one or two production buildings"; point 3: the scripted lines' "a Bio Portal
    /// warps in on the first Beacon", the golden game `colonists_opening`'s): the buildings it warps in, each once it
    /// has this many gathering Drones, in their order of priority for the bank. An Egress Node and a Robo Portal (the
    /// Starting Conditions' own, warped in again if lost), a Bio Portal on its first Beacon, a second Robo Portal at 8
    /// Drones (slice A2d), and two more Bio Portals as its income grows: a second at 12 Drones, and a third at 18, 2 a
    /// patch (`factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`: "Colonists at 2 Drones per patch ≈ 23
    /// Space Crystals per second for 18 Drones", 32 fully saturated). Three Bio Portals and two Robo Portals spend 40 a
    /// second. The Robo Portals' Drones fill the field and deploy the Beacons (`factions/colonists/units/drone.md >
    /// BeaconCapacityCost`: "roughly every 6th Drone must deploy as a Beacon to keep Drone production going"): with
    /// one, standard's bank averaged up to 1,055 Space Crystals from 3:00 to 10:00 in the mirror on
    /// `maps/playtest.json`, Beacon Capacity holding its portals back, and under 350 with two (`crates/ai/tests/games.rs`,
    /// `ai_games`).
    pub const WARP_INS: [(ObjectKind, usize); 6] = [
        (ObjectKind::EgressNode, 0),
        (ObjectKind::RoboPortal, 0),
        (ObjectKind::BioPortal, 0),
        (ObjectKind::RoboPortal, 8),
        (ObjectKind::BioPortal, 12),
        (ObjectKind::BioPortal, 18),
    ];
    /// Placeholder (Q0147 point 2: "training its basic units without pause"): the Sabre Legionnaires each Bio Portal
    /// keeps queued, while Beacon Capacity has room for them.
    pub const TRAIN_QUEUE: usize = 2;
    /// Placeholder (Q0147 point 2: "its basic units", the Sabre Legionnaire and the Rifle Legionnaire it converts
    /// into, `factions/colonists/units/weapon_conversion.md`): one Legionnaire in this many is a Rifle. Sabres alone
    /// trade evenly with a passive player's at its staging point, so that on `dev_skirmish` neither side wins in 20
    /// minutes; with half of them Rifles, firing over the Sabres in front, the waves beat it within 15 minutes
    /// (`crates/ai/tests/games.rs`, `ai_games`).
    pub const RIFLE_EVERY: usize = 2;
    /// Placeholder (not in the design: Beacon Capacity kept ahead of the army, `factions/colonists/units/drone.md >
    /// BeaconCapacityCost`: "roughly every 6th Drone must deploy as a Beacon to keep Drone production going"): a
    /// Drone deploys once the room under Beacon Capacity, the Beacons on their way counted, falls short of what the
    /// queued units take plus this many of each portal's next units: two, a portal's next 20 seconds (slice A2d). With
    /// one, standard's bank averaged up to 503 Space Crystals from 3:00 to 10:00 against the GDO's passive on
    /// `maps/playtest.json`; with two, under 150 there and in the mirror (`crates/ai/tests/games.rs`, `ai_games`).
    pub const BEACON_LEAD: u32 = 2;
    /// Placeholder (not in the design: where Beacons go): how far from the Robo Portal the Beacon sites reach, each
    /// way, tiles. A site takes a Beacon's 2 x 2 (a Bio Portal's 3 x 3 while one to come lacks a Beacon) and a free
    /// tile round it, so that 16 tiles each way hold the 40 buildings of Beacon Capacity's 200
    /// (`factions/colonists/overview.md > ColonistResources > Beacon Capacity`) and more.
    pub const SITE_REACH: i32 = 16;
    /// Placeholder (not in the design: the most deploy checks one search for a Beacon site makes, so that a full base
    /// costs little).
    pub const SITE_CHECKS: usize = 24;
    /// Placeholder (not in the design: a Drone sent to deploy that hasn't begun its deployment this long after its
    /// order is stopped, its Deploy failed), ticks (20 seconds: a walk across the Beacon sites' reach takes 10).
    pub const DEPLOY_PATIENCE_TICKS: Tick = 320;
    /// Placeholder (not in the design: a Drone whose Deploy failed, and that Deploy's site, are left out of the next
    /// Deploys this long), ticks (1 minute).
    pub const FAILED_DEPLOY_TICKS: Tick = 960;
    /// Placeholder (Q0147 point 2's rush: "a small early army and an early first attack"; slice A2d): the Legionnaires
    /// of the rush's first wave, Sabres its Bio Portal trains before the Robo Portal's Drones (half of them convert
    /// into Rifles, [`RIFLE_EVERY`]), which leave as soon as they wait at staging (`FactionAi::waves`). Smaller than
    /// standard's 8 ([`super::army::FIRST_WAVE_SIZE`]), so that it goes sooner.
    pub const RUSH_WAVE: usize = 6;
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
    /// Tunnels, `factions/syndicate/overview.md > SyndicateResources > Supplies`: "Spent on Tunnel construction"), for
    /// each Tunnel on a field (slice A2d), so that Tunnel Space grows with the income. With one in all, an Agent's
    /// Supply a trip from a starting Tunnel to the centre's station (about 1 a minute, of the station's 4) paid for the
    /// sixth Tunnel only at about 16 minutes: on `maps/playtest.json`, with two crystal hatches at work, Tunnel Space
    /// was full from 3 minutes, and the bank piled up to 1,350 Space Crystals. Two on every map, the passive player's
    /// larger army held out for 20 minutes against standard's in the mirror on `dev_skirmish`, from slot 1.
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
    /// Placeholder (Q0147 point 2's rush: "a small early army and an early first attack"; slice A2d): the Guards of the
    /// rush's first wave, which it trains before it wants a runner or builds a second Tunnel, and which leave as soon
    /// as they wait at staging (`FactionAi::waves`).
    pub const RUSH_WAVE: usize = 6;
    /// Placeholder (not in the design: an Agent standing idle is given a job again at most this often), ticks (2
    /// seconds).
    pub const JOB_RETRY_TICKS: Tick = 32;
    /// Placeholder (not in the design: a new Tunnel's site lies within this distance of where it is wanted), grid
    /// units.
    pub const SITE_REACH_GU: f32 = 16.0;
    /// Placeholder (not in the design; slice A2d: a new Tunnel adds crystal income at patches its crystal hatch is
    /// nearer than every other hatch of the player's by this much at least, centre to centre, so that the Agents
    /// mining them, standing at their edge, deliver there), grid units.
    pub const INCOME_MARGIN_GU: f32 = 1.0;
    /// Placeholder (not in the design; slice A2d: the patches a new Tunnel adds income at lie within this distance of
    /// its crystal hatch, centre to centre; design change 0294: "3 Agents saturate out to about 3 GU", from the
    /// patch's edge), grid units.
    pub const INCOME_REACH_GU: f32 = 4.5;
    /// Placeholder (not in the design: the most build checks one search for a Tunnel's site makes, so that it costs
    /// little).
    pub const SITE_CHECKS: usize = 400;
    /// Placeholder (not in the design: after a search for a Tunnel's site finds none, the next waits this long), ticks
    /// (10 seconds).
    pub const SITE_RETRY_TICKS: Tick = 160;
}
