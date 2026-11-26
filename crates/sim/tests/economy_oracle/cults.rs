//! The Cults (slices M5-L, M5-S2a and M5-S3): the economy check of `factions/cults/economy.md > Economy check` (design
//! changes 0346 and 0385) on the reference layouts of `sc_sim::testing::cults`, played through the gate by its Cults
//! line ([`Cults`]), with `TestSim`'s invariants after every tick and player 1's books balanced every step (`Books`).
//!
//! **The reference field** (design change 0385, answering Q0144 in part; [`line_field`]): "**8 patches in a contiguous
//! line**, mined from **both long sides**. It should seat about **32 Recruits** at full efficiency (8 tiles × 2 sides ×
//! about 2 per tile). Storages stand along both long sides only: **none at the ends**". Three Storage sites a side
//! ([`SITES`]), two seats a patch on each side ([`SEATS_PER_PATCH`]): the room at a Storage is the design's "about 2
//! Recruits per tile of face length", 6 at a long face, 4 at a short one.
//!
//! **Two scenarios** ("Both **start with only the one starting Storage**, and add Storages **as they're needed**:
//! whenever the mining Recruits outnumber the room at the existing Storages. Each Storage is paid for and built by
//! Recruits taken off mining"):
//! 1. **One Hall** ([`one_hall`]): "the starting Hall only; every Recruit mines once it has room".
//! 2. **Two Halls** ([`two_halls`]): "save for a second Recruitment Hall as early as affordable, built by the Recruits on
//!    the patches farthest from the starting Storage; then Storages as needed. With 40 Recruits against about 32 seats,
//!    the second Hall's Recruits fill the field". Its target: "Two Halls may fall behind early, but it should
//!    **overtake One Hall by about 4:00**", checked with the four lines' table in `compared.rs`.
//!
//! **How the lines play** them (`sc_sim::testing::cults`'s module docs and `seats.rs`; where the design is silent,
//! slice M5-S3's literal reading, reported):
//! - **Seats.** Every Recruit mines: each idle one is dealt the next free seat, the near side's first (the starting
//!   Storage's, then near W's, then near E's), then the far side's (M, W, E), each site's patches nearest the starting
//!   Storage first. A Recruit on a near seat whose Storage doesn't stand yet mines at a loss, taking its loads to the
//!   Storage nearest to itself (`factions/cults/units/recruit.md > Gathering`, Which Storage). The far side's seats are
//!   dealt only once their own Storage is complete: a Recruit there takes its loads to the Storage nearest to itself,
//!   which would be on the near side, round the line, and its leg back to the patch would then take it to the near
//!   face. "Every Recruit mines once it has room" is read so: each Recruit mines as soon as the field has a seat for
//!   it, all 20 of One Hall's.
//! - **Room, and when a Storage is needed.** A site's seats are room while its Storage stands, from the order on.
//!   Whenever the player's Recruits outnumber the room, the line orders the next site's Storage once nothing else of
//!   its is being built and the bank covers it.
//! - **Who builds.** The Recruits that mine at a loss or not at all, those whose seat has no room yet and those without
//!   a seat ([`Crew::Far`]): "Recruits taken off mining", and for the second Hall "the Recruits on the patches farthest
//!   from the starting Storage". Each goes back to its seat after.
//! - **The far side** is reached as a player would send Recruits there: a Move to the lane in front of the patch, round
//!   the line's end, then the Gather ([`Seats::by`]); every Recruit is sent to its seat that way, the near side's too.
//! - **Two Halls** orders its second Hall the moment the bank holds 300 and no Storage before it; the Storages follow
//!   once it is complete (one building at a time, "then Storages as needed"). Its site, west of the line, is
//!   [`SECOND_HALL`].
//!
//! How each figure is measured ("**Distance** is measured **edge to edge**, patch to Storage, in whole tiles"):
//! - **A Recruit's income** at a gap: one Recruit on [`face_field`], its trip the time between two deliveries once it
//!   has settled (from its third on, all the same), its income its load over its trip.
//! - **A face's efficiency** ("full efficiency at about **2 Recruits per tile of face length** (designer's estimate
//!   for the 3x2 Storage, 2026-11-27: ~6 on a 3-tile face, ~4 on a 2-tile face [...]; re-measure)"): 6 Recruits on a
//!   Storage's long side and 4 on its short side ([`face_field`]), dealt evenly over its patches, their income over
//!   0:20 to 5:00 against as many Recruits' alone.
//! - **The field's seats** ("It should seat about **32 Recruits** at full efficiency"): 16 Recruits a side on
//!   [`full_line`], every Storage standing, two to a patch, their income over 0:20 to 5:00 against 32 Recruits' alone.
//! - **Moving a Storage closer** (`factions/cults/structures/storage.md`: "Moving a Storage one tile closer (2-tile gap
//!   → 1-tile gap) gains ~0.86 SC/s per Recruit, ~+4 SC/s per face, paying back the 100 SC in ~25 s plus build time";
//!   "The per-face figures above (written for 2x2) are to be re-measured"): one Recruit's incomes at a 1- and a 2-tile
//!   gap; the 6 Recruits of a long side, the face a 3x2 Storage is laid out to serve, at both gaps; and the Storage's
//!   price over that face's gain. (A short side's 4 are printed.)
//! - **The Hall's fill** ("a starting Hall fills from 6 to 20 in 168 s"): the 14th Recruit One Hall's Hall makes.
//! - **The earliest second Hall** ("~0:55–1:15"; `factions/cults/structures/recruitment_hall.md >
//!   BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`: "because the 6 starting Recruits and saving
//!   the first 300 SC dominate"): Two Halls' second Hall, "as early as affordable", complete. The range passes from 15%
//!   under its low end to 15% over its high end.
//! - **Level with, then ahead of, the GDO and the Colonists** ("roughly level with the GDO and the Colonists until
//!   ~1:30 and slightly ahead by 2:48"; still open: "whether ~0.9 of the GDO and the Colonists by 2:48 is acceptable, to
//!   be decided on the new figures"): what One Hall's Recruits delivered by 1:30 and by 2:48 over what the GDO's
//!   reference line mined (`played::plates_first`) and the Colonists' delivered (`colonists::Line`), each on its own
//!   reference field. Level is a ratio of 1, within the tolerance; slightly ahead, above 1 by at most 15%.
//!
//! `cargo test -p sc_sim --release --test sim economy_oracle::cults -- --ignored --nocapture` prints the
//! measurements behind `docs/economy_oracle.md > The Cults`.

