use super::player::tile_span;
use super::*;
use crate::defs::ObjectKind;
const FOG_DELAY_TICKS: crate::units::Tick = crate::tuning::vision::BUILTIN.fog_delay_ticks;
const REVEAL_TICKS: crate::units::Tick = crate::tuning::vision::BUILTIN.reveal_ticks;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn unit(id: u32, kind: ObjectKind, owner: Option<PlayerId>, pos: Vec2) -> Object {
    Object {
        id: ObjectId(id),
        kind,
        owner,
        pos,
        prev_pos: pos,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        shields: kind.def().max_shields,
        placement: None,
        resource: None,
        orders: None,
        mover: None,
        attack: None,
        riot_shield: None,
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
        tunnel: crate::tunnel::fresh(kind),
        construction: None,
        decay: None,
        popup: None,
        lineage: Vec::new(),
        augmentation: Default::default(),
        energy: None,
        kept_research: None,
        buffs: Vec::new(),
        stock: None,
        tether: None,
        drone: None,
        cooldown: None,
        aircraft: None,
        illusion_of: None,
        tanks: None,
        thrust: None,
        chopper: None,
        schedule: None,
    }
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn visible(vision: &Vision, player: PlayerId) -> usize {
    vision
        .player(player)
        .unwrap()
        .states()
        .filter(|&s| s == VisibilityState::Visible)
        .count()
}

/// How many of `player`'s tiles are Visible only for the fog delay.
fn lingering(vision: &Vision, player: PlayerId) -> usize {
    vision.player(player).unwrap().lingering().count()
}

/// A box covers the tiles it overlaps: touching an edge isn't overlapping, a box of no width covers the tile it
/// lies in, and the span is clamped to the map.
#[test]
fn tile_spans() {
    assert_eq!(tile_span(64.0, 128.0, 10), (1, 2));
    assert_eq!(tile_span(63.0, 128.0, 10), (0, 2));
    assert_eq!(tile_span(64.0, 129.0, 10), (1, 3));
    assert_eq!(tile_span(80.0, 80.0, 10), (1, 2));
    assert_eq!(tile_span(64.0, 64.0, 10), (1, 2));
    assert_eq!(tile_span(-10.0, 10.0, 10), (0, 1));
    assert_eq!(tile_span(-100.0, -50.0, 10), (0, 0));
    assert_eq!(tile_span(700.0, 800.0, 10), (10, 10));
}

/// A disc of radius 5 around a tile centre holds the 81 tiles whose centres are within 5 grid units: the lattice
/// points with dx² + dy² ≤ 25, the 3-4-5 ones on the boundary included.
#[test]
fn a_disc_of_tiles() {
    let map = Map::new(20, 20);
    let mut vision = Vision::new(&map, [P1]);
    vision.update(&map, [&unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10))], 0);
    assert_eq!(visible(&vision, P1), 81);
    let state = |x, y| vision.state(P1, TilePos::new(x, y));
    assert_eq!(state(13, 14), VisibilityState::Visible);
    assert_eq!(state(14, 14), VisibilityState::Unexplored);
    assert_eq!(state(15, 10), VisibilityState::Visible);
    assert_eq!(state(16, 10), VisibilityState::Unexplored);
}

/// Air units are exempt from the ElevationModifier (`core/vision.md > ElevationModifier`): no M1 unit flies, so
/// this stamps an exempt eye directly. From the same spot a ground eye on level 1 sees one grid unit further over
/// level 0 and one less up to level 2.
#[test]
fn an_exempt_eye_sees_its_sight_range_over_any_ground() {
    // Columns 0–9 at level 0, 10–19 at level 1, 20–29 at level 2.
    let levels: Vec<u8> = (0..30 * 3).map(|i| ((i % 30) / 10) as u8).collect();
    let row = |eye: &Eye| {
        let mut tiles = Vec::new();
        stamp(eye, 30, 3, &levels, &mut tiles);
        let row: Vec<u32> = tiles.into_iter().filter(|i| i / 30 == 1).map(|i| i % 30).collect();
        (row[0], row[row.len() - 1])
    };
    let mut eye = Eye {
        player: 0,
        at: centre(15, 1),
        sight: 5.0,
        elevation: Elevation::Exempt,
    };
    assert_eq!(row(&eye), (10, 20));
    eye.elevation = Elevation::Level(1);
    assert_eq!(row(&eye), (9, 19), "6 over level 0, 4 up to level 2");
}

