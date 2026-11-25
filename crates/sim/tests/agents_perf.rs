//! Timings of whole ticks with a Syndicate economy at work and its Tunnel Network busy (slice M4-M,
//! `artifacts/director/m4-plan.md > M: M4 QA`): 4 Tunnels on 2 shared fields, 4 Agents gathering for each, and 60
//! Guards cycling through Enter and Eject at one Tunnel; then the same with 40 Grenadiers shelling a Tunnel field.
//! Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test agents_perf -- --ignored --nocapture --test-threads=1
//! ```
//!
//! On a flat 60 × 32 map player 1 has two fields side by side, each the stacked layout of the Syndicate economy check
//! (`factions/syndicate/units/agent.md > Gathering > Space Crystals > Syndicate economy check`,
//! `testing::syndicate::stacked_field`): two patches between two Tier 1 Tunnels whose crystal hatches face them, 3
//! grid units away (design change 0294), and 4 Agents for each Tunnel, sent to its patch in tick 1. The 60 Guards stand
//! north of the second field's western Tunnel and go in by its Side A in tick 1 (`systems/controls/commands.md >
//! Enter`). Each tick that a Guard inside is free to leave, the Tunnel ejects one, the network's longest inside
//! (`factions/syndicate/tunnels.md > EjectMenu`). A Guard that came out walks off at once, as a player would clear the
//! exit, round to the entry's side and in again (16 ticks after a walk in failed, it tries again). So the Guards go
//! round all along: the line at the entry path, the transit delay inside, the eject queue, the exit spot.
//!
//! The first part times that alone. Then 40 of player 2's Grenadiers appear south of the first field, each already in
//! Range of its point on the middle line of a lane between a hatch and its patch, where the Agents walk, and attack the
//! ground there (`systems/controls/commands.md > AttackGround`; they keep firing, design change 0293): a grenade about
//! every tick, each blast weighed against every object (`sc_sim::combat::area`). The points lie a grid unit or more
//! from the Tunnels, so that the blasts never reach them; an Agent the blasts kill is replaced by a debug spawn north
//! of the field and sent to its patch, so that the field keeps its 8 Agents at work. The second part times that.
//!
//! It prints, for each part, the mean, the 99th percentile and the worst whole tick, with what went on in it. Timings
//! are printed, never asserted. New in M4-M, so its figures are absolute ones: compare them by A/B runs.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind, PlayerId, Rotation,
    SU_PER_GU, Scenario, SimEvent, SpawnAt, SpawnSettings, Target, Tick, TilePos, TunnelCommand, Vec2,
};

const WIDTH: usize = 60;
const HEIGHT: usize = 32;
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The fields' offsets in x, tiles. The first is the one the Grenadiers shell.
const FIELDS: [i32; 2] = [0, 30];
/// Their offset in y, tiles: room north of the Tunnels for the Guards.
const OY: i32 = 12;
/// Grid units from each crystal hatch to its patch, edge to edge.
const DISTANCE: i32 = 3;
const AGENTS: u32 = 4;
const GUARDS: u32 = 60;
const GRENADIERS: u32 = 40;
/// The Tunnel the Guards cycle through: the second field's western one ([`scenario`]'s fifth spawn).
const GUARD_TUNNEL: ObjectId = ObjectId(5);
/// The first field's patches, western and eastern.
const SHELLED_PATCHES: [ObjectId; 2] = [ObjectId(2), ObjectId(3)];
/// Ticks each part is timed: a minute of game.
const PART: Tick = 1024;
/// A Guard whose walk in failed is sent in again this many ticks later.
const RETRY: Tick = 16;

