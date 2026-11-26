//! A soak of the Syndicate against the GDO (slice M4-M, `artifacts/director/m4-plan.md > M: M4 QA`): the golden game
//! `syndicate_vs_gdo`'s players (`sc_sim::testing::syndicate::skirmish`'s module docs) played for [`MINUTES`] minutes
//! of `dev_skirmish` from its real start, each through the gate with their own commands, as players would. The golden
//! game ends at 3:07, player 1's two Tunnels lost, so the soak's players do more, and fight over a front all game:
//! - **Player 1's line** (`Syndicate`): the golden's opening (four more Agents, the second Tunnel at [`TUNNEL_SITE`] by
//!   the [`BUILDER`]th, who then gathers Supplies, and the first Tunnel's Optics Lab); then two pop-up Laser Turrets in
//!   the forward Tunnel's area ([`POPUPS`]) and a Chemical Plant in the first's ([`CHEMICAL_PLANT_SITE`]), the first
//!   Tunnel's upgrade to Tier 2, and [`GUARDS`] Guards, [`GRENADIERS`] Grenadiers and [`RESERVES`] more Guards behind
//!   their prerequisites. (Slice M4-S1's upright Laser Turret, which an Agent built at the front, outside every Tunnel
//!   Area, went with design changes 0359 and 0360: only a Tunnel builds a turret, inside its area.)
//! - **The front** is the foot of the plateau's west ramp, below the forward Tunnel. Its units go in and out by the
//!   forward Tunnel while it is in operation, else by the first. The first [`GUARDS_OUT`] Guards out hold
//!   [`GUARD_POST`]; one below half its MaxHP walks back in, heals in the network (`network::Held::regenerate`), and
//!   comes out again once it has three quarters, as the next one out, so that the network takes units in and lets them
//!   out all game. The Grenadiers come out as they are made and stand at [`GRENADIER_POST`]; while a GDO unit is within
//!   [`SHELL_REACH`] of it, every other one Attack Grounds where the nearest stands, and the others pick their own
//!   targets.
//! - **Player 2** plays the golden's GDO with [`PEACEKEEPERS`] Peacekeepers instead of its 16, which attack-move to the
//!   front in waves of [`WAVE`], from [`FIRST_WAVE`] on, one every [`WAVE_GAP`]: the front stays where it is when the
//!   forward Tunnel falls, and player 1's first Tunnel is never a target, so the game goes on.
//!
//! It checks, after every tick:
//! - `TestSim`'s invariants: the movement invariants, V1, K1, O1, and P1, "every projectile lands within its flight
//!   time" (`testing::invariants`, the brief's acceptance);
//! - the network's ([`check_network`]): no held unit on the map, nor named there, and every held unit idle; each
//!   unit held at a site builds it and each site's builder is held there; each eject queue holds its owner's units in
//!   the network; no two Tunnel Areas overlap (the reserved ones: an upgrade's future area included); each owned
//!   expansion, a Laser Turret among them, lies in one of its owner's Tunnel Areas, and only a neutral expansion decays;
//!   every Laser Turret has its pop-up state, lowered while it isn't active (`Sim::is_active_turret`, design change
//!   0360);
//! - that nothing names a removed object (`testing::dangling_references`);
//! - that the gate takes every command the players give, and that the books balance: both players' Space Crystals
//!   (`testing::build_order::Books`; the GDO's only while no patch under its plates could run out in the step) and
//!   player 1's Supplies (`testing::syndicate::SupplyBooks`);
//! - that nothing is stuck ([`Watch`]): every order that ends by itself ends within [`ORDER_TICKS`]; an Agent on a
//!   Gather goes from one step of its job to the next within [`STEP_TICKS`], bar waiting; it waits only at a patch
//!   another Agent mines, at an empty station, or for at most [`RECHECK_TICKS`];
//!
//! at the end, that the game went on all along with what it is meant to have in it (grenades at a target and at the
//! ground, blasts, pop-ups rising and firing, units in and out of the network, deaths on either side); and over the
//! whole game, that it is deterministic: a second run gives the same hash at every checkpoint, and the first run's
//! replay gives its last hash.
//!
//! Ignored: two games of 9 600 ticks.
//! `cargo test -p sc_sim --release --test sim syndicate_soak -- --ignored --nocapture`.

use std::collections::{BTreeMap, BTreeSet};
use std::mem::{Discriminant, discriminant};

use sc_sim::network::Place;
use sc_sim::orders::state::GatherPhase;
use sc_sim::orders::{ActiveOrder, Behaviour, OrderGroup};
use sc_sim::popup::PopUp;
use sc_sim::production::ProductionCommand;
use sc_sim::testing::build_order::{Books, BuildOrder, Order};
use sc_sim::testing::syndicate::skirmish::{
    self, AGENTS, BUILDER, GDO, GDO_STAGING, GUARDS, GUARDS_OUT, OPTICS_LAB_SITE, PATCH, STATION, SYNDICATE,
    TUNNEL_SITE, TURNED,
};
use sc_sim::testing::syndicate::{Done, Job, SupplyBooks, Syndicate};
use sc_sim::testing::{self, TestSim};
use sc_sim::tunnel::{Resource, area};
use sc_sim::units::TilePos;
use sc_sim::{
    Command, ConstructionCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Reject,
    ResourceState, SU_PER_GU, Sim, SimEvent, TICKS_PER_SECOND, Target, Tick, TunnelCommand, Vec2,
};