/// The update changes what differs, and the revision only when a tile's state does. A tile no source sees any more
/// stays Visible for the fog delay (design change 0178), and turns Explored in the update of the tick its delay ends.
#[test]
fn updates_change_only_what_moved() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let revision = |v: &Vision, p| v.player(p).unwrap().revision();
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    vision.update(&map, [&scout], 1);
    assert_eq!((revision(&vision, P1), revision(&vision, P2)), (1, 0));
    // Turning changes nothing.
    scout.facing = 1.0;
    vision.update(&map, [&scout], 2);
    assert_eq!(revision(&vision, P1), 1);
    // 10 su east: the 7 tiles exactly on the boundary to the north, west and south drop out of sight, and 2 to the
    // east come in. Those that dropped out stay Visible.
    scout.pos += Vec2::new(10.0, 0.0);
    vision.update(&map, [&scout], 3);
    assert_eq!(
        (revision(&vision, P1), visible(&vision, P1), lingering(&vision, P1)),
        (2, 83, 7)
    );
    assert!(vision.player(P1).unwrap().lingers(TilePos::new(5, 10)));
    // A step that sees the same tiles changes nothing.
    scout.pos += Vec2::new(1.0, 0.0);
    vision.update(&map, [&scout], 4);
    assert_eq!((revision(&vision, P1), visible(&vision, P1)), (2, 83));
    // They stay Visible at the end of ticks 3 to 10, and tick 11's update turns them Explored.
    vision.update(&map, [&scout], 2 + FOG_DELAY_TICKS);
    assert_eq!((revision(&vision, P1), visible(&vision, P1)), (2, 83));
    vision.check(&map, [&scout], 2 + FOG_DELAY_TICKS).unwrap();
    vision.update(&map, [&scout], 3 + FOG_DELAY_TICKS);
    assert_eq!(
        (revision(&vision, P1), visible(&vision, P1), lingering(&vision, P1)),
        (3, 76, 0)
    );
    assert_eq!(vision.state(P1, TilePos::new(5, 10)), VisibilityState::Explored);
    // On the next tile's centre: the east edge Visible at once.
    scout.pos = centre(11, 10);
    vision.update(&map, [&scout], 12);
    assert_eq!(revision(&vision, P1), 4);
    assert_eq!(vision.state(P1, TilePos::new(16, 10)), VisibilityState::Visible);
    vision.check(&map, [&scout], 12).unwrap();
    // Gone: everything it saw stays Visible for the delay (the 4 tiles it left in tick 12 too), and is Explored after.
    vision.update(&map, [], 13);
    assert_eq!((revision(&vision, P1), lingering(&vision, P1)), (4, 81 + 4));
    vision.check(&map, [], 13).unwrap();
    vision.update(&map, [], 13 + FOG_DELAY_TICKS);
    assert_eq!(visible(&vision, P1), 0);
    assert_eq!(vision.state(P1, TilePos::new(11, 10)), VisibilityState::Explored);
    assert_eq!(revision(&vision, P1), 5);
    assert_eq!(revision(&vision, P2), 0, "the other player saw nothing change");
}