/// Wall-clock time of `work`. Only perf tests read a clock: they measure the sim from outside, and nothing in the
/// sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn gu(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// The two fields (ids 1 to 8: each field's western Tunnel, its two patches, its eastern Tunnel, which is flipped left
/// and right so that its hatch faces west), then each field's Agents (ids 9 to 24: the western Tunnel's 4, the
/// eastern's 4, in rows south of the field), then the Guards (ids 25 to 84, 10 by 6, north of [`GUARD_TUNNEL`]).
fn scenario() -> Scenario {
    let map = vec![".".repeat(WIDTH); HEIGHT].join("\n");
    let mut builder = ScenarioBuilder::new(&map)
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate);
    let patch = || SpawnSettings::CrystalPatch {
        starting_amount: 1_000_000,
    };
    for ox in FIELDS {
        builder = builder
            .spawn(ObjectKind::Tunnel, Some(P1), tile(6 + ox, 6 + OY))
            .spawn_with(patch(), tile(10 + DISTANCE + ox, 8 + OY))
            .spawn_with(patch(), tile(12 + DISTANCE + ox, 8 + OY))
            .spawn(ObjectKind::Tunnel, Some(P1), tile(13 + 2 * DISTANCE + ox, 6 + OY));
    }
    for ox in FIELDS {
        for (from, step) in [(10 + DISTANCE + ox, -1), (12 + DISTANCE + ox, 1)] {
            for i in 0..AGENTS as i32 {
                let at = SpawnAt::Point(TilePos::new(from + step * i, 13 + OY).center());
                builder = builder.spawn(ObjectKind::Agent, Some(P1), at);
            }
        }
    }
    for i in 0..GUARDS {
        let at = gu(34.0 + 0.75 * (i % 10) as f32, 10.0 + 0.75 * (i / 10) as f32);
        builder = builder.spawn(ObjectKind::Guard, Some(P1), SpawnAt::Point(at));
    }
    let mut scenario = builder.build();
    for eastern in [3, 7] {
        scenario.map.spawns[eastern].orientation = BuildingOrientation {
            rotation: Rotation::R0,
            flip_h: true,
            flip_v: false,
        };
    }
    scenario
}

/// Where Grenadier `i` stands, and the point it attacks: 20 for each of the first field's lanes, the western one's
/// for even `i`, in three rows of 7 south of the lane. Its point is the one of the lane's middle line, from a grid unit
/// off the hatch's Tunnel to the patch (x 11 to 12.5 and 16.5 to 18), nearest to it: within its Range of 4 grid units.
fn grenadier(i: u32) -> (Vec2, Vec2) {
    let (lane, k) = (i % 2, i / 2);
    let x = [11.75, 17.25][lane as usize] - 2.25 + 0.75 * (k % 7) as f32;
    let at = gu(x, (10 + OY) as f32 + 0.75 + 0.75 * (k / 7) as f32);
    let from = [11.0, 16.5][lane as usize];
    let aim = from + (((x - from) / 0.375).round().clamp(0.0, 4.0)) * 0.375;
    (at, gu(aim, (8 + OY) as f32 + 0.5))
}

/// The way round of a Guard that came out, by a count: a spot in a block north-east of [`GUARD_TUNNEL`]'s Side A,
/// where its exit path's spot is, then one in a block north-west, where its entry path's is. So the Guards walking
/// away from the exit don't cross those walking up to the entry.
fn way_round(n: u32) -> [Vec2; 2] {
    let (dx, dy) = (0.75 * (n % 7) as f32, 0.75 * (n / 7 % 5) as f32);
    [gu(41.0 + dx, 10.5 + dy), gu(30.5 + dx, 10.5 + dy)]
}

/// What the test keeps up between ticks: the Guards to send away and in again, and the first field's Agents by patch.
struct Loop {
    away: BTreeMap<Tick, Vec<ObjectId>>,
    back_in: BTreeMap<Tick, Vec<ObjectId>>,
    waited: u32,
    shelled: [Vec<ObjectId>; 2],
    agents: BTreeSet<ObjectId>,
    /// How many of [`TestSim::events`] have been read.
    read: usize,
    replaced: u32,
}

/// What a part saw.
#[derive(Default)]
struct Part {
    samples: Vec<Duration>,
    entered: u32,
    left: u32,
    failed: u32,
    delivered: u32,
    launched: u32,
    blasts: u32,
    agents_died: u32,
    others_died: u32,
}

