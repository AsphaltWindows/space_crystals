//! The Cults (slices M5-L and M5-S2a): the economy check of `factions/cults/economy.md > Economy check` (design change
//! 0346, answering Q0103) on the reference layouts of `sc_sim::testing::cults`, played through the gate by its Cults
//! line ([`Cults`]), with `TestSim`'s invariants after every tick and player 1's books balanced every step (`Books`).
//! "Two scenarios, on an 8-patch field", since slice M5-S2a with design change 0380's 3x2 Storage, three faces of 3,
//! 3 and 2 patches:
//! - **Scenario 1, ideal mining** ("one Storage per face, all Recruits mining"): the check's targets ([`TARGETS`]),
//!   each "within 15% of its target either way" ([`TOLERANCE`]). A figure that can't pass is brought to the designer
//!   with its measured value and pinned meanwhile ([`OUT_OF_TOLERANCE`]); the setup is not bent to pass it (design
//!   changes 0108 and 0346). A change that moves a figure in or out of the tolerance fails here.
//! - **Scenario 2, the realistic opening** (the "user's build line"): played, then measured against the GDO, the
//!   Colonists and scenario 1 (design change 0381: "it should **overtake ideal mining (scenario 1) by about
//!   4:00**").
//! - **The comparison** (design changes 0381 to 0383, answering Q0135 with 0380): both scenarios, the GDO and the
//!   Colonists at 1:30, 2:48, 4:00, 5:00 and 6:00 ([`CHECKPOINTS`]), each line's [`Figures`] there, pinned
//!   ([`COMPARED`]): "The user will set further targets from it".
//!
//! How each figure is measured ("**Distance** is measured **edge to edge**, patch to Storage, in whole tiles"):
//! - **A Recruit's income** at a gap: one Recruit on [`face_field`], its trip the time between two deliveries once it
//!   has settled (from its third on, all the same), its income its load over its trip.
//! - **A face's efficiency** ("full efficiency at about **2 Recruits per tile of face length** (designer's estimate
//!   for the 3x2 Storage, 2026-11-27: ~6 on a 3-tile face, ~4 on a 2-tile face [...]; re-measure)"): 6 Recruits on a
//!   Storage's long side and 4 on its short side ([`face_field`]), dealt evenly over its patches, their income over
//!   0:20 to 5:00 against as many Recruits' alone.
//! - **Moving a Storage closer** (`factions/cults/structures/storage.md`: "Moving a Storage one tile closer (2-tile gap
//!   → 1-tile gap) gains ~0.86 SC/s per Recruit, ~+4 SC/s per face, paying back the 100 SC in ~25 s plus build time";
//!   "The per-face figures above (written for 2x2) are to be re-measured"): one Recruit's incomes at a 1- and a 2-tile
//!   gap; the 6 Recruits of a long side, the face a 3x2 Storage is laid out to serve, at both gaps; and the Storage's
//!   price over that face's gain. (A short side's 4 are printed.)
//! - **The Hall's fill** ("a starting Hall fills from 6 to 20 in 168 s"): the 14th Recruit the starting Hall of
//!   scenario 1 makes.
//! - **The earliest second Hall** ("~0:55–1:15"; `factions/cults/structures/recruitment_hall.md >
//!   BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`: "because the 6 starting Recruits and saving
//!   the first 300 SC dominate"): scenario 1, with a Hall ordered at [`SECOND_HALL`] the moment the bank holds its
//!   price, built by the 4 Recruits nearest its site (any number may build, `factions/cults/economy.md > Cults
//!   Building Mechanics > Group construction rules`); the time it is complete. The range passes from 15% under its
//!   low end to 15% over its high end.
//! - **Level with, then ahead of, the GDO and the Colonists** ("roughly level with the GDO and the Colonists until
//!   ~1:30 and slightly ahead by 2:48"; the same section: "in crystals mined", "best case, all Recruits mining with
//!   ideal Storage placement"): what scenario 1's Recruits delivered by 1:30 and by 2:48 over what the GDO's reference
//!   line mined (`played::plates_first`) and the Colonists' delivered (`colonists::Line`), each on its own reference
//!   field. Level is a ratio of 1, within the tolerance; slightly ahead, above 1 by at most 15%.
//!
//! **The seats** ([`SEATS`]): scenario 1 deals all of the starting Hall's 20 Recruits ("all Recruits mining"), 2.5
//! to a patch as slice M5-L's 5 to a 2-tile face did: 7 on face A, 8 on B and 5 on C. That is a Recruit or two over
//! the design's "about 2 Recruits per tile of face length" on each face, as M5-L's fifth was over its "4–5".
//!
//! **Scenario 2's line** (the check: "start with the one starting Storage; save for a second Recruitment Hall without
//! building any extra Storages, spreading new Recruits to patches farther from the starting Storage; build the second
//! Hall with all the non-optimal Recruits (those on the far patches); then build Storages around the patches, one per
//! face"; design change 0346: "Where the second Hall goes, and how the far Recruits are assigned, is your choice
//! within the description"): the Recruits dealt face by face outward, B's (the starting Storage's) first, then A's and
//! C's together, on scenario 1's seats, a Recruit without a free seat left idle; the second Hall at [`SECOND_HALL`]
//! once the bank holds 300, by every Recruit whose patch has no own Storage a tile from it ([`Crew::Far`]); then the
//! Storages of C and A in turn, each by the Recruits still on such patches. The other dealing orders and crews that
//! [`cults_measurements`] tries gather less.
//!
//! `cargo test -p sc_sim --release --test sim economy_oracle::cults -- --ignored --nocapture` prints the
//! measurements behind `docs/economy_oracle.md > The Cults`.

