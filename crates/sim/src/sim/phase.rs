//! The tick pipeline (`docs/architecture/tick-pipeline.md > Tick pipeline`): [`Sim::step`] runs the phases of a tick in their fixed
//! order, and [`Phase`] names them, for measurements ([`Sim::step_observed`]).

use super::Sim;
use crate::augmentation::Research;
use crate::command::TickInputs;
use crate::economy::defeat;
use crate::ids::ObjectId;
use crate::{buffs, combat, economy, energy, movement, orders, patches, paths, shields};

/// The parts of a tick, in the order [`Sim::step`](super::Sim::step) runs them (`docs/architecture/tick-pipeline.md > Tick pipeline`):
/// [`Sim::step_observed`](super::Sim::step_observed) marks each as it starts, then [`Phase::End`] when the tick is
/// done, so the time from one mark to the next is that phase's. Only measurements look at them (slice M2-J2,
/// `tests/structures_perf.rs`).
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phase {
    /// The tick's bookkeeping: the events cleared, (M5-X1b) the tick's rules (`sim/rules.rs`), the positions kept for
    /// interpolation.
    Start,
    /// The defeat check's first walk: who owns structures as the tick starts (`economy::defeat`).
    DefeatOwners,
    /// The players' commands, through the gate.
    Commands,
    /// The modes (the Riot Shield, the Weapon Conversion) and the behaviours (5a).
    Behaviours,
    /// The true grids follow the footprints (5b).
    NavSync,
    /// Path serving (5c).
    PathServing,
    /// Movement (5d).
    Movement,
    /// Combat (5e).
    Combat,
    /// The dead leave the game and the true grids follow their footprints (5e).
    Deaths,
    /// The pop-up turrets rise and retract, and the true grids follow their footprints (slice M4-S2, `sim/popups.rs`).
    PopUps,
    /// The Shields regenerate (`shields`, slice M3-B).
    Shields,
    /// (M5-N4a) The energy pools regenerate (`energy`).
    Energy,
    /// (M5-N5) The buffs work, and those whose time is up end (`buffs`).
    Buffs,
    /// The economy: construction, mining, the workers, the transformations, then the producers (plan decision T5).
    Economy,
    /// The defeat check's second walk, which ends the economy phase.
    Defeat,
    /// Vision (5f), the supply stations, the records of what each player sees and the rally points that follow them.
    Vision,
    /// Path knowledge: what each player has seen of the map's resources, and its planning grids.
    PathKnowledge,
    /// The tick is done.
    End,
}

impl Phase {
    /// Every phase, in order, [`Phase::End`] last.
    pub const ALL: [Phase; 18] = [
        Phase::Start,
        Phase::DefeatOwners,
        Phase::Commands,
        Phase::Behaviours,
        Phase::NavSync,
        Phase::PathServing,
        Phase::Movement,
        Phase::Combat,
        Phase::Deaths,
        Phase::PopUps,
        Phase::Shields,
        Phase::Energy,
        Phase::Buffs,
        Phase::Economy,
        Phase::Defeat,
        Phase::Vision,
        Phase::PathKnowledge,
        Phase::End,
    ];
}

impl Sim {
    /// Advance exactly one tick. Phase order is fixed and documented in ARCHITECTURE.md.
    pub fn step(&mut self, inputs: &TickInputs) {
        self.step_observed(inputs, |_| {});
    }