impl Loop {
    fn new(t: &TestSim) -> Self {
        let agents: BTreeSet<ObjectId> = (t.sim().objects())
            .filter(|o| o.kind == ObjectKind::Agent)
            .map(|o| o.id)
            .collect();
        let first: Vec<ObjectId> = agents.iter().copied().take(2 * AGENTS as usize).collect();
        Self {
            away: BTreeMap::new(),
            back_in: BTreeMap::new(),
            waited: 0,
            shelled: [first[..AGENTS as usize].to_vec(), first[AGENTS as usize..].to_vec()],
            agents,
            read: 0,
            replaced: 0,
        }
    }

    /// The commands of the tick after the last step, then that step, timed.
    fn tick(&mut self, t: &mut TestSim, part: &mut Part) {
        let next = t.sim().tick() + 1;
        for (tick, e) in &t.events()[self.read..] {
            match e {
                SimEvent::EnteredNetwork { .. } => part.entered += 1,
                SimEvent::LeftNetwork { unit, .. } => {
                    part.left += 1;
                    self.away.entry(tick + 1).or_default().push(*unit);
                }
                SimEvent::EnterFailed { unit, .. } => {
                    part.failed += 1;
                    self.back_in.entry(tick + RETRY).or_default().push(*unit);
                }
                SimEvent::ResourcesDelivered { amount, .. } => part.delivered += amount,
                SimEvent::ProjectileLaunched { .. } => part.launched += 1,
                SimEvent::Blast { .. } => part.blasts += 1,
                SimEvent::Died { kind, .. } => match kind {
                    ObjectKind::Agent => part.agents_died += 1,
                    _ => part.others_died += 1,
                },
                _ => {}
            }
        }
        self.read = t.events().len();
        // The Guards that just came out walk round to the entry and go in again (queued orders, as with Shift); those
        // whose walk in failed try again from the north-west.
        let walk = |to: Vec2| OrderKind::Move(Target::Point(to));
        let enter = OrderKind::Enter(GUARD_TUNNEL);
        for unit in self.away.remove(&next).unwrap_or_default() {
            let [east, west] = way_round(self.waited);
            self.waited += 1;
            t.order_at(next, P1, &[unit], walk(east), false)
                .order_at(next, P1, &[unit], walk(west), true)
                .order_at(next, P1, &[unit], enter, true);
        }
        for unit in self.back_in.remove(&next).unwrap_or_default() {
            if t.sim().object(unit).is_some() {
                let [_, west] = way_round(self.waited);
                self.waited += 1;
                t.order_at(next, P1, &[unit], walk(west), false)
                    .order_at(next, P1, &[unit], enter, true);
            }
        }
        // One more Guard out, if one is free to leave.
        if t.sim().next_out(P1, ObjectKind::Guard).is_ok() {
            let eject = Command::Tunnel {
                tunnel: GUARD_TUNNEL,
                command: TunnelCommand::Eject {
                    kind: ObjectKind::Guard,
                },
            };
            t.at(next, P1, eject);
        }
        self.replace(t, next);
        let ((), took) = timed(|| {
            t.step();
        });
        part.samples.push(took);
    }

    /// The first field keeps its Agents: a new Agent (a replacement that just appeared) is sent to the patch of the
    /// lane it is north of, and each one the blasts killed is replaced north of its patch.
    fn replace(&mut self, t: &mut TestSim, next: Tick) {
        let sim = t.sim();
        let new: Vec<(ObjectId, f32)> = (sim.objects())
            .filter(|o| o.kind == ObjectKind::Agent && !self.agents.contains(&o.id))
            .map(|o| (o.id, o.pos.x))
            .collect();
        for (id, x) in new {
            self.agents.insert(id);
            self.replaced += 1;
            let lane = usize::from(x > 14.5 * SU_PER_GU);
            self.shelled[lane].push(id);
            t.order_at(next, P1, &[id], OrderKind::Gather(SHELLED_PATCHES[lane]), false);
        }
        // A spot taken (by the replacement before, not yet off it) refuses the spawn: it comes a tick later.
        for lane in 0..2 {
            self.shelled[lane].retain(|id| t.sim().object(*id).is_some());
            for i in self.shelled[lane].len() as u32..AGENTS {
                let x = [13.0 - i as f32, 15.0 + i as f32][lane] + 0.5;
                let spawn = DebugCommand::Spawn {
                    kind: ObjectKind::Agent,
                    owner: Some(P1),
                    at: SpawnAt::Point(gu(x, (3 + OY) as f32 + 0.5)),
                    orientation: Default::default(),
                };
                t.debug_at(next, spawn);
            }
        }
    }
}

