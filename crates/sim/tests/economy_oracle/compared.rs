//! The economies compared (design changes 0381 to 0383 and 0385; `factions/cults/economy.md > Economy check`, its
//! Timeline and What to compare): both Cults scenarios of `cults.rs`, the GDO's reference line (`played::plates_first`)
//! and the Colonists' (`colonists::Line`), each on its own reference field, out to 6:00, with each one's figures at
//! the five checkpoints. Design change 0385 (answering Q0144 in part): "Report the new table as a question. Nothing is
//! tuned until the user has seen it."

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::Entry;
use sc_sim::testing::colonists;
use sc_sim::testing::cults::{FIELD_PLAYER, line_field};
use sc_sim::{Scenario, SimConfig, Tick};

use super::colonists::Line;
use super::cults::{Game, one_hall, two_halls};
use super::played::{plates_first, reference_base};
use super::{at, clock, seconds};

/// The comparison's checkpoints (design change 0381: "at **1:30, 2:48, 4:00, 5:00 and 6:00**").
const CHECKPOINTS: [Tick; 5] = [at(1, 30), at(2, 48), at(4, 0), at(5, 0), at(6, 0)];
/// Its lines, in the order of [`COMPARED`].
const LINES: [&str; 4] = ["Cults, One Hall", "Cults, Two Halls", "GDO", "Colonists"];
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