/// Design change 0178: a tile that comes back into sight within the delay simply stays Visible, and its delay
/// starts again the next time it leaves sight. The update touches only the tiles that left sight or whose delay
/// ends, through a queue in the order the delays end.
#[test]
fn a_tile_back_in_sight_stays_visible() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1]);
    let revision = |v: &Vision| v.player(P1).unwrap().revision();
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    let west = TilePos::new(5, 10);
    vision.update(&map, [&scout], 1);
    scout.pos = centre(11, 10);
    vision.update(&map, [&scout], 2);
    assert!(vision.player(P1).unwrap().lingers(west));
    let moved = revision(&vision);
    // Back before the delay ends: Visible all along, with no change of state.
    scout.pos = centre(10, 10);
    vision.update(&map, [&scout], 5);
    assert!(!vision.player(P1).unwrap().lingers(west));
    // Away again in tick 6: its old entry (due at tick 10) is stale, and the tile turns Explored at tick 14.
    scout.pos = centre(11, 10);
    vision.update(&map, [&scout], 6);
    vision.update(&map, [&scout], 2 + FOG_DELAY_TICKS);
    assert_eq!(vision.state(P1, west), VisibilityState::Visible);
    vision.check(&map, [&scout], 2 + FOG_DELAY_TICKS).unwrap();
    vision.update(&map, [&scout], 5 + FOG_DELAY_TICKS);
    assert_eq!(vision.state(P1, west), VisibilityState::Visible);
    vision.update(&map, [&scout], 6 + FOG_DELAY_TICKS);
    assert_eq!(vision.state(P1, west), VisibilityState::Explored);
    assert_eq!(
        vision.player(P1).unwrap().linger.queued(),
        0,
        "the queue held nothing but the due entries"
    );
    assert!(revision(&vision) > moved);
    vision.check(&map, [&scout], 6 + FOG_DELAY_TICKS).unwrap();
}

/// Two sources on one tile count twice: it stays Visible while either sees it.
#[test]
fn overlapping_sources_count_each() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1]);
    let a = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    let b = unit(2, ObjectKind::Guard, Some(P1), centre(14, 10));
    vision.update(&map, [&a, &b], 1);
    vision.update(&map, [&b], 2);
    assert_eq!(vision.state(P1, TilePos::new(12, 10)), VisibilityState::Visible);
    assert!(vision.player(P1).unwrap().lingers(TilePos::new(8, 10)));
    assert!(!vision.player(P1).unwrap().lingers(TilePos::new(12, 10)));
    vision.update(&map, [&b], 2 + FOG_DELAY_TICKS);
    assert_eq!(vision.state(P1, TilePos::new(12, 10)), VisibilityState::Visible);
    assert_eq!(vision.state(P1, TilePos::new(8, 10)), VisibilityState::Explored);
    vision.check(&map, [&b], 2 + FOG_DELAY_TICKS).unwrap();
}

/// `core/entities.md > Object Type > SightRange`: vision goes to the owner. A neutral unit and a resource (sight 0)
/// see nothing; a unit that changes hands takes its sight to the new owner, and what it saw stays Visible to the old
/// one for the fog delay.
#[test]
fn only_owned_objects_with_sight_give_vision() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let neutral = unit(1, ObjectKind::Guard, None, centre(5, 5));
    let patch = unit(2, ObjectKind::CrystalPatch, Some(P1), centre(20, 5));
    let mut turncoat = unit(3, ObjectKind::Peacekeeper, Some(P1), centre(15, 12));
    vision.update(&map, [&neutral, &patch, &turncoat], 1);
    assert_eq!(visible(&vision, P1), 81);
    turncoat.owner = Some(P2);
    vision.update(&map, [&neutral, &patch, &turncoat], 2);
    assert_eq!((visible(&vision, P1), visible(&vision, P2)), (81, 81));
    assert_eq!(lingering(&vision, P1), 81);
    vision.update(&map, [&neutral, &patch, &turncoat], 2 + FOG_DELAY_TICKS);
    assert_eq!((visible(&vision, P1), visible(&vision, P2)), (0, 81));
    vision
        .check(&map, [&neutral, &patch, &turncoat], 2 + FOG_DELAY_TICKS)
        .unwrap();
}

