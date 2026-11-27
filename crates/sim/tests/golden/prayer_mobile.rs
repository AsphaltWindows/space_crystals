//! The Prayer-mobile's golden game (slice M5-N11, `factions/cults/units/prayer_mobile.md`): one Prayer-mobile inspires
//! a group of Recruits, which walk to a Temple and punch it faster, and another suppresses a fight ahead of it.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use sc_sim::AreaAbility;
use sc_sim::buffs::Buff;

use super::*;

/// Player 1's Prayer-mobile of the north (id 1), on tile (12, 3), facing east, which casts Inspire.
const NORTH: ObjectId = ObjectId(1);
/// Player 1's four Recruits, in the west, which gather by [`NORTH`] and then punch player 2's [`TEMPLE`].
const RECRUITS: [u32; 4] = [2, 3, 4, 5];
/// Player 2's Temple, on tiles (17, 1) to (19, 3); its Storage stands east of it.
const TEMPLE: ObjectId = ObjectId(6);
/// Player 1's Prayer-mobile of the south (id 8), on tile (6, 12), facing east, which casts Suppress.
const SOUTH: ObjectId = ObjectId(8);
/// Player 1's three Militants, 3 tiles ahead of [`SOUTH`], north to south.
const MILITANTS: [u32; 3] = [9, 10, 11];
/// Player 2's six Recruits, at the map's eastern edge, which charge the Militants.
const CHARGERS: [u32; 6] = [12, 13, 14, 15, 16, 17];
/// [`NORTH`] has its 75 at the end of tick 800: it casts Inspire in this tick, and the Recruits set off for the Temple.
const INSPIRE: Tick = 801;
/// Inspire's 10 s, in ticks.
const INSPIRE_TICKS: Tick = 160;
/// Player 2's Recruits AttackMove onto the Militants in this tick.
const CHARGE: Tick = 1290;
/// [`SOUTH`] has its 100 at the end of tick 1600: it casts Suppress in this tick, over the fight.
const SUPPRESS: Tick = 1601;
/// Suppress's 4 s, in ticks.
const SUPPRESS_TICKS: Tick = 64;
/// A Recruit's FiringDuration, 8 frames (`factions/cults/units/recruit.md`): the longest a blow begun before the cast
/// takes to land (`sc_sim::suppress`).
const BLOW: Tick = 8;
/// The game's last tick: 10 s after the Suppress is over.
const END: Tick = SUPPRESS + SUPPRESS_TICKS + 160;

thread_local! {
    /// The units Suppress took in, as it was cast.
    static SUPPRESSED: RefCell<BTreeSet<ObjectId>> = const { RefCell::new(BTreeSet::new()) };
    /// The hits of the suppressed units after the 4 s.
    static AFTER: RefCell<u32> = const { RefCell::new(0) };
    /// The ticks of each of player 1's Recruits' hits on the Temple, and the longest step any of them took while
    /// inspired.
    static PUNCHES: RefCell<BTreeMap<u32, Vec<Tick>>> = const { RefCell::new(BTreeMap::new()) };
    static STRIDE: RefCell<f32> = const { RefCell::new(0.0) };
}

/// Player 1's `caster` casts `ability` from where it stands (`Command::Cast`; the client sends that point).
fn cast(caster: ObjectId, ability: AreaAbility, at: Vec2) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Cast {
            casters: vec![caster],
            ability,
            at,
        },
    }
}

/// The buffs `id` has now, if it is on the map.
fn buffs(sim: &Sim, id: ObjectId) -> Vec<Buff> {
    sim.object(id).map_or_else(Vec::new, |o| o.buffs.clone())
}

