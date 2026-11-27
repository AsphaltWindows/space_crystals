//! The cast (`factions/cults/units/disciple.md > Spells > Mirage`: "An **instant** spell: the Disciple creates **4
//! illusions** of itself"; `> Placeholder stats > Mirage numbers`), where its illusions go, what they start with, what
//! the gate asks of it, and a recast.

use super::*;
use sc_sim::{Augmentation, DebugCommand};

/// Design change 0448's Verification, step 1, as far as the sim goes: "Select a Disciple with at least 50 energy and
/// press F. Four illusions appear north, east, south and west of it, and energy drops by 50." The Disciple starts at
/// its 50 (`> Placeholder stats`: "starts at 50") and casts in tick 1. Its illusions (2 to 5) come in that tick, each
/// after its `Spawned` and before the `SimEvent::Mirage`, "**1 grid unit** from the Disciple's centre (centre to
/// centre)", north, east, south and west ("the Disciple stays where it is"): Disciples of its player's, facing its
/// way, linked to it, idle, with no energy of their own ("Illusions have no energy of their own"). Its energy is 50
/// below a game without the cast, and a second cast at once is refused, `Reject::NotEnoughEnergy`.
#[test]
fn f_makes_four_illusions_north_east_south_and_west_1_gu_off_for_50_energy() {
    let mut without = lone();
    without.step();
    let mut t = lone();
    assert_eq!(energy(&t, 1), COST, "it starts with its 50");
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    assert_eq!(illusions(&t, 1), [2, 3, 4, 5]);
    let compass = [
        AT - Vec2::Y * GU,
        AT + Vec2::X * GU,
        AT + Vec2::Y * GU,
        AT - Vec2::X * GU,
    ];
    assert_eq!([2, 3, 4, 5].map(|id| t.pos(id)), compass, "north, east, south, west");
    assert_eq!(t.pos(1), AT, "the Disciple stays where it is");
    let parent = t.sim().object(ObjectId(1)).cloned().expect("the Disciple");
    for id in [2, 3, 4, 5] {
        let o = t.sim().object(ObjectId(id)).expect("an illusion");
        assert_eq!(
            (o.kind, o.owner, o.illusion_of, o.facing),
            (DISCIPLE, Some(P1), Some(ObjectId(1)), parent.facing),
            "#{id}"
        );
        assert!(o.energy.is_none(), "#{id}: no energy of its own");
        let idle = (o.orders.as_ref()).is_none_or(|orders| orders.current.is_none() && orders.queue.is_empty());
        assert!(idle, "#{id}: idle");
    }
    assert_eq!(energy(&without, 1) - energy(&t, 1), COST, "50 spent");
    assert_eq!(energy(&t, 1), A_TICK, "the tick's regeneration left");
    let tick_1: Vec<SimEvent> = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::Spawned(_) | SimEvent::Mirage { .. }))
        .map(|(_, e)| e.clone())
        .collect();
    let mut expected: Vec<SimEvent> = (2..=5).map(|id| SimEvent::Spawned(ObjectId(id))).collect();
    expected.push(SimEvent::Mirage {
        disciple: ObjectId(1),
        illusions: ids(&[2, 3, 4, 5]),
    });
    assert_eq!(tick_1, expected);
    assert_eq!(validate(&t, P1, mirage(&[1])), Err(Reject::NotEnoughEnergy));
}

