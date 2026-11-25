//! The filter's table (slice M4-V): every kind of `SimEvent`, and what each viewer perceives of it. The world and the
//! test are here, the rows in `tests/table.rs`.

use serde::Deserialize;
use serde::de::{self, Visitor};
use std::fmt;

use sc_sim::testing::TestSim;
use sc_sim::{DebugCommand, Faction, GameSetup, ObjectKind, Slot, SpawnAt, TilePos, scenarios};

use super::*;

mod table;

use table::table;

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);

/// The objects of [`world`], by id: player 1's unit, its Tunnel and its Hideout (hidden from the others), player 2's
/// unit, which sees them where they are, player 3's, far off in the east, which sees none of them, and player 1's
/// Laser Turret, a structure that shoots (slice M4-S1), which player 2 sees too.
const UNIT: ObjectId = ObjectId(1);
const TUNNEL: ObjectId = ObjectId(2);
const HIDEOUT: ObjectId = ObjectId(3);
const WATCHER: ObjectId = ObjectId(4);
const FAR: ObjectId = ObjectId(5);
const TURRET: ObjectId = ObjectId(6);
/// Player 1's objects that the sim no longer has: one that died in the frame, where player 2 sees (its `Died` is among
/// the frame's events); a site gone without a death (cancelled), as the filter last saw it, where player 2 sees; a unit
/// off the map, inside player 1's network; and a Hideout that died, hidden from the others.
const DEAD: ObjectId = ObjectId(90);
const GONE: ObjectId = ObjectId(91);
const HELD: ObjectId = ObjectId(92);
const HIDDEN_DEAD: ObjectId = ObjectId(93);
/// A death the event says was hidden, whatever its kind: what a pop-up turret's will say while it is fully lowered
/// (`factions/syndicate/structures/laser_turret.md > Upright vs Pop-up (by location)`), once `Hiding::of` has that arm.
const LOWERED: ObjectId = ObjectId(94);

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 40 × 10 map with three players, player 1 the Syndicate, auto-targeting off, the objects of [`UNIT`] to
/// [`TURRET`] spawned in the first tick: player 1's Peacekeeper on (4, 5), its Tunnel on (6, 1)–(9, 4) and its Hideout
/// on (6, 6)–(7, 7); player 2's Peacekeeper on (8, 5), whose SightRange of 5 takes in all three; player 3's on
/// (35, 5); player 1's Laser Turret on (4, 3), in player 2's sight too.
fn world() -> Sim {
    let mut scenario = scenarios::empty(40, 10);
    scenario.map.slots.push(Slot::new(TilePos::new(35, 5)));
    scenario.setup = GameSetup::default_for(&scenario.map);
    scenario.setup.players[0].faction = Faction::Syndicate;
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    let spawn = |kind, owner, at| DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at,
        orientation: Default::default(),
    };
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(4, 5))
        .debug_at(1, spawn(ObjectKind::Tunnel, P1, tile(6, 1)))
        .debug_at(1, spawn(ObjectKind::Hideout, P1, tile(6, 6)))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(8, 5))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P3, centre(35, 5))
        .debug_at(1, spawn(ObjectKind::LaserTurret, P1, tile(4, 3)))
        .run_until(2);
    t.sim().clone()
}

/// What a viewer perceives of an event, as the table says it: all of it, along its path over their Visible tiles, or
/// nothing.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Seen {
    Whole,
    Path,
    Nothing,
}

use Seen::{Nothing as N, Path as P, Whole as W};

impl Seen {
    fn of(perception: Perception) -> Self {
        match perception {
            Perception::Whole => W,
            Perception::Clipped { .. } => P,
            Perception::Nothing => N,
        }
    }
}

