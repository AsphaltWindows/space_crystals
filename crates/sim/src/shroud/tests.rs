//! The clouds' bookkeeping on its own (`super`): components, matching, the timer, the releases and (M6-CM) the linger.
//! In games: `crates/sim/tests/shroud/`.

use super::*;

/// The channel of unit `id`, since tick 1, at (`x`, 500) with a 3-GU radius.
fn at(id: u32, x: f32) -> Member {
    Member {
        id: ObjectId(id),
        since: 1,
        at: Vec2::new(x, 500.0),
        radius: 192.0,
        until: None,
    }
}

fn ids(cloud: &Cloud) -> Vec<u32> {
    cloud.members.iter().map(|m| m.id.0).collect()
}

/// The built-in linger: 24 frames (`Spells::linger`).
const LINGER: Tick = 24;

/// "Clouds are connected when their circles **touch or overlap**": 5 GU apart (320 su) one cloud, touching (384 su) one
/// cloud, a hair beyond apart; and the components in id order, whatever the order along the map.
#[test]
fn circles_that_touch_or_overlap_are_one_cloud() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(
        vec![at(1, 100.0), at(2, 420.0), at(3, 804.0), at(4, 1189.0)],
        1,
        &mut events,
    );
    let list: Vec<Vec<u32>> = clouds.list().iter().map(ids).collect();
    assert_eq!(
        list,
        [vec![1, 2, 3], vec![4]],
        "5 GU apart, then touching, then just apart"
    );
    assert_eq!(events.len(), 4, "four started: {events:?}");
    let mut clouds = Clouds::default();
    clouds.update(
        vec![at(1, 1000.0), at(2, 100.0), at(3, 900.0), at(4, 200.0)],
        1,
        &mut events,
    );
    let list: Vec<Vec<u32>> = clouds.list().iter().map(ids).collect();
    assert_eq!(list, [vec![1, 3], vec![2, 4]]);
    assert_eq!(clouds.over(Vec2::new(1000.0, 692.0)), Some(0), "the edge is under it");
    assert_eq!(clouds.over(Vec2::new(1000.0, 693.0)), None);
}

/// The buffer and its timer: the first absorbed hit starts it, the release 8 frames after takes the buffer, and the next
/// absorbed hit starts it again; a cloud that goes on as it was keeps both.
#[test]
fn a_cloud_releases_8_frames_after_its_first_absorbed_hit() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(vec![at(1, 100.0)], 1, &mut events);
    clouds.absorb(0, 4.0, 10);
    clouds.absorb(0, 6.0, 12);
    assert_eq!((clouds.list()[0].buffer, clouds.list()[0].release_at), (10.0, Some(18)));
    clouds.update(vec![at(1, 100.0)], 13, &mut events);
    assert_eq!(clouds.list()[0].release_at, Some(18), "the same cloud goes on");
    assert!(!clouds.due(17) && clouds.due(18));
    assert!(clouds.take_releases(17).is_empty());
    let released = clouds.take_releases(18);
    assert_eq!((released.len(), released[0].buffer), (1, 10.0));
    assert_eq!((clouds.list()[0].buffer, clouds.list()[0].release_at), (0.0, None));
    assert!(!clouds.due(26), "it waits for its next absorbed hit");
    clouds.absorb(0, 1.0, 30);
    assert_eq!(clouds.list()[0].release_at, Some(38));
}

