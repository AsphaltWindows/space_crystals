//! The Cults (slice M5-L): the economy check of `factions/cults/economy.md > Economy check` (design change 0346,
//! answering Q0103) on the reference layouts of `sc_sim::testing::cults`, played through the gate by its Cults line
//! ([`Cults`]), with `TestSim`'s invariants after every tick and player 1's books balanced every step (`Books`). "Two
//! scenarios, on an 8-patch field":
//! - **Scenario 1, ideal mining** ("one Storage per face, all Recruits mining"): the check's targets ([`TARGETS`]),
//!   each "within 15% of its target either way" ([`TOLERANCE`]). A figure that can't pass is brought to the designer
//!   with its measured value and pinned meanwhile ([`OUT_OF_TOLERANCE`]); the setup is not bent to pass it (design
//!   changes 0108 and 0346). A change that moves a figure in or out of the tolerance fails here.
//! - **Scenario 2, the realistic opening** (the "user's build line"): played, then "measured against the GDO and the
//!   Colonists at 1:30 and 2:48 and reported (the docs have no target for it yet)".
//!
//! How each figure is measured ("**Distance** is measured **edge to edge**, patch to Storage, in whole tiles"):
//! - **A Recruit's income** at a gap: one Recruit on [`face_field`], its trip the time between two deliveries once it
//!   has settled (from its third on, all the same), its income its load over its trip.
//! - **A face's efficiency** ("4–5 Recruits per 2-tile face at full efficiency"): 4 or 5 Recruits on [`face_field`],
//!   3 and 2 to a patch as [`Cults::deal`] seats them, their income over 0:20 to 5:00 against as many Recruits' alone.
//! - **Moving a Storage closer** (`factions/cults/structures/storage.md`: "Moving a Storage one tile closer (2-tile gap
//!   → 1-tile gap) gains ~0.86 SC/s per Recruit, ~+4 SC/s per face, paying back the 100 SC in ~25 s plus build time"):
//!   one Recruit's incomes at a 1- and a 2-tile gap; 4 Recruits' (the most a face holds at full efficiency at both
//!   gaps); and the Storage's price over that face's gain.
//! - **The Hall's fill** ("a starting Hall fills from 6 to 20 in 168 s"): the 14th Recruit the starting Hall of
//!   scenario 1 makes.
//! - **The earliest second Hall** ("~0:55–1:15"; `factions/cults/structures/recruitment_hall.md >
//!   BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`: "because the 6 starting Recruits and saving
//!   the first 300 SC dominate"): scenario 1, with a Hall ordered at [`SECOND_HALL`] the moment the bank holds its
//!   price, built by the 4 Recruits nearest its site (one face's worth; any number may build,
//!   `factions/cults/economy.md > Cults Building Mechanics > Group construction rules`); the time it is complete. The
//!   range passes from 15% under its low end to 15% over its high end.
//! - **Level with, then ahead of, the GDO and the Colonists** ("roughly level with the GDO and the Colonists until
//!   ~1:30 and slightly ahead by 2:48"; the same section: "in crystals mined", "best case, all Recruits mining with
//!   ideal Storage placement"): what scenario 1's Recruits delivered by 1:30 and by 2:48 over what the GDO's reference
//!   line mined (`played::plates_first`) and the Colonists' delivered (`colonists::Line`), each on its own reference
//!   field. Level is a ratio of 1, within the tolerance; slightly ahead, above 1 by at most 15%.
//!
//! **Scenario 2's line** (the check: "start with the one starting Storage; save for a second Recruitment Hall without
//! building any extra Storages, spreading new Recruits to patches farther from the starting Storage; build the second
//! Hall with all the non-optimal Recruits (those on the far patches); then build Storages around the patches, one per
//! face"; design change 0346: "Where the second Hall goes, and how the far Recruits are assigned, is your choice
//! within the description"): the Recruits dealt face by face outward (B's, the starting Storage's, first, then C's and
//! A's together, then D's), 5 to a face, a Recruit without a free seat left idle; the second Hall at [`SECOND_HALL`]
//! once the bank holds 300, by every Recruit whose patch has no own Storage a tile from it ([`Crew::Far`]); then the
//! Storages of C, A and D in turn, each by the Recruits still on such patches. The other dealing orders and crews that
//! [`cults_measurements`] tries gather less, and none nears the GDO or the Colonists.
//!
//! `cargo test -p sc_sim --release --test sim economy_oracle::cults -- --ignored --nocapture` prints the
//! measurements behind `docs/economy_oracle.md > The Cults`.