/// The table (`tests/table.rs`), through the filter: every kind of event, as the frame brings it, reaches its owner,
/// an enemy who sees where it is, an enemy in the fog and that enemy with the fog off as the table says; a path is
/// clipped to the event's own. Player 1's own business reaches player 1 alone, fog or none; what happens to an object
/// or at a place reaches whoever sees the object or the place, never one the object is hidden from (the Hideout, and a
/// death that says it was hidden); every shot and projectile is clipped for every viewer, its shooter's owner too
/// (design change 0293). And every kind of event has a row: a new kind fails here until it has one, as it doesn't
/// compile until [`Arrival::audience`] has its rule.
#[test]
fn every_kind_of_event_reaches_whom_the_table_says() {
    let sim = world();
    for (id, kind, owner) in [
        (UNIT, ObjectKind::Peacekeeper, P1),
        (TUNNEL, ObjectKind::Tunnel, P1),
        (HIDEOUT, ObjectKind::Hideout, P1),
        (WATCHER, ObjectKind::Peacekeeper, P2),
        (FAR, ObjectKind::Peacekeeper, P3),
        (TURRET, ObjectKind::LaserTurret, P1),
    ] {
        let o = sim.object(id).unwrap_or_else(|| panic!("no object {id:?}"));
        assert_eq!((o.kind, o.owner), (kind, Some(owner)), "{id:?}");
    }
    let sight = |player, fog| Sight::new(&sim, player, fog);
    let (p2, p3) = (sight(P2, true), sight(P3, true));
    let object = |id| sim.object(id).unwrap();
    assert!(
        p2.sees(object(UNIT)) && p2.sees(object(TUNNEL)) && p2.sees(object(TURRET)),
        "player 2 sees player 1's objects"
    );
    assert!(!p2.sees(object(HIDEOUT)), "but not the Hideout, hidden from it");
    assert_eq!(
        p2.at(object(HIDEOUT).pos),
        VisibilityState::Visible,
        "though it sees its tiles"
    );
    assert!(
        !p3.sees(object(UNIT)) && !p3.sees(object(TUNNEL)) && !p3.sees(object(TURRET)),
        "player 3 sees none of it"
    );
    assert_eq!(p3.at(centre(5, 5)), VisibilityState::Unexplored);

    let rows = table(&sim);
    let batch: Vec<SimEvent> = rows.iter().map(|(event, _)| event.clone()).collect();
    let mut known = Vec::new();
    take_objects(&sim, &mut known);
    known.push((GONE, Sighting::object_at(Some(P1), Hiding::default(), centre(5, 6))));
    known.push((HELD, Sighting::off_map(Some(P1))));
    known.sort_unstable_by_key(|&(id, _)| id);
    let arrival = Arrival::new(&sim, batch.iter(), &known);
    let viewers = [sight(P1, true), p2, p3, sight(P3, false)];
    for (event, expected) in &rows {
        let audience = arrival.audience(event);
        let perceived = viewers.map(|viewer| audience.perceived_by(&sim, &viewer));
        assert_eq!(perceived.map(Seen::of), *expected, "{event:?}: {audience:?}");
        if let SimEvent::Attacked { from, to, .. } | SimEvent::ProjectileLaunched { from, to, .. } = *event {
            assert_eq!(perceived[0], Perception::Clipped { from, to }, "its own path");
        }
    }

    let named: Vec<&str> = rows.iter().map(|(event, _)| variant(event)).collect();
    let missing: Vec<&&str> = every_variant().iter().filter(|name| !named.contains(name)).collect();
    assert!(missing.is_empty(), "no row for {missing:?}");
}

/// The name of `event`'s kind, as serde writes it: the key of its externally tagged form.
fn variant(event: &SimEvent) -> &'static str {
    let value = serde_json::to_value(event).expect("an event serializes");
    let name = match &value {
        serde_json::Value::Object(map) if map.len() == 1 => map.keys().next().cloned(),
        serde_json::Value::String(name) => Some(name.clone()),
        _ => None,
    };
    let name = name.unwrap_or_else(|| panic!("not an enum's form: {value}"));
    every_variant()
        .iter()
        .find(|known| **known == name)
        .unwrap_or_else(|| panic!("no such kind: {name}"))
}

/// The names of every kind of `SimEvent`, as its `Deserialize` lists them: a kind added to the enum is among them
/// without anyone listing it. They come from a deserializer that asks for nothing but them.
fn every_variant() -> &'static [&'static str] {
    #[derive(Debug)]
    struct Stop;

    impl fmt::Display for Stop {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("only the variants' names are asked for")
        }
    }

    impl std::error::Error for Stop {}

    impl de::Error for Stop {
        fn custom<T: fmt::Display>(_: T) -> Self {
            Stop
        }
    }

    struct Names(Option<&'static [&'static str]>);

    impl<'de> de::Deserializer<'de> for &mut Names {
        type Error = Stop;

        fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value, Stop> {
            Err(Stop)
        }

        fn deserialize_enum<V: Visitor<'de>>(
            self,
            _: &'static str,
            variants: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Stop> {
            self.0 = Some(variants);
            Err(Stop)
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option unit
            unit_struct newtype_struct seq tuple tuple_struct map struct identifier ignored_any
        }
    }

    let mut names = Names(None);
    assert!(SimEvent::deserialize(&mut names).is_err());
    names.0.expect("SimEvent deserializes as an enum")
}