use std::collections::BTreeMap;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, Entry};
use sc_sim::testing::colonists;
use sc_sim::testing::cults::{
    Build, Crew, Cults, Done, FIELD_FACES, FIELD_PLAYER, FIELD_STORAGES, SECOND_HALL, Side, adjacent_field,
    crowd_field, face_field, reference_field,
};
use sc_sim::{ObjectId, ObjectKind, Scenario, SimConfig, SimEvent, Tick};

use super::colonists::Line;
use super::played::{plates_first, reference_base};
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

/// `factions/cults/economy.md > Economy check`'s "Targets", scenario 1's, by figure; a Storage's gain and payback from
/// `factions/cults/structures/storage.md`'s rationale.
const TARGETS: [(&str, Target); 14] = [
    ("SC/s per Recruit at a 1-tile gap", Target::Near(1.45)),
    ("SC/s per Recruit at a 2-tile gap", Target::Near(0.59)),
    ("SC/s per Recruit at a 3-tile gap", Target::Near(0.37)),
    ("efficiency of 6 Recruits on a 3-tile face", Target::Near(1.0)),
    ("efficiency of 4 Recruits on a 2-tile face", Target::Near(1.0)),
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
/// The figures out of the tolerance, as slice M5-S2a measured them (`docs/economy_oracle.md > The Cults`): by 2:48
/// scenario 1 has 0.88 of the GDO's crystals and 0.86 of the Colonists' (0.90 and 0.88 with the 2x2 Storage, slice
/// M5-L; M5-L's other three, a face's fifth Recruit, its gain and its Storage's payback, pass on the 3x2's long side).
const OUT_OF_TOLERANCE: [&str; 2] = [
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

    /// Player 1's Recruits with no order at all.
    fn idle(&self) -> usize {
        let own = |o: &&sc_sim::Object| o.owner == Some(FIELD_PLAYER) && o.kind == ObjectKind::Recruit;
        (self.t.sim().objects())
            .filter(|o| own(o) && o.orders.as_ref().is_none_or(|o| o.is_idle()))
            .count()
    }
}

/// `recruits` seats spread evenly over `patches`, the earlier patches taking one more where they don't divide.
fn spread(patches: &[ObjectId], recruits: u32) -> Vec<(ObjectId, u32)> {
    let n = patches.len() as u32;
    (patches.iter().enumerate())
        .map(|(i, &patch)| (patch, recruits / n + u32::from((i as u32) < recruits % n)))
        .collect()
}

/// A Recruit's income at a `gap`-tile gap: its load over its trip once settled, on the first patch of a long side.
fn alone(gap: i32) -> f64 {
    let line = Cults::new(FIELD_PLAYER).deal([(ObjectId(1), 1)]);
    let game = Game::play(face_field(Side::Long, gap, 1), line, at(1, 0));
    let recruit = ObjectId(Side::Long.patches() + 2);
    let trips = &game.deliveries()[&recruit];
    let settled: Vec<(Tick, u32)> = (trips.windows(2).skip(1)).map(|w| (w[1].0 - w[0].0, w[1].1)).collect();
    assert!(
        settled.len() >= 3 && settled.windows(2).all(|w| w[0] == w[1]),
        "{trips:?}"
    );
    let (trip, load) = settled[0];
    println!("a Recruit at a {gap}-tile gap: a trip of {trip} ticks, {load} a trip");
    f64::from(load) / seconds(trip)
}

/// `recruits` Recruits' income on one face of a Storage's `side`, the Storage at a `gap`-tile gap, over 0:20 to 5:00.
fn face(side: Side, gap: i32, recruits: u32) -> f64 {
    let patches: Vec<ObjectId> = (1..=side.patches()).map(ObjectId).collect();
    let line = Cults::new(FIELD_PLAYER).deal(spread(&patches, recruits));
    let game = Game::play(face_field(side, gap, recruits), line, at(5, 0));
    game.income(at(0, 20), at(5, 0))
}

/// Scenario 1's seats, by face from the west (module docs): 2.5 to a patch, all 20 of the starting Hall's Recruits.
const SEATS: [u32; 3] = [7, 8, 5];

/// The seats on the faces `faces` names (indices into [`FIELD_FACES`]), [`SEATS`] spread over each.
fn seats(faces: &[usize]) -> Vec<(ObjectId, u32)> {
    (faces.iter())
        .flat_map(|&face| spread(FIELD_FACES[face], SEATS[face]))
        .collect()
}

/// Scenario 1: every face's Storage, every Recruit dealt over the field, in one group (the patch with the fewest
/// first).
fn ideal() -> Cults {
    Cults::new(FIELD_PLAYER).deal(seats(&[1, 0, 2]))
}

/// Scenario 2's line (module docs), the Recruits dealt to `groups` in turn and the Storages built by `crew`.
fn realistic(groups: &[Vec<(ObjectId, u32)>], crew: Crew) -> Cults {
    let mut line = Cults::new(FIELD_PLAYER);
    for group in groups {
        line = line.deal(group.clone());
    }
    line = line.build(Build::new(ObjectKind::RecruitmentHall, SECOND_HALL, Crew::Far));
    for face in [2, 0] {
        line = line.build(Build::new(ObjectKind::Storage, FIELD_STORAGES[face], crew));
    }
    line
}

/// Scenario 2's dealing: B's face, then A's and C's.
fn outward() -> Vec<Vec<(ObjectId, u32)>> {
    vec![seats(&[1]), seats(&[0, 2])]
}

/// The starting Hall of [`reference_field`]: the first object after the patches and the pre-placed Storages.
fn starting_hall(game: &Game) -> ObjectId {
    let hall = game.t.sim().objects().find(|o| o.kind == ObjectKind::RecruitmentHall);
    hall.expect("the starting Hall").id
}

#[test]
fn cults_trips_and_faces() {
    let [one, two, three] = [alone(1), alone(2), alone(3)];
    let (near, far) = (face(Side::Long, 1, 6), face(Side::Long, 2, 6));
    let (short, short_far) = (face(Side::Short, 1, 4), face(Side::Short, 2, 4));
    let price = {
        let t = TestSim::with_config(face_field(Side::Long, 1, 0), SimConfig::default());
        let price = t.sim().build_price(FIELD_PLAYER, ObjectKind::Storage).expect("a price");
        f64::from(price.amount)
    };
    println!(
        "a long side, 6 Recruits: {near:.2} SC/s at a 1-tile gap, {far:.2} at 2; a short side, 4: {short:.2} and \
         {short_far:.2}, a gain of {:.2}, paid back in {:.1} s",
        short - short_far,
        price / (short - short_far)
    );
    check(&[
        ("SC/s per Recruit at a 1-tile gap", one),
        ("SC/s per Recruit at a 2-tile gap", two),
        ("SC/s per Recruit at a 3-tile gap", three),
        ("efficiency of 6 Recruits on a 3-tile face", near / (6.0 * one)),
        ("efficiency of 4 Recruits on a 2-tile face", short / (4.0 * one)),
        ("SC/s per Recruit gained from a 2-tile gap to 1", one - two),
        ("SC/s per face gained from a 2-tile gap to 1", near - far),
        ("seconds a Storage takes to pay back", price / (near - far)),
    ]);
}

#[test]
fn cults_ideal_mining_against_the_gdo_and_the_colonists() {
    let end = at(2, 48);
    let game = Game::play(reference_field(&[0, 1, 2]), ideal(), end);
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
    let game = Game::play(reference_field(&[0, 1, 2]), ideal().build(build), at(1, 30));
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

/// Scenario 2 (module docs), its builds: the second Hall first, then a Storage for each other face, by 2:48; the
/// second Hall's Effectiveness. Its crystals are in [`the_economies_compared_out_to_six_minutes`].
#[test]
fn cults_realistic_opening() {
    let end = at(2, 48);
    let game = Game::play(reference_field(&[]), realistic(&outward(), Crew::Far), end);
    let halls = game.built(ObjectKind::RecruitmentHall);
    let storages = game.built(ObjectKind::Storage);
    let [(_, hall_built, hall)] = halls[..] else {
        panic!("one second Hall: {:?}", game.line.done());
    };
    assert_eq!(storages.len(), 2, "a Storage for each other face by 2:48");
    assert!(
        storages.iter().all(|s| s.0 >= hall_built),
        "no Storage before the second Hall"
    );
    let state = game.t.sim().hall_state(hall).expect("a Hall");
    assert_eq!(state.effectiveness_percent(), 100, "its whole area its own");
    for (ordered, built, site) in halls.iter().chain(&storages) {
        println!("{site:?} ordered {}, complete {}", clock(*ordered), clock(*built));
    }
    for (tick, done) in game.line.done() {
        if let Done::Ordered { build, crew, .. } = done {
            println!(
                "{:?} ordered at {} from {} Recruits",
                build.kind,
                clock(*tick),
                crew.len()
            );
        }
    }
    println!("{} Recruits idle at 2:48", game.idle());
}

/// The comparison's checkpoints (design change 0381: "at **1:30, 2:48, 4:00, 5:00 and 6:00**").
const CHECKPOINTS: [Tick; 5] = [at(1, 30), at(2, 48), at(4, 0), at(5, 0), at(6, 0)];
/// Its lines, in the order of [`COMPARED`].
const LINES: [&str; 4] = ["Cults, ideal", "Cults, realistic", "GDO", "Colonists"];
/// Design change 0382: "**Income/s**: delivered over the preceding 10 s (160 frames), divided by 10".
const INCOME_WINDOW: Tick = 160;

/// One line's figures at a checkpoint (design change 0382), in Space Crystals: what it gathered over the
/// [`INCOME_WINDOW`] before it, ten times its income per second; "**Total mined**: delivered since 0:00, excluding the
/// starting bank" (the GDO's: what its plates mined); "**Bank**: unspent crystals"; and "Spent on economy", which
/// design change 0383 makes "**everything spent**" ("total crystals spent, net of refunds").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Figures {
    window: u32,
    mined: u32,
    bank: u32,
    spent: u32,
}

impl Figures {
    /// A line's figures at tick `at`, from its books, its bank starting at `start`. Checks design change 0383's
    /// Verification: "Bank + spent = starting bank + total mined, at every checkpoint".
    fn of(books: &[Entry], start: u32, at: Tick) -> Self {
        let gathered = |e: &Entry| e.mined + e.delivered;
        let until = &books[..at as usize];
        let figures = Self {
            window: until[(at - INCOME_WINDOW) as usize..].iter().map(gathered).sum(),
            mined: until.iter().map(gathered).sum(),
            bank: until.last().expect("a step").bank,
            spent: until.iter().map(|e| e.spent).sum::<u32>() - until.iter().map(|e| e.refunded).sum::<u32>(),
        };
        assert_eq!(
            figures.bank + figures.spent,
            start + figures.mined,
            "at {}: bank + spent = starting bank + total mined",
            clock(at)
        );
        figures
    }

    fn income(&self) -> f64 {
        f64::from(self.window) / seconds(INCOME_WINDOW)
    }

    /// Whether these figures are near `pinned`'s: an income within 10%, or 1 SC/s, as ten seconds of loads of 2 come
    /// and go; an amount within 5%, or 20 Space Crystals.
    fn near(&self, pinned: &Self) -> bool {
        let near = |measured: u32, pinned: u32, share: f64, floor: u32| {
            f64::from(measured.abs_diff(pinned)) <= f64::from(floor).max(share * f64::from(pinned))
        };
        near(self.window, pinned.window, 0.10, 10)
            && near(self.mined, pinned.mined, 0.05, 20)
            && near(self.bank, pinned.bank, 0.05, 20)
            && near(self.spent, pinned.spent, 0.05, 20)
    }
}

/// [`Figures`] as a table row writes them.
const fn row(window: u32, mined: u32, bank: u32, spent: u32) -> Figures {
    Figures {
        window,
        mined,
        bank,
        spent,
    }
}

/// The comparison as slice M5-S2a measured it (`docs/economy_oracle.md > The Cults > The economies compared`), by
/// line ([`LINES`]) and checkpoint ([`CHECKPOINTS`]): what each gathered over the 10 seconds before, since 0:00, its
/// bank and what it spent.
const COMPARED: [[Figures; 5]; 4] = [
    [
        row(130, 918, 918, 0),
        row(190, 2362, 2362, 0),
        row(236, 3904, 3904, 0),
        row(212, 5278, 5278, 0),
        row(222, 6572, 6572, 0),
    ],
    [
        row(96, 676, 376, 300),
        row(214, 1848, 1348, 500),
        row(216, 3424, 2924, 500),
        row(200, 4628, 4128, 500),
        row(216, 5942, 5442, 500),
    ],
    [
        row(190, 980, 280, 800),
        row(290, 2670, 1320, 1450),
        row(290, 4590, 2740, 1950),
        row(290, 6190, 3940, 2350),
        row(290, 7790, 5440, 2450),
    ],
    [
        row(160, 1032, 682, 450),
        row(248, 2744, 1994, 850),
        row(280, 4624, 3524, 1200),
        row(304, 6312, 4912, 1500),
        row(320, 8224, 6524, 1800),
    ],
];

/// The four lines played to 6:00: each one's figures at the checkpoints.
fn compared() -> [[Figures; 5]; 4] {
    let end = CHECKPOINTS[4];
    let start = |scenario: Scenario| {
        let t = TestSim::with_config(scenario, SimConfig::default());
        t.sim().player(FIELD_PLAYER).expect("player 1").crystals
    };
    let ideal = Game::play(reference_field(&[0, 1, 2]), ideal(), end);
    let real = Game::play(reference_field(&[]), realistic(&outward(), Crew::Far), end);
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    let lines: [(&[Entry], u32); 4] = [
        (&ideal.books, start(reference_field(&[0, 1, 2]))),
        (&real.books, start(reference_field(&[]))),
        (&gdo.books, start(reference_base())),
        (colonists.books(), start(colonists::reference_field(4))),
    ];
    let built: Vec<String> = (real.built(ObjectKind::RecruitmentHall).iter())
        .chain(&real.built(ObjectKind::Storage))
        .map(|(_, built, _)| clock(*built))
        .collect();
    println!(
        "the realistic opening: built {built:?}; {} Recruits idle at 6:00, of {}",
        real.idle(),
        (real.t.sim().objects())
            .filter(|o| o.kind == ObjectKind::Recruit)
            .count()
    );
    lines.map(|(books, start)| CHECKPOINTS.map(|at| Figures::of(books, start, at)))
}

/// Design changes 0381 to 0383 (answering Q0135 with 0380; module docs): the four lines out to 6:00, each one's
/// figures at the five checkpoints, consistent (`Figures::of`) and pinned ([`COMPARED`]). And 0381's Verification: "It
/// flags whether the realistic opening has overtaken ideal mining by 4:00", by income and by total mined; and,
/// as 0381 asks to report, whether ideal mining still lands at ~0.9 of the GDO and the Colonists by 2:48.
#[test]
fn the_economies_compared_out_to_six_minutes() {
    let table = compared();
    println!("line | checkpoint | income/s | total mined | bank | spent");
    for (line, row) in LINES.iter().zip(&table) {
        for (at, f) in CHECKPOINTS.iter().zip(row) {
            println!(
                "{line} | {} | {:.1} | {} | {} | {}",
                &clock(*at)[..4],
                f.income(),
                f.mined,
                f.bank,
                f.spent
            );
        }
    }
    let [ideal, real, gdo, colonists] = &table;
    let at_4 = 2;
    let overtaken = (
        real[at_4].window > ideal[at_4].window,
        real[at_4].mined > ideal[at_4].mined,
    );
    println!(
        "by 4:00 the realistic opening has overtaken ideal mining: by income/s {} ({:.1} against {:.1}), by total \
         mined {} ({} against {})",
        overtaken.0,
        real[at_4].income(),
        ideal[at_4].income(),
        overtaken.1,
        real[at_4].mined,
        ideal[at_4].mined
    );
    let at_2_48 = 1;
    let over = |other: &[Figures; 5]| f64::from(ideal[at_2_48].mined) / f64::from(other[at_2_48].mined);
    println!(
        "ideal mining by 2:48: {:.2} of the GDO's, {:.2} of the Colonists'",
        over(gdo),
        over(colonists)
    );
    assert_eq!(
        overtaken, OVERTAKEN,
        "the realistic opening against ideal mining at 4:00"
    );
    for ((line, row), pinned) in LINES.iter().zip(&table).zip(&COMPARED) {
        for ((at, f), pin) in CHECKPOINTS.iter().zip(row).zip(pinned) {
            assert!(
                f.near(pin),
                "{line} at {}: {f:?}, pinned {pin:?}: re-measure, update COMPARED and docs/economy_oracle.md, and \
                 tell the designer (design change 0381: the comparison is reported)",
                clock(*at)
            );
        }
    }
}

/// Whether the realistic opening has overtaken ideal mining by 4:00, by income/s and by total mined, as measured.
const OVERTAKEN: (bool, bool) = (false, false);

#[test]
#[ignore = "the measurements for docs/economy_oracle.md: run it in release with --ignored --nocapture"]
fn cults_measurements() {
    for (side, gap, recruits) in [
        (Side::Long, 1, 5),
        (Side::Long, 1, 6),
        (Side::Long, 1, 7),
        (Side::Long, 1, 8),
        (Side::Long, 2, 6),
        (Side::Long, 2, 7),
        (Side::Short, 1, 4),
        (Side::Short, 1, 5),
        (Side::Short, 2, 4),
        (Side::Short, 2, 5),
    ] {
        println!(
            "a {side:?} side at a {gap}-tile gap, {recruits} Recruits: {:.2} SC/s",
            face(side, gap, recruits)
        );
    }
    for builders in [1, 2, 4, 6] {
        let build = Build::new(ObjectKind::RecruitmentHall, SECOND_HALL, Crew::Nearest(builders));
        let game = Game::play(reference_field(&[0, 1, 2]), ideal().build(build), at(3, 0));
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
    let ideal = Game::play(reference_field(&[0, 1, 2]), ideal(), end);
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
    // Scenario 1's seats on A's and C's patches, the patches nearest to B's Storage first: A's east one and C's west
    // one, then the others.
    let ([a0, a1, a2], [c0, c1]) = (
        <[_; 3]>::try_from(seats(&[0])).expect("face A"),
        <[_; 2]>::try_from(seats(&[2])).expect("face C"),
    );
    let nearest = vec![seats(&[1]), vec![a2, c0], vec![a1, c1, a0]];
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
        println!(
            "{name}: by 1:30 {}, by 2:48 {}; built {built:?}; {} Recruits idle at 2:48",
            game.delivered(0, at(1, 30)),
            game.delivered(0, end),
            game.idle()
        );
    }
}

/// The free columns (`docs/economy_oracle.md > The Cults`, "Why the free columns"): the field's income over 0:20 to
/// 5:00 with 16 and 20 Recruits dealt as in scenario 1, its faces a free column apart ([`crowd_field`]) and side by
/// side ([`adjacent_field`]), the Storages under the row or B's north of it; and the loads each Storage took then, by
/// face from the west.
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
        ("8 adjacent patches, B's Storage north of the row", |n| {
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