use ObjectKind::{
    Agent, Barracks, ChemicalPlant, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Grenadier, Guard,
    Hideout, LaserTurret, OpticsLab, Peacekeeper, PeacekeeperShielded, SupplyDeliveryStation, Tunnel,
};

/// How long the game runs.
const MINUTES: u32 = 10;
const END: Tick = MINUTES * 60 * TICKS_PER_SECOND;
/// A hash is noted at every multiple of this tick.
const CHECKPOINT: Tick = 64;
/// The longest an order that ends by itself may run: the time a Peacekeeper (4 su a tick, the slowest unit here)
/// takes to walk twice round `dev_skirmish`'s 64 × 48 tiles of 64 su, far more than the longest way across it and the
/// fights on the way.
const ORDER_TICKS: Tick = 2 * (64 + 48) * 64 / 4;
/// The longest an Agent may take over one step of a Gather but waiting: a leg to a patch, the station or a hatch, its
/// MiningDuration or DropOffDuration (64 frames each), or its wait at the crystal hatch behind the others (design
/// change 0255: "the first to arrive delivers next"), two Agents at most ahead of it.
const STEP_TICKS: Tick = 30 * TICKS_PER_SECOND;
/// A waiting Agent looks round again every 16 ticks (`tuning::gathering::RECHECK_TICKS`).
const RECHECK_TICKS: Tick = 16;

// The front (module docs). The plateau's west ramp comes down through the gap in its cliffs at (26, 22) and (26, 23),
// and its slope reaches down to (24, 20)–(25, 25). The Peacekeepers come over the plateau and down the ramp, and the
// defence waits below, on flat ground, out of sight of its top (a Peacekeeper sees 5 grid units): there it reaches the
// units coming down all at once. A defence within sight of the top stops them up there, where little of it reaches.

/// The pop-up turrets' tiles, in the forward Tunnel's Tier 1 area ((18, 14) to (25, 21)): one north of the ramp's
/// foot, one east of the Tunnel's Side A doorway, clear of the doorway (south) and of the square the Agents deliver
/// Supplies from (east, (24, 17)).
const POPUPS: [TilePos; 2] = [TilePos::new(24, 19), TilePos::new(23, 21)];
/// The Chemical Plant's top-left tile, in the south-west corner of the first Tunnel's area ((4, 4) to (11, 11)), off
/// the Agents' way to the field (north-west) and the Tunnel's doorway (south).
const CHEMICAL_PLANT_SITE: TilePos = TilePos::new(4, 10);
/// Where the Guards stand: west of the ramp's foot (the golden game's hold its top, `skirmish::GUARD_POST`).
const GUARD_POST: TilePos = TilePos::new(22, 22);
/// The Guards that replace the fallen, trained after the Grenadiers, and kept in the network until needed.
const RESERVES: usize = 6;
/// The Grenadiers, behind the Chemical Plant.
const GRENADIERS: usize = 8;
/// Where they stand: west of the ramp's foot, east of the Guards.
const GRENADIER_POST: TilePos = TilePos::new(23, 23);
/// The Grenadiers shell the GDO unit nearest their post within this of it, space units: one step past their Range of
/// 4 grid units.
const SHELL_REACH: f32 = 5.0 * SU_PER_GU;
/// And aim again at the nearest every this many ticks, while they shell.
const REAIM_TICKS: Tick = 32;
/// Player 2's Peacekeepers, and its waves: [`WAVE`] at a time, the first no sooner than [`FIRST_WAVE`], and each next
/// one no sooner than [`WAVE_GAP`] after the one before. The golden game's waves are of 6 (`skirmish::WAVE`), and
/// set off as soon as there are that many, two a minute, which no defence of the line's holds.
const PEACEKEEPERS: usize = 40;
const WAVE: usize = 3;
const FIRST_WAVE: Tick = 3 * 60 * TICKS_PER_SECOND;
const WAVE_GAP: Tick = 40 * TICKS_PER_SECOND;

/// The soak's players (module docs).
struct Players {
    syndicate: Syndicate,
    gdo: BuildOrder,
    /// Whether the Hideout's rally point was cleared for the Guards.
    cleared: bool,
    /// Whether player 2's Barracks has its rally point.
    rallied: bool,
    /// Player 2's Peacekeepers sent with a wave.
    sent: BTreeSet<ObjectId>,
    /// The tick player 2's latest wave set off in.
    last_wave: Option<Tick>,
}

