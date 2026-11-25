//! Cancelling an expansion (`factions/syndicate/tunnels.md > Expansions under construction`, design change 0254:
//! "while building, the expansion offers only **X Cancel Construction**, refunding **75%** of its cost (rounded down).
//! The Tunnel's X cancels whatever operation it is running"), and the site gone otherwise.

use super::*;

/// The cancels so far: each Tunnel, site, kind and refund.
fn cancels(t: &TestSim) -> Vec<(ObjectId, ObjectId, ObjectKind, u32)> {
    events(t, |e| match *e {
        SimEvent::ExpansionCancelled {
            tunnel,
            site,
            kind,
            refund,
        } => Some((tunnel, site, kind, refund)),
        _ => None,
    })
}

/// 0254's verification: "Place a Hideout (200 SC), then cancel: 150 SC come back", by the site's own Cancel
/// Construction, and the same through the Tunnel's X; each time the site leaves the game, its Tunnel Space comes back,
/// the operation is freed, and the Tunnel builds again.
#[test]
fn cancelling_gives_back_150_of_200() {
    let mut t = started(base(), 400);
    next(&mut t, P1, expand(2, 6));
    let first = building(&t);
    t.run(50);
    assert_eq!((crystals(&t, P1), tunnel_space(&t, P1)), (200, (1, 10)));
    next(&mut t, P1, cancel_site(first));
    assert_eq!(
        (crystals(&t, P1), tunnel_space(&t, P1), operation(&t)),
        (350, (1, 11), None)
    );
    assert!(t.sim().object(first).is_none());
    next(&mut t, P1, expand(6, 12));
    let second = building(&t);
    assert_eq!(crystals(&t, P1), 150);
    next(&mut t, P1, cancel_operation());
    assert_eq!((crystals(&t, P1), operation(&t)), (300, None));
    assert!(t.sim().object(second).is_none());
    assert_eq!(cancels(&t), [(T1, first, HIDEOUT, 150), (T1, second, HIDEOUT, 150)]);
    assert_eq!(
        gate(&t, P1, cancel_operation()),
        Err(Reject::NoOperation(T1)),
        "nothing left to cancel"
    );
    assert_eq!(gate(&t, P1, expand(2, 6)), Ok(()));
}

/// Its own Cancel Construction is the owner's alone, as any command; the other player's is refused.
#[test]
fn only_the_owner_cancels() {
    let mut t = started(base(), 400);
    next(&mut t, P1, expand(2, 6));
    let site = building(&t);
    assert_eq!(gate(&t, P2, cancel_site(site)), Err(Reject::NotOwned(site)));
    let x = Command::Tunnel {
        tunnel: T1,
        command: TunnelCommand::Cancel,
    };
    assert_eq!(gate(&t, P2, x), Err(Reject::NotOwned(T1)));
}

/// The site gone any other way (a debug removal stands for its destruction, `Sim::remove_objects`): the operation
/// ends with it, nothing is refunded, and the Tunnel builds again.
#[test]
fn a_site_gone_frees_the_operation() {
    let mut t = started(base(), 400);
    next(&mut t, P1, expand(2, 6));
    let site = building(&t);
    next_debug(&mut t, DebugCommand::Remove { id: site });
    assert_eq!((operation(&t), crystals(&t, P1), cancels(&t)), (None, 200, vec![]));
    assert_eq!(gate(&t, P1, expand(2, 6)), Ok(()));
}
