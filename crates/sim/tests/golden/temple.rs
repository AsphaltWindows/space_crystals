//! The Temple's golden game (slice M5-N3, `factions/cults/structures/temple.md`): a staffed Temple completes Enhanced
//! Constitution Level 1 while Enhanced Constitution Militants fight Agents.

use std::cell::RefCell;
use std::collections::BTreeMap;

use sc_sim::{Augmentation, DebugCommand, RecruitKind, SimConfig, Tech};

use super::*;

/// The Temple (id 1), player 1's, on tiles (14, 2) to (16, 4): its entrance, side B, to the east, its exit, side C, to
/// the west.
const TEMPLE: ObjectId = ObjectId(1);
/// Player 1's six Recruits, east of the Temple, which staff it.
const STAFF: [u32; 6] = [2, 3, 4, 5, 6, 7];
/// Player 1's three Militants, given Enhanced Constitution in tick 1, south-west of the Temple.
const MILITANTS: [u32; 3] = [8, 9, 10];
/// Player 2's three Agents, at the map's western edge.
const AGENTS: [u32; 3] = [11, 12, 13];
/// The research is queued in this tick, with the six inside.
const RESEARCH: Tick = 80;
/// Enhanced Constitution Level 1 completes in this tick: its 960 frames over six Recruits, the queue's tick the first.
const COMPLETED: Tick = RESEARCH + 159;
/// Player 2's Agents AttackMove onto the Militants from this tick on.
const ATTACK: Tick = 120;
/// The game's last tick: the last Agent fell at 0:24 (tick 391).
const END: Tick = 400;

thread_local! {
    /// The Militants' HP as the tick before left it, by id, for the tick the research completes in.
    static BEFORE: RefCell<BTreeMap<u32, f32>> = const { RefCell::new(BTreeMap::new()) };
}

/// Player 1 plays the Cults, player 2 the Syndicate, with debug commands on: the Cults start with no Space Crystals
/// (`factions/cults/economy.md > Starting Conditions`), and a debug command gives player 1 the research's 100, so that
/// the game is about the Temple, not about gathering; another gives the Militants their Augmentation, which the Altar's
/// game plays out (`altar.rs`).
fn cults_p1_syndicate_p2(builder: ScenarioBuilder) -> Scenario {
    let mut scenario = builder.config(SimConfig::DEBUG).build();
    scenario.setup.players[0].faction = Faction::Cults;
    scenario.setup.players[1].faction = Faction::Syndicate;
    scenario
}

