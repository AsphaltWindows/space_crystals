//! The Shadebringer's golden game (slice M6-SB, plan decisions T93 and T98; `factions/syndicate/units/shadebringer.md >
//! Sludge (toggle)`, `factions/syndicate/structures/vault.md`; design changes 0369, 0424 and 0425): two Shadebringers
//! lay their trails across the way of four Peacekeepers, slowed in them and for 2 s after while a fifth, away from
//! them, isn't; one backs up over its own trail, slowed too; then both drive into a Tier 2 Tunnel, the one with its
//! Sludge on switched off as it goes in, and its Vault refills their tanks, both at once, until they are full.
//! Auto-targeting is off: nobody fights.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use sc_sim::buffs::BuffKind;
use sc_sim::defs::PatchKind;
use sc_sim::patches::{Tank, trail_point};
use sc_sim::{DebugCommand, SimConfig};

use super::*;

/// Player 1's Tunnel (id 1), unturned on tiles (40, 10) to (43, 13): Side A faces north, its entry path's spot at
/// (2658, 640). Its Hideout (id 2) on (44, 14), and its Vault (id 3), finished, on (44, 8) to (45, 9), in its Tier 1
/// area.
const TUNNEL: ObjectId = ObjectId(1);
const VAULT: ObjectId = ObjectId(3);
/// Player 1's two Shadebringers, A on the line y = 420 and B on y = 468 beside it, west of the Tunnel, facing east.
const A: ObjectId = ObjectId(4);
const B: ObjectId = ObjectId(5);
/// Player 2's four Peacekeepers north of the trails, each sent south across both, and a fifth far east, sent south
/// where there is no trail.
const CROSSING: [ObjectId; 4] = [ObjectId(6), ObjectId(7), ObjectId(8), ObjectId(9)];
const CROSSING_X: [f32; 4] = [1100.0, 1200.0, 1300.0, 1400.0];
const AWAY: ObjectId = ObjectId(10);
const AWAY_X: f32 = 3300.0;
/// Where the Peacekeepers start and where they go, y.
const NORTH: f32 = 150.0;
const SOUTH: f32 = 1000.0;
/// Both switch their Sludge on and drive east along their lines; B switches its off by hand; the Peacekeepers set
/// off.
const ON: Tick = 2;
const B_OFF: Tick = 70;
const GO: Tick = 60;
/// A's tank is empty in this tick, 10 s of use after it was switched on, and its toggle off by itself.
const A_EMPTY: Tick = ON + 159;
/// A backs up 160 su over its own trail, still on the ground.
const A_REVERSE: Tick = 180;
/// B switches its Sludge on again, with charge left, and drives into the Tunnel; A follows it in later.
const B_IN: Tick = 180;
const A_IN: Tick = 240;
/// The game's last tick, once both tanks are full.
const END: Tick = 720;
/// `factions/syndicate/units/shadebringer.md > Placeholder stats`, fixed point (`sc_sim::patches`): "**Tanks**: 100
/// each; drain **10 per second** while on [...]; refill **5 per second** inside the network with a Vault", a tick's
/// of each; "each patch lasts **X = 10 s**", "trail width 48 su".
const CAPACITY: u32 = 100 << 16;
const DRAIN: u32 = (10 << 16) / 16;
const REFILL: u32 = (5 << 16) / 16;
const LIFE: Tick = 160;
const RADIUS: f32 = 24.0;
/// "slowed by **Y = 40%**": a slowed Peacekeeper's move in a tick, at most, of its MaxSpeed of 4.
const SLOWED_PK: f32 = 4.0 * 0.6;

/// The game's events of interest so far.
#[derive(Default)]
struct Seen {
    /// The ticks each unit ended holding the sludge's slow.
    slowed: BTreeMap<ObjectId, Vec<Tick>>,
    /// The Shadebringers' tanks as the last tick left them, held or not.
    tanks: BTreeMap<ObjectId, Tank>,
    /// The ticks they went into the network.
    entered: BTreeMap<ObjectId, Tick>,
    /// The ticks both were held and filling.
    both_filling: usize,
    /// The tick each Peacekeeper reached its point.
    arrived: BTreeMap<ObjectId, Tick>,
    /// The patches laid, by whom, and where.
    laid: BTreeSet<(ObjectId, u32, u32)>,
}

thread_local! {
    static SEEN: RefCell<Seen> = RefCell::new(Seen::default());
}

fn player_debug(command: DebugCommand) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Debug(command),
    }
}

/// The Sludge switched on or off for `units`.
fn sludge(units: &[ObjectId], on: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Ability {
            units: units.to_vec(),
            ability: Ability::Sludge { on },
        },
    }
}

/// Unit `id`, on the map or held.
fn unit(sim: &Sim, id: ObjectId) -> Option<&sc_sim::Object> {
    sim.object(id).or_else(|| sim.held_unit(id).map(|held| &held.object))
}

