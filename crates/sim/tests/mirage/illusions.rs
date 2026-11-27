//! What an illusion is (`factions/cults/units/disciple.md > Spells > Mirage`; `sc_sim::mirage`): no Unit Control, a
//! Disciple's sight, double damage, the Altar's refusal and the Shrine's healing, and how it ends.

use super::*;
use sc_sim::buffs::BuffKind;
use sc_sim::{AreaAbility, Target, VisibilityState};

/// Design change 0448's Verification, step 2: "Unit Control is unchanged. Move the illusions around: they reveal the
/// map like a Disciple" ("They use **no Unit Control**"; "Sight: as the Disciple's"). The Cults' Unit Control player 1
/// uses (`Sim::army_cap`) is the Disciple's before the cast and after it. Then the east illusion walks 20 grid units
/// east, a normal controllable unit, and sees what a second Disciple does that walks there from the same spot, in a
/// game without the cast: each tile of player 1's beyond the Disciple's own sight is alike in both games.
#[test]
fn illusions_use_no_unit_control_and_reveal_the_map_as_a_disciple_does() {
    let goal = Vec2::new(AT.x + 21.0 * GU, AT.y);
    let game = |mirage: bool| {
        let mut builder = flat(44, 20).spawn(DISCIPLE, Some(P1), point(AT.x, AT.y));
        if !mirage {
            builder = builder.spawn(DISCIPLE, Some(P1), point(AT.x + GU, AT.y));
        }
        let mut t = quiet(builder.start());
        let used = |t: &TestSim| t.sim().army_cap(P1).expect("in the game").used;
        let alone = used(&t);
        let walker = if mirage {
            cast(&mut t, 1, &[1]);
            step_checked(&mut t);
            assert_eq!(illusions(&t, 1), [2, 3, 4, 5]);
            assert_eq!(used(&t), alone, "Unit Control unchanged");
            assert!(alone > 0, "the Disciple's own");
            3
        } else {
            step_checked(&mut t);
            2
        };
        order(&mut t, 2, P1, &[walker], OrderKind::Move(Target::Point(goal)));
        for _ in 0..600 {
            step_checked(&mut t);
        }
        assert!(t.pos(walker).distance(goal) < 1.0, "#{walker} at {}", t.pos(walker));
        let far: Vec<(TilePos, VisibilityState)> = (20..44)
            .flat_map(|x| (0..20).map(move |y| TilePos::new(x, y)))
            .map(|tile| (tile, t.sim().vision().state(P1, tile)))
            .collect();
        (t.pos(walker), far)
    };
    let (illusion_at, illusion_sees) = game(true);
    let (disciple_at, disciple_sees) = game(false);
    assert_eq!(illusion_at, disciple_at, "the same walk");
    let visible = |sees: &[(TilePos, VisibilityState)]| {
        sees.iter()
            .filter(|(_, state)| *state == VisibilityState::Visible)
            .count()
    };
    assert!(visible(&illusion_sees) > 100, "{}", visible(&illusion_sees));
    assert_eq!(illusion_sees, disciple_sees);
}

/// Design change 0448's Verification, step 3: "Shoot an illusion: it loses twice the HP a real Disciple would"
/// ("illusions take **double damage from all sources** (Shroud releases included)"). Player 2's Militants, each south
/// east of its target, 2.5 grid units both ways, shoot the Disciple and its east illusion: 4 a hit off the Disciple, 8
/// off the illusion. Then the Disciple, back to 10 energy, channels, its illusions mirroring it under its cloud, and the
/// first Militant shoots it again from outside the cloud: the release's share is the area minimum, 0.5, on the
/// Disciple (`tests/shroud`'s `absorption`), and twice that, 1, on each illusion.
#[test]
fn an_illusion_takes_double_damage_a_release_s_share_too() {
    let mut t = quiet(
        flat(20, 20)
            .spawn(DISCIPLE, Some(P1), point(AT.x, AT.y))
            .spawn(MILITANT, Some(P2), point(AT.x + 2.5 * GU, AT.y + 2.5 * GU))
            .spawn(MILITANT, Some(P2), point(AT.x + 3.5 * GU, AT.y + 2.5 * GU))
            .start(),
    );
    cast(&mut t, 1, &[1]);
    order(&mut t, 2, P2, &[2], OrderKind::Attack(ObjectId(1)));
    order(&mut t, 2, P2, &[3], OrderKind::Attack(ObjectId(5)));
    step_until(&mut t, 100, "a hit on each", |t| {
        !hits_on(t, 1).is_empty() && !hits_on(t, 5).is_empty()
    });
    let now = t.sim().tick();
    order(&mut t, now + 1, P2, &[2, 3], OrderKind::Stop);
    t.run(1);
    let damage = |hits: Vec<(Tick, u32, f32)>| hits.iter().map(|hit| hit.2).collect::<Vec<f32>>();
    let on_disciple = damage(hits_on(&t, 1));
    let on_illusion = damage(hits_on(&t, 5));
    assert!(on_disciple.iter().all(|&d| d == 4.0), "{on_disciple:?}");
    assert!(on_illusion.iter().all(|&d| d == 8.0), "{on_illusion:?}");
    assert_eq!(hp(&t, 5), 40.0 - 8.0 * on_illusion.len() as f32);

    regenerate(&mut t, 1, 10.5);
    let channel = t.sim().tick() + 1;
    order(&mut t, channel, P1, &[1], OrderKind::Shroud);
    order(&mut t, channel + 1, P2, &[2], OrderKind::Attack(ObjectId(1)));
    let released = |t: &TestSim| hits_on(t, 1).iter().any(|hit| hit.1 == 1);
    step_until(&mut t, 100, "a release", released);
    let release = t.sim().tick();
    let share = |t: &TestSim, id: u32| -> Vec<f32> {
        (hits_on(t, id).iter())
            .filter(|hit| hit.0 == release && hit.1 == 1)
            .map(|hit| hit.2)
            .collect()
    };
    assert_eq!(clouds(&t), [vec![1]], "the Disciple's cloud alone");
    assert_eq!(share(&t, 1), [0.5], "the Disciple's share");
    for id in [4, 5, 6, 7] {
        assert!(stands(&t, id), "#{id} mirrors the channel");
        assert_eq!(share(&t, id), [1.0], "#{id}'s share, doubled");
    }
}

