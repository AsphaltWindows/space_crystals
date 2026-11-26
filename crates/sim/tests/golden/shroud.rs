//! Shroud's golden game (slice M5-N4b, `factions/cults/units/disciple.md > Spells > Shroud`): three Disciples shroud
//! Militants under the fire of Guards and a Grenadier; the middle one's cloud joins the two others', its Stop splits
//! them, it joins them again, and they run dry.

use std::cell::RefCell;

use sc_sim::shroud::Cloud;

use super::*;

/// Player 1's Disciples, west to east: 1 and 3, 8 grid units apart, two clouds; 2, 4 grid units from each, whose
/// circle joins them.
const DISCIPLES: [u32; 3] = [1, 2, 3];
/// Player 1's Militants, holding: 4 and 5 under 1's circle and 8 and 9 under 3's, in front, 6 and 7 under 2's alone,
/// behind.
const MILITANTS: [u32; 6] = [4, 5, 6, 7, 8, 9];
/// Player 2's three Guards and its Grenadier, south of them, which AttackMove onto them from tick 1.
const ATTACKERS: [u32; 4] = [10, 11, 12, 13];
/// Disciple 2 channels in this tick, which joins the two clouds into one, both holding fire.
const JOIN: Tick = 67;
/// Disciple 2 is given a Stop in this tick, which splits the cloud, its buffer holding fire.
const SPLIT: Tick = 112;
/// Disciple 2 channels again in this tick, which joins the two clouds again, both holding fire.
const REJOIN: Tick = 148;
/// 1 and 3 run dry in this tick: the 50 energy a Disciple starts with pays 177 ticks of the drain, from tick 1
/// (`factions/cults/units/disciple.md > Placeholder stats > Shroud numbers`), which leaves 2's cloud alone.
const DRY: Tick = 178;
/// 2 runs dry in this tick, its buffer holding fire.
const DRY_2: Tick = 292;
/// Militants 6, 7 and 9, the three left, AttackMove onto the Grenadier in this tick, which has stood out of their
/// range, shooting Disciple 2.
const FINISH: Tick = 310;
/// The game's last tick: the Grenadier fell at 0:20.5 (tick 328).
const END: Tick = 336;

thread_local! {
    /// The clouds as the tick before left them: each one's members' ids, and its buffer.
    static BEFORE: RefCell<Vec<(Vec<u32>, f32)>> = const { RefCell::new(Vec::new()) };
}

/// The clouds: each one's members' ids, and its buffer.
fn clouds(sim: &Sim) -> Vec<(Vec<u32>, f32)> {
    let ids = |c: &Cloud| c.members.iter().map(|m| m.id.0).collect();
    sim.clouds().list().iter().map(|c| (ids(c), c.buffer)).collect()
}

/// The clouds `tick` has, as their members' ids, and who is under them: the Disciples channelling and the Militants
/// whose centres are inside their circles.
fn phase(tick: Tick) -> (Vec<Vec<u32>>, Vec<u32>) {
    let apart = (vec![vec![1], vec![3]], vec![1, 3, 4, 5, 8, 9]);
    let joined = (vec![vec![1, 2, 3]], (1..=9).collect());
    match tick {
        ..JOIN => apart,
        JOIN..SPLIT => joined,
        SPLIT..REJOIN => apart,
        REJOIN..DRY => joined,
        DRY..DRY_2 => (vec![vec![2]], vec![2, 6, 7]),
        _ => (vec![], vec![]),
    }
}

