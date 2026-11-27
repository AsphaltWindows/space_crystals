//! The Shrine's golden game (slice M5-N5, `factions/cults/structures/shrine.md`): a manned Shrine heals wounded
//! Militants mid-fight and buffs a Disciple's energy.

use std::cell::RefCell;
use std::collections::BTreeMap;

use sc_sim::AreaAbility;
use sc_sim::buffs::Buff;
use sc_sim::energy::ONE;

use super::*;

/// The Shrine (id 1), player 1's, on tiles (20, 2) to (21, 3): its entrance, side B, to the east, its exit, side C, to
/// the west.
const SHRINE: ObjectId = ObjectId(1);
/// Player 1's eight Recruits, east of the Shrine, which go into it in tick 1.
const RECRUITS: [u32; 8] = [2, 3, 4, 5, 6, 7, 8, 9];
/// Player 1's three Militants, south-west of the Shrine, north to south.
const MILITANTS: [u32; 3] = [10, 11, 12];
/// Player 1's Disciple, east of the Militants, out of the fight.
const DISCIPLE: ObjectId = ObjectId(13);
/// Player 2's three Agents, at the map's western edge.
const AGENTS: [u32; 3] = [14, 15, 16];
/// Player 2's Agents AttackMove onto the Militants in this tick.
const ATTACK: Tick = 520;
/// The Shrine casts Health regeneration over the Militants in this tick, mid-fight, at [`HEAL_AT`]: it has its 75 from
/// tick 664.
const HEAL: Tick = 680;
/// Where: the middle Militant's spot.
const HEAL_AT: (i32, i32) = (12, 9);
/// The Shrine casts Energy regeneration on the Disciple in this tick, at its spot: it has its 50 again from tick 1064.
const BOOST: Tick = 1072;
/// The buffs' 15 s and 20 s, in ticks (`> Placeholder stats`).
const HEAL_TICKS: Tick = 240;
const BOOST_TICKS: Tick = 320;
/// What eight Recruits inside add to the Shrine's energy in a tick: 0.25 a second each (`> Energy`), 2 a second.
const FULL: u32 = 8 * ONE / 64;
/// What a Disciple regenerates in a tick, 0.5 a second (`factions/cults/units/disciple.md > Energy`), and
/// what Energy regeneration adds to it, 1 a second.
const DISCIPLE_RATE: u32 = ONE / 32;
const BOOST_RATE: u32 = ONE / 16;
/// The game's last tick: a second after the Disciple's buff ends.
const END: Tick = BOOST + BOOST_TICKS + 16;

/// What the last tick left: the Shrine's energy and how many were inside it, each Militant's HP by id, and the
/// Disciple's energy.
#[derive(Default)]
struct Before {
    shrine: u32,
    inside: usize,
    militants: BTreeMap<u32, f32>,
    disciple: u32,
}

thread_local! {
    static BEFORE: RefCell<Before> = RefCell::new(Before::default());
}

/// Player 1's Shrine casts `ability` at the centre of tile `at`.
fn cast(ability: AreaAbility, at: (i32, i32)) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Cast {
            casters: vec![SHRINE],
            ability,
            at: point(at.0, at.1),
        },
    }
}

/// The buffs `id` has now, if it is on the map.
fn buffs(sim: &Sim, id: u32) -> Vec<Buff> {
    sim.object(ObjectId(id)).map_or_else(Vec::new, |o| o.buffs.clone())
}