/// Player 1 (the Syndicate) has a Tunnel, set to Tier 2 by a debug command in tick 1 (its upgrade isn't what the game
/// is about), with a Hideout and a Vault, and two Shadebringers west of it; player 2 (the GDO), five Peacekeepers in
/// the north. At [`ON`] both Shadebringers switch their Sludge on and drive east, laying their trails, side by side,
/// one patch every 24 su; B switches its off at [`B_OFF`], and A's tank is empty at [`A_EMPTY`], its toggle off by
/// itself. From [`GO`] the Peacekeepers walk south: the four crossing the trails hold the slow from the tick their
/// centres are in a patch, walking 2.4 su a tick at most, 0.6 of their 4, until 2 s after they are out; the fifth,
/// away from the trails, never, and arrives first. At [`A_REVERSE`] A backs up over its own trail, slowed. At
/// [`B_IN`] B switches its Sludge on again and drives into the Tunnel, laying a trail to it: as it goes in, the
/// network switches it off. A follows at [`A_IN`]. Inside, the Vault refills their tanks 5 a second, both at once,
/// until both are full; every patch is gone 10 s after it was laid.
pub(crate) fn syndicate_sludge() -> Game {
    let mut builder = ScenarioBuilder::new(&open(56, 24))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Gdo)
        .spawn(ObjectKind::Tunnel, Some(P1), SpawnAt::Tile(TilePos::new(40, 10)))
        .spawn(ObjectKind::Hideout, Some(P1), SpawnAt::Tile(TilePos::new(44, 14)))
        .spawn(ObjectKind::Vault, Some(P1), SpawnAt::Tile(TilePos::new(44, 8)))
        .spawn(
            ObjectKind::Shadebringer,
            Some(P1),
            SpawnAt::Point(Vec2::new(900.0, 420.0)),
        )
        .spawn(
            ObjectKind::Shadebringer,
            Some(P1),
            SpawnAt::Point(Vec2::new(900.0, 468.0)),
        );
    for x in CROSSING_X.into_iter().chain([AWAY_X]) {
        builder = builder.spawn(PK, Some(P2), SpawnAt::Point(Vec2::new(x, NORTH)));
    }
    let move_to = |x: f32, y: f32| OrderKind::Move(Target::Point(Vec2::new(x, y)));
    let mut commands = vec![
        (1, player_debug(DebugCommand::SetAutoTargeting { on: false })),
        (1, player_debug(DebugCommand::SetTier { id: TUNNEL, tier: 2 })),
        (ON, sludge(&[A, B], true)),
        (ON, order(P1, [A.0], move_to(2100.0, 420.0))),
        (ON, order(P1, [B.0], move_to(2200.0, 468.0))),
        (B_OFF, sludge(&[B], false)),
    ];
    for (pk, x) in CROSSING
        .into_iter()
        .chain([AWAY])
        .zip(CROSSING_X.into_iter().chain([AWAY_X]))
    {
        commands.push((GO, order(P2, [pk.0], move_to(x, SOUTH))));
    }
    commands.extend([
        (
            A_REVERSE,
            order(P1, [A.0], OrderKind::Reverse(Vec2::new(2020.0, 420.0))),
        ),
        (B_IN, sludge(&[B], true)),
        (B_IN, order(P1, [B.0], OrderKind::Enter(TUNNEL))),
        (A_IN, order(P1, [A.0], OrderKind::Enter(TUNNEL))),
    ]);
    Game {
        scenario: builder.config(SimConfig::DEBUG).build(),
        commands,
        script: None,
        end: END,
        during: |tick, sim| {
            if tick == 1 {
                SEEN.set(Seen::default());
            }
            SEEN.with_borrow_mut(|seen| during(seen, tick, sim));
        },
        expect: |sim| {
            SEEN.with_borrow(|seen| {
                let slowed = |id: ObjectId| seen.slowed.get(&id).map_or(0, Vec::len);
                for pk in CROSSING {
                    assert!(slowed(pk) > 32, "{pk:?} slowed for {} ticks", slowed(pk));
                }
                assert_eq!(slowed(AWAY), 0, "the Peacekeeper away from the trails");
                let backing =
                    (seen.slowed.get(&A).into_iter().flatten()).filter(|&&tick| (A_REVERSE..A_IN).contains(&tick));
                assert!(backing.count() > 0, "A over its own trail");
                let arrived = |id| {
                    seen.arrived
                        .get(&id)
                        .copied()
                        .unwrap_or_else(|| panic!("{id:?} arrived"))
                };
                for pk in CROSSING {
                    assert!(arrived(pk) > arrived(AWAY), "{pk:?} slowed on its way");
                }
                let laid = |id: ObjectId| seen.laid.iter().filter(|&&(by, ..)| by == id).count();
                assert!(laid(A) > 30 && laid(B) > 10, "{} and {} patches", laid(A), laid(B));
                assert!(seen.entered.contains_key(&A) && seen.entered.contains_key(&B));
                assert!(seen.both_filling > 100, "{} ticks filling both", seen.both_filling);
            });
            for sb in [A, B] {
                let held = sim.held_unit(sb).unwrap_or_else(|| panic!("{sb:?} in the network"));
                let tank = held.object.tanks.expect("its tanks").sludge;
                assert_eq!((tank.used, tank.on), (0, false), "{sb:?} full");
            }
            assert!(sim.patches().is_empty(), "every patch gone");
            let vault = sim.object(VAULT).expect("the Vault");
            assert_eq!(
                (vault.kind, sim.parent_tunnel(VAULT)),
                (ObjectKind::Vault, Some(TUNNEL))
            );
        },
    }
}