use std::collections::BTreeMap;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, Entry};
use sc_sim::testing::cults::{
    Build, Crew, Cults, Done, FIELD_PAIRS, FIELD_PLAYER, FIELD_STORAGES, SECOND_HALL, adjacent_field, crowd_field,
    face_field, reference_field,
};
use sc_sim::{ObjectId, ObjectKind, Scenario, SimConfig, SimEvent, Tick};

use super::colonists::Line;
use super::played::plates_first;
use super::{TOLERANCE, at, clock, seconds};

/// How a figure passes (`factions/cults/economy.md > Economy check`: "a measured figure passes if it is within 15% of
/// its target either way").
#[derive(Clone, Copy, Debug)]
enum Target {
    /// Within 15% of it.
    Near(f64),
    /// From 15% under the first to 15% over the second.
    Between(f64, f64),
    /// "Slightly ahead": above 1, by at most 15%.
    Ahead,
}

impl Target {
    fn passes(self, measured: f64) -> bool {
        match self {
            Target::Near(target) => ((measured - target) / target).abs() <= TOLERANCE,
            Target::Between(lo, hi) => (lo * (1.0 - TOLERANCE)..=hi * (1.0 + TOLERANCE)).contains(&measured),
            Target::Ahead => measured > 1.0 && measured <= 1.0 + TOLERANCE,
        }
    }
}

/// `factions/cults/economy.md > Economy check`'s "Targets", scenario 1's, by figure.
const TARGETS: [(&str, Target); 14] = [
    ("SC/s per Recruit at a 1-tile gap", Target::Near(1.45)),
    ("SC/s per Recruit at a 2-tile gap", Target::Near(0.59)),
    ("SC/s per Recruit at a 3-tile gap", Target::Near(0.37)),
    ("efficiency of 4 Recruits on a face", Target::Near(1.0)),
    ("efficiency of 5 Recruits on a face", Target::Near(1.0)),
    ("SC/s per Recruit gained from a 2-tile gap to 1", Target::Near(0.86)),
    ("SC/s per face gained from a 2-tile gap to 1", Target::Near(4.0)),
    ("seconds a Storage takes to pay back", Target::Near(25.0)),
    ("seconds the starting Hall takes from 6 to 20", Target::Near(168.0)),
    ("seconds to the earliest second Hall", Target::Between(55.0, 75.0)),
    ("crystals over the GDO's by 1:30", Target::Near(1.0)),
    ("crystals over the Colonists' by 1:30", Target::Near(1.0)),
    ("crystals over the GDO's by 2:48", Target::Ahead),
    ("crystals over the Colonists' by 2:48", Target::Ahead),
];
/// The figures out of the tolerance, as M5-L measured them (`docs/economy_oracle.md > The Cults`): the 5th Recruit
/// of a face mines at 0.82 of its rate alone; a face of 4 gains 3.0 SC/s from a 2-tile gap to 1, and its Storage pays
/// back in 33 s; by 2:48 scenario 1 has 0.90 of the GDO's crystals and 0.88 of the Colonists'.
const OUT_OF_TOLERANCE: [&str; 5] = [
    "efficiency of 5 Recruits on a face",
    "SC/s per face gained from a 2-tile gap to 1",
    "seconds a Storage takes to pay back",
    "crystals over the GDO's by 2:48",
    "crystals over the Colonists' by 2:48",
];

