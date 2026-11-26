//! The clouds' bookkeeping on its own (`super`): components, matching, the timer and the releases. In games:
//! `crates/sim/tests/shroud/`.

use super::*;

/// The channel of unit `id`, since tick 1, at (`x`, 500) with a 3-GU radius.
fn at(id: u32, x: f32) -> Member {
    Member {
        id: ObjectId(id),
        since: 1,
        at: Vec2::new(x, 500.0),
        radius: 192.0,
    }
}

fn ids(cloud: &Cloud) -> Vec<u32> {
    cloud.members.iter().map(|m| m.id.0).collect()
}

/// "Clouds are connected when their circles **touch or overlap**": 5 GU apart (320 su) one cloud, touching (384 su) one
/// cloud, a hair beyond apart; and the components in id order, whatever the order along the map.
#[test]
fn circles_that_touch_or_overlap_are_one_cloud() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    clouds.update(
        vec![at(1, 100.0), at(2, 420.0), at(3, 804.0), at(4, 1189.0)],
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
    clouds.update(vec![at(1, 100.0)], &mut events);
    clouds.absorb(0, 4.0, 10);
    clouds.absorb(0, 6.0, 12);
    assert_eq!((clouds.list()[0].buffer, clouds.list()[0].release_at), (10.0, Some(18)));
    clouds.update(vec![at(1, 100.0)], &mut events);
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
/// over the cloud as it was; a cloud that ends with nothing in its buffer releases nothing.
#[test]
fn joins_splits_and_ends_release_the_buffers_as_the_clouds_were() {
    let mut clouds = Clouds::default();
    let mut events = Vec::new();
    // Two clouds a short gap apart, each with a buffer.
    clouds.update(vec![at(1, 100.0), at(3, 600.0)], &mut events);
    clouds.absorb(0, 2.0, 5);
    clouds.absorb(1, 3.0, 6);
    // A third channels in between.
    clouds.update(vec![at(1, 100.0), at(2, 350.0), at(3, 600.0)], &mut events);
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
    clouds.update(vec![at(1, 100.0), at(3, 600.0)], &mut events);
    let ended: Vec<(Vec<u32>, f32)> = clouds.ended().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(ended, [(vec![1, 2, 3], 5.0)]);
    let list: Vec<(Vec<u32>, f32)> = clouds.list().iter().map(|c| (ids(c), c.buffer)).collect();
    assert_eq!(list, [(vec![1], 0.0), (vec![3], 0.0)]);
    clouds.take_releases(9);
    // A channel that ends with an empty buffer releases nothing; a new Shroud order is a new channel.
    clouds.update(vec![at(1, 100.0)], &mut events);
    assert!(clouds.ended().is_empty() && clouds.list().len() == 1);
    clouds.absorb(0, 1.0, 10);
    let recast = Member {
        since: 11,
        ..at(1, 100.0)
    };
    events.clear();
    clouds.update(vec![recast], &mut events);
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
    clouds.update(Vec::new(), &mut events);
    assert!(clouds.is_empty());
}
