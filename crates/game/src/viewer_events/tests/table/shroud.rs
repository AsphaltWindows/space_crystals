//! (M5-N4b) The rows of Shroud's events in the filter's table (`super::table`), player 1's unit standing for a
//! Disciple: its channel starting and ending happen to it, as to any object, and to one that died in the frame where it
//! was; a release reaches every player, whose drawing keeps it to their Visible tiles (`crate::shroud_view`). (M6-CM)
//! And a Mirage, which names the illusions: to the Disciple's owner alone, even an enemy who sees it seeing only the
//! Disciples come (`factions/cults/units/disciple.md > Spells > Mirage`: "enemies can't tell them apart").

use super::*;

/// The rows of a Disciple's channel and a cloud's release, and of a Mirage, `unit` where player 1's unit stands.
pub(super) fn rows(unit: Vec2) -> Vec<(SimEvent, [Seen; 4])> {
    vec![
        (
            SimEvent::Mirage {
                disciple: UNIT,
                illusions: vec![WATCHER, DEAD],
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ShroudStarted {
                disciple: UNIT,
                at: unit,
            },
            [W, W, N, W],
        ),
        (SimEvent::ShroudEnded { disciple: UNIT }, [W, W, N, W]),
        (SimEvent::ShroudEnded { disciple: DEAD }, [W, W, N, W]),
        (
            SimEvent::ShroudReleased {
                circles: vec![(unit, 192.0), (unit + Vec2::new(320.0, 0.0), 192.0)],
                damage: 4.0,
            },
            [W, W, W, W],
        ),
    ]
}
