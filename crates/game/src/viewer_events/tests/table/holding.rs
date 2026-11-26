//! The rows of the buildings that hold units (slices M5-F1, M5-F2, M5-N1b and M5-N3) in the filter's table
//! (`super::table`).

use sc_sim::{AugmentOrder, Tech};

use super::*;

/// The rows of the events of a building that holds units, as the Tunnel stands for one in this world.
pub(super) fn rows() -> Vec<(SimEvent, [Seen; 4])> {
    let order = AugmentOrder::Both;
    vec![
        // (M5-F1) A building that holds units, as the Tunnel stands for one here: its pool full, the unit going in,
        // coming out of its doorway, and evacuated as it goes.
        (
            SimEvent::EnterFailed {
                unit: UNIT,
                target: TUNNEL,
                full: true,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::EnteredBuilding {
                unit: HELD,
                building: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::LeftBuilding {
                unit: UNIT,
                building: TUNNEL,
            },
            [W, W, N, W],
        ),
        (
            SimEvent::Evacuated {
                unit: UNIT,
                building: GONE,
            },
            [W, W, N, W],
        ),
        // (M5-F2) A pool building's Release, queued and taken back: its owner's, as its production's news.
        (
            SimEvent::ReleaseQueued {
                producer: TUNNEL,
                unit: UNIT,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ReleaseCancelled {
                producer: TUNNEL,
                unit: UNIT,
            },
            [W, N, N, N],
        ),
        // (M5-N1b) An Altar's augmentation order, queued and taken back: its owner's, as its production's news.
        (
            SimEvent::AugmentQueued {
                producer: TUNNEL,
                unit: UNIT,
                order,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::AugmentCancelled {
                producer: TUNNEL,
                unit: UNIT,
                order,
                refund: 50,
            },
            [W, N, N, N],
        ),
        // (M5-N3) A Temple's research, queued, taken back and completed: its owner's, as its production's news.
        (
            SimEvent::ResearchQueued {
                producer: TUNNEL,
                tech: Tech::EnhancedConstitutionL1,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ResearchCancelled {
                producer: TUNNEL,
                tech: Tech::EnhancedConstitutionL1,
                refund: 100,
            },
            [W, N, N, N],
        ),
        (
            SimEvent::ResearchCompleted {
                producer: TUNNEL,
                tech: Tech::ImprovedMetabolismL2,
            },
            [W, N, N, N],
        ),
    ]
}