use std::collections::BTreeMap;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::{Books, Entry};
use sc_sim::testing::cults::{
    Build, Crew, Cults, Done, FIELD_PLAYER, LINE, LineSide, SECOND_HALL, SECOND_HALL_TURNED, SITES, Seats, Side,
    face_field, full_line, line_field,
};
use sc_sim::{ObjectId, ObjectKind, OrderKind, Scenario, SimConfig, SimEvent, Tick, TilePos};

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

/// `factions/cults/economy.md > Economy check`'s "Targets" and its reference field's seats, by figure; a Storage's
/// gain and payback from `factions/cults/structures/storage.md`'s rationale.
const TARGETS: [(&str, Target); 15] = [
    ("SC/s per Recruit at a 1-tile gap", Target::Near(1.45)),
    ("SC/s per Recruit at a 2-tile gap", Target::Near(0.59)),
    ("SC/s per Recruit at a 3-tile gap", Target::Near(0.37)),
    ("efficiency of 6 Recruits on a 3-tile face", Target::Near(1.0)),
    ("efficiency of 4 Recruits on a 2-tile face", Target::Near(1.0)),
    ("efficiency of 32 Recruits on the line's two sides", Target::Near(1.0)),
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
/// The figures out of the tolerance, as slice M5-S3 measured them (`docs/economy_oracle.md > The Cults`): One Hall's
/// crystals by 1:30 are 0.83 of the Colonists', and by 2:48 0.87 of the GDO's and 0.84 of the Colonists' (the
/// design's "Still open: whether ~0.9 of the GDO and the Colonists by 2:48 is acceptable, to be decided on the new
/// figures").
const OUT_OF_TOLERANCE: [&str; 3] = [
    "crystals over the Colonists' by 1:30",
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
    let mut pinned: Vec<&str> = (OUT_OF_TOLERANCE.iter().copied())
        .filter(|name| measured.iter().any(|m| m.0 == *name))
        .collect();
    off.sort_unstable();
    pinned.sort_unstable();
    assert_eq!(
        off, pinned,
        "the figures outside the tolerance changed: bring them to the designer (design change 0108), pin them here and \
         update docs/economy_oracle.md"
    );
}

/// A Cults line played on a layout: the game, the line, and player 1's books (`books[i]` is tick `i + 1`'s step).
pub(super) struct Game {
    pub(super) t: TestSim,
    pub(super) line: Cults,
    pub(super) books: Vec<Entry>,
}

impl Game {
    /// `line` played on `scenario` to tick `until`: every command one the gate takes, given at the next tick.
    pub(super) fn play(scenario: Scenario, mut line: Cults, until: Tick) -> Self {
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

    /// When the line's `kind` builds were ordered and completed, their sites and how many Recruits built each, in
    /// order.
    fn built(&self, kind: ObjectKind) -> Vec<(Tick, Tick, ObjectId, usize)> {
        let ordered = (self.line.done().iter()).filter_map(|(tick, done)| match done {
            Done::Ordered { build, crew, .. } if build.kind == kind => Some((*tick, build.origin, crew.len())),
            _ => None,
        });
        let built: Vec<(Tick, ObjectId, TilePos)> = (self.line.done().iter())
            .filter_map(|(tick, done)| match done {
                Done::Built { build, site } if build.kind == kind => Some((*tick, *site, build.origin)),
                _ => None,
            })
            .collect();
        // The last order of each building that was completed (one lost before its site began is ordered again).
        let mut runs = Vec::new();
        for &(done, site, origin) in &built {
            let last = (ordered.clone())
                .filter(|&(tick, o, _)| o == origin && tick <= done)
                .max_by_key(|&(tick, ..)| tick);
            let (ordered, _, crew) = last.expect("ordered");
            runs.push((ordered, done, site, crew));
        }
        runs
    }

    /// Player 1's Recruits, those mining (a Gather their current order) and those with no order at all.
    pub(super) fn recruits(&self) -> (usize, usize, usize) {
        let own = (self.t.sim().objects()).filter(|o| o.owner == Some(FIELD_PLAYER) && o.kind == ObjectKind::Recruit);
        let (mut all, mut mining, mut idle) = (0, 0, 0);
        for o in own {
            all += 1;
            let current = o.orders.as_ref().and_then(|orders| orders.current.as_ref());
            mining += usize::from(current.is_some_and(|a| matches!(a.order.kind, OrderKind::Gather(_))));
            idle += usize::from(o.orders.as_ref().is_none_or(|o| o.is_idle()));
        }
        (all, mining, idle)
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

/// The seats on a patch, a side (design change 0385: "8 tiles × 2 sides × about 2 per tile").
const SEATS_PER_PATCH: u32 = 2;

/// The reference field's seats (module docs), a group for each of [`SITES`] in its order: two Recruits to a patch on
/// the site's side, its patches nearest the starting Storage first, the room of the Storage on the site, which the
/// line builds once it is needed; the far side's dealt only once their Storage is complete; every Recruit sent to its
/// seat by the lane in front of its patch.
fn sides() -> impl Iterator<Item = Seats> {
    SITES.iter().map(|site| {
        let storage = Build::turned(ObjectKind::Storage, site.origin, site.orientation(), Crew::Far);
        let seats = site.patches.iter().map(|&patch| (patch, SEATS_PER_PATCH));
        let group = Seats::new(seats).facing(storage).by(site.side.lane());
        if site.side == LineSide::Far {
            group.waiting()
        } else {
            group
        }
    })
}

/// Scenario 1, One Hall (module docs).
pub(super) fn one_hall() -> Cults {
    sides().fold(Cults::new(FIELD_PLAYER), Cults::seats)
}

/// Scenario 2, Two Halls (module docs): One Hall's line with the second Hall first, built by the Recruits that mine
/// at a loss.
pub(super) fn two_halls() -> Cults {
    let hall = Build::turned(ObjectKind::RecruitmentHall, SECOND_HALL, SECOND_HALL_TURNED, Crew::Far);
    one_hall().build(hall)
}

/// [`full_line`]'s seats with `near` and `far` Recruits a side, spread evenly over each side's patches, each sent by
/// its lane.
fn line_seats(near: u32, far: u32) -> Cults {
    let side = |side: LineSide, n| Seats::new(spread(&LINE, n)).by(side.lane());
    Cults::new(FIELD_PLAYER)
        .seats(side(LineSide::Near, near))
        .seats(side(LineSide::Far, far))
}

/// The reference field's income over 0:20 to 5:00 with `near` and `far` Recruits a side ([`line_seats`]).
fn line_income(near: u32, far: u32) -> f64 {
    let game = Game::play(full_line(near, far), line_seats(near, far), at(5, 0));
    game.income(at(0, 20), at(5, 0))
}

/// The starting Hall of [`line_field`]: the first Hall of the game.
fn starting_hall(game: &Game) -> ObjectId {
    let hall = game.t.sim().objects().find(|o| o.kind == ObjectKind::RecruitmentHall);
    hall.expect("the starting Hall").id
}

/// The name of the Storage site at `origin` ([`SITES`]).
fn site_name(origin: TilePos) -> String {
    let i = SITES.iter().position(|s| s.origin == origin).expect("a site");
    let side = match SITES[i].side {
        LineSide::Near => "near",
        LineSide::Far => "far",
    };
    let at = ["M", "W", "E"][i % 3];
    format!("{side} {at}")
}

/// The Storages `game`'s line built: each site's name, when it was ordered and complete, and how many Recruits built
/// it.
fn storages(game: &Game) -> Vec<(String, Tick, Tick, usize)> {
    let built = game.built(ObjectKind::Storage);
    let named = built.iter().map(|&(ordered, done, site, crew)| {
        let placed = game.t.sim().object(site).and_then(|o| o.placement);
        (site_name(placed.expect("standing").origin), ordered, done, crew)
    });
    named.collect()
}

/// Prints `line`'s Storages ([`storages`]) and returns their sites' names.
fn print_storages<'a>(line: &str, storages: &'a [(String, Tick, Tick, usize)]) -> Vec<&'a str> {
    for (name, ordered, done, crew) in storages {
        println!(
            "{line}: {name} ordered {} by {crew}, complete {}",
            clock(*ordered),
            clock(*done)
        );
    }
    storages.iter().map(|(name, ..)| name.as_str()).collect()
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

/// The reference field's seats (module docs): 16 Recruits a side, two to a patch, against 32 Recruits' alone.
#[test]
fn cults_the_line_seats_32() {
    let one = alone(1);
    let income = line_income(16, 16);
    println!("the line, 16 Recruits a side: {income:.2} SC/s");
    check(&[(
        "efficiency of 32 Recruits on the line's two sides",
        income / (32.0 * one),
    )]);
}

/// One Hall (module docs): its Hall's fill, the Storages it builds, its Recruits all mining by 4:00 (design change 0385's
/// Verification: "One Hall levels off at 20 Recruits mining, below the field's ~32 seats"), and its crystals against the
/// GDO's and the Colonists'.
#[test]
fn cults_one_hall() {
    let end = at(4, 0);
    let game = Game::play(line_field(), one_hall(), end);
    let made = game.made_by(starting_hall(&game));
    assert_eq!(made.len(), 14, "the starting Hall fills by 2:48");
    let built = storages(&game);
    let sites = print_storages("One Hall", &built);
    assert_eq!(sites, ["near W", "near E", "far M"], "room for its 20");
    let (all, mining, idle) = game.recruits();
    println!("One Hall at 4:00: {all} Recruits, {mining} mining, {idle} idle");
    assert_eq!((all, mining, idle), (20, 20, 0), "levels off at 20 Recruits mining");
    let by_2_48 = at(2, 48);
    let (gdo, colonists) = (plates_first(by_2_48), Line::play(by_2_48, false));
    let over = |by: Tick, other: u32| f64::from(game.delivered(0, by)) / f64::from(other);
    let by_1_30 = at(1, 30);
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

/// Two Halls (module docs): its second Hall, ordered the moment the bank holds its price, by the Recruits that mine at
/// a loss, complete as the earliest second Hall, its whole Recruitment Area its own; the Storages it builds; and
/// design change 0385's Verification: "Two Halls reaches ~32 mining, and builds roughly 5–6 Storages along both sides".
#[test]
fn cults_two_halls() {
    let end = at(6, 0);
    let game = Game::play(line_field(), two_halls(), end);
    let [(ordered, built, hall, crew)] = game.built(ObjectKind::RecruitmentHall)[..] else {
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
    println!(
        "Two Halls: the second Hall ordered {} by {crew}, complete {}",
        clock(ordered),
        clock(built)
    );
    let state = game.t.sim().hall_state(hall).expect("a Hall");
    assert_eq!(state.effectiveness_percent(), 100, "its whole area its own");
    let storages = storages(&game);
    let sites = print_storages("Two Halls", &storages);
    assert!(
        storages.iter().all(|&(_, ordered, ..)| ordered > built),
        "then Storages as needed"
    );
    assert_eq!(sites, ["near W", "near E", "far M", "far W", "far E"], "every site's");
    let (all, mining, idle) = game.recruits();
    println!("Two Halls at 6:00: {all} Recruits, {mining} mining, {idle} idle");
    assert_eq!(
        (all, mining, idle),
        (40, 32, 8),
        "32 mining, the field's seats, and 8 without a seat"
    );
    check(&[("seconds to the earliest second Hall", seconds(built))]);
}

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
    let one = alone(1);
    let both = [8, 12, 14, 15, 16, 17, 18, 20, 24].map(|n| (n, n));
    for (near, far) in both.into_iter().chain([(14, 0), (16, 0), (18, 0)]) {
        let income = line_income(near, far);
        let n = f64::from(near + far);
        println!(
            "the line, {near} Recruits on the near side and {far} on the far: {income:.2} SC/s, {:.2} of {n} Recruits \
             alone",
            income / (n * one)
        );
    }
    let end = at(2, 48);
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    let one = Game::play(line_field(), one_hall(), end);
    let two = Game::play(line_field(), two_halls(), end);
    println!("time, GDO, Colonists, One Hall, Two Halls (crystals gathered since 0:00)");
    for by in [at(0, 30), at(1, 0), at(1, 30), at(2, 0), at(2, 30), end] {
        let row = [
            gdo.mined(0, by),
            colonists.delivered(0, by),
            one.delivered(0, by),
            two.delivered(0, by),
        ];
        println!("{}: {row:?}", clock(by));
    }
}