impl Players {
    /// The players, with `more` Agents trained beyond the line's [`AGENTS`], all on crystals.
    fn new(more: usize) -> Self {
        let gdo = BuildOrder::new(GDO).track(
            [
                Order::new(ExtractionFacility, ExtractionPlate),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Generator),
                Order::new(DeploymentCenter, Barracks),
            ]
            .into_iter()
            .chain([Order::new(Barracks, Peacekeeper); PEACEKEEPERS]),
        );
        let line = BuildOrder::new(SYNDICATE).track(vec![Order::new(Hideout, Agent); AGENTS + more]);
        let syndicate = Syndicate::new(line)
            .gather(PATCH)
            .rally(Target::Object(PATCH))
            .job(BUILDER, Job::Tunnel(TUNNEL_SITE, TURNED))
            .eject_builders(Job::Supplies(STATION))
            .expand(0, OpticsLab, OPTICS_LAB_SITE)
            .expand(1, LaserTurret, POPUPS[0])
            .expand(0, ChemicalPlant, CHEMICAL_PLANT_SITE)
            .expand(1, LaserTurret, POPUPS[1])
            .upgrade(0)
            .after(OpticsLab, vec![Order::new(Hideout, Guard); GUARDS])
            .after(ChemicalPlant, vec![Order::new(Hideout, Grenadier); GRENADIERS])
            .after(OpticsLab, vec![Order::new(Hideout, Guard); RESERVES]);
        Self {
            syndicate,
            gdo,
            cleared: false,
            rallied: false,
            sent: BTreeSet::new(),
            last_wave: None,
        }
    }

    /// What `player`'s commands of the step to tick `tick` paid in Space Crystals.
    fn spent_in(&self, player: PlayerId, tick: Tick) -> u32 {
        if player == SYNDICATE {
            return self.syndicate.spent_in(tick);
        }
        (self.gdo.issued().iter())
            .filter(|i| i.tick == tick)
            .map(|i| i.cost)
            .sum()
    }

    /// Both players' commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs).
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let plays = |player| sim.outcome().is_none() && sim.player(player).is_some_and(|p| !p.is_defeated());
        let mut commands = self.syndicate.commands(sim);
        if plays(SYNDICATE) {
            // The Tunnel the front's units go in and out by: the forward one while it is in operation, else the first.
            let tunnels = || {
                sim.objects()
                    .filter(|o| o.owner == Some(SYNDICATE) && o.kind == Tunnel && o.is_operational())
            };
            let forward = tunnels().find(|o| o.placement.is_some_and(|p| p.origin == TUNNEL_SITE));
            let exit = forward.or_else(|| tunnels().next()).map(|o| o.id);
            self.guards(sim, tick, exit, &mut commands);
            grenadiers(sim, tick, exit, &mut commands);
        }
        commands.extend(self.gdo.commands(sim));
        if plays(GDO) {
            self.peacekeepers(sim, &mut commands);
        }
        commands
    }

    /// Player 1's Guards (module docs): the Hideout's rally point cleared as the first is ordered, so that they stay
    /// in the network; [`GUARDS_OUT`] of them out of `exit`, one ejected a tick, each sent to [`GUARD_POST`] once out;
    /// a hurt one back in, and out again once healed.
    fn guards(&mut self, sim: &Sim, tick: Tick, exit: Option<ObjectId>, commands: &mut Vec<PlayerCommand>) {
        let command = |command| PlayerCommand {
            player: SYNDICATE,
            command,
        };
        let first = (self.syndicate.done().iter()).find_map(|&(at, done)| match done {
            Done::Train { by, order } if at == tick && order.kind == Guard => Some(by),
            _ => None,
        });
        if !self.cleared
            && let Some(hideout) = first
        {
            // The producer itself as the target clears its rally point (design change 0086).
            commands.push(command(Command::Production {
                objects: vec![hideout],
                command: ProductionCommand::SetRally(Target::Object(hideout)),
            }));
            self.cleared = true;
        }
        let out = left_network(sim, Guard);
        if !out.is_empty() {
            commands.push(command(Command::Order {
                units: out.clone(),
                kind: OrderKind::AttackMove(GUARD_POST.center()),
                queue: false,
            }));
        }
        let Some(exit) = exit else { return };
        let guards = || sim.objects().filter(|o| o.owner == Some(SYNDICATE) && o.kind == Guard);
        let entering = |o: &Object| current(o).is_some_and(|c| matches!(c.order.kind, OrderKind::Enter(_)));
        let hurt = |o: &Object| o.hp.unwrap_or(0.0) * 2.0 < max_hp(o.kind);
        let back: Vec<ObjectId> = guards()
            .filter(|o| hurt(o) && !entering(o) && !out.contains(&o.id))
            .map(|o| o.id)
            .collect();
        let holding = guards().filter(|o| !hurt(o) && !entering(o)).count();
        if !back.is_empty() {
            commands.push(command(Command::Order {
                units: back,
                kind: OrderKind::Enter(exit),
                queue: false,
            }));
        }
        let queued = queued_out(sim, exit, Guard);
        let healed = |id| {
            sim.held_unit(id)
                .is_some_and(|h| h.object.hp.unwrap_or(0.0) * 4.0 >= 3.0 * max_hp(Guard))
        };
        if holding + queued < GUARDS_OUT
            && let Ok(next) = sim.next_out(SYNDICATE, Guard)
            && healed(next)
        {
            commands.push(command(Command::Tunnel {
                tunnel: exit,
                command: TunnelCommand::Eject { kind: Guard },
            }));
        }
    }

    /// Player 2's Peacekeepers: the Barracks rallied to [`GDO_STAGING`], as in the golden game; a wave of the [`WAVE`]
    /// idle ones first trained that never set off, once the wave is due (module docs), attack-moving to the centre
    /// of the forward Tunnel's footprint, whether the Tunnel stands or not: the front, which the Syndicate's units hold
    /// all game. The golden game's waves go to player 1's Tunnel nearest the staging point, and once the forward one
    /// has fallen, on to the first, which ends the game.
    fn peacekeepers(&mut self, sim: &Sim, commands: &mut Vec<PlayerCommand>) {
        let command = |command| PlayerCommand { player: GDO, command };
        let own = || sim.objects().filter(|o| o.owner == Some(GDO));
        if !self.rallied
            && let Some(barracks) = own().find(|o| o.kind == Barracks)
        {
            commands.push(command(Command::Production {
                objects: vec![barracks.id],
                command: ProductionCommand::SetRally(Target::Point(GDO_STAGING.center())),
            }));
            self.rallied = true;
        }
        let idle = |o: &&Object| {
            matches!(o.kind, Peacekeeper | PeacekeeperShielded)
                && o.orders.as_ref().is_none_or(|orders| orders.is_idle())
        };
        let waiting: Vec<ObjectId> = own()
            .filter(idle)
            .filter(|o| !self.sent.contains(&o.id))
            .map(|o| o.id)
            .collect();
        let tick = sim.tick() + 1;
        let due = self.last_wave.map_or(FIRST_WAVE, |last| last + WAVE_GAP);
        if waiting.len() < WAVE || tick < due {
            return;
        }
        let going = waiting[..WAVE].to_vec();
        self.sent.extend(&going);
        self.last_wave = Some(tick);
        let front = Vec2::new(TUNNEL_SITE.x as f32 + 2.0, TUNNEL_SITE.y as f32 + 2.0) * SU_PER_GU;
        commands.push(command(Command::Order {
            units: going,
            kind: OrderKind::AttackMove(front),
            queue: false,
        }));
    }
}