/// V1 catches a count that drifted from the objects, a Visible tile that isn't Explored, a tile lingering in sight
/// or past its delay, and a reveal past its end.
#[test]
fn the_check_finds_drift() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(10, 10));
    let foe = unit(2, ObjectKind::Guard, Some(P2), centre(25, 10));
    vision.update(&map, [&scout, &foe], 1);
    scout.pos = centre(11, 10);
    vision.update(&map, [&scout, &foe], 2);
    vision.reveal(P1, &foe, 2);
    vision.check(&map, [&scout, &foe], 2).unwrap();
    let mut drifted = vision.clone();
    drifted.players[0].seers[0] = 1;
    let (at, message) = drifted.check(&map, [&scout, &foe], 2).unwrap_err();
    assert_eq!(at, centre(0, 0));
    assert!(message.contains("fresh count"), "{message}");
    let mut forgotten = vision.clone();
    forgotten.players[0].explored = TileBits::new(30 * 20);
    let (_, message) = forgotten.check(&map, [&scout, &foe], 2).unwrap_err();
    assert!(message.contains("not Explored"), "{message}");
    let mut seen = vision.clone();
    seen.players[0].linger.start(10 * 30 + 11, 2);
    let (at, message) = seen.check(&map, [&scout, &foe], 2).unwrap_err();
    assert_eq!(at, centre(11, 10));
    assert!(message.contains("lingers, though 1 sources see it"), "{message}");
    // The tiles the scout left in tick 2, (10, 5) first, linger until tick 10: then they are still Visible, and
    // later a reveal is still on.
    let (at, message) = vision.check(&map, [&scout, &foe], 2 + FOG_DELAY_TICKS).unwrap_err();
    assert_eq!(at, centre(10, 5));
    assert!(message.contains("outside the delay"), "{message}");
    let mut revealed = vision.clone();
    revealed.update(&map, [&scout, &foe], 2 + FOG_DELAY_TICKS);
    revealed.check(&map, [&scout, &foe], 2 + FOG_DELAY_TICKS).unwrap();
    let (at, message) = revealed.check(&map, [&scout, &foe], 2 + REVEAL_TICKS).unwrap_err();
    assert_eq!(at, foe.pos);
    assert!(message.contains("#2 is revealed until tick 26"), "{message}");
}

