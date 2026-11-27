//! (M6-CM) The cloud's linger: design change 0448's Verification, step 8, "Shroud linger: channel, then Stop. The cloud
//! stays 1.5 s where it was, and a hit fired into it during that time is still absorbed. The buffer releases when the
//! cloud vanishes. The same happens when the Disciple dies while channelling" (`factions/cults/units/disciple.md >
//! Spells > Shroud`: "**The cloud lingers**: when the channel ends for any reason (cancel, out of energy, or the
//! Disciple's death), the cloud **stays for 1.5 s (24 frames, placeholder)** and keeps absorbing as usual").

use super::*;

/// How the channel of [`game`]'s Disciple ends.
#[derive(Clone, Copy, Debug)]
enum End {
    Never,
    /// A Stop in that tick.
    Stop(Tick),
    /// Its death, to player 2's Militants under its cloud, whose fire hits it in full.
    Death,
}

/// Player 1's Disciple (1) channels from tick 1 over their Militant (2), 80 su south of it; player 2's Militant (3),
/// 320 su south of the Disciple and outside its cloud, attacks 2 once, its order given `lead` ticks before the shot is
/// to land: in tick `order` for [`End::Never`] and [`End::Stop`], 20 ticks after the death for [`End::Death`], for
/// which player 2's four Militants under the cloud (4 to 7) shoot the Disciple from tick 2. Run until the shot is
/// absorbed: the game and the tick the channel ended in, if it did.
fn game(end: End, order_at: Tick, lead: Tick) -> (TestSim, Option<Tick>) {
    let mut builder = flat(20, 16)
        .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
        .spawn(MILITANT, Some(P1), point(400.0, 480.0))
        .spawn(MILITANT, Some(P2), point(400.0, 720.0));
    if let End::Death = end {
        for at in [(320.0, 400.0), (480.0, 400.0), (400.0, 320.0), (330.0, 330.0)] {
            builder = builder.spawn(MILITANT, Some(P2), point(at.0, at.1));
        }
    }
    let mut t = quiet(builder.start());
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    let ended = match end {
        End::Never => {
            order(&mut t, order_at, P2, &[3], OrderKind::Attack(ObjectId(2)));
            None
        }
        End::Stop(stop) => {
            order(&mut t, stop, P1, &[1], OrderKind::Stop);
            order(&mut t, order_at, P2, &[3], OrderKind::Attack(ObjectId(2)));
            Some(stop)
        }
        End::Death => {
            order(&mut t, 2, P2, &[4, 5, 6, 7], OrderKind::Attack(ObjectId(1)));
            step_until(&mut t, 100, "the Disciple dies", |t| {
                t.sim().object(ObjectId(1)).is_none()
            });
            let death = t.sim().tick();
            order(&mut t, death + 20 - lead, P2, &[3], OrderKind::Attack(ObjectId(2)));
            Some(death)
        }
    };
    step_until(&mut t, 200, "the shot absorbed", |t| buffer(t, 1) > 0.0);
    let shot = t.sim().tick();
    order(&mut t, shot + 1, P2, &[3], OrderKind::Stop);
    (t, ended)
}

/// Step 8, both ways (module docs). A first game, the channel going on, finds when player 2's Militant's one shot
/// lands. Then the channel ends 20 ticks before that, by a Stop, and in another game by the Disciple's death: either
/// way its circle lingers where it was, 24 ticks from the tick it ended in, the shot is absorbed into it as without the
/// end, nothing hits player 1's Militant then, and nothing is released 8 ticks on; the buffer goes as the circle
/// vanishes, 4 over the circle as it was, 0.5 of it on the Militant, and the cloud is gone.
#[test]
fn a_cloud_lingers_after_a_stop_or_its_disciple_s_death_absorbing_and_releasing_as_it_vanishes() {
    let (plain, _) = game(End::Never, 40, 0);
    let shot = plain.sim().tick();
    let lead = shot - 40;
    assert!(lead < 20, "the shot lands {lead} ticks after its order");
    for end in [End::Stop(shot - 20), End::Death] {
        let (mut t, ended) = game(end, 40, lead);
        let ended = ended.expect("the channel ended");
        assert_eq!(t.sim().tick(), ended + 20, "{end:?}: the shot 20 ticks after the end");
        let ends = (ended, SimEvent::ShroudEnded { disciple: ObjectId(1) });
        assert!(t.events().contains(&ends), "{end:?}: the channel ended then");
        let lingering = t.sim().clouds().list()[0].members[0];
        assert_eq!(
            (lingering.id, lingering.at, lingering.until),
            (ObjectId(1), Vec2::new(400.0, 400.0), Some(ended + LINGER)),
            "{end:?}: where it was, for 24 ticks"
        );
        assert_eq!(buffer(&t, 1), 4.0, "{end:?}: absorbed");
        assert_eq!((hp(&t, 2), hits_on(&t, 2)), (40.0, vec![]), "{end:?}: nothing hits it");
        t.run_until(ended + LINGER - 1);
        assert_eq!(releases(&t), [], "{end:?}: not 8 ticks after the shot");
        t.step();
        assert_eq!(
            releases(&t),
            [(ended + LINGER, vec![Vec2::new(400.0, 400.0)], 4.0)],
            "{end:?}: as it vanishes, over the circle as it was"
        );
        assert_eq!(hp(&t, 2), 39.5, "{end:?}: the area minimum");
        assert!(clouds(&t).is_empty(), "{end:?}: gone");
    }
}