/// Checks `measured`'s figures: each within the tolerance but for those pinned out of it.
fn check(measured: &[(&'static str, f64)]) {
    let mut off = Vec::new();
    for &(name, value) in measured {
        let (_, target) = TARGETS.iter().find(|t| t.0 == name).expect("a figure of the check");
        let passes = target.passes(value);
        println!(
            "{name}: {value:.3}, target {target:?}{}",
            if passes { "" } else { ": OUT" }
        );
        if !passes {
            off.push(name);
        }
    }
    let pinned: Vec<&str> = (OUT_OF_TOLERANCE.iter().copied())
        .filter(|name| measured.iter().any(|m| m.0 == *name))
        .collect();
    off.sort_unstable();
    let mut pinned = pinned;
    pinned.sort_unstable();
    assert_eq!(
        off, pinned,
        "the figures outside the tolerance changed: bring them to the designer (design change 0108), pin them here and \
         update docs/economy_oracle.md"
    );
}

/// A Cults line played on a layout: the game, the line, and player 1's books (`books[i]` is tick `i + 1`'s step).
struct Game {
    t: TestSim,
    line: Cults,
    books: Vec<Entry>,
}

impl Game {
    /// `line` played on `scenario` to tick `until`: every command one the gate takes, given at the next tick.
    fn play(scenario: Scenario, mut line: Cults, until: Tick) -> Self {
        let mut t = TestSim::with_config(scenario, SimConfig::default());
        let mut books = Vec::new();
        while t.sim().tick() < until {
            let tick = t.sim().tick() + 1;
            let open = Books::open(t.sim(), FIELD_PLAYER);
            for c in line.commands(t.sim()) {
                if let Err(reject) = t.sim().validate(&c) {
                    panic!("tick {tick}: the gate refuses {c:?}: {reject:?}");
                }
                t.at(tick, c.player, c.command);
            }
            t.step();
            books.push(open.close(t.sim(), line.spent_in(tick)));
        }
        Self { t, line, books }
    }

    /// What player 1's Recruits delivered from tick `from + 1` to tick `to`.
    fn delivered(&self, from: Tick, to: Tick) -> u32 {
        self.books[from as usize..to as usize].iter().map(|e| e.delivered).sum()
    }

    fn income(&self, from: Tick, to: Tick) -> f64 {
        f64::from(self.delivered(from, to)) / seconds(to - from)
    }

    /// The ticks of each worker's deliveries.
    fn deliveries(&self) -> BTreeMap<ObjectId, Vec<(Tick, u32)>> {
        let mut by: BTreeMap<ObjectId, Vec<(Tick, u32)>> = BTreeMap::new();
        for (i, e) in self.books.iter().enumerate() {
            for &(worker, amount) in &e.deliveries {
                by.entry(worker).or_default().push((i as Tick + 1, amount));
            }
        }
        by
    }

    /// The ticks the Recruitment Hall `hall` released a Recruit in.
    fn made_by(&self, hall: ObjectId) -> Vec<Tick> {
        (self.t.events().iter())
            .filter_map(|(tick, e)| match *e {
                SimEvent::UnitProduced { producer, .. } if producer == hall => Some(*tick),
                _ => None,
            })
            .collect()
    }

    /// When the line's `kind` build was ordered and completed, and its site.
    fn built(&self, kind: ObjectKind) -> Vec<(Tick, Tick, ObjectId)> {
        let ordered = (self.line.done().iter()).filter_map(|(tick, done)| match done {
            Done::Ordered { build, .. } if build.kind == kind => Some(*tick),
            _ => None,
        });
        let built = (self.line.done().iter()).filter_map(|(tick, done)| match done {
            Done::Built { build, site } if build.kind == kind => Some((*tick, *site)),
            _ => None,
        });
        ordered
            .zip(built)
            .map(|(ordered, (built, site))| (ordered, built, site))
            .collect()
    }
}

/// A Recruit's income at a `gap`-tile gap: its load over its trip once settled.
fn alone(gap: i32) -> f64 {
    let line = Cults::new(FIELD_PLAYER).deal([(FIELD_PAIRS[0][0], 1)]);
    let game = Game::play(face_field(gap, 1), line, at(1, 0));
    let trips = &game.deliveries()[&ObjectId(4)];
    let settled: Vec<(Tick, u32)> = (trips.windows(2).skip(1)).map(|w| (w[1].0 - w[0].0, w[1].1)).collect();
    assert!(
        settled.len() >= 3 && settled.windows(2).all(|w| w[0] == w[1]),
        "{trips:?}"
    );
    let (trip, load) = settled[0];
    println!("a Recruit at a {gap}-tile gap: a trip of {trip} ticks, {load} a trip");
    f64::from(load) / seconds(trip)
}

/// `recruits` Recruits' income on one face, its Storage at a `gap`-tile gap, over 0:20 to 5:00.
fn face(gap: i32, recruits: u32) -> f64 {
    let line = Cults::new(FIELD_PLAYER).deal([(FIELD_PAIRS[0][0], 3), (FIELD_PAIRS[0][1], 2)]);
    let game = Game::play(face_field(gap, recruits), line, at(5, 0));
    game.income(at(0, 20), at(5, 0))
}

/// Seats on the faces of `pairs` (indices into `FIELD_PAIRS`): 3 on a pair's west patch, 2 on its east, the west
/// patches first.
fn seats(pairs: &[usize]) -> Vec<(ObjectId, u32)> {
    let west = pairs.iter().map(|&p| (FIELD_PAIRS[p][0], 3));
    west.chain(pairs.iter().map(|&p| (FIELD_PAIRS[p][1], 2))).collect()
}

/// Scenario 1: every face's Storage, every Recruit dealt over the field, each face 5 seats.
fn ideal() -> Cults {
    Cults::new(FIELD_PLAYER).deal(seats(&[1, 2, 0, 3]))
}

/// Scenario 2's line (module docs), the Recruits dealt to `groups` in turn and the Storages built by `crew`.
fn realistic(groups: &[Vec<(ObjectId, u32)>], crew: Crew) -> Cults {
    let mut line = Cults::new(FIELD_PLAYER);
    for group in groups {
        line = line.deal(group.clone());
    }
    line = line.build(Build::new(ObjectKind::RecruitmentHall, SECOND_HALL, Crew::Far));
    for pair in [2, 0, 3] {
        line = line.build(Build::new(ObjectKind::Storage, FIELD_STORAGES[pair], crew));
    }
    line
}

/// Scenario 2's dealing: B's face, then C's and A's, then D's.
fn outward() -> Vec<Vec<(ObjectId, u32)>> {
    vec![seats(&[1]), seats(&[2, 0]), seats(&[3])]
}

/// The starting Hall of [`reference_field`]: the first object after the patches and the pre-placed Storages.
fn starting_hall(game: &Game) -> ObjectId {
    let hall = game.t.sim().objects().find(|o| o.kind == ObjectKind::RecruitmentHall);
    hall.expect("the starting Hall").id
}

#[test]
fn cults_trips_and_faces() {
    let [one, two, three] = [alone(1), alone(2), alone(3)];
    let (near, far) = (face(1, 4), face(2, 4));
    let fifth = face(1, 5);
    let price = {
        let t = TestSim::with_config(face_field(1, 0), SimConfig::default());
        let price = t.sim().build_price(FIELD_PLAYER, ObjectKind::Storage).expect("a price");
        f64::from(price.amount)
    };
    check(&[
        ("SC/s per Recruit at a 1-tile gap", one),
        ("SC/s per Recruit at a 2-tile gap", two),
        ("SC/s per Recruit at a 3-tile gap", three),
        ("efficiency of 4 Recruits on a face", near / (4.0 * one)),
        ("efficiency of 5 Recruits on a face", fifth / (5.0 * one)),
        ("SC/s per Recruit gained from a 2-tile gap to 1", one - two),
        ("SC/s per face gained from a 2-tile gap to 1", near - far),
        ("seconds a Storage takes to pay back", price / (near - far)),
    ]);
}

#[test]
fn cults_ideal_mining_against_the_gdo_and_the_colonists() {
    let end = at(2, 48);
    let game = Game::play(reference_field(&[0, 1, 2, 3]), ideal(), end);
    let made = game.made_by(starting_hall(&game));
    assert_eq!(made.len(), 14, "the starting Hall fills by 2:48");
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    let over = |by: Tick, other: u32| f64::from(game.delivered(0, by)) / f64::from(other);
    let (by_1_30, by_2_48) = (at(1, 30), end);
    check(&[
        ("seconds the starting Hall takes from 6 to 20", seconds(made[13])),
        ("crystals over the GDO's by 1:30", over(by_1_30, gdo.mined(0, by_1_30))),
        (
            "crystals over the Colonists' by 1:30",
            over(by_1_30, colonists.delivered(0, by_1_30)),
        ),
        ("crystals over the GDO's by 2:48", over(by_2_48, gdo.mined(0, by_2_48))),
        (
            "crystals over the Colonists' by 2:48",
            over(by_2_48, colonists.delivered(0, by_2_48)),
        ),
    ]);
}

/// The earliest second Hall (module docs), which also checks that the line orders it the moment the bank holds its
/// price and that its four builders go back to their patches.
#[test]
fn cults_earliest_second_hall() {
    let build = Build::new(ObjectKind::RecruitmentHall, SECOND_HALL, Crew::Nearest(4));
    let game = Game::play(reference_field(&[0, 1, 2, 3]), ideal().build(build), at(1, 30));
    let [(ordered, built, _)] = game.built(ObjectKind::RecruitmentHall)[..] else {
        panic!("one second Hall: {:?}", game.line.done());
    };
    // The order is given at tick `ordered`, on the bank tick `ordered - 1` left (`books[ordered - 2]`).
    let banks = [
        game.books[ordered as usize - 3].bank,
        game.books[ordered as usize - 2].bank,
    ];
    assert!(
        banks[0] < 300 && banks[1] >= 300,
        "ordered as soon as affordable: {banks:?}"
    );
    println!("the second Hall: ordered {}, complete {}", clock(ordered), clock(built));
    check(&[("seconds to the earliest second Hall", seconds(built))]);
    let gathering = (game.t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Recruit && o.orders.as_ref().is_some_and(|o| !o.is_idle()))
        .count();
    let recruits = game.t.sim().objects().filter(|o| o.kind == ObjectKind::Recruit).count();
    assert_eq!(gathering, recruits, "every Recruit at work");
}

/// Scenario 2 (module docs), measured and reported: the line's builds in order, the second Hall's Effectiveness, and
/// its crystals against the GDO's and the Colonists' by 1:30 and 2:48.
#[test]
fn cults_realistic_opening() {
    let end = at(2, 48);
    let game = Game::play(reference_field(&[]), realistic(&outward(), Crew::Far), end);
    let halls = game.built(ObjectKind::RecruitmentHall);
    let storages = game.built(ObjectKind::Storage);
    let [(_, hall_built, hall)] = halls[..] else {
        panic!("one second Hall: {:?}", game.line.done());
    };
    assert_eq!(storages.len(), 3, "a Storage for each other face by 2:48");
    assert!(
        storages.iter().all(|s| s.0 >= hall_built),
        "no Storage before the second Hall"
    );
    let state = game.t.sim().hall_state(hall).expect("a Hall");
    assert_eq!(state.effectiveness_percent(), 100, "its whole area its own");
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    for by in [at(1, 30), end] {
        let cults = game.delivered(0, by);
        let (g, c) = (gdo.mined(0, by), colonists.delivered(0, by));
        println!(
            "by {}: the Cults {cults}, the GDO {g} ({:.2}), the Colonists {c} ({:.2})",
            clock(by),
            f64::from(cults) / f64::from(g),
            f64::from(cults) / f64::from(c)
        );
    }
    for (ordered, built, site) in halls.iter().chain(&storages) {
        println!("{site:?} ordered {}, complete {}", clock(*ordered), clock(*built));
    }
}

#[test]
#[ignore = "the measurements for docs/economy_oracle.md: run it in release with --ignored --nocapture"]
fn cults_measurements() {
    for (gap, recruits) in [(1, 4), (1, 5), (2, 4), (2, 5), (3, 4)] {
        println!(
            "a face at a {gap}-tile gap, {recruits} Recruits: {:.2} SC/s",
            face(gap, recruits)
        );
    }
    for builders in [1, 2, 4, 6] {
        let build = Build::new(ObjectKind::RecruitmentHall, SECOND_HALL, Crew::Nearest(builders));
        let game = Game::play(reference_field(&[0, 1, 2, 3]), ideal().build(build), at(3, 0));
        let [(ordered, built, _)] = game.built(ObjectKind::RecruitmentHall)[..] else {
            panic!("the second Hall by {builders}, not complete by 3:00")
        };
        println!(
            "the second Hall by {builders}: ordered {}, complete {}",
            clock(ordered),
            clock(built)
        );
    }
    let end = at(2, 48);
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    let ideal = Game::play(reference_field(&[0, 1, 2, 3]), ideal(), end);
    let real = Game::play(reference_field(&[]), realistic(&outward(), Crew::Far), end);
    println!("time, GDO, Colonists, Cults ideal, Cults realistic (crystals gathered since 0:00)");
    for by in [at(0, 30), at(1, 0), at(1, 30), at(2, 0), at(2, 30), end] {
        let row = [
            gdo.mined(0, by),
            colonists.delivered(0, by),
            ideal.delivered(0, by),
            real.delivered(0, by),
        ];
        println!("{}: {row:?}", clock(by));
    }
    let [a, b, c, d] = FIELD_PAIRS;
    let nearest = vec![
        vec![(b[0], 3), (b[1], 2)],
        vec![(c[0], 3), (a[1], 3)],
        vec![(c[1], 2), (a[0], 2)],
        vec![(d[0], 3), (d[1], 2)],
    ];
    let lines = [
        (
            "outward, Storages by the far Recruits (scenario 2)",
            outward(),
            Crew::Far,
        ),
        ("outward, Storages by the 4 nearest", outward(), Crew::Nearest(4)),
        (
            "nearest patches first, Storages by the far Recruits",
            nearest.clone(),
            Crew::Far,
        ),
        (
            "nearest patches first, Storages by the 3 nearest",
            nearest,
            Crew::Nearest(3),
        ),
    ];
    for (name, groups, crew) in lines {
        let game = Game::play(reference_field(&[]), realistic(&groups, crew), end);
        let built: Vec<String> = (game.built(ObjectKind::RecruitmentHall).iter())
            .chain(&game.built(ObjectKind::Storage))
            .map(|(_, built, _)| clock(*built))
            .collect();
        let idle = (game.t.sim().objects())
            .filter(|o| o.kind == ObjectKind::Recruit && o.orders.as_ref().is_none_or(|o| o.is_idle()))
            .count();
        println!(
            "{name}: by 1:30 {}, by 2:48 {}; built {built:?}; {idle} Recruits idle at 2:48",
            game.delivered(0, at(1, 30)),
            game.delivered(0, end)
        );
    }
}

/// The free columns (`docs/economy_oracle.md > The Cults`, "Why the free columns"): the field's income over 0:20 to
/// 5:00 with 16 and 20 Recruits dealt as in scenario 1, its faces a free column apart ([`crowd_field`]) and side by
/// side ([`adjacent_field`]), the Storages under the row or on alternate sides; and the loads each Storage took then,
/// by pair from the west.
#[test]
#[ignore = "the measurements for docs/economy_oracle.md: run it in release with --ignored --nocapture"]
fn cults_free_columns() {
    /// A layout with a number of Recruits.
    type Layout = fn(u32) -> Scenario;
    let layouts: [(&str, Layout); 3] = [
        ("the faces a free column apart", crowd_field),
        ("8 adjacent patches, the Storages under the row", |n| {
            adjacent_field(n, false)
        }),
        ("8 adjacent patches, the Storages on alternate sides", |n| {
            adjacent_field(n, true)
        }),
    ];
    let (from, to) = (at(0, 20), at(5, 0));
    for (name, layout) in layouts {
        for recruits in [16, 20] {
            let game = Game::play(layout(recruits), ideal(), to);
            let storages = (game.t.sim().objects()).filter(|o| o.kind == ObjectKind::Storage);
            let mut loads: BTreeMap<ObjectId, u32> = storages.map(|o| (o.id, 0)).collect();
            for (tick, e) in game.t.events() {
                if let SimEvent::ResourcesDelivered { drop_off, .. } = e
                    && (from + 1..=to).contains(tick)
                {
                    *loads.entry(*drop_off).or_default() += 1;
                }
            }
            println!(
                "{name}, {recruits} Recruits: {:.2} SC/s, loads by Storage {:?}",
                game.income(from, to),
                loads.values().collect::<Vec<_>>()
            );
        }
    }
}