/// Player 1's Grenadiers (module docs): one ejected a tick by `exit` while any is in the network, each sent to
/// [`GRENADIER_POST`] once out; while a GDO unit is within [`SHELL_REACH`] of the post, every other one out (by id) Attack
/// Grounds where the nearest stands, aiming again every [`REAIM_TICKS`], and goes back to its post once none is.
fn grenadiers(sim: &Sim, tick: Tick, exit: Option<ObjectId>, commands: &mut Vec<PlayerCommand>) {
    let command = |command| PlayerCommand {
        player: SYNDICATE,
        command,
    };
    let post = GRENADIER_POST.center();
    let out = left_network(sim, Grenadier);
    if !out.is_empty() {
        commands.push(command(Command::Order {
            units: out.clone(),
            kind: OrderKind::AttackMove(post),
            queue: false,
        }));
    }
    if let Some(exit) = exit
        && sim.next_out(SYNDICATE, Grenadier).is_ok()
    {
        commands.push(command(Command::Tunnel {
            tunnel: exit,
            command: TunnelCommand::Eject { kind: Grenadier },
        }));
    }
    let nearest = (sim.objects())
        .filter(|o| o.owner == Some(GDO) && o.mover.is_some() && o.pos.distance(post) <= SHELL_REACH)
        .min_by(|a, b| (a.pos.distance_squared(post)).total_cmp(&b.pos.distance_squared(post)));
    let shelling = |o: &Object| current(o).is_some_and(|c| matches!(c.order.kind, OrderKind::AttackGround(_)));
    // Every other one shells; the others pick their own targets (`systems/controls/behaviors.md > BaseAutoTargeting`).
    let grenadiers = (sim.objects())
        .filter(|o| o.owner == Some(SYNDICATE) && o.kind == Grenadier && o.id.0 % 2 == 0 && !out.contains(&o.id));
    let (units, kind): (Vec<ObjectId>, _) = match nearest {
        Some(enemy) => (
            grenadiers
                .filter(|o| !shelling(o) || tick.is_multiple_of(REAIM_TICKS))
                .map(|o| o.id)
                .collect(),
            OrderKind::AttackGround(enemy.pos),
        ),
        None => (
            grenadiers.filter(|o| shelling(o)).map(|o| o.id).collect(),
            OrderKind::AttackMove(post),
        ),
    };
    if !units.is_empty() {
        commands.push(command(Command::Order {
            units,
            kind,
            queue: false,
        }));
    }
}

/// Player 1's units of `kind` that came out of the network in the latest tick.
fn left_network(sim: &Sim, kind: ObjectKind) -> Vec<ObjectId> {
    (sim.events().iter())
        .filter_map(|e| match *e {
            SimEvent::LeftNetwork { unit, .. } => Some(unit),
            _ => None,
        })
        .filter(|&unit| {
            sim.object(unit)
                .is_some_and(|o| o.owner == Some(SYNDICATE) && o.kind == kind)
        })
        .collect()
}