/// Design change 0334: "when clouds join ... each joining cloud releases its buffer **immediately** over its own area.
/// The joined cloud then starts a **fresh joint buffer**"; a split, or a channel's end, releases the buffer at once
/// over the cloud as it was; a cloud that ends with nothing in its buffer releases nothing. (M6-CM) So it is with no
/// linger, the clouds as they were before it (`Clouds::follow` with 0).
#[test]
fn with_no_linger_joins_splits_and_ends_release_the_buffers_as_the_clouds_were() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    let update = |clouds: &mut Clouds, channelling: Vec<Member>, now: Tick, events: &mut Vec<SimEvent>| {
        clouds.follow(channelling, now, 0, events)
    };
    // Two clouds a short gap apart, each with a buffer.
    update(&mut clouds, vec![at(1, 100.0), at(3, 600.0)], 1, &mut events);
    clouds.absorb(0, 2.0, 5);
    clouds.absorb(1, 3.0, 6);
    // A third channels in between.
    update(
        &mut clouds,
        vec![at(1, 100.0), at(2, 350.0), at(3, 600.0)],
        7,
        &mut events,
    );
    assert_eq!(clouds.list().len(), 1);
    assert_eq!((ids(&clouds.list()[0]), clouds.list()[0].buffer), (vec![1, 2, 3], 0.0));
    assert_eq!(clouds.list()[0].release_at, None, "a fresh joint buffer");
    let ended: Vec<(Vec<u32>, f32)> = clouds.ended().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(
        ended,
        [(vec![1], 2.0), (vec![3], 3.0)],
        "both release, each over its own area"
    );
    assert!(clouds.due(7));
    assert_eq!(clouds.take_releases(7).len(), 2);
    // They buffer together, then the middle one stops: the split releases the joint buffer over all three circles.
    clouds.absorb(0, 5.0, 8);
    update(&mut clouds, vec![at(1, 100.0), at(3, 600.0)], 8, &mut events);
    let ended: Vec<(Vec<u32>, f32)> = clouds.ended().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(ended, [(vec![1, 2, 3], 5.0)]);
    let list: Vec<(Vec<u32>, f32)> = clouds.list().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(list, [(vec![1], 0.0), (vec![3], 0.0)]);
    clouds.take_releases(9);
    // A channel that ends with an empty buffer releases nothing; a new Shroud order is a new channel.
    update(&mut clouds, vec![at(1, 100.0)], 9, &mut events);
    assert!(clouds.ended().is_empty() && clouds.list().len() == 1);
    clouds.absorb(0, 1.0, 10);
    let recast = Member {
        since: 11,
        ..at(1, 100.0)
    };
    events.clear();
    update(&mut clouds, vec![recast], 11, &mut events);
    assert_eq!(clouds.ended().len(), 1, "the channel under way ended");
    assert_eq!(
        events,
        [
            SimEvent::ShroudEnded { disciple: ObjectId(1) },
            SimEvent::ShroudStarted {
                disciple: ObjectId(1),
                at: Vec2::new(100.0, 500.0)
            }
        ]
    );
    clouds.take_releases(11);
    update(&mut clouds, Vec::new(), 12, &mut events);
    assert!(clouds.is_empty());
    assert!(
        (clouds.clouds.iter().chain(&clouds.ended)).all(|c| c.members.iter().all(|m| !m.lingers())),
        "nothing ever lingers"
    );
}

/// (M6-CM) Design change 0448: "when the channel ends, by the Disciple's death or for any other reason, the cloud
/// **stays for 1.5 s (24 frames, placeholder)** and keeps absorbing as usual. [...] When it vanishes, its buffer is
/// released": the cloud goes on, its buffer and its timer with it, absorbing and releasing on its timer, until the
/// update 24 ticks after the end, which drops it, its buffer released over its circle as it was.
#[test]
fn a_cloud_lingers_24_frames_after_its_channel_ends_absorbing_as_usual() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(vec![at(1, 100.0)], 1, &mut events);
    clouds.absorb(0, 4.0, 5);
    events.clear();
    clouds.update(Vec::new(), 10, &mut events);
    assert_eq!(events, [SimEvent::ShroudEnded { disciple: ObjectId(1) }]);
    let lingering = Member {
        until: Some(10 + LINGER),
        ..at(1, 100.0)
    };
    assert_eq!(clouds.list().len(), 1, "it stays");
    assert_eq!(clouds.list()[0].members, [lingering]);
    assert_eq!(
        (clouds.list()[0].buffer, clouds.list()[0].release_at),
        (4.0, Some(13)),
        "its buffer and its timer go on"
    );
    assert!(clouds.ended().is_empty());
    // It releases on its timer, and absorbs again.
    assert_eq!(clouds.take_releases(13)[0].buffer, 4.0);
    assert_eq!(clouds.over(Vec2::new(100.0, 600.0)), Some(0), "still over its circle");
    clouds.absorb(0, 3.0, 30);
    // The same as it goes on: an update with nothing new changes nothing, nor does the end of a tick.
    let (before, mut quiet) = (clouds.clone(), Vec::new());
    clouds.update(Vec::new(), 33, &mut quiet);
    clouds.settle(|_| None, 33, &mut quiet);
    assert_eq!((&clouds, quiet.as_slice()), (&before, &[][..]));
    // 24 ticks after the end, it vanishes, and its buffer waits for this tick's release over its circle as it was.
    clouds.update(Vec::new(), 34, &mut events);
    assert!(clouds.list().is_empty());
    let ended: Vec<(Vec<Member>, f32)> = clouds.ended().iter().map(|c| (c.members.clone(), c.buffer)).collect();
    assert_eq!(ended, [(vec![lingering], 3.0)]);
    assert!(clouds.due(34));
    clouds.take_releases(34);
    assert!(clouds.is_empty());
    assert_eq!(events.len(), 1, "no more events: {events:?}");
}