/// "Each illusion starts with the Disciple's **current HP**, and has its MaxHP and speed (augmentations included)";
/// "can't be augmented, but copies the parent's augmented stats when created" (design change 0448). A Disciple with
/// both Augmentations (a debug command: EnhancedConstitution's MaxHP, ImprovedMetabolism's speed,
/// `factions/cults/augmentation.md`), shot by player 2's Militant to 8 HP under its MaxHP, casts: each illusion has its
/// HP, its MaxHP and its top speed, all above a plain Disciple's but the HP.
#[test]
fn an_illusion_starts_with_its_parent_s_hp_max_hp_and_speed_augmentations_included() {
    let mut t = quiet(
        flat(20, 20)
            .spawn(DISCIPLE, Some(P1), point(AT.x, AT.y))
            .spawn(MILITANT, Some(P2), point(AT.x, AT.y + 3.5 * GU))
            .start(),
    );
    let both = Augmentation {
        constitution: true,
        metabolism: true,
    };
    t.debug_at(
        1,
        DebugCommand::SetAugmentation {
            id: ObjectId(1),
            augmentation: both,
        },
    );
    order(&mut t, 2, P2, &[2], OrderKind::Attack(ObjectId(1)));
    let max_hp = |t: &TestSim, id: u32| (t.sim().object(ObjectId(id))).and_then(|o| o.max_hp_with(t.sim().research()));
    let speed = |t: &TestSim, id: u32| t.sim().object(ObjectId(id)).map(|o| o.max_speed(t.sim().research()));
    t.run(1);
    let full = max_hp(&t, 1).expect("destructible");
    assert!(full > 40.0, "EnhancedConstitution's MaxHP: {full}");
    step_until(&mut t, 100, "two hits", |t| hits_on(t, 1).len() == 2);
    let now = t.sim().tick();
    order(&mut t, now + 1, P2, &[2], OrderKind::Stop);
    cast(&mut t, now + 2, &[1]);
    t.run(2);
    let wounded = hp(&t, 1);
    assert_eq!(wounded, full - 8.0, "two hits of 4");
    let plain_speed = DISCIPLE.def().unit.as_ref().expect("a unit").movement.max_speed();
    assert!(
        speed(&t, 1).expect("a unit") > plain_speed,
        "ImprovedMetabolism's speed"
    );
    assert_eq!(illusions(&t, 1), [3, 4, 5, 6]);
    for id in [3, 4, 5, 6] {
        assert_eq!(
            (hp(&t, id), max_hp(&t, id), speed(&t, id)),
            (wounded, Some(full), speed(&t, 1)),
            "#{id}"
        );
        let o = t.sim().object(ObjectId(id)).expect("an illusion");
        assert_eq!(o.augmentation, both, "#{id}");
    }
}

/// "A blocked spot puts that illusion at the **nearest free spot**" (`sc_sim::mirage`; Placeholder (not in the design:
/// how finely and how far it looks): rings 8 su apart round the spot, out to 3 grid units, each from the point straight
/// on from the Disciple). A Mountain on tile (11, 10), east of the Disciple, covers the east spot: no point within 44
/// su of it fits an illusion's 24 × 24 box, so the first ring with a free point is the sixth, 48 su out, and its first
/// point, straight on east, is free. The others take their spots.
#[test]
fn a_blocked_spot_puts_its_illusion_at_the_nearest_free_one() {
    let mut rows = vec![".".repeat(20); 20];
    rows[10].replace_range(11..12, "M");
    let mut t = quiet(
        cults(&rows.join("\n"))
            .spawn(DISCIPLE, Some(P1), point(AT.x, AT.y))
            .start(),
    );
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    assert_eq!(illusions(&t, 1), [2, 3, 4, 5]);
    let east = t.pos(3);
    let expected = AT + Vec2::new(GU + 48.0, 0.0);
    assert!(east.distance(expected) < 1e-3, "east, past the Mountain: {east}");
    assert_eq!(
        [2, 4, 5].map(|id| t.pos(id)),
        [AT - Vec2::Y * GU, AT + Vec2::Y * GU, AT - Vec2::X * GU]
    );
}