/// How many units of `kind` Tunnel `tunnel`'s eject queue holds.
fn queued_out(sim: &Sim, tunnel: ObjectId, kind: ObjectKind) -> usize {
    let queue = sim
        .object(tunnel)
        .and_then(|o| o.tunnel.as_ref())
        .map(|t| t.eject.as_slice());
    (queue.unwrap_or_default().iter())
        .filter(|entry| sim.held_unit(entry.unit).is_some_and(|h| h.object.kind == kind))
        .count()
}

fn max_hp(kind: ObjectKind) -> f32 {
    kind.def().max_hp.unwrap_or(0.0)
}

/// A unit's current order.
fn current(o: &Object) -> Option<&ActiveOrder> {
    o.orders.as_ref()?.current.as_ref()
}

/// The network's invariants (module docs), which `TestSim` doesn't check.
fn check_network(sim: &Sim) {
    let tick = sim.tick();
    for held in sim.held() {
        let unit = held.object.id;
        assert!(
            sim.object(unit).is_none(),
            "tick {tick}: held unit {unit:?} is on the map too"
        );
        let idle = held.object.orders.as_ref().is_none_or(|orders| orders.is_idle());
        assert!(idle, "tick {tick}: held unit {unit:?} has orders");
        for o in sim.objects() {
            let named = testing::referenced_ids(o).contains(&unit);
            assert!(!named, "tick {tick}: {:?} names held unit {unit:?}", o.id);
        }
        if let Place::Site(site) = held.place {
            let o = (sim.object(site)).unwrap_or_else(|| panic!("tick {tick}: {unit:?} is held at {site:?}, gone"));
            assert_eq!(o.owner, held.object.owner, "tick {tick}: {unit:?} is held at {site:?}");
            let builder = o.construction.and_then(|c| c.builder.unit());
            assert_eq!(
                builder,
                Some(unit),
                "tick {tick}: {unit:?} is held at {site:?}, which it doesn't build"
            );
        }
    }
    let tunnels: Vec<&Object> = sim.objects().filter(|o| o.kind == Tunnel).collect();
    for o in sim.objects() {
        if let Some(worker) = o.construction.and_then(|c| c.builder.unit()) {
            let held = sim.held_unit(worker);
            assert!(
                held.is_some_and(|h| h.place == Place::Site(o.id) && h.object.owner == o.owner),
                "tick {tick}: site {:?}'s builder {worker:?} isn't held in it",
                o.id
            );
        }
        for entry in o.tunnel.iter().flat_map(|t| &t.eject) {
            let held = sim.held_unit(entry.unit);
            assert!(
                held.is_some_and(|h| h.place == Place::Network && h.object.owner == o.owner),
                "tick {tick}: {:?}'s eject queue holds {:?}, not its owner's in the network",
                o.id,
                entry.unit
            );
        }
        let expansion = sc_sim::tunnel::is_expansion(o.kind);
        assert_eq!(
            o.decay.is_some(),
            expansion && o.owner.is_none(),
            "tick {tick}: {:?}, a {:?}: only a neutral expansion decays",
            o.id,
            o.kind
        );
        if expansion && o.owner.is_some() {
            let parent = area::parent(o, tunnels.iter().copied());
            assert!(
                parent.is_some(),
                "tick {tick}: expansion {:?} lies in none of its owner's areas",
                o.id
            );
        }
        if o.kind == LaserTurret {
            // Every turret has its state from its first tick, and one that isn't active (a site, a neutral one) is
            // down: an inactive turret is lowered at once (`sc_sim::popup`).
            let active = sim.is_active_turret(o);
            let ok = o.popup.is_some() && (active || o.popup == Some(PopUp::LOWERED));
            assert!(ok, "tick {tick}: turret {:?} (active: {active}) is {:?}", o.id, o.popup);
        }
    }
    for (i, a) in tunnels.iter().enumerate() {
        for b in &tunnels[i + 1..] {
            let (Some(ra), Some(rb)) = (area::reserved(a), area::reserved(b)) else {
                continue;
            };
            assert!(
                !area::overlaps(ra, rb),
                "tick {tick}: Tunnels {:?} and {:?}' areas overlap",
                a.id,
                b.id
            );
        }
    }
}

/// What a unit is doing, to see when it changes: its current order (its group and kind), and for a Gather the step
/// of its job. A Gather that goes on to another patch changes its kind (the order follows the worker from patch to
/// patch, `orders::gather`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Doing {
    group: OrderGroup,
    kind: OrderKind,
    step: Option<Discriminant<GatherPhase>>,
}

impl Doing {
    fn of(current: &ActiveOrder) -> Self {
        let step = match &current.state {
            Behaviour::Gather(gather) => Some(discriminant(&gather.phase)),
            _ => None,
        };
        Self {
            group: current.order.group,
            kind: current.order.kind,
            step,
        }
    }
}