/// Player 1 (the Cults) has a Shrine (id 1), its eight Recruits east of it (2–9), three Militants south-west of it
/// (10–12) and a Disciple east of them (13); player 2 (the Syndicate) three Agents at the map's western edge (14–16).
/// The Recruits go into the Shrine in tick 1 (`factions/cults/structures/shrine.md > Ritual Participation`), all eight
/// by tick 94, and charge its energy, 0.25 a second each (`> Energy`): 2 a second, every tick of the game from then on.
/// At [`ATTACK`] the Agents AttackMove onto the Militants, and the middle ones of both sides fall together before the
/// Shrine has its 75. At [`HEAL`] it casts Health regeneration over the two Militants left, both wounded and under fire
/// (`> Placeholder stats`: "+2 HP per second for 15 seconds"): they get its buff, and the Agents in its circle don't
/// ("Only the caster's own units are affected"); for its 240 ticks each heals 0.125 HP a tick after the tick's hits,
/// up to its MaxHP, and the two kill the two Agents, which would have killed them without it (as the game played when
/// it was made). At [`BOOST`] the Shrine casts Energy regeneration on the Disciple ("+1 per second for 20 seconds"):
/// its energy rises by 1.5 a second for 320 ticks, its own 0.5 and the buff's 1, then by 0.5 again.
pub(crate) fn cults_shrine() -> Game {
    let mut builder = (ScenarioBuilder::new(&open(30, 14)))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Shrine, Some(P1), SpawnAt::Tile(TilePos::new(20, 2)));
    for y in 1..=8 {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(24, y));
    }
    for y in [8, 9, 10] {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(12, y));
    }
    builder = builder.spawn(ObjectKind::Disciple, Some(P1), at(19, 9));
    for y in [7, 9, 11] {
        builder = builder.spawn(ObjectKind::Agent, Some(P2), at(1, y));
    }
    let commands = vec![
        (1, order(P1, RECRUITS, OrderKind::Enter(SHRINE))),
        (ATTACK, order(P2, AGENTS, OrderKind::AttackMove(point(12, 9)))),
        (HEAL, cast(AreaAbility::HealthRegeneration, HEAL_AT)),
        (BOOST, cast(AreaAbility::EnergyRegeneration, (19, 9))),
    ];
    Game {
        scenario: builder.build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            let shrine = sim.object(SHRINE).expect("the Shrine stands");
            let energy = shrine.energy.expect("an energy pool");
            let inside = sim.held_in(SHRINE).len();
            let casts: Vec<&SimEvent> = (sim.events().iter())
                .filter(|e| matches!(e, SimEvent::Cast { .. }))
                .collect();
            let cast_event = |ability, at: (i32, i32)| SimEvent::Cast {
                caster: SHRINE,
                ability,
                at: point(at.0, at.1),
                facing: 0.0,
            };
            match tick {
                HEAL => assert_eq!(casts, [&cast_event(AreaAbility::HealthRegeneration, HEAL_AT)]),
                BOOST => assert_eq!(casts, [&cast_event(AreaAbility::EnergyRegeneration, (19, 9))]),
                _ => assert_eq!(casts, Vec::<&SimEvent>::new(), "tick {tick}"),
            }
            // Only the caster's own units, and none of player 2's, ever.
            for id in AGENTS {
                assert_eq!(buffs(sim, id), [], "tick {tick}: Agent #{id}");
            }
            let heal = Buff {
                ability: AreaAbility::HealthRegeneration.into(),
                until: HEAL + HEAL_TICKS,
            };
            // A buff works in the tick of its cast and every tick before its `until`, and ends with the last of them.
            let healing = (HEAL..HEAL + HEAL_TICKS).contains(&tick);
            let heal_held = (HEAL..HEAL + HEAL_TICKS - 1).contains(&tick);
            let boost = Buff {
                ability: AreaAbility::EnergyRegeneration.into(),
                until: BOOST + BOOST_TICKS,
            };
            let boosted = (BOOST..BOOST + BOOST_TICKS).contains(&tick);
            let boost_held = (BOOST..BOOST + BOOST_TICKS - 1).contains(&tick);
            let hit = |unit: u32| -> f32 {
                (sim.events().iter())
                    .filter_map(|e| match *e {
                        SimEvent::Hit { target, damage, .. } if target == ObjectId(unit) => Some(damage),
                        _ => None,
                    })
                    .sum()
            };
            let disciple = sim.object(DISCIPLE).expect("the Disciple stays out of the fight");
            let disciple_energy = disciple.energy.expect("an energy pool");
            assert_eq!(
                disciple.buffs,
                if boost_held { vec![boost] } else { vec![] },
                "tick {tick}"
            );
            BEFORE.with_borrow_mut(|before| {
                if tick == 1 {
                    *before = Before {
                        disciple: 50 * ONE,
                        militants: MILITANTS.iter().map(|&id| (id, 40.0)).collect(),
                        ..Before::default()
                    };
                }
                // The Shrine's energy: 2 a second with the eight inside, less what each cast costs.
                if before.inside == 8 {
                    let cost = match tick {
                        HEAL => 75 * ONE,
                        BOOST => 50 * ONE,
                        _ => 0,
                    };
                    assert_eq!(energy + cost, before.shrine + FULL, "tick {tick}");
                    assert_eq!(inside, 8, "tick {tick}: they stay inside");
                } else {
                    assert!(energy >= before.shrine, "tick {tick}");
                }
                // The Militants: under the buff, 0.125 HP a tick after the tick's hits, up to the MaxHP; else only hits.
                for id in MILITANTS {
                    let Some(unit) = sim.object(ObjectId(id)) else {
                        before.militants.remove(&id);
                        continue;
                    };
                    let hp = unit.hp.expect("destructible");
                    let then = before.militants[&id] - hit(id);
                    let has = buffs(sim, id);
                    assert_eq!(
                        has,
                        if heal_held { vec![heal] } else { vec![] },
                        "tick {tick}: Militant #{id}"
                    );
                    if healing {
                        assert_eq!(
                            hp,
                            if then < 40.0 { (then + 0.125).min(40.0) } else { then },
                            "tick {tick}: #{id}"
                        );
                    } else {
                        assert_eq!(hp, then, "tick {tick}: #{id}");
                    }
                    if tick == HEAL {
                        assert!(then < 40.0, "tick {tick}: #{id} is wounded");
                    }
                    before.militants.insert(id, hp);
                }
                if tick == HEAL {
                    assert_eq!(
                        before.militants.keys().copied().collect::<Vec<u32>>(),
                        [10, 12],
                        "the two left"
                    );
                    assert!(
                        AGENTS.iter().any(|&id| sim.object(ObjectId(id)).is_some()),
                        "the fight is on"
                    );
                }
                // The Disciple: its own 0.5 a second, and the buff's 1 while it lasts.
                let rate = if boosted {
                    DISCIPLE_RATE + BOOST_RATE
                } else {
                    DISCIPLE_RATE
                };
                assert_eq!(disciple_energy, before.disciple + rate, "tick {tick}");
                *before = Before {
                    shrine: energy,
                    inside,
                    disciple: disciple_energy,
                    militants: std::mem::take(&mut before.militants),
                };
            });
        },
        expect: |sim| {
            for id in AGENTS {
                assert!(sim.object(ObjectId(id)).is_none(), "Agent #{id} is dead");
            }
            let alive: Vec<u32> = MILITANTS
                .iter()
                .copied()
                .filter(|&id| sim.object(ObjectId(id)).is_some())
                .collect();
            assert_eq!(alive, [10, 12], "the two healed Militants win");
            let disciple = sim.object(DISCIPLE).expect("the Disciple lives");
            let gained = END * DISCIPLE_RATE + BOOST_TICKS * BOOST_RATE;
            assert_eq!(
                disciple.energy,
                Some(50 * ONE + gained),
                "0.5 a second, and 20 from the buff"
            );
            assert_eq!(sim.held_in(SHRINE).len(), 8, "the participants stay inside");
            assert!(sim.objects().all(|o| o.buffs.is_empty()), "every buff ended");
        },
    }
}