/// (M6-CM) The linger at the end of a tick (`Clouds::settle`): a channel that ends there, its unit gone, lingers from
/// that tick, and vanishes at the first update 24 ticks after, in 5a or at the end of a tick.
#[test]
fn a_channel_that_ends_with_its_unit_lingers_from_the_end_of_its_tick() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(vec![at(1, 100.0)], 1, &mut events);
    clouds.settle(|_| None, 5, &mut events);
    assert_eq!(clouds.list()[0].members[0].until, Some(5 + LINGER));
    clouds.settle(|_| None, 5 + LINGER - 1, &mut events);
    assert_eq!(clouds.list().len(), 1, "a tick before its time");
    clouds.settle(|_| None, 5 + LINGER, &mut events);
    assert!(clouds.is_empty(), "nothing in its buffer, nothing to release");
}

/// (M6-CM) "for connect/join/split/timers it's an ordinary cloud": a new channel next to a lingering cloud joins it
/// (each releases its buffer, the joined cloud starts fresh); its circle vanishing later is the loss of a member, which
/// releases the joint buffer over all its circles as they were; and a Disciple that channels again where its circle
/// lingers has both, its new channel's a new member.
#[test]
fn a_lingering_cloud_joins_and_splits_as_any_other() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(vec![at(1, 100.0)], 1, &mut events);
    clouds.update(Vec::new(), 10, &mut events);
    clouds.absorb(0, 2.0, 11);
    // Disciple 2 channels next to it, at tick 12: the join.
    let two = Member {
        since: 12,
        ..at(2, 350.0)
    };
    clouds.update(vec![two], 12, &mut events);
    let list: Vec<(Vec<u32>, f32)> = clouds.list().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(list, [(vec![1, 2], 0.0)], "joined, fresh");
    let ended: Vec<(Vec<u32>, f32)> = clouds.ended().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(ended, [(vec![1], 2.0)], "the lingering cloud released as it joined");
    clouds.take_releases(12);
    clouds.absorb(0, 5.0, 30);
    // Its circle vanishes: the cloud loses a member, and releases over both circles.
    clouds.update(vec![two], 10 + LINGER, &mut events);
    let ended: Vec<(Vec<u32>, f32)> = clouds.ended().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(ended, [(vec![1, 2], 5.0)]);
    let list: Vec<(Vec<u32>, f32)> = clouds.list().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(list, [(vec![2], 0.0)]);
    clouds.take_releases(10 + LINGER);
    // Disciple 2 channels again where it stood: its old circle lingers beside its new channel's, one cloud.
    let again = Member { since: 40, ..two };
    events.clear();
    clouds.update(vec![again], 40, &mut events);
    assert_eq!(
        events,
        [
            SimEvent::ShroudEnded { disciple: ObjectId(2) },
            SimEvent::ShroudStarted {
                disciple: ObjectId(2),
                at: two.at
            }
        ]
    );
    let lingering = Member {
        until: Some(40 + LINGER),
        ..two
    };
    assert_eq!(clouds.list().len(), 1);
    assert_eq!(clouds.list()[0].members, [lingering, again]);
    clouds.update(vec![again], 40 + LINGER, &mut events);
    assert_eq!(clouds.list()[0].members, [again], "the old circle gone");
}

/// (M6-CM) A channel whose circle moves under it, its unit teleported (`DebugCommand::SetPosition`) or its radius
/// changed by new rules, goes on: its old circle gives way to the new one and doesn't linger, its channel not having
/// ended (as before the linger: ended and started anew, the cloud fresh, its buffer released over the old circle). The
/// seed sweep's seed 87 found it: a lingering copy of a channel under way listed it twice.
#[test]
fn a_channel_moved_under_it_leaves_no_lingering_circle() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(vec![at(1, 100.0)], 1, &mut events);
    clouds.absorb(0, 3.0, 2);
    events.clear();
    let moved = at(1, 900.0);
    clouds.update(vec![moved], 5, &mut events);
    assert_eq!(clouds.list().len(), 1);
    assert_eq!(clouds.list()[0].members, [moved], "the new circle alone");
    assert_eq!(clouds.list()[0].buffer, 0.0, "fresh");
    let ended: Vec<(Vec<Member>, f32)> = clouds.ended().iter().map(|c| (c.members.clone(), c.buffer)).collect();
    assert_eq!(ended, [(vec![at(1, 100.0)], 3.0)], "released over the old circle");
    assert_eq!(
        events,
        [
            SimEvent::ShroudEnded { disciple: ObjectId(1) },
            SimEvent::ShroudStarted {
                disciple: ObjectId(1),
                at: moved.at
            }
        ]
    );
    let wider = Member { radius: 256.0, ..moved };
    clouds.update(vec![wider], 6, &mut events);
    assert_eq!(clouds.list()[0].members, [wider], "resized: the new circle alone");
    clouds.update(Vec::new(), 7, &mut events);
    let lingering = Member {
        until: Some(7 + LINGER),
        ..wider
    };
    assert_eq!(clouds.list()[0].members, [lingering], "its end lingers, once");
}