/// A readable name for what a unit does, and the longest it may do it.
fn label(sim: &Sim, current: &ActiveOrder) -> (String, Tick) {
    if let Behaviour::Gather(gather) = &current.state {
        let step = format!("{:?}", gather.phase);
        let step = step.split([' ', '{']).next().unwrap_or_default().to_string();
        let order = match current.order.kind {
            OrderKind::DropOff(_) => "Drop Off",
            _ => "Gather",
        };
        // Waiting lasts while the patches are busy or the station empty ([`Watch::note`] checks that they are).
        let bound = match gather.phase {
            GatherPhase::Waiting { .. } => Tick::MAX,
            _ => STEP_TICKS,
        };
        return (format!("{order}, {step}"), bound);
    }
    let kind = current.order.kind;
    // Hold and Patrol last until the next order, and so does a Move to a unit, which follows it, and an Attack Ground,
    // which keeps firing (design change 0293).
    let bound = match kind {
        OrderKind::Hold | OrderKind::Patrol(_) | OrderKind::AttackGround(_) => Tick::MAX,
        OrderKind::Move(Target::Object(id)) if sim.object(id).is_some_and(|o| o.mover.is_some()) => Tick::MAX,
        _ => ORDER_TICKS,
    };
    let name = format!("{kind:?}");
    (
        name.split(['(', ' ', '{']).next().unwrap_or_default().to_string(),
        bound,
    )
}

/// What the soak watches, tick by tick (module docs).
#[derive(Default)]
struct Watch {
    /// Whether the checks that nothing is stuck are off: the variants with more Agents than the game is played with
    /// ([`more_crystal_agents_never_stop_the_first_tunnel`]), whose Agents wait longer for their turn at the hatch.
    lenient: bool,
    /// What each unit is doing, and since when.
    doing: BTreeMap<ObjectId, (Doing, Tick)>,
    /// Each Agent that waits at a patch nobody mines (or that is gone), or at a station with Supplies, since when.
    idle: BTreeMap<ObjectId, Tick>,
    /// The longest each order and each step of a Gather lasted, by name: how long, the unit, and the tick.
    longest: BTreeMap<String, (Tick, ObjectId, Tick)>,
    /// What happened, by name.
    counts: BTreeMap<&'static str, u32>,
    /// The turrets that rose, by id: each one's state as the latest tick left it.
    popups: BTreeMap<ObjectId, Option<Discriminant<PopUp>>>,
}

impl Watch {
    fn count(&mut self, what: &'static str, n: u32) {
        *self.counts.entry(what).or_default() += n;
    }

    /// Notes what the units do at the end of the tick, and checks that none is stuck.
    fn note(&mut self, sim: &Sim) {
        let tick = sim.tick();
        let stuck = |o: &Object, what: &str| {
            // What stands round it, within 3 grid units: id, kind and where, in grid units.
            let near: Vec<_> = (sim.objects())
                .filter(|a| a.id != o.id && a.pos.distance(o.pos) < 3.0 * SU_PER_GU)
                .map(|a| {
                    let doing = current(a).map(|c| (c.order.kind, c.since, c.state.clone()));
                    let moving = a.mover.as_ref().map(|m| (m.status, m.goal));
                    (a.id.0, a.kind, a.pos / SU_PER_GU, doing, moving)
                })
                .collect();
            format!(
                "tick {tick}: #{} ({:?} of player {:?}) is stuck: {what}: {o:?}; round it: {near:?}",
                o.id.0,
                o.kind,
                o.owner.map(|p| p.0)
            )
        };
        // The patches a worker mines.
        let mined: BTreeSet<ObjectId> = (sim.objects().filter_map(current))
            .filter_map(|c| match (c.order.kind, &c.state) {
                (OrderKind::Gather(patch), Behaviour::Gather(g)) if matches!(g.phase, GatherPhase::Mining { .. }) => {
                    Some(patch)
                }
                _ => None,
            })
            .collect();
        let empty_station = |id: ObjectId| {
            sim.object(id).is_some_and(|o| {
                o.kind == SupplyDeliveryStation
                    && matches!(
                        o.resource,
                        Some(ResourceState::SupplyStation {
                            current_supplies: 0,
                            ..
                        })
                    )
            })
        };
        let (mut doing, mut idle) = (BTreeMap::new(), BTreeMap::new());
        for o in sim.objects() {
            let Some(c) = current(o) else { continue };
            let now = Doing::of(c);
            let since = match self.doing.get(&o.id) {
                Some(&(was, since)) if was == now => since,
                _ => tick,
            };
            doing.insert(o.id, (now, since));
            let (name, bound) = label(sim, c);
            let age = tick - since;
            let longest = self.longest.entry(name.clone()).or_insert((0, o.id, tick));
            if age > longest.0 {
                *longest = (age, o.id, tick);
            }
            assert!(
                self.lenient || age <= bound,
                "{}",
                stuck(o, &format!("{name} for {age} ticks"))
            );
            let Behaviour::Gather(gather) = &c.state else { continue };
            // It waits at a patch nobody mines, or at a station with Supplies: it gathers in the next tick, or it
            // waits short of it and tries again at its next look (or looks round, if the patch is gone).
            if let (OrderKind::Gather(target), GatherPhase::Waiting { since: waiting }) = (c.order.kind, gather.phase)
                && !mined.contains(&target)
                && !empty_station(target)
            {
                let since = (self.idle.get(&o.id).copied().unwrap_or(tick)).max(waiting);
                idle.insert(o.id, since);
                let age = tick - since;
                assert!(
                    self.lenient || age <= RECHECK_TICKS,
                    "{}",
                    stuck(
                        o,
                        &format!("waiting at #{} for {age} ticks, which nobody mines", target.0)
                    )
                );
            }
        }
        (self.doing, self.idle) = (doing, idle);

        for o in sim.objects().filter(|o| o.kind == LaserTurret && !o.is_site()) {
            let state = o.popup.map(|state| discriminant(&state));
            let was = self.popups.insert(o.id, state).flatten();
            let rising = discriminant(&PopUp::Rising(sc_sim::Progress::new(1)));
            if state == Some(rising) && was != Some(rising) {
                self.count("pop-up rises", 1);
            }
        }
        for e in sim.events() {
            let what = match *e {
                SimEvent::ProjectileLaunched { target: None, .. } => "grenades at the ground",
                SimEvent::ProjectileLaunched { .. } => "grenades at a target",
                SimEvent::Blast { .. } => "blasts",
                SimEvent::Attacked { attacker, .. } => match sim.object(attacker) {
                    Some(o) if o.kind == LaserTurret => "pop-up shots",
                    _ => continue,
                },
                SimEvent::EnteredNetwork { .. } => "entered the network",
                SimEvent::LeftNetwork { .. } => "left the network",
                SimEvent::EnterFailed { .. } => "Enters failed",
                SimEvent::ExpansionCompleted { .. } => "expansions completed",
                SimEvent::UpgradeCompleted { .. } => "upgrades completed",
                SimEvent::SiteCompleted { .. } => "sites completed",
                SimEvent::Died {
                    owner: Some(SYNDICATE), ..
                } => "player 1's losses",
                SimEvent::Died { owner: Some(GDO), .. } => "player 2's losses",
                SimEvent::Died { owner: None, .. } => "neutral losses",
                SimEvent::ResourcesDelivered { amount, resource, .. } => {
                    let what = match resource {
                        Resource::Crystals => "Space Crystals delivered",
                        Resource::Supplies => "Supplies delivered",
                    };
                    self.count(what, amount);
                    continue;
                }
                _ => continue,
            };
            self.count(what, 1);
        }
    }
}