/// A save keeps each player's Explored tiles as rows of symbols, and its tiles that linger and its reveals as
/// lists; loading rebuilds the rest from the objects.
#[test]
fn explored_rows_round_trip() {
    let map = Map::new(30, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let mut scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(6, 6));
    let foe = unit(2, ObjectKind::Guard, Some(P2), centre(28, 18));
    vision.update(&map, [&scout, &foe], 1);
    scout.pos = centre(20, 12);
    vision.update(&map, [&scout, &foe], 2);
    vision.reveal(P1, &foe, 2);
    let saved = vision.saved();
    assert_eq!(saved[0].explored.len(), 20);
    assert_eq!(&saved[0].explored[6][..13], ".###########.");
    assert!(
        saved[0]
            .lingering
            .iter()
            .any(|l| l.tile == TilePos::new(6, 6) && l.ends == 10)
    );
    assert_eq!(
        saved[0].revealed,
        [save::SavedReveal {
            id: ObjectId(2),
            ends: 26
        }]
    );
    let loaded = load(&map, &[P1, P2], saved.clone(), 2, [&scout, &foe]).unwrap();
    assert_eq!(loaded, vision);
    assert_eq!(loaded.state(P1, TilePos::new(6, 6)), VisibilityState::Visible);
    assert!(loaded.sees(P1, &foe), "revealed");
    loaded.check(&map, [&scout, &foe], 2).unwrap();
    // Once the delay is over, in a save of tick 10.
    vision.update(&map, [&scout, &foe], 10);
    let later = vision.saved();
    let loaded = load(&map, &[P1, P2], later.clone(), 10, [&scout, &foe]).unwrap();
    assert_eq!(loaded, vision);
    assert_eq!(loaded.state(P1, TilePos::new(6, 6)), VisibilityState::Explored);
    assert_eq!(loaded.state(P1, TilePos::new(20, 12)), VisibilityState::Visible);
    loaded.check(&map, [&scout, &foe], 10).unwrap();

    // Tiles Visible now are Explored, even if the save forgot them.
    let mut forgetful = later.clone();
    forgetful[0].explored = vec![save::UNEXPLORED_SYMBOL.to_string().repeat(30); 20];
    let loaded = load(&map, &[P1, P2], forgetful, 10, [&scout, &foe]).unwrap();
    assert_eq!(loaded.state(P1, TilePos::new(20, 12)), VisibilityState::Visible);
    loaded.check(&map, [&scout, &foe], 10).unwrap();
    // A tile a source sees no longer lingers.
    let mut seen = later.clone();
    let tile = TilePos::new(20, 12);
    seen[0].lingering = vec![save::SavedLinger { tile, ends: 12 }];
    let loaded = load(&map, &[P1, P2], seen, 10, [&scout, &foe]).unwrap();
    assert!(!loaded.player(P1).unwrap().lingers(tile));
    loaded.check(&map, [&scout, &foe], 10).unwrap();

    let fails = |saved: Vec<SavedVision>, tick| load(&map, &[P1, P2], saved, tick, [&scout, &foe]).unwrap_err();
    assert!(
        fails(saved.clone(), 10).contains("lingers until tick 10"),
        "a delay over"
    );
    assert!(
        fails(saved.clone(), 0).contains("outside the 8 ticks"),
        "a delay too long"
    );
    let mut twice = saved.clone();
    let first = twice[0].lingering[0].clone();
    twice[0].lingering.insert(0, first);
    assert!(fails(twice, 2).contains("out of order, or twice"));
    let mut off = saved.clone();
    off[0].lingering[0].tile = TilePos::new(30, 0);
    assert!(fails(off, 2).contains("off the map"));
    assert!(
        fails(later.clone(), 26).contains("reveal of #2 ends at tick 26"),
        "a reveal over"
    );
    let mut again = later.clone();
    let reveal = again[0].revealed[0].clone();
    again[0].revealed.push(reveal);
    assert!(fails(again, 10).contains("out of order, or twice"));
    assert!(
        load(&map, &[P2, P1], later.clone(), 10, [&scout, &foe]).is_err(),
        "players out of order"
    );
    assert!(
        load(&map, &[P1], later.clone(), 10, [&scout, &foe]).is_err(),
        "a player too many"
    );
    let mut short = later.clone();
    short[1].explored.pop();
    assert!(load(&map, &[P1, P2], short, 10, [&scout, &foe]).is_err());
    let mut odd = later;
    odd[1].explored[0].replace_range(0..1, "?");
    assert!(load(&map, &[P1, P2], odd, 10, [&scout, &foe]).is_err());
}