/// What a step gathered: the workers' deliveries, or what the plates mined.
fn gathered(e: &Entry) -> u32 {
    e.mined + e.delivered
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

/// The comparison as slice M5-S3 measured it (`docs/economy_oracle.md > The Cults > The economies compared`), by
/// line ([`LINES`]) and checkpoint ([`CHECKPOINTS`]): what each gathered over the 10 seconds before, since 0:00, its
/// bank and what it spent.
const COMPARED: [[Figures; 5]; 4] = [
    [
        row(146, 860, 660, 200),
        row(214, 2316, 2016, 300),
        row(266, 4162, 3862, 300),
        row(264, 5748, 5448, 300),
        row(264, 7338, 7038, 300),
    ],
    [
        row(90, 708, 208, 500),
        row(272, 2180, 1480, 700),
        row(362, 4676, 3876, 800),
        row(390, 6906, 6106, 800),
        row(388, 9150, 8350, 800),
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

/// Whether Two Halls has overtaken One Hall by 4:00, by income/s and by total mined, as slice M5-S3 measured it.
const OVERTAKEN: (bool, bool) = (true, true);

/// The four lines played to 6:00.
struct Compared {
    /// Each one's figures at the checkpoints.
    table: [[Figures; 5]; 4],
    /// The ticks from which Two Halls stays ahead of One Hall to 6:00: by income/s (over the [`INCOME_WINDOW`]
    /// before) and by total mined.
    ahead_from: (Option<Tick>, Option<Tick>),
}

fn compared() -> Compared {
    let end = CHECKPOINTS[4];
    let start = |scenario: Scenario| {
        let t = TestSim::with_config(scenario, SimConfig::default());
        t.sim().player(FIELD_PLAYER).expect("player 1").crystals
    };
    let one = Game::play(line_field(), one_hall(), end);
    let two = Game::play(line_field(), two_halls(), end);
    let (gdo, colonists) = (plates_first(end), Line::play(end, false));
    let lines: [(&[Entry], u32); 4] = [
        (&one.books, start(line_field())),
        (&two.books, start(line_field())),
        (&gdo.books, start(reference_base())),
        (colonists.books(), start(colonists::reference_field(4))),
    ];
    for (name, game) in [("One Hall", &one), ("Two Halls", &two)] {
        let (all, mining, idle) = game.recruits();
        println!("{name}: {all} Recruits at 6:00, {mining} mining, {idle} idle");
    }
    let ((one_window, one_mined), (two_window, two_mined)) = (running(&one.books), running(&two.books));
    Compared {
        table: lines.map(|(books, start)| CHECKPOINTS.map(|at| Figures::of(books, start, at))),
        ahead_from: (ahead_from(&two_window, &one_window), ahead_from(&two_mined, &one_mined)),
    }
}

/// What `books` gathered over the [`INCOME_WINDOW`] before each tick, and since 0:00: `[i]` is tick `i + 1`'s.
fn running(books: &[Entry]) -> (Vec<u32>, Vec<u32>) {
    let mut total = 0;
    let mined: Vec<u32> = (books.iter())
        .map(|e| {
            total += gathered(e);
            total
        })
        .collect();
    let window = INCOME_WINDOW as usize;
    let windows = (0..mined.len()).map(|i| mined[i] - if i >= window { mined[i - window] } else { 0 });
    (windows.collect(), mined)
}

/// The tick from which `ahead`'s figure stays above `behind`'s to the end (`[i]` is tick `i + 1`'s); `None` if it
/// isn't above at the end.
fn ahead_from(ahead: &[u32], behind: &[u32]) -> Option<Tick> {
    match ahead.iter().zip(behind).rposition(|(a, b)| a <= b) {
        None => Some(1),
        Some(last) if last + 1 < ahead.len() => Some(last as Tick + 2),
        Some(_) => None,
    }
}

/// Design change 0385 (answering Q0144 in part): the four lines out to 6:00, each one's figures at the five
/// checkpoints, consistent (`Figures::of`) and pinned ([`COMPARED`]). And whether Two Halls has overtaken One Hall by
/// 4:00 ("Two Halls may fall behind early, but it should **overtake One Hall by about 4:00**"), by income and by total
/// mined ([`OVERTAKEN`]).
#[test]
fn the_economies_compared_out_to_six_minutes() {
    let Compared { table, ahead_from } = compared();
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
    let [one, two, gdo, colonists] = &table;
    let at_4 = 2;
    let overtaken = (two[at_4].window > one[at_4].window, two[at_4].mined > one[at_4].mined);
    println!(
        "by 4:00 Two Halls has overtaken One Hall: by income/s {} ({:.1} against {:.1}), by total mined {} ({} against \
         {})",
        overtaken.0,
        two[at_4].income(),
        one[at_4].income(),
        overtaken.1,
        two[at_4].mined,
        one[at_4].mined
    );
    let when = |tick: Option<Tick>| tick.map_or("never".to_string(), clock);
    println!(
        "Two Halls stays ahead of One Hall from {} by income/s, from {} by total mined",
        when(ahead_from.0),
        when(ahead_from.1)
    );
    let at_6 = 4;
    println!(
        "at 6:00 Two Halls against One Hall: {:.1} against {:.1} SC/s, {} against {} mined, {} against {} in the bank",
        two[at_6].income(),
        one[at_6].income(),
        two[at_6].mined,
        one[at_6].mined,
        two[at_6].bank,
        one[at_6].bank
    );
    let at_2_48 = 1;
    let over =
        |line: &[Figures; 5], other: &[Figures; 5]| f64::from(line[at_2_48].mined) / f64::from(other[at_2_48].mined);
    println!(
        "by 2:48 One Hall has {:.2} of the GDO's crystals and {:.2} of the Colonists'; Two Halls {:.2} and {:.2}",
        over(one, gdo),
        over(one, colonists),
        over(two, gdo),
        over(two, colonists)
    );
    assert_eq!(overtaken, OVERTAKEN, "Two Halls against One Hall at 4:00");
    for ((line, row), pinned) in LINES.iter().zip(&table).zip(&COMPARED) {
        for ((at, f), pin) in CHECKPOINTS.iter().zip(row).zip(pinned) {
            assert!(
                f.near(pin),
                "{line} at {}: {f:?}, pinned {pin:?}: re-measure, update COMPARED and docs/economy_oracle.md, and \
                 tell the designer (design change 0385: the table is reported)",
                clock(*at)
            );
        }
    }
}
