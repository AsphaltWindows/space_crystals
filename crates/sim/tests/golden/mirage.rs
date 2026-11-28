//! Mirage's golden game (slice M6-CM, `factions/cults/units/disciple.md > Spells > Mirage`): a Disciple's illusions
//! walk into two Guards' fire and die of double damage; the Disciple casts again, its two others vanishing for a new
//! set, and walks into the Guards' fire itself: its death takes its new set along.

use std::cell::RefCell;

use super::*;

/// Player 1's Disciple, alone of its player's at the start.
const DISCIPLE: u32 = 1;
/// Player 2's Guards, on the east side.
const GUARDS: [u32; 2] = [2, 3];
/// The first Mirage's illusions, cast in tick 1: north, east, south and west of the Disciple.
const FIRST: [u32; 4] = [4, 5, 6, 7];
/// The two of them that walk into the Guards' fire, north and east, and the two that stay, south and west. The east
/// one, nearer, dies in tick 113 (0:07.1), the north one in tick 137 (0:08.6).
const SENT: [u32; 2] = [4, 5];
const KEPT: [u32; 2] = [6, 7];
/// The second Mirage's illusions.
const SECOND: [u32; 4] = [8, 9, 10, 11];
/// The Disciple casts again in this tick, its 50 energy back (0.5 a second from tick 1, 1600 ticks, and a margin).
const RECAST: Tick = 1610;
/// The Disciple walks into the Guards' fire in this tick, its new illusions staying where they came.
const WALK: Tick = 1620;
/// The Disciple dies in this tick, its new illusions with it, at 1:49.6.
const DEATH: Tick = 1754;
/// The game's last tick.
const END: Tick = DEATH + 16;

thread_local! {
    /// What the game did so far: each Mirage as `(tick, illusions)`, each death and each removal as `(tick, id)`.
    static STORY: RefCell<Story> = RefCell::new(Story::default());
}

#[derive(Default)]
struct Story {
    mirages: Vec<(Tick, Vec<u32>)>,
    deaths: Vec<(Tick, u32)>,
    removals: Vec<(Tick, u32)>,
}

/// The ids removed in `tick`, of `removals`.
fn removed_in(removals: &[(Tick, u32)], tick: Tick) -> Vec<u32> {
    removals.iter().filter(|r| r.0 == tick).map(|r| r.1).collect()
}

/// Player 1 (the Cults) has a Disciple (1) on the west side of an open map; player 2 (the Syndicate) two Guards (2,
/// 3), idle, 12 grid units east, out of its sight. The Disciple casts Mirage in tick 1 ("An **instant** spell: the
/// Disciple creates **4 illusions** of itself", 50 energy), and its north and east illusions (4, 5) walk into the
/// Guards' reach: the Guards shoot them, and each hit on an illusion takes twice what the Guard's Damage does to a
/// Disciple from that side ("illusions take **double damage from all sources**"; `systems/combat.md >
/// DirectionalArmor`: 0.75 from the front, 1 from a side, 1.25 from behind), and they die. At [`RECAST`] the Disciple,
/// its energy back, casts again: its south and west illusions (6, 7) vanish, with no death, and four new ones come
/// (8–11) ("the old set vanishes and a new set of 4 appears; a Disciple never has more than one set"). At [`WALK`] it
/// walks into the Guards' reach alone, and the Guards kill it at [`DEATH`]; its illusions leave the game with it, with
/// no death ("an illusion lasts until it **dies**, its **parent dies**, or the parent **casts Mirage again**").
pub(crate) fn cults_mirage() -> Game {
    let builder = (ScenarioBuilder::new(&open(24, 15)))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Syndicate)
        .spawn(ObjectKind::Disciple, Some(P1), at(5, 7))
        .spawn(GUARD, Some(P2), at(17, 6))
        .spawn(GUARD, Some(P2), at(17, 8));
    let mirage = || PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: vec![ObjectId(DISCIPLE)],
            ability: Ability::Mirage,
        },
    };
    let commands = vec![
        (1, mirage()),
        (2, order(P1, SENT, OrderKind::Move(Target::Point(point(15, 7))))),
        (RECAST, mirage()),
        (
            WALK,
            order(P1, [DISCIPLE], OrderKind::Move(Target::Point(point(15, 7)))),
        ),
    ];
    Game {
        scenario: builder.build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            STORY.with_borrow_mut(|story| {
                if tick == 1 {
                    *story = Story::default();
                }
                for event in sim.events() {
                    match event {
                        SimEvent::Mirage { illusions, .. } => {
                            story.mirages.push((tick, illusions.iter().map(|id| id.0).collect()));
                        }
                        SimEvent::Died { id, .. } => story.deaths.push((tick, id.0)),
                        SimEvent::Removed(id) => story.removals.push((tick, id.0)),
                        &SimEvent::Hit {
                            attacker,
                            target,
                            damage,
                            ..
                        } => {
                            assert!(GUARDS.contains(&attacker.0), "tick {tick}: #{} fired", attacker.0);
                            let sides = [0.75, 1.0, 1.25].map(|side| 18.0 * side);
                            let illusion = FIRST.contains(&target.0) || SECOND.contains(&target.0);
                            let factor = if illusion { 2.0 } else { 1.0 };
                            assert!(
                                sides.iter().any(|&side| damage == side * factor),
                                "tick {tick}: #{} took {damage}",
                                target.0
                            );
                        }
                        _ => {}
                    }
                }
            });
        },
        expect: |sim| {
            STORY.with_borrow(|story| {
                assert_eq!(story.mirages, [(1, FIRST.to_vec()), (RECAST, SECOND.to_vec())]);
                let dead: Vec<u32> = story.deaths.iter().map(|d| d.1).collect();
                assert_eq!(
                    dead,
                    [SENT[1], SENT[0], DISCIPLE],
                    "the two sent died, the east one first, then the Disciple"
                );
                assert!(story.deaths.iter().take(2).all(|d| d.0 < RECAST), "{:?}", story.deaths);
                assert_eq!(removed_in(&story.removals, RECAST), KEPT, "the old set at the recast");
                assert_eq!(story.deaths.last(), Some(&(DEATH, DISCIPLE)));
                let mut with_it = vec![DISCIPLE];
                with_it.extend(SECOND);
                assert_eq!(removed_in(&story.removals, DEATH), with_it, "the new set with it");
            });
            assert!(units_of(sim, P1).is_empty(), "nothing of player 1's left");
            assert_eq!(units_of(sim, P2).len(), 2, "the Guards stand");
        },
    }
}