/// Design change 0177: an attacker that shoots one of the player's objects from outside its vision is revealed to
/// the player, it alone and not its tiles, from the tick of the shot through the 23 after; the tick after that
/// ends it, unless another shot from outside renews it. One the player sees by its tiles (the fog delay
/// included) is not revealed, nor one the player owns.
#[test]
fn a_shot_from_outside_vision_reveals_the_attacker() {
    fn p1(vision: &Vision) -> &PlayerVision {
        vision.player(P1).unwrap()
    }
    let map = Map::new(40, 20);
    let mut vision = Vision::new(&map, [P1, P2]);
    let mut victim = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(5, 10));
    let mut sniper = unit(2, ObjectKind::Guard, Some(P2), centre(20, 10));
    let friend = unit(3, ObjectKind::Guard, Some(P1), centre(30, 10));
    let end_of = |vision: &mut Vision, tick, victim: &Object, sniper: &Object| {
        vision.update(&map, [victim, sniper, &friend], tick);
        vision.check(&map, [victim, sniper, &friend], tick).unwrap();
    };
    end_of(&mut vision, 1, &victim, &sniper);
    assert!(!vision.sees(P1, &sniper));
    // One shot, in phase 5e of tick 2.
    vision.reveal(P1, &sniper, 2);
    vision.reveal(P1, &friend, 2);
    end_of(&mut vision, 2, &victim, &sniper);
    assert!(vision.sees(P1, &sniper));
    assert_eq!(p1(&vision).object_state(&sniper), VisibilityState::Visible);
    assert!(!p1(&vision).is_revealed(friend.id), "its own");
    assert_eq!(
        vision.state(P1, TilePos::new(20, 10)),
        VisibilityState::Unexplored,
        "only the attacker"
    );
    end_of(&mut vision, 1 + REVEAL_TICKS, &victim, &sniper);
    assert!(vision.sees(P1, &sniper), "through tick 25");
    end_of(&mut vision, 2 + REVEAL_TICKS, &victim, &sniper);
    assert!(!vision.sees(P1, &sniper), "ended in tick 26");
    // Shots in ticks 30 and 40: the second renews the reveal, to the end of tick 63.
    vision.reveal(P1, &sniper, 30);
    end_of(&mut vision, 30, &victim, &sniper);
    vision.reveal(P1, &sniper, 40);
    end_of(&mut vision, 40, &victim, &sniper);
    end_of(&mut vision, 30 + REVEAL_TICKS, &victim, &sniper);
    assert!(vision.sees(P1, &sniper), "renewed");
    end_of(&mut vision, 39 + REVEAL_TICKS, &victim, &sniper);
    assert!(vision.sees(P1, &sniper), "through tick 63");
    end_of(&mut vision, 40 + REVEAL_TICKS, &victim, &sniper);
    assert!(!vision.sees(P1, &sniper), "ended in tick 64");
    // In sight by its tiles: no reveal. Visible only for the fog delay: revealed, the timers being independent, so
    // it stays Visible past the delay's end in tick 79, to the end of tick 95.
    sniper.pos = centre(8, 10);
    end_of(&mut vision, 70, &victim, &sniper);
    vision.reveal(P1, &sniper, 71);
    assert!(!p1(&vision).is_revealed(sniper.id));
    victim.pos = centre(5, 0);
    end_of(&mut vision, 71, &victim, &sniper);
    assert!(p1(&vision).lingers(TilePos::new(8, 10)));
    vision.reveal(P1, &sniper, 72);
    assert!(
        p1(&vision).is_revealed(sniper.id),
        "the fog delay isn't the attacker's sources"
    );
    end_of(&mut vision, 72, &victim, &sniper);
    end_of(&mut vision, 71 + FOG_DELAY_TICKS, &victim, &sniper);
    assert_eq!(vision.state(P1, TilePos::new(8, 10)), VisibilityState::Explored);
    assert!(vision.sees(P1, &sniper), "revealed");
    end_of(&mut vision, 71 + REVEAL_TICKS, &victim, &sniper);
    assert!(vision.sees(P1, &sniper), "through tick 95");
    end_of(&mut vision, 72 + REVEAL_TICKS, &victim, &sniper);
    assert!(!vision.sees(P1, &sniper));
}

/// The tiles that linger and the reveals are history: two visions that differ in either alone hash apart.
#[test]
fn the_fog_delay_and_the_reveals_are_hashed() {
    use crate::hash::{StateHash, StateHasher};
    let hash = |vision: &Vision| {
        let mut h = StateHasher::new();
        vision.hash(&mut h);
        h.finish()
    };
    let map = Map::new(20, 10);
    let scout = unit(1, ObjectKind::Peacekeeper, Some(P1), centre(5, 5));
    let sniper = unit(2, ObjectKind::Guard, Some(P2), centre(15, 5));
    let mut stayed = Vision::new(&map, [P1, P2]);
    stayed.update(&map, [&scout, &sniper], 1);
    let mut left = stayed.clone();
    stayed.update(&map, [&scout, &sniper], 2);
    left.update(&map, [&sniper], 2);
    // The disc's 81 tiles but one row off the map's south edge.
    assert_eq!(
        (visible(&stayed, P1), visible(&left, P1)),
        (80, 80),
        "the same tiles Visible"
    );
    assert_eq!((lingering(&stayed, P1), lingering(&left, P1)), (0, 80));
    assert_ne!(hash(&stayed), hash(&left));
    let mut revealed = stayed.clone();
    revealed.reveal(P1, &sniper, 2);
    assert_eq!(hash(&stayed.clone()), hash(&stayed));
    assert_ne!(hash(&revealed), hash(&stayed));
}