/// "Illusions can't be augmented at the Altar themselves" (`sc_sim::mirage`). Player 1's Altar (1), as the Altar tests
/// have it (tiles (12, 10) to (14, 11), its entrance east), and their Disciple east of it: the gate refuses an Enter of
/// its illusions' into the Altar, `Reject::NotAccepted`, and takes one of the Disciple's and theirs, which the
/// Disciple alone carries out. It goes in; its illusions stay on the map, with no Enter of their own.
#[test]
fn the_altar_refuses_illusions() {
    let mut t = quiet(
        flat(30, 22)
            .spawn(ObjectKind::Altar, Some(P1), tile(12, 10))
            .spawn(DISCIPLE, Some(P1), point(1150.0, 704.0))
            .start(),
    );
    cast(&mut t, 1, &[2]);
    step_checked(&mut t);
    assert_eq!(illusions(&t, 2), [3, 4, 5, 6]);
    let altar = ObjectId(1);
    let enter = |units: &[u32]| Command::Order {
        units: ids(units),
        kind: OrderKind::Enter(altar),
        queue: false,
    };
    assert_eq!(validate(&t, P1, enter(&[3])), Err(Reject::NotAccepted(altar)));
    assert_eq!(validate(&t, P1, enter(&[3, 4, 5, 6])), Err(Reject::NotAccepted(altar)));
    let all = [2, 3, 4, 5, 6];
    assert_eq!(validate(&t, P1, enter(&all)), Ok(()));
    t.at(2, P1, enter(&all));
    step_until(&mut t, 300, "the Disciple in", |t| !t.sim().held_in(altar).is_empty());
    t.run(20);
    let inside: Vec<u32> = t.sim().held_in(altar).iter().map(|held| held.object.id.0).collect();
    assert_eq!(inside, [2], "the Disciple alone");
    assert_eq!(illusions(&t, 2), [3, 4, 5, 6], "on the map");
    for id in [3, 4, 5, 6] {
        let o = t.sim().object(ObjectId(id)).expect("on the map");
        let current = (o.orders.as_ref()).and_then(|orders| orders.current.as_ref());
        assert!(current.is_none_or(|a| a.order.kind != OrderKind::Enter(altar)), "#{id}");
    }
}

