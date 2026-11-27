//! The Null Chamber's golden game (slice M6-SN, plan decision T98; `factions/syndicate/structures/null_chamber.md >
//! Null Chamber Exit Buff`, `factions/syndicate/structures/null_chamber.md > Placeholder stats`): a Tier 2 Tunnel
//! builds a Null Chamber behind its Hideout while five Guards go in and wait; the chamber finished, they take their
//! turns, four at a time, come out buffed and fight three Peacekeepers beside a sixth Guard that never went in. A
//! Peacekeeper's hit takes 7 HP of a buffed Guard and 9 of the other, and each buff ends 20 s after its Guard came out.
//! Auto-targeting is off, so that every attack is an order's.

use std::cell::RefCell;
use std::collections::BTreeSet;

use sc_sim::buffs::BuffKind;
use sc_sim::{BuildingOrientation, DebugCommand, SimConfig, TunnelCommand};

use super::*;

/// Player 1's Tunnel (id 1), unturned on tiles (12, 10) to (15, 13): Side A faces north. Its Hideout (id 2) on (16,
/// 14), the Null Chamber's TechPrerequisite.
const TUNNEL: ObjectId = ObjectId(1);
/// The five Guards that go in, and the sixth, which stays out.
const INSIDE: [ObjectId; 5] = [ObjectId(3), ObjectId(4), ObjectId(5), ObjectId(6), ObjectId(7)];
const OUTSIDE: ObjectId = ObjectId(8);
/// Player 2's three Peacekeepers on Hold north-east of the Tunnel, in sight of where the Guards come out.
const PEACEKEEPERS: [ObjectId; 3] = [ObjectId(9), ObjectId(10), ObjectId(11)];
const GROUP_AT: Vec2 = Vec2::new(1060.0, 380.0);
/// The Null Chamber (id 12), which the Tunnel expands on tiles (9, 11) to (10, 12), in its Tier 2 area, complete 480
/// frames after the click.
const CHAMBER: ObjectId = ObjectId(12);
const CHAMBER_AT: TilePos = TilePos::new(9, 11);
const EXPAND: Tick = 2;
const BUILT: Tick = EXPAND + 480;
/// "each Null Chamber buffs **4 units at a time**, each buff taking **2 s**", "duration **20 s**", ticks.
const TURN: Tick = 32;
const DURATION: Tick = 320;
/// The five sent in, all in before the chamber is finished; ejected once their turns are done, the fifth's 2 s after
/// the first four's.
const ENTER: Tick = 300;
const EJECT: Tick = BUILT + 2 * TURN + 16;
/// The Peacekeepers attack: one the Guard that stayed out, two the first two buffed; 3 s later the six Guards attack
/// them, the Peacekeeper on the Guard that stayed out last.
const FIGHT: Tick = EJECT + 60;
const ANSWER: Tick = FIGHT + 48;
/// The game's last tick, after every buff's end.
const END: Tick = EJECT + DURATION + 120;
/// A Peacekeeper's hit (`factions/gdo/units/peacekeeper.md`: "Damage: 10") on a Guard (PointArmor 1): 10 − 1, and on a
/// buffed one 10 × 0.8 − 1 ("damage taken = Attack Damage × (1 − reduction) − armor").
const PLAIN_HIT: f32 = 9.0;
const BUFFED_HIT: f32 = 7.0;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    /// The Guards' ticks into the network and out of it.
    entries: Vec<(Tick, ObjectId)>,
    exits: Vec<(Tick, ObjectId)>,
    /// The Guards holding the buff at the end of the last tick: the hits of this one meet the reduction.
    buffed: BTreeSet<ObjectId>,
    /// The Peacekeepers' hits on buffed Guards and on the others.
    reduced: usize,
    plain: usize,
    /// The buffs seen to end, each in the 319th tick after its Guard came out.
    ended: usize,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

fn player_debug(command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    }
}