/// Player 1 (the Cults) has three Disciples in a row (ids 1–3) and six Militants on Hold (4–9); player 2 (the
/// Syndicate) three Guards and a Grenadier (10–13), 7 grid units south, which AttackMove onto them at tick 1. Disciples
/// 1 and 3 channel Shroud from tick 1 ("**Clouds connect**: overlapping or touching circles form one cloud"; 8 grid
/// units apart, theirs don't): the fire at the Militants under them goes into their buffers and comes out 8 frames
/// later over each cloud's area (`> Spells > Shroud`). At [`JOIN`] Disciple 2, between them, channels too: its circle
/// overlaps both, which joins the three into one cloud, and each of the two releases its buffer at once over its own
/// area (design change 0334, Q0113); at [`SPLIT`] its Stop splits the cloud, which releases the joint buffer at once
/// over its three circles; at [`REJOIN`] it joins them again. At [`DRY`] 1 and 3 run dry, which leaves 2's cloud, over
/// the Militants behind, and at [`DRY_2`] 2 runs dry. While a unit is under a cloud, every hit on it is a release; at
/// each of those ticks every buffer goes at once, over its cloud as it was. Two of the Guards fall while the clouds
/// are up, and no unit of player 1's; in the open, the third Guard and the Grenadier kill the three Militants in front,
/// and the Guard falls; the Grenadier, the last, has stood out of the holding Militants' range, and falls once they
/// AttackMove onto it at [`FINISH`].
pub(crate) fn cults_shroud() -> Game {
    let mut builder = (ScenarioBuilder::new(&open(24, 15)))
        .faction(P1, Faction::Cults)
        .faction(P2, Faction::Syndicate);
    for x in [7, 11, 15] {
        builder = builder.spawn(ObjectKind::Disciple, Some(P1), at(x, 6));
    }
    for (x, y) in [(6, 7), (8, 7), (10, 5), (12, 5), (14, 7), (16, 7)] {
        builder = builder.spawn(ObjectKind::Militant, Some(P1), at(x, y));
    }
    for x in [7, 11, 15] {
        builder = builder.spawn(GUARD, Some(P2), at(x, 13));
    }
    builder = builder.spawn(ObjectKind::Grenadier, Some(P2), at(12, 13));
    let shroud = |units| order(P1, units, OrderKind::Shroud);
    let commands = vec![
        (1, shroud(vec![1, 3])),
        (1, order(P1, MILITANTS, OrderKind::Hold)),
        (1, order(P2, ATTACKERS, OrderKind::AttackMove(point(11, 7)))),
        (JOIN, shroud(vec![2])),
        (SPLIT, order(P1, [2], OrderKind::Stop)),
        (REJOIN, shroud(vec![2])),
        (FINISH, order(P1, [6, 7, 9], OrderKind::AttackMove(point(12, 11)))),
    ];
    Game {
        scenario: builder.build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            let now = clouds(sim);
            let (expected, covered) = phase(tick);
            let members: Vec<Vec<u32>> = now.iter().map(|c| c.0.clone()).collect();
            assert_eq!(members, expected, "tick {tick}");
            let released: Vec<(usize, f32)> = (sim.events().iter())
                .filter_map(|e| match e {
                    SimEvent::ShroudReleased { circles, damage } => Some((circles.len(), *damage)),
                    _ => None,
                })
                .collect();
            BEFORE.with_borrow_mut(|before| {
                if tick == 1 {
                    before.clear();
                }
                let had: Vec<(usize, f32)> = (before.iter())
                    .filter(|c| c.1 > 0.0)
                    .map(|c| (c.0.len(), c.1))
                    .collect();
                if [JOIN, SPLIT, REJOIN, DRY, DRY_2].contains(&tick) {
                    assert_eq!(
                        released, had,
                        "tick {tick}: every buffer goes at once, over its cloud as it was"
                    );
                }
                if [JOIN, SPLIT, REJOIN, DRY_2].contains(&tick) {
                    assert!(!had.is_empty(), "tick {tick}: a buffer holds fire");
                }
                *before = now;
            });
            for event in sim.events() {
                if let SimEvent::Hit { attacker, target, .. } = event
                    && covered.contains(&target.0)
                {
                    assert!(
                        DISCIPLES.contains(&attacker.0),
                        "tick {tick}: #{} hit by #{}",
                        target.0,
                        attacker.0
                    );
                }
            }
        },
        expect: |sim| {
            assert!(sim.clouds().is_empty(), "the Disciples ran dry");
            assert!(units_of(sim, P2).is_empty(), "player 2's units are all dead");
            let alive = |ids: &[u32]| -> Vec<u32> {
                (ids.iter().copied())
                    .filter(|&id| sim.object(ObjectId(id)).is_some())
                    .collect()
            };
            assert_eq!(alive(&DISCIPLES), DISCIPLES, "the Disciples all stand");
            assert_eq!(
                alive(&MILITANTS),
                [6, 7, 9],
                "the three Militants in front fell in the open"
            );
        },
    }
}