/// "Other effects: the Shrine's health regeneration works on them (they are own Cults infantry)". Player 1's Disciple,
/// shot by player 2's Militant to 32 HP, casts, and its illusions start at 32 too. Player 1's Shrine (1), as the Shrine
/// tests have it (tiles (6, 9) to (7, 10)), manned by eight Recruits and charged to 75, casts its Q at the Disciple
/// (`factions/cults/structures/shrine.md > Abilities`: "Health regeneration (name TBD) | Own Cults infantry | Grants
/// health regeneration [...] | 75 | radius 4 grid units | 15 s"): the Disciple and its illusions get the buff in the
/// cast's tick and heal alike, 30 HP over 15 s capped at their MaxHP, an illusion's HP its parent's every tick.
#[test]
fn the_shrine_heals_illusions() {
    let at = Vec2::new(900.0, 640.0);
    let mut t = quiet(
        flat(32, 20)
            .spawn(ObjectKind::Shrine, Some(P1), tile(6, 9))
            .spawn(DISCIPLE, Some(P1), point(at.x, at.y))
            .spawn(MILITANT, Some(P2), point(at.x + 3.0 * GU, at.y + 1.5 * GU))
            .start(),
    );
    let shrine = ObjectId(1);
    for i in 0..8 {
        t.spawn_unit_at(1, RECRUIT, P1, Vec2::new(650.0, 528.0 + 32.0 * i as f32));
    }
    let recruits: Vec<u32> = (4..=11).collect();
    let enter = Command::Order {
        units: ids(&recruits),
        kind: OrderKind::Enter(shrine),
        queue: false,
    };
    t.at(2, P1, enter);
    order(&mut t, 2, P2, &[3], OrderKind::Attack(ObjectId(2)));
    step_until(&mut t, 100, "two hits", |t| hits_on(t, 2).len() == 2);
    let now = t.sim().tick();
    order(&mut t, now + 1, P2, &[3], OrderKind::Stop);
    cast(&mut t, now + 2, &[2]);
    t.run(2);
    assert_eq!(hp(&t, 2), 32.0);
    assert_eq!(illusions(&t, 2), [12, 13, 14, 15]);
    assert!(
        [12, 13, 14, 15].iter().all(|&id| hp(&t, id) == 32.0),
        "the parent's current HP"
    );
    step_until(&mut t, 300, "the Shrine manned", |t| t.sim().held_in(shrine).len() == 8);
    let charged = |t: &TestSim| {
        let o = t.sim().object(shrine).expect("the Shrine");
        sc_sim::energy::of(o).is_some_and(|(energy, _)| energy >= 75.0)
    };
    step_until(&mut t, 2000, "the Shrine charged", charged);
    let q = t.sim().tick() + 1;
    let heal = Command::Cast {
        casters: vec![shrine],
        ability: AreaAbility::HealthRegeneration,
        at,
    };
    t.at(q, P1, heal);
    t.run_until(q);
    let healed = [2, 12, 13, 14, 15];
    for id in healed {
        let o = t.sim().object(ObjectId(id)).expect("on the map");
        let kinds: Vec<BuffKind> = o.buffs.iter().map(|buff| buff.ability).collect();
        assert_eq!(kinds, [BuffKind::Area(AreaAbility::HealthRegeneration)], "#{id}");
        assert_eq!(hp(&t, id), 32.125, "#{id}: healed in the cast's tick");
    }
    for _ in 0..80 {
        step_checked(&mut t);
        let parent = hp(&t, 2);
        assert!(healed.iter().all(|&id| hp(&t, id) == parent), "tick {}", t.sim().tick());
    }
    assert_eq!(hp(&t, 2), 40.0, "capped at its MaxHP");
}

/// Design change 0448's Verification, step 6: "Kill the parent: all its illusions vanish" ("an illusion lasts until it
/// **dies**, its **parent dies**, or the parent **casts Mirage again**"). Player 2's two Militants kill the east
/// illusion first: 8 a hit, its 40 HP in five, and it dies as any unit does (`SimEvent::Died`), the others staying.
/// Then they kill the Disciple: in the tick it dies its three other illusions leave the game with it, with no death.
#[test]
fn an_illusion_dies_alone_and_its_parent_s_death_takes_the_rest() {
    let mut t = quiet(
        flat(20, 20)
            .spawn(DISCIPLE, Some(P1), point(AT.x, AT.y))
            .spawn(MILITANT, Some(P2), point(AT.x + 2.5 * GU, AT.y + 2.5 * GU))
            .spawn(MILITANT, Some(P2), point(AT.x + 3.5 * GU, AT.y + 2.5 * GU))
            .start(),
    );
    cast(&mut t, 1, &[1]);
    order(&mut t, 2, P2, &[2, 3], OrderKind::Attack(ObjectId(5)));
    step_checked(&mut t);
    assert_eq!(illusions(&t, 1), [4, 5, 6, 7]);
    step_until(&mut t, 200, "the east illusion dies", |t| {
        t.sim().object(ObjectId(5)).is_none()
    });
    let first = t.sim().tick();
    assert_eq!(died(&t), [(first, 5)]);
    let hits: Vec<f32> = hits_on(&t, 5).iter().map(|hit| hit.2).collect();
    let in_last_tick = hits_on(&t, 5).iter().filter(|hit| hit.0 == first).count();
    assert!(hits.iter().all(|&d| d == 8.0), "8 a hit: {hits:?}");
    assert_eq!(
        hits.len() - in_last_tick,
        4,
        "the fifth hit kills it: {:?}",
        hits_on(&t, 5)
    );
    assert_eq!(illusions(&t, 1), [4, 6, 7], "the others stay");
    order(&mut t, first + 1, P2, &[2, 3], OrderKind::Attack(ObjectId(1)));
    step_until(&mut t, 300, "the Disciple dies", |t| {
        t.sim().object(ObjectId(1)).is_none()
    });
    let death = t.sim().tick();
    assert_eq!(died(&t), [(first, 5), (death, 1)], "no illusion's death");
    let gone: Vec<u32> = (removed(&t).iter())
        .filter(|(tick, _)| *tick == death)
        .map(|&(_, id)| id)
        .collect();
    assert_eq!(gone, [1, 4, 6, 7], "with it");
    assert!(illusions(&t, 1).is_empty(), "none left");
    assert!(
        t.sim().objects().all(|o| o.owner != Some(P1)),
        "nothing of player 1's left"
    );
}