fn report(name: &str, part: &mut Part) {
    let ticks = part.samples.len();
    let total: Duration = part.samples.iter().sum();
    part.samples.sort_unstable();
    let p99 = part.samples[ticks * 99 / 100];
    let worst = part.samples[ticks - 1];
    println!(
        "{name}: {ticks} ticks: mean {:.3} ms, p99 {:.3} ms, worst {:.3} ms per whole sim tick; {} Guards went in, {} \
         came out, {} walks in failed; {} Space Crystals delivered; {} grenades thrown, {} blasts; {} Agents and {} \
         others died",
        total.as_secs_f64() * 1e3 / ticks as f64,
        p99.as_secs_f64() * 1e3,
        worst.as_secs_f64() * 1e3,
        part.entered,
        part.left,
        part.failed,
        part.delivered,
        part.launched,
        part.blasts,
        part.agents_died,
        part.others_died,
    );
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_4_tunnels_16_agents_60_guards_cycling_then_40_grenadiers() {
    let mut t = TestSim::new(scenario());
    t.check_invariants(false);
    for field in 0..2 {
        for side in 0..2 {
            let first = 9 + (2 * field + side) * AGENTS;
            let agents: Vec<ObjectId> = (first..first + AGENTS).map(ObjectId).collect();
            let patch = ObjectId(2 + 4 * field + side);
            t.order_at(1, P1, &agents, OrderKind::Gather(patch), false);
        }
    }
    let guards: Vec<ObjectId> = (25..25 + GUARDS).map(ObjectId).collect();
    t.order_at(1, P1, &guards, OrderKind::Enter(GUARD_TUNNEL), false);
    let mut state = Loop::new(&t);

    let mut network = Part::default();
    for _ in 0..PART {
        state.tick(&mut t, &mut network);
    }
    let held = t.sim().network(P1).count();
    report("network", &mut network);

    // The Grenadiers, south of the first field, out of the Guards' sight; each attacks its point from the next tick.
    let tick = t.sim().tick() + 1;
    for i in 0..GRENADIERS {
        t.spawn_unit_at(tick, ObjectKind::Grenadier, P2, grenadier(i).0);
    }
    let mut shelled = Part::default();
    state.tick(&mut t, &mut shelled);
    let grenadiers: Vec<ObjectId> = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Grenadier)
        .map(|o| o.id)
        .collect();
    assert_eq!(grenadiers.len(), GRENADIERS as usize);
    let tick = t.sim().tick() + 1;
    for (i, id) in grenadiers.iter().enumerate() {
        let target = grenadier(i as u32).1;
        t.order_at(tick, P2, &[*id], OrderKind::AttackGround(target), false);
    }
    for _ in 1..PART {
        state.tick(&mut t, &mut shelled);
    }
    report("shelled", &mut shelled);
    let standing = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Tunnel && o.hp == o.max_hp())
        .count();
    println!(
        "{held} Guards inside after the first part; {} Agents replaced; {standing} of the 4 Tunnels unhurt",
        state.replaced
    );
    // The shelling never reached a Tunnel, nor a Grenadier left its point, and the Guards kept going round.
    assert_eq!(standing, 4);
    assert!(network.left > GUARDS && shelled.left > GUARDS, "the Guards stopped");
}