/// Player 1's six Recruits (ids 2–7) go into its Temple (id 1) at tick 1, and at [`RESEARCH`] it queues Enhanced
/// Constitution Level 1 (`factions/cults/structures/temple.md > Recruit Staffing`: "100 SC, 960 frames"), all its 100
/// Space Crystals, which six Recruits research in 160 frames (`core/tech_and_research.md > Research > Faction
/// Exceptions > Cults — research is staffed by Recruits`: "Linear scaling"). Its three Militants (ids 8–10), Enhanced
/// Constitution since tick 1 (60 MaxHP: 40, +2 Constitution), stand south-west of it; at [`ATTACK`] three Agents of
/// player 2's (ids 11–13) AttackMove onto them. The Militants outrange them, 4 grid units to 2, and fire first, but the
/// Agents close in and wear #9 down, and the fight is on when the research completes at [`COMPLETED`]: every Militant
/// has 80 MaxHP from that tick (+4 Constitution) and 20 HP more, at once, its damage kept
/// (`factions/cults/augmentation.md > Research applies to everyone`). #9 falls all the same; the two others kill the
/// Agents.
pub(crate) fn cults_temple() -> Game {
    let mut builder =
        ScenarioBuilder::new(&open(28, 12)).spawn(ObjectKind::Temple, Some(P1), SpawnAt::Tile(TilePos::new(14, 2)));
    for y in 1..=6 {
        builder = builder.spawn(ObjectKind::Recruit, Some(P1), at(18, y));
    }
    for y in [7, 8, 9] {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(10, y));
    }
    for y in [6, 8, 10] {
        builder = builder.spawn(ObjectKind::Agent, Some(P2), at(1, y));
    }
    let debug = |command| PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    };
    let bank = debug(DebugCommand::SetResources {
        player: P1,
        crystals: 100,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let mut commands = vec![(1, bank)];
    for id in MILITANTS {
        let augmentation = Augmentation::of(RecruitKind::Constitution);
        let id = ObjectId(id);
        commands.push((1, debug(DebugCommand::SetAugmentation { id, augmentation })));
    }
    commands.push((1, order(P1, STAFF, OrderKind::Enter(TEMPLE))));
    let research = PlayerCommand {
        player: P1,
        command: Command::Production {
            objects: vec![TEMPLE],
            command: ProductionCommand::Research(Tech::EnhancedConstitutionL1),
        },
    };
    commands.push((RESEARCH, research));
    commands.push((ATTACK, order(P2, AGENTS, OrderKind::AttackMove(point(10, 8)))));
    Game {
        scenario: cults_p1_syndicate_p2(builder),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            let completed: Vec<&SimEvent> = (sim.events().iter())
                .filter(|e| matches!(e, SimEvent::ResearchCompleted { .. }))
                .collect();
            let techs = &sim.player(P1).expect("in the game").techs;
            if tick == RESEARCH {
                assert_eq!(sim.held_in(TEMPLE).len(), 6, "the six went in");
                let queued = SimEvent::ResearchQueued {
                    producer: TEMPLE,
                    tech: Tech::EnhancedConstitutionL1,
                };
                assert!(sim.events().contains(&queued), "queued");
                assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "paid");
            }
            if tick == COMPLETED {
                let done = SimEvent::ResearchCompleted {
                    producer: TEMPLE,
                    tech: Tech::EnhancedConstitutionL1,
                };
                assert_eq!(completed, [&done], "tick {tick}");
                assert!(!units_of(sim, P2).is_empty(), "the fight is on");
            } else {
                assert_eq!(completed, Vec::<&SimEvent>::new(), "tick {tick}");
            }
            assert_eq!(techs.len(), usize::from(tick >= COMPLETED), "tick {tick}");
            let research = sim.research();
            let max_hp = if tick >= COMPLETED { 80.0 } else { 60.0 };
            let hit = |unit: ObjectId| -> f32 {
                (sim.events().iter())
                    .filter_map(|e| match *e {
                        SimEvent::Hit { target, damage, .. } if target == unit => Some(damage),
                        _ => None,
                    })
                    .sum()
            };
            BEFORE.with_borrow_mut(|before| {
                if tick == 1 {
                    before.clear();
                }
                for id in MILITANTS {
                    let Some(unit) = sim.object(ObjectId(id)) else {
                        before.remove(&id);
                        continue;
                    };
                    let hp = unit.hp.expect("destructible");
                    assert_eq!(unit.max_hp_with(research), Some(max_hp), "tick {tick}: #{id}");
                    if tick == COMPLETED {
                        let then = before[&id];
                        assert_eq!(hp, then - hit(unit.id) + 20.0, "tick {tick}: #{id}'s HP 20 up");
                    }
                    before.insert(id, hp);
                }
                if tick == COMPLETED {
                    assert!(before.values().any(|&hp| hp < max_hp), "a Militant is hurt");
                }
            });
        },
        expect: |sim| {
            assert!(units_of(sim, P2).is_empty(), "player 2's Agents are all dead");
            let militants: Vec<&sc_sim::Object> = MILITANTS.iter().filter_map(|&id| sim.object(ObjectId(id))).collect();
            assert!(!militants.is_empty(), "a Militant survives");
            let research = sim.research();
            for militant in militants {
                assert_eq!(militant.max_hp_with(research), Some(80.0), "#{}", militant.id.0);
            }
            let techs = &sim.player(P1).expect("in the game").techs;
            assert!(techs.contains(&Tech::EnhancedConstitutionL1), "researched");
            assert_eq!(sim.held_in(TEMPLE).len(), 6, "the staff stay inside");
        },
    }
}