/// Whether `player`'s books can follow the step: no patch under one of its plates holds 10 or less, the most a
/// plate takes in a step, which the step could empty (`Books::close`).
fn followable(sim: &Sim, player: PlayerId) -> bool {
    let left = |patch| match sim.object(patch)?.resource {
        Some(ResourceState::CrystalPatch { remaining }) => Some(remaining),
        _ => None,
    };
    (sim.objects().filter(|o| o.owner == Some(player)))
        .filter_map(|o| left(o.extractor?.patch))
        .all(|left| left > 10)
}

/// One game, and what it came to.
struct Run {
    t: TestSim,
    hashes: Vec<(Tick, u64)>,
    watch: Watch,
    /// The tick from which the GDO's books were no longer kept, if they stopped.
    books_stopped: Option<Tick>,
}

/// The game, with `more` Agents on crystals than it is played with (4); the checks that nothing is stuck are for the
/// game as played.
fn play(more: usize) -> Run {
    let scenario = skirmish::scenario();
    let config = scenario.setup.config;
    let mut t = TestSim::with_config(scenario, config);
    let mut players = Players::new(more);
    let mut watch = Watch {
        lenient: more > 0,
        ..Watch::default()
    };
    let mut hashes = vec![(0, t.sim().state_hash())];
    let mut books_stopped = None;
    while t.sim().tick() < END {
        let tick = t.sim().tick() + 1;
        let commands = players.commands(t.sim());
        // The gate takes every command, as the ones before it in the step leave the game; this check sees the game
        // before the step, so a constructor placing its ready building may take its next Construct in the same step
        // (as in the golden games).
        let placing: Vec<ObjectId> = (commands.iter())
            .filter_map(|c| match &c.command {
                Command::Construction {
                    objects,
                    command: ConstructionCommand::Place { .. },
                } => Some(objects.clone()),
                _ => None,
            })
            .flatten()
            .collect();
        for c in &commands {
            match t.sim().validate(c) {
                Ok(()) => {}
                Err(Reject::ConstructorBusy(id)) if placing.contains(&id) => {}
                Err(reject) => panic!("tick {tick}: the gate refuses {c:?}: {reject:?}"),
            }
        }
        let mut books = Vec::new();
        for player in [SYNDICATE, GDO] {
            if followable(t.sim(), player) {
                books.push((Books::open(t.sim(), player), players.spent_in(player, tick)));
            } else {
                books_stopped.get_or_insert(tick);
            }
        }
        let supplies = SupplyBooks::open(t.sim(), SYNDICATE);
        for c in commands {
            t.at(tick, c.player, c.command);
        }
        t.step();
        for (books, spent) in books {
            books.close(t.sim(), spent);
        }
        supplies.close(t.sim(), players.syndicate.supplies_spent_in(tick));
        assert_eq!(testing::dangling_references(t.sim()), [], "tick {tick}");
        check_network(t.sim());
        watch.note(t.sim());
        if tick.is_multiple_of(CHECKPOINT) {
            hashes.push((tick, t.sim().state_hash()));
        }
    }
    Run {
        t,
        hashes,
        watch,
        books_stopped,
    }
}