/// What the game checks after each tick: the patches, the toggles and tanks, the slows, the network's entries and the
/// Vault's refill (the game's doc comment).
fn during(seen: &mut Seen, tick: Tick, sim: &Sim) {
    // Each patch new in this tick is laid where its Shadebringer's tick left its rear (behind it as it drives on, its
    // front as it backs up), and no patch stays on the ground more than 10 s after it was laid or renewed.
    for patch in sim.patches().iter() {
        assert_eq!(
            (patch.kind, patch.radius),
            (PatchKind::Sludge, RADIUS),
            "tick {tick}: {patch:?}"
        );
        assert!(patch.by == A || patch.by == B, "tick {tick}: {patch:?}");
        assert!(
            tick < patch.until && patch.until <= tick + LIFE,
            "tick {tick}: {patch:?}"
        );
        let new = seen
            .laid
            .insert((patch.by, patch.centre.x.to_bits(), patch.centre.y.to_bits()));
        if new && let Some(o) = sim.object(patch.by) {
            assert_eq!(
                (patch.until, patch.centre),
                (tick + LIFE, trail_point(o)),
                "tick {tick}: {patch:?}"
            );
        }
    }
    for e in sim.events() {
        if let SimEvent::EnteredNetwork { unit, .. } = *e {
            seen.entered.insert(unit, tick);
            let before = seen.tanks[&unit];
            let now = sim
                .held_unit(unit)
                .and_then(|held| held.object.tanks)
                .expect("held")
                .sludge;
            assert!(!now.on, "tick {tick}: {unit:?} switched off going in");
            if unit == B {
                assert!(
                    before.on && before.used < CAPACITY,
                    "tick {tick}: B went in laying, {before:?}"
                );
            }
        }
    }
    let mut filling = 0;
    for sb in [A, B] {
        let tank = unit(sim, sb)
            .and_then(|o| o.tanks)
            .expect("a Shadebringer's tanks")
            .sludge;
        let expected_a = match tick {
            ..ON => 0,
            ON..=A_EMPTY => (tick - ON + 1) * DRAIN,
            _ => CAPACITY,
        };
        if sb == A && tick <= A_EMPTY {
            assert_eq!(
                (tank.used, tank.on),
                (expected_a, (ON..A_EMPTY).contains(&tick)),
                "tick {tick}: A"
            );
        }
        if sb == B && (ON..B_OFF).contains(&tick) {
            assert_eq!((tank.used, tank.on), ((tick - ON + 1) * DRAIN, true), "tick {tick}: B");
        }
        if sb == B && (B_OFF..B_IN).contains(&tick) {
            assert_eq!((tank.used, tank.on), ((B_OFF - ON) * DRAIN, false), "tick {tick}: B");
        }
        // In the network from the tick it goes in, filling 5 a second until it is full.
        let before = seen.tanks.get(&sb).copied().unwrap_or_default();
        if seen.entered.contains_key(&sb) {
            assert_eq!(
                tank.used,
                before.used.saturating_sub(REFILL),
                "tick {tick}: {sb:?} filling"
            );
            if before.used > 0 {
                filling += 1;
            }
        }
        seen.tanks.insert(sb, tank);
    }
    if filling == 2 {
        seen.both_filling += 1;
    }
    for o in sim.objects() {
        if o.buffs.iter().any(|buff| buff.ability == BuffKind::Sludge) {
            seen.slowed.entry(o.id).or_default().push(tick);
            if o.kind == PK {
                let moved = o.pos.distance(o.prev_pos);
                assert!(moved <= SLOWED_PK + 1e-3, "tick {tick}: {:?} moved {moved}", o.id);
            }
        }
        if o.kind == PK && (o.pos.y - SOUTH).abs() < 1.0 && !seen.arrived.contains_key(&o.id) {
            seen.arrived.insert(o.id, tick);
        }
    }
}
