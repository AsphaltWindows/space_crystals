//! The Egress Node's F, Select Returning Drones (`factions/colonists/economy.md > Egress Node > Egress Node
//! ObjectInterfaceState`; design change 0124: "Egress Node: F"): a StateOnlyTransition that replaces the selection
//! with "all Drones whose AssignedDropOff is this Egress Node and which are currently executing a Gather command —
//! whether mining, walking to the patch, carrying resources back, or dropping off" (`sc_sim::Sim::returning_drones`),
//! hidden while there are none.

use super::*;
use KeyCode::{KeyF, KeyX};

/// Drones A and B gather patch 3 for node 1, their nearest, one mining it while the other waits: F on node 1 selects
/// exactly them, whatever phase of their Gather it is pressed in. Not Drone C, which gathered for node 1 and was
/// stopped, nor D, which gathers patch 10 for node 2, its nearest: node 2's F selects D alone. F is hidden while node 1
/// has no returning Drone, before the Gathers and once A and B are stopped, and its key does nothing then.
#[test]
fn f_selects_the_drones_returning_to_its_node_in_every_phase_of_their_gather() {
    let mut app = gathering_game();
    click(&mut app, home(NODE), &[]);
    assert_eq!(selected(&app), [NODE]);
    assert!(!shows_f(&mut app), "no Drone gathers yet");
    tap(&mut app, &[], KeyF);
    assert_eq!(selected(&app), [NODE], "F is hidden");

    click(&mut app, home(C), &[]);
    right_click(&mut app, home(PATCH), &[]);
    assert_eq!(drop_off(&app, C), Some(NODE));
    tap(&mut app, &[], KeyX);
    assert_eq!(current(&app, C), None);
    click(&mut app, home(D), &[]);
    right_click(&mut app, home(FAR_PATCH), &[]);
    assert_eq!(drop_off(&app, D), Some(NODE_2));
    select_around(&mut app, &[home(A), home(B)], &[A, B]);
    right_click(&mut app, home(PATCH), &[]);

    let phases: [(&str, Phases); 5] = [
        ("walking to the patch", |phase| matches!(phase, GatherPhase::ToPatch)),
        ("waiting at the patch", |phase| {
            matches!(phase, GatherPhase::Waiting { .. })
        }),
        ("mining", |phase| matches!(phase, GatherPhase::Mining { .. })),
        ("carrying a load back", |phase| matches!(phase, GatherPhase::ToDropOff)),
        ("unloading at the node", |phase| {
            matches!(phase, GatherPhase::DroppingOff { .. })
        }),
    ];
    for (what, phase) in phases {
        click(&mut app, home(NODE), &[]);
        assert_eq!(selected(&app), [NODE], "{what}");
        run_until(&mut app, 400, what, |app| {
            in_phase(app, A, phase) || in_phase(app, B, phase)
        });
        assert!(shows_f(&mut app), "{what}");
        tap(&mut app, &[], KeyF);
        assert_eq!(selected(&app), [A, B], "{what}");
    }

    click(&mut app, home(NODE_2), &[]);
    tap(&mut app, &[], KeyF);
    assert_eq!(selected(&app), [D], "node 2's");

    click(&mut app, home(NODE), &[]);
    tap(&mut app, &[], KeyF);
    tap(&mut app, &[], KeyX);
    click(&mut app, home(NODE), &[]);
    assert!(!shows_f(&mut app), "A and B stopped");
    tap(&mut app, &[], KeyF);
    assert_eq!(selected(&app), [NODE]);
}