/// The units and structures each player has, by kind, the network's included.
fn census(sim: &Sim) -> BTreeMap<(u8, ObjectKind), usize> {
    let mut counts = BTreeMap::new();
    for o in sim.objects().chain(sim.held().map(|held| &held.object)) {
        if let Some(owner) = o.owner {
            *counts.entry((owner.0, o.kind)).or_default() += 1;
        }
    }
    counts
}

#[test]
#[ignore = "a soak of two 10-minute games: run it in release"]
fn ten_minutes_of_the_syndicate_against_the_gdo() {
    let run = play(0);
    let (sim, w) = (run.t.sim(), &run.watch);
    let fallen: Vec<(Tick, ObjectKind, u32)> = (run.t.events().iter())
        .filter_map(|(tick, e)| match *e {
            SimEvent::Died { id, kind, owner, .. } if owner != Some(GDO) && kind.def().structure.is_some() => {
                Some((*tick, kind, id.0))
            }
            _ => None,
        })
        .collect();
    println!("at tick {}: {:?}", sim.tick(), census(sim));
    println!("player 1's structures that fell (tick, kind, id): {fallen:?}");
    println!(
        "banks {:?}, player 1's Supplies {}; the GDO's books kept to tick {:?}",
        [SYNDICATE, GDO].map(|p| sim.player(p).map_or(0, |p| p.crystals)),
        sim.player(SYNDICATE).map_or(0, |p| p.supplies),
        run.books_stopped
    );
    println!("what happened: {:?}", w.counts);
    println!("the longest each order or step of a Gather lasted (ticks, the unit, at tick):");
    for (name, (ticks, id, at)) in &w.longest {
        println!("  {name:<24} {ticks:>5} (#{} at tick {at})", id.0);
    }

    // The game went on all along, with grenades, the pop-up turrets and the network in it.
    assert!(sim.outcome().is_none(), "the game ended: {:?}", sim.outcome());
    let count = |what| w.counts.get(what).copied().unwrap_or(0);
    for what in [
        "grenades at a target",
        "grenades at the ground",
        "blasts",
        "pop-up rises",
        "pop-up shots",
        "entered the network",
        "left the network",
        "player 1's losses",
        "player 2's losses",
    ] {
        assert!(count(what) > 0, "no {what}");
    }

    // Deterministic: the same game again, and the replay of the first.
    let again = play(0);
    assert_eq!(again.hashes, run.hashes, "a second run");
    assert_eq!(run.t.replay().run().state_hash(), sim.state_hash(), "the replay");
}

/// The first Tunnel's crystal income of `run`: Space Crystals per second after the first minute, and the longest
/// stretch without a delivery (ticks; from the start, between two, and to the end).
fn first_tunnel_income(run: &Run) -> (f32, Tick) {
    let mut tunnel = None;
    let mut loads: Vec<(Tick, u32)> = Vec::new();
    for (tick, e) in run.t.events() {
        if let SimEvent::ResourcesDelivered { drop_off, amount, .. } = *e
            && *tunnel.get_or_insert(drop_off) == drop_off
        {
            loads.push((*tick, amount));
        }
    }
    let minute = 60 * TICKS_PER_SECOND;
    let amount: u32 = loads.iter().filter(|load| load.0 > minute).map(|load| load.1).sum();
    let ticks: Vec<Tick> = [0]
        .into_iter()
        .chain(loads.iter().map(|load| load.0))
        .chain([END])
        .collect();
    let longest = ticks.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap_or(END);
    (amount as f32 * TICKS_PER_SECOND as f32 / (END - minute) as f32, longest)
}

/// Design change 0324's Verification: "The `dev_skirmish` soak, with 4 and 6 Agents: no Tunnel stops for good." The
/// game with 4 Agents on crystals (as played), 5 and 6 (one and two more trained): the first Tunnel never goes a
/// minute without a delivery. It prints the soak's rows of `docs/playtest/m4.md > Known issues`. With more Agents than
/// played, the soak's own stuck checks are off: six Agents at one hatch wait longer for their turn than they allow.
///
/// Ignored: three games of 9 600 ticks.
/// `cargo test -p sc_sim --release --test sim syndicate_soak -- --ignored --nocapture`.
#[test]
#[ignore = "a soak of three 10-minute games: run it in release"]
fn more_crystal_agents_never_stop_the_first_tunnel() {
    for more in 0..=2 {
        let run = play(more);
        let (income, longest) = first_tunnel_income(&run);
        println!(
            "{} Agents on crystals: {income:.1} Space Crystals per second after the first minute, longest gap {longest} ticks",
            4 + more
        );
        assert!(
            longest < 60 * TICKS_PER_SECOND,
            "{} Agents: no delivery for {longest} ticks",
            4 + more
        );
    }
}