/// Player 1 (the Cults) has two Prayer-mobiles, [`NORTH`] and [`SOUTH`] (`factions/cults/units/prayer_mobile.md`),
/// facing east as they came into the game; four Recruits in the west (2–5), which gather 2 tiles west of [`NORTH`]
/// in tick 1; and three Militants ahead of [`SOUTH`] (9–11). Player 2 (the Cults) has a Temple in the north-east
/// ([`TEMPLE`]), a Storage east of it (7) and six Recruits at the eastern edge (12–17).
///
/// At [`INSPIRE`] [`NORTH`] casts Inspire (`> Placeholder stats`: "every own infantry unit within **4 grid units** of
/// the vehicle gets **+1.5 su/frame MaxSpeed** for **10 s**; Recruits also attack **25% faster**"): the four Recruits
/// get it, and walk to the Temple at 5.5 su a tick and punch it every 9 ticks, until it ends. At [`CHARGE`] player 2's
/// Recruits AttackMove onto the Militants, which shoot at them as they come; at [`SUPPRESS`], as they fight hand to
/// hand, [`SOUTH`] casts Suppress ("an arc of **90° total** in front of the vehicle's current facing, **5 grid units**
/// long; units whose centre is inside can't attack for **4 s**"): every unit of the fight is in it, of both players
/// ("enemies and own alike"), and none of them hits anything until it is over (but with a blow begun before it, which
/// lands within a Recruit's FiringDuration, [`BLOW`]), when the fight goes on.
pub(crate) fn cults_prayer_mobile() -> Game {
    let mut builder = (ScenarioBuilder::new(&open(30, 16)))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Cults)
        .spawn(ObjectKind::PrayerMobile, Some(P1), at(12, 3));
    for y in 1..=4 {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(2, y));
    }
    builder = builder
        .spawn(ObjectKind::Temple, Some(P2), SpawnAt::Tile(TilePos::new(17, 1)))
        .spawn(ObjectKind::Storage, Some(P2), SpawnAt::Tile(TilePos::new(25, 1)))
        .spawn(ObjectKind::PrayerMobile, Some(P1), at(6, 12));
    for y in [11, 12, 13] {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(9, y));
    }
    for y in 9..=14 {
        builder = builder.spawn(ObjectKind::Recruit, Some(P2), at(27, y));
    }
    let commands = vec![
        (1, order(P1, RECRUITS, OrderKind::Move(Target::Point(point(10, 3))))),
        (INSPIRE, cast(NORTH, AreaAbility::Inspire, point(12, 3))),
        (INSPIRE, order(P1, RECRUITS, OrderKind::Attack(TEMPLE))),
        (CHARGE, order(P2, CHARGERS, OrderKind::AttackMove(point(9, 12)))),
        (SUPPRESS, cast(SOUTH, AreaAbility::Suppress, point(6, 12))),
    ];
    Game {
        scenario: builder.build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SUPPRESSED.set(BTreeSet::new());
                AFTER.set(0);
                PUNCHES.set(BTreeMap::new());
                STRIDE.set(0.0);
            }
            let inspire = Buff {
                ability: AreaAbility::Inspire,
                until: INSPIRE + INSPIRE_TICKS,
            };
            // A buff works in the tick of its cast and every tick before its `until`, and ends with the last of them.
            let inspired = (INSPIRE..INSPIRE + INSPIRE_TICKS - 1).contains(&tick);
            for id in RECRUITS {
                let expected = if inspired { vec![inspire] } else { vec![] };
                assert_eq!(buffs(sim, ObjectId(id)), expected, "tick {tick}: Recruit #{id}");
            }
            assert_eq!(
                (buffs(sim, NORTH), buffs(sim, SOUTH)),
                (vec![], vec![]),
                "tick {tick}: casters"
            );
            // Inspired, they walk at 4 + 1.5 su a tick at most, and punch the Temple.
            for id in RECRUITS {
                let o = sim.object(ObjectId(id)).expect("out of the fight");
                let step = o.pos.distance(o.prev_pos);
                let top = if (INSPIRE..INSPIRE + INSPIRE_TICKS).contains(&tick) {
                    5.5
                } else {
                    4.0
                };
                assert!(step <= top + 1e-3, "tick {tick}: #{id} stepped {step}");
                if tick > INSPIRE {
                    STRIDE.with_borrow_mut(|stride| *stride = stride.max(step));
                }
            }
            for e in sim.events() {
                if let SimEvent::Hit { attacker, target, .. } = *e
                    && target == TEMPLE
                {
                    PUNCHES.with_borrow_mut(|punches| punches.entry(attacker.0).or_default().push(tick));
                }
            }
            let hits_by = |units: &BTreeSet<ObjectId>| {
                (sim.events().iter())
                    .filter(|e| matches!(e, SimEvent::Hit { attacker, .. } if units.contains(attacker)))
                    .count() as u32
            };
            if tick == SUPPRESS {
                let suppress = Buff {
                    ability: AreaAbility::Suppress,
                    until: SUPPRESS + SUPPRESS_TICKS,
                };
                let taken: BTreeSet<ObjectId> = (sim.objects())
                    .filter(|o| o.buffs.contains(&suppress))
                    .map(|o| o.id)
                    .collect();
                let militants: BTreeSet<ObjectId> = MILITANTS.iter().map(|&id| ObjectId(id)).collect();
                let chargers = (taken.iter()).filter(|id| CHARGERS.contains(&id.0)).count();
                assert!(taken.is_superset(&militants), "the Militants: {taken:?}");
                assert!(chargers >= 2, "player 2's Recruits in the fight: {taken:?}");
                assert_eq!(taken.len(), militants.len() + chargers, "no one else: {taken:?}");
                SUPPRESSED.set(taken);
            }
            if (SUPPRESS + BLOW..SUPPRESS + SUPPRESS_TICKS).contains(&tick) {
                let held = SUPPRESSED.with_borrow(|units| hits_by(units));
                assert_eq!(held, 0, "tick {tick}: they hold their fire");
            }
            if tick >= SUPPRESS + SUPPRESS_TICKS {
                let hits = SUPPRESSED.with_borrow(|units| hits_by(units));
                AFTER.with_borrow_mut(|after| *after += hits);
            }
        },
        expect: |sim| {
            assert!(AFTER.with_borrow(|after| *after) > 0, "the fight went on");
            let stride = STRIDE.with_borrow(|stride| *stride);
            assert!((stride - 5.5).abs() < 1e-3, "they walked inspired: {stride}");
            // Every 9 ticks while inspired (the attack cycle of 12 × 0.75), every 12 once it is over.
            let end = INSPIRE + INSPIRE_TICKS;
            PUNCHES.with_borrow(|punches| {
                assert_eq!(
                    punches.keys().copied().collect::<Vec<u32>>(),
                    RECRUITS,
                    "all four punch it"
                );
                for (id, ticks) in punches {
                    let gaps = |from: Tick, to: Tick| -> Vec<Tick> {
                        let inside: Vec<Tick> = (ticks.iter().copied()).filter(|t| (from..to).contains(t)).collect();
                        inside.windows(2).map(|w| w[1] - w[0]).collect()
                    };
                    let (during, after) = (gaps(INSPIRE, end), gaps(end + 12, END + 1));
                    assert!(
                        !during.is_empty() && during.iter().all(|&gap| gap == 9),
                        "#{id}: {during:?}"
                    );
                    assert!(
                        !after.is_empty() && after.iter().all(|&gap| gap == 12),
                        "#{id}: {after:?}"
                    );
                }
            });
            let punched = sim.object(TEMPLE).and_then(|o| o.hp);
            assert!(punched < Some(450.0), "the Temple was punched: {punched:?}");
        },
    }
}