/// The gate (`sc_sim::mirage`): Mirage wants its 50 energy ("Cost: **50 energy**"; `Reject::NotEnoughEnergy`), and
/// "**No spells**: illusions can't cast" (`Reject::IllusionsCantCast`), neither Mirage nor Shroud. Of a selection of a
/// Disciple and its illusions the Disciple alone casts, once it has the energy again: one `SimEvent::Mirage`, its new
/// set in place of the old.
#[test]
fn the_gate_wants_50_energy_and_no_illusion_casts() {
    let mut t = lone();
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    assert_eq!(validate(&t, P1, mirage(&[1])), Err(Reject::NotEnoughEnergy));
    for id in [2, 3, 4, 5] {
        assert_eq!(validate(&t, P1, mirage(&[id])), Err(Reject::IllusionsCantCast), "#{id}");
        assert_eq!(validate(&t, P1, shroud(&[id])), Err(Reject::IllusionsCantCast), "#{id}");
    }
    regenerate(&mut t, 1, COST);
    assert_eq!(validate(&t, P1, mirage(&[2])), Err(Reject::IllusionsCantCast));
    let all = [1, 2, 3, 4, 5];
    assert_eq!(validate(&t, P1, mirage(&all)), Ok(()));
    let now = t.sim().tick() + 1;
    cast(&mut t, now, &all);
    step_checked(&mut t);
    assert_eq!(mirages(&t).len(), 2, "one cast: {:?}", mirages(&t));
    assert_eq!(mirages(&t)[1], (now, 1, vec![6, 7, 8, 9]));
    assert_eq!(illusions(&t, 1), [6, 7, 8, 9], "the new set alone");
}

/// Design change 0448's Verification, step 5: "Press F while the parent channels. The channel ends, the old illusions
/// vanish and four new ones appear" ("**Casting it cancels Shroud**: casting Mirage ends the Disciple's channel (the
/// cloud then lingers as usual)"; "the old set vanishes and a new set of 4 appears; a Disciple never has more than one
/// set"). The Disciple, back to 51 energy, channels, its illusions mirroring it, and casts again two ticks on: its
/// channel ends in that tick (`SimEvent::ShroudEnded`) and its cloud lingers; its old illusions leave the game with no
/// death, before the new ones come to the same spots; it is idle.
#[test]
fn f_while_channelling_ends_the_channel_and_replaces_the_set() {
    let mut t = lone();
    cast(&mut t, 1, &[1]);
    step_checked(&mut t);
    regenerate(&mut t, 1, COST + 1.0);
    let channel = t.sim().tick() + 1;
    order(&mut t, channel, P1, &[1], OrderKind::Shroud);
    cast(&mut t, channel + 2, &[1]);
    t.run(2);
    assert_eq!(clouds(&t), [vec![1]], "channelling");
    assert!(
        [1, 2, 3, 4, 5].iter().all(|&id| stands(&t, id)),
        "the illusions mirror it"
    );
    let before: Vec<Vec2> = [2, 3, 4, 5].map(|id| t.pos(id)).to_vec();
    step_checked(&mut t);
    let recast = channel + 2;
    assert_eq!(t.sim().tick(), recast);
    let ended = (recast, SimEvent::ShroudEnded { disciple: ObjectId(1) });
    assert!(t.events().contains(&ended), "the channel ends");
    let lingering = t.sim().clouds().list()[0].members[0];
    assert_eq!(
        (lingering.id, lingering.until),
        (ObjectId(1), Some(recast + 24)),
        "its cloud lingers"
    );
    assert!(!stands(&t, 1), "the Disciple idles");
    let gone: Vec<(Tick, u32)> = removed(&t);
    assert_eq!(gone, [2, 3, 4, 5].map(|id| (recast, id)), "the old set");
    assert_eq!(died(&t), [], "with no death");
    assert_eq!(mirages(&t).last(), Some(&(recast, 1, vec![6, 7, 8, 9])));
    assert_eq!(illusions(&t, 1), [6, 7, 8, 9], "one set");
    assert_eq!([6, 7, 8, 9].map(|id| t.pos(id)).to_vec(), before, "on the same spots");
    let events: Vec<&SimEvent> = (t.events().iter())
        .filter(|(tick, _)| *tick == recast)
        .map(|(_, e)| e)
        .collect();
    let at = |event: SimEvent| events.iter().position(|e| **e == event).expect("in the tick");
    assert!(
        at(SimEvent::Removed(ObjectId(5))) < at(SimEvent::Spawned(ObjectId(6))),
        "the old go before the new come: {events:?}"
    );
}