/// Player 1 (the Syndicate) has a Tunnel, set to Tier 2 and given 200 Space Crystals by debug commands in tick 1 (its
/// upgrade and its mining aren't what the game is about), a Hideout and six Guards north of the Tunnel; player 2 (the
/// GDO), three Peacekeepers on Hold north-east of it. At [`EXPAND`] the Tunnel expands a Null Chamber (200, 480
/// frames), complete at [`BUILT`]. Five Guards go in from [`ENTER`] on and take no turn until it is finished; then the
/// first four in take theirs, done 2 s later, and the fifth begins, done 2 s after them. Ejected at [`EJECT`], the five
/// come out one after another, each holding the buff for 20 s from the tick it comes out. At [`FIGHT`] the Peacekeepers
/// attack: one the sixth Guard, two two of the buffed; at [`ANSWER`] the six Guards attack them in turn and kill them.
/// A hit takes 7 HP of a Guard that held the buff as the tick began, and 9 of the sixth. Each buff ends in the 319th
/// tick after its Guard came out, and none is left at [`END`].
pub(crate) fn syndicate_null_chamber() -> Game {
    let mut builder = ScenarioBuilder::new(&open(40, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(12, 10)))
        .spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(TilePos::new(16, 14)));
    for i in 0..5 {
        let at = Vec2::new(866.0 + 40.0 * (i % 2) as f32, 520.0 - 44.0 * i as f32);
        builder = builder.spawn(GUARD, Some(P1), SpawnAt::Point(at));
    }
    builder = builder.spawn(GUARD, Some(P1), SpawnAt::Point(Vec2::new(1160.0, 520.0)));
    for dx in [-40.0, 0.0, 40.0] {
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(GROUP_AT + Vec2::new(dx, 0.0)));
    }
    let bank = player_debug(DebugCommand::SetResources {
        player: P1,
        crystals: 200,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let tier = player_debug(DebugCommand::SetTier { id: TUNNEL, tier: 2 });
    let tunnel = |command: TunnelCommand| PlayerCommand {
        player: P1,
        command: Command::Tunnel {
            tunnel: TUNNEL,
            command,
        },
    };
    let expand = tunnel(TunnelCommand::Expand {
        kind: ObjectKind::NullChamber,
        origin: CHAMBER_AT,
        orientation: BuildingOrientation::default(),
    });
    let mut commands = vec![
        (1, player_debug(DebugCommand::SetAutoTargeting { on: false })),
        (1, bank),
        (1, tier),
        (1, order(P2, PEACEKEEPERS.map(|id| id.0), OrderKind::Hold)),
        (EXPAND, expand),
        (ENTER, order(P1, INSIDE.map(|id| id.0), OrderKind::Enter(TUNNEL))),
    ];
    for _ in INSIDE {
        commands.push((EJECT, tunnel(TunnelCommand::Eject { kind: GUARD })));
    }
    let targets = [OUTSIDE, INSIDE[0], INSIDE[1]];
    for (pk, target) in PEACEKEEPERS.into_iter().zip(targets) {
        commands.push((FIGHT, order(P2, [pk.0], OrderKind::Attack(target))));
    }
    let guards = || INSIDE.into_iter().chain([OUTSIDE]).collect::<Vec<ObjectId>>();
    for (n, pk) in [PEACEKEEPERS[1], PEACEKEEPERS[2], PEACEKEEPERS[0]]
        .into_iter()
        .enumerate()
    {
        let attack = Command::Order {
            units: guards(),
            kind: OrderKind::Attack(pk),
            queue: n > 0,
        };
        commands.push((
            ANSWER,
            PlayerCommand {
                player: P1,
                command: attack,
            },
        ));
    }
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
            }
            let buff_of = |o: &sc_sim::Object| {
                (o.buffs.iter())
                    .find(|buff| buff.ability == BuffKind::NullChamber)
                    .map(|buff| buff.until)
            };
            for e in sim.events() {
                match *e {
                    SimEvent::ExpansionStarted { tunnel, site } => {
                        assert_eq!((tick, tunnel, site), (EXPAND, TUNNEL, CHAMBER));
                    }
                    SimEvent::ExpansionCompleted { tunnel, site } => {
                        assert_eq!((tick, tunnel, site), (BUILT, TUNNEL, CHAMBER));
                    }
                    SimEvent::EnteredNetwork { unit, .. } => {
                        assert!(INSIDE.contains(&unit) && tick < BUILT, "tick {tick}: {unit:?} in");
                        SEEN.with_borrow_mut(|seen| seen.entries.push((tick, unit)));
                    }
                    SimEvent::LeftNetwork { unit, .. } => {
                        let o = sim.object(unit).expect("out");
                        assert_eq!(buff_of(o), Some(tick + DURATION), "tick {tick}: {unit:?} out buffed");
                        SEEN.with_borrow_mut(|seen| seen.exits.push((tick, unit)));
                    }
                    SimEvent::Hit {
                        attacker,
                        target,
                        damage,
                        ..
                    } if PEACEKEEPERS.contains(&attacker) => SEEN.with_borrow_mut(|seen| {
                        let buffed = seen.buffed.contains(&target);
                        let expected = if buffed { BUFFED_HIT } else { PLAIN_HIT };
                        assert_eq!(
                            damage, expected,
                            "tick {tick}: {attacker:?} on {target:?}, buffed {buffed}"
                        );
                        *(if buffed { &mut seen.reduced } else { &mut seen.plain }) += 1;
                    }),
                    _ => {}
                }
            }
            // No turn before the chamber is finished; four done 2 s after it, the fifth 2 s later.
            let charged = sim.held().filter(|held| held.service.charged()).count();
            if tick < BUILT {
                assert!(sim.held().all(|held| held.service.is_none()), "tick {tick}: a turn");
            } else if tick == BUILT + TURN - 2 {
                assert_eq!(charged, 0, "tick {tick}");
            } else if tick == BUILT + TURN - 1 || tick == BUILT + 2 * TURN - 2 {
                assert_eq!(charged, 4, "tick {tick}");
            } else if tick == BUILT + 2 * TURN - 1 {
                assert_eq!(charged, 5, "tick {tick}");
            }
            SEEN.with_borrow_mut(|seen| {
                for &(out, unit) in &seen.exits {
                    let held = sim.object(unit).and_then(buff_of);
                    if tick == out + DURATION - 2 {
                        assert_eq!(held, Some(out + DURATION), "tick {tick}: {unit:?}'s buff lasts");
                    } else if tick == out + DURATION - 1 {
                        assert_eq!(held, None, "tick {tick}: {unit:?}'s buff is over");
                        seen.ended += 1;
                    }
                }
                seen.buffed = (sim.objects()).filter(|o| buff_of(o).is_some()).map(|o| o.id).collect();
            });
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                let came_in: Vec<ObjectId> = seen.entries.iter().map(|&(_, unit)| unit).collect();
                let out: Vec<ObjectId> = seen.exits.iter().map(|&(_, unit)| unit).collect();
                assert_eq!(came_in.len(), 5);
                assert_eq!(out, came_in, "the five out, the first in first");
                assert!(seen.exits.iter().all(|&(tick, _)| tick >= EJECT));
                assert!(
                    seen.reduced >= 2 && seen.plain >= 2,
                    "{} reduced hits, {} plain",
                    seen.reduced,
                    seen.plain
                );
                assert_eq!(seen.ended, 5, "every buff ended");
                assert!(seen.buffed.is_empty());
            });
            assert!(
                PEACEKEEPERS.iter().all(|&id| sim.object(id).is_none()),
                "player 2's Peacekeepers are dead"
            );
            assert!(
                INSIDE.iter().chain([&OUTSIDE]).all(|&id| sim.object(id).is_some()),
                "the six Guards live"
            );
            let chamber = sim.object(CHAMBER).expect("the Null Chamber");
            assert_eq!(
                (chamber.kind, chamber.hp, sim.parent_tunnel(CHAMBER)),
                (ObjectKind::NullChamber, Some(400.0), Some(TUNNEL))
            );
            assert_eq!(sim.player(P1).map(|p| p.crystals), Some(0), "all spent");
        },
    }
}