    /// [`Sim::step`], calling `observe` as each [`Phase`] of the tick starts and with [`Phase::End`] when it is done,
    /// for a measurement to time them (`tests/structures_perf.rs`). The sim does the same work either way and never
    /// reads anything back, so an observer can't change the game.
    pub fn step_observed(&mut self, inputs: &TickInputs, mut observe: impl FnMut(Phase)) {
        observe(Phase::Start);
        self.events.clear();
        // (M5-X1b) The rules of the whole tick (`sim/rules.rs`): those of the last tick's SetRules, every object
        // following its new maxima, or the game's, made the current ones whatever another game stepped on before.
        self.start_rules();
        self.nav.begin_tick();
        self.tick += 1;
        for o in self.objects.values_mut() {
            o.prev_pos = o.pos;
            o.prev_facing = o.facing;
        }
        // `docs/architecture/tick-pipeline.md > Tick pipeline`: objects that commands create during this tick are first updated in
        // the next one. Ids only grow, so they are exactly the ids from here on.
        let first_new = ObjectId(self.next_id);
        // The defeat check compares with the structures each player owns as the tick starts (`economy::defeat`).
        observe(Phase::DefeatOwners);
        let owned_before = defeat::structure_owners(&self.objects, &self.players, self.outcome.as_ref());
        observe(Phase::Commands);
        for cmd in &inputs.commands {
            self.apply_command(cmd);
        }
        observe(Phase::Behaviours);
        // Modes (`sim/modes.rs`): after the commands, so that a toggle counts from its own tick; before 5a, so that
        // the behaviours, the movement and the combat of the tick all see the mode it leaves.
        self.step_modes(first_new);
        // (M6-GA) The vehicle deploys under way take a frame, and those done complete (`sim/vehicle_deploy.rs`): after
        // the Modes step, as T44 has it; before 5a, so that a unit that has deployed acts as its new type from here on,
        // its held order first.
        self.step_vehicle_deploys(first_new);
        // The scheduled choppers that leave now get their orders (`sim/supply_run.rs`): before 5a, so that
        // they lift off in it, as for a command's order.
        self.step_schedules(first_new);
        // Phases 5a–5f, `docs/movement.md > 4`.
        // 5a: after the commands, so an order acts in its own tick; before movement, so goals have one writer.
        // The behaviours read the positions and the vision the last tick left (5f).
        let mut reach = self.nav.take_reach();
        let world = orders::world::World::new(
            &self.map,
            &self.vision,
            &self.nav,
            &self.objects,
            self.tick,
            self.auto_targeting,
        )
        .with_knowledge(&self.path_knowledge, self.nav_version)
        .with_research(Research::new(&self.players))
        .with_occupants(&combat::Threats::of(&self.objects, &self.held));
        let channelling = orders::update(&mut self.objects, first_new, &world, &mut reach);
        self.nav.put_reach(reach);
        // The workers that got to their sites in 5a deploy or start their sites, or their orders fail
        // (`sim/builds.rs`); before 5b claims the footprints.
        self.step_deployments(first_new);
        // (M5-E) The units that work at a footprint from a spot beside it get their spots and walk there
        // (`sim/spot_work.rs`): after the arrivals, which may have started their site; before 5c serves their paths.
        self.step_spots(first_new);
        // The units that walked into the entry path of the structure being entered in 5a leave the map for it (the
        // network, a building's pool), and the Enter orders that can't go on end (`sim/network.rs`). Here, as the
        // deployments: before 5b to 5f, so that no grid, collision, shot or sight of this tick includes a unit that
        // has gone in.
        self.step_entries(first_new);
        // The choppers attach to their towers, evicting the others there (`sim/supply_run.rs`): before the landings,
        // which fly and land them by the supply loop's rules on its spots.
        self.step_supply_runs(first_new);
        // (M6-GH2) The landings under way take a frame, and every aircraft flying to a spot, on a structure or on open
        // ground, flies on to it and lands (`sim/landing.rs`, `crate::landing`): after 5a, which starts the flights and
        // lifts the aircraft off; before 5b to 5f, so that the tick's movement flies each to its spot and its combat
        // sees each as it is left.
        self.step_landings(first_new);
        // The Attacks 5a found a unit can't carry out, its target too close (design change 0182), fail.
        self.step_too_close(first_new);
        // (M6-GD) The Launches 5a started launch their drones (`sim/drones.rs`, `crate::tether`): before 5b, so that a
        // drone has its grid from its first tick; it moves and attacks from the next one, as every new object.
        self.step_launches(first_new);
        // (M5-N4b) The clouds follow the channels as 5a leaves them (`crate::shroud`, plan decision T77): a channel that
        // started in it protects in this tick's 5e; (M6-CM) one that ended lingers, and a circle whose linger is over
        // vanishes, its cloud's buffer released there.
        self.clouds.update(channelling, self.tick, &mut self.events);
        // 5b: the movement below must collide with this tick's obstacles (the true grids), and every unit that came
        // into the game or was queued since the last tick has its grid from here on (slice M5-Z2, `paths`).
        observe(Phase::NavSync);
        paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
        self.cover_grids();
        // 5c: after the goals, before the movement that follows the paths. Each unit plans on what its owner knew
        // at the end of the last tick (`paths::knowledge`).
        observe(Phase::PathServing);
        paths::serve(
            &mut self.nav,
            &self.map,
            &mut self.objects,
            first_new,
            &self.path_knowledge,
            self.nav_version,
            self.tick,
            &mut self.events,
        );
        // 5d: the one movement pipeline, the single place that moves units.
        observe(Phase::Movement);
        // (M6-SB) The units in sludge, where the last tick left them, are slowed from this tick on
        // (`patches::slow`): before they move, by top speeds that their slow lowers.
        patches::slow(&mut self.objects, &self.patches, first_new, self.tick);
        let (nav, scratch) = self.nav.for_movement();
        let crushed = movement::step(
            &mut self.objects,
            first_new,
            &self.map,
            nav,
            scratch,
            self.tick,
            Research::new(&self.players),
            &mut self.events,
        );
        // (M5-V4) The crushed die in 5d, before 5e (`movement::crush`): they leave through the removal path, the grids
        // follow as after 5e's deaths, and a Disciple's Shroud ends with its death (`shroud::Clouds::settle`), its
        // circle lingering from there (M6-CM; with no linger, its buffer released in this tick's 5e).
        if !crushed.is_empty() {
            self.remove_objects(&crushed);
            paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
            self.clouds
                .settle(|id| self.objects.get(&id), self.tick, &mut self.events);
        }
        // (M6-GD) The drones whose end comes fall (`sim/drones.rs`, `crate::tether`): after 5d, which moved them past
        // their leash; before 5e, so that a drone that falls attacks nothing in the tick, and the blast of one past its
        // leash lands in it.
        self.step_tethers(first_new);
        // (M6-SB) The patches whose time is over go, and the Shadebringers with a toggle on lay their trails where 5d
        // left them and drain (`patches::lay`): after the crushed left, who lay nothing; before 5f, which shows them.
        patches::lay(&mut self.objects, &mut self.patches, first_new, self.tick);
        // 5e: shoots from the positions movement committed, and the projectiles due land (`combat::projectiles`). The
        // dead leave through the removal path; a footprint they leave behind no longer blocks, so the grids follow as
        // in 5b.
        // It reads the vision of the end of the last tick (5f below), as the design's ValidTarget asks, and its hits
        // reveal the attackers their victims' owners don't see (design change 0177, `vision::reveal`).
        // A unit turns at most its TurnRate per tick in all: the aim has what 5d's turn left (`movement::Turns`).
        observe(Phase::Combat);
        let turns = self.nav.turns();
        let mut emptied = Vec::new();
        let dead = combat::step(
            &mut self.objects,
            &mut self.held,
            &self.map,
            &mut self.vision,
            &mut self.projectiles,
            turns,
            first_new,
            self.tick,
            self.auto_targeting,
            Research::new(&self.players),
            &mut self.clouds,
            &mut emptied,
            &mut self.events,
        );
        observe(Phase::Deaths);
        if !dead.is_empty() {
            self.remove_objects(&dead);
            paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
        }
        // (M6-GH1) The aircraft whose last round went in 5e fly home by themselves (`sim/aircraft.rs`): after the
        // removal of the dead, which unassigns an Airfield's aircraft as it falls.
        if !emptied.is_empty() {
            self.send_home(&emptied);
        }
        // The pop-up turrets (slices M4-S2 and M5-Y, `sim/popups.rs`): after 5e, where each picked its target, and its
        // removals, so that a turret whose Tunnel is lost in this tick is down at its end; after 5d, so that the
        // footprint-clear test reads where the units stand as the tick ends; before 5f, which sees a turret from the
        // tick it starts to rise.
        observe(Phase::PopUps);
        self.step_popups(first_new);
        // The Shields regenerate (plan decision T18): after 5e's hits and the removal of the dead, so that the dead
        // never do; before the economy, whose transformations then raise them with their maxima.
        observe(Phase::Shields);
        shields::step(&mut self.objects, first_new);
        // (M5-N4a) The energy pools regenerate (plan decision T75), right after the Shields and for the same reasons:
        // the dead never do, and the units the economy brings into the game start at their start in their first tick
        // (a Disciple trained as it leaves its Cathedral, `Sim::step_holding`).
        observe(Phase::Energy);
        energy::step(&mut self.objects, &mut self.held, first_new);
        // (M5-N5) The buffs work (plan decision T75), beside the Shields and the energy and for the same reasons: the
        // dead never heal. After the energy, which reads a buff's bonus in the buff's last tick too.
        observe(Phase::Buffs);
        buffs::step(
            &mut self.objects,
            &mut self.held,
            Research::new(&self.players),
            first_new,
            self.tick,
        );
        // The economy (plan decision T5): after 5e, so that the buildings destroyed this tick count for nothing in
        // it; before 5f, so that what it brings into the game is seen in its own tick.
        observe(Phase::Economy);
        let depleted = economy::step(&mut self.objects, &mut self.players, first_new, &mut self.events);
        // `core/map.md > SpaceCrystalsPatch`: the patches the workers emptied disappear, and their tiles open
        // (design change 0038), so the grids follow as after 5e's deaths.
        if !depleted.is_empty() {
            self.remove_objects(&depleted);
            paths::sync(&mut self.nav, &self.objects, &mut self.nav_version);
        }
        // (M5-N8) The Tunnel expansions regenerate (`sim/expansions.rs`): before the sites, the upgrades and the decay,
        // so that one completed or taken back by a Tunnel in the tick regenerates from the next one on, and keeps the
        // damage it was built with, or that its decay left it, in this one.
        self.regenerate_expansions();
        // The transformations (slice M3-E, `crate::transform`), part of the economy phase after construction (plan
        // decision T5's position): here, because completing one turns its functions on from the sim's objects.
        self.step_transforms(first_new);
        // The sites (`sim/sites.rs`), beside the transformations and for the same reason: a site completed in the tick
        // is a Tunnel in operation for what follows (its Agent in the network, for the network's step).
        self.step_sites(first_new);
        // (M5-N8) The repairs (`sim/repairs.rs`), beside the sites, whose builders they are counted as: after 5e, so
        // that the HP an object lost in the tick is what its repairers restore; a site completed in the tick has no
        // repairers yet.
        self.step_repairs();
        // The Tunnels' upgrades (slice M4-G, `sim/upgrades.rs`), beside the sites and for the same reason: a Tunnel
        // upgraded in the tick is at its new tier for what follows (its Tunnel Space, the network's transit).
        self.step_upgrades(first_new);
        // The neutral expansions decay (slice M4-H, `sim/tunnels.rs`): after the sites and the upgrades, so that one a
        // Tunnel finished over in this tick is its owner's again first.
        self.step_decay();
        // The producers (slice M2-D2), part of the economy phase: here, not inside `economy::step`, because a
        // finished unit is inserted into the game. Before 5f, so that it is seen in its own tick.
        self.step_production(first_new);
        // (M6-GD) The stocks' purchases (`sim/drones.rs`, `crate::stock`), beside the producers: a unit's stock holds
        // the item from the tick its frames are done.
        self.step_stocks(first_new);
        // The Tunnel Network (`sim/network.rs`), part of the economy phase: the units inside regenerate, then each
        // Tunnel's eject queue lets its first unit out onto the exit path if that is clear. After the producers, so
        // that a unit a Hideout finished in this tick with a surface rally comes out in it; after 5d, so that a unit
        // never moves in the tick it appears; before 5f, so that it is seen in it.
        self.step_network();
        // The buildings that hold units (slice M5-F1, `sim/holding.rs`): each lets its next unit ready to leave into its
        // exit doorway if the doorway is free: one Eject All took out, one a Release lets out, or one trained as it
        // comes out (slice M5-F2, `sim/producers/pool.rs`). After the producers, so that an entry finished in this
        // tick lets its unit out in it, as a producer's finished unit, and for the same reasons: after 5d, before 5f.
        self.step_holding();
        // (M6-ST1) The carriers on an Unload All that 5d brought to rest put their passengers on the map round them
        // (`sim/holding/carriers.rs`, `crate::carry`), beside the buildings and for the same reasons.
        self.step_unloads(first_new);
        // The defeat check ends the economy phase (plan decisions T5 and T8, slice M2-F2): after every way a
        // structure leaves the game; before 5f, so that the units a defeat makes neutral are no one's eyes in its
        // own tick.
        observe(Phase::Defeat);
        defeat::step(
            &mut self.objects,
            &mut self.players,
            &mut self.outcome,
            &owned_before,
            self.tick,
            &mut self.events,
        );
        // A defeated player's network goes with their last structure (`sim/network.rs`).
        self.clear_defeated_networks();
        // (M5-N4b) The clouds of the channels that ended since 5a, by a death in 5e or a defeat's cleared orders
        // (`shroud::Clouds::settle`; (M5-V4) a crush in 5d settled them before 5e): after every way a channel ends;
        // (M6-CM) their circles linger from there, and those whose linger is over vanish, their buffers waiting for the
        // next tick's 5e.
        self.clouds
            .settle(|id| self.objects.get(&id), self.tick, &mut self.events);
        // 5f: sees the tick's final positions, this tick's new objects included; the fog delays and reveals that end
        // in this tick end (`vision::linger`, `vision::reveal`).
        observe(Phase::Vision);
        self.vision.update(&self.map, self.objects.values(), self.tick);
        self.update_supply_stations(first_new);
        // Last: the records of what each player sees keep the tick's final state, the deliveries included.
        self.vision.remember(self.tick, |id| self.objects.get(&id));
        // Rally points on enemies follow them only while their owner sees them (design change 0088): on the
        // vision this tick leaves.
        self.follow_targets();
        // Path knowledge, after everything it follows from (the objects, the vision and its records): what each
        // player has seen by now of the map's resources, then its grids for the next tick's plans, rebuilt at most
        // once per player and tick. Nothing after this may change the objects or the vision.
        observe(Phase::PathKnowledge);
        self.path_knowledge.observe(&self.vision);
        // Every unit the tick brought has its grid, in the true grids and each player's (slice M5-Z2).
        self.cover_grids();
        paths::sync_known(&mut self.nav, &self.objects, &self.vision, &mut self.path_knowledge);
        debug_assert!(
            self.nav.matches(&self.objects),
            "a blocking footprint changed after phase 5b: sync the grids again (`paths::sync`)"
        );
        observe(Phase::End);
    }
}
