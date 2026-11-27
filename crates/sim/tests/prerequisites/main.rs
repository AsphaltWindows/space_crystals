//! The Syndicate's prerequisites in play (slice M4-K; design changes 0229, 0260 and 0264;
//! `core/tech_and_research.md > Tech Prerequisites`), through the gate, with the design's own types: the Optics Lab
//! and the Chemical Plant built at a Tunnel (`factions/syndicate/structures/optics_lab.md`,
//! `factions/syndicate/structures/chemical_plant.md`), the Hideout's Guard behind the Optics Lab
//! (`factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: "W: Build Guard [...] Only available
//! if the player owns at least one Optics Lab") and its Grenadier behind the Chemical Plant ("E: Build Grenadier [...]
//! Only available if the player owns at least one Chemical Plant", slice M4-U), what losing either does, which labs
//! count, and what the other player sees of either: nothing. The rule's cases that need a requirement no type of the
//! design has yet (a GDO unit's and building's, a Colonists Warp In's, a worker's Build, an expansion's) are the sim's
//! own unit tests (`sc_sim::tech`'s), since only those can change a def.
//!
//! Every test runs under the invariants of `TestSim`.

use sc_sim::network::Place;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tunnel::Operation;
use sc_sim::vision::VisibilityState;
use sc_sim::{
    BuildingOrientation, Command, DebugCommand, Faction, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand,
    PlayerId, ProductionCommand, Reject, SimEvent, SpawnAt, Target, Tick, TilePos, TunnelCommand,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const AGENT: ObjectKind = ObjectKind::Agent;
const GUARD: ObjectKind = ObjectKind::Guard;
const HIDEOUT: ObjectKind = ObjectKind::Hideout;
const TUNNEL: ObjectKind = ObjectKind::Tunnel;
const OPTICS_LAB: ObjectKind = ObjectKind::OpticsLab;
const CHEMICAL_PLANT: ObjectKind = ObjectKind::ChemicalPlant;
const GRENADIER: ObjectKind = ObjectKind::Grenadier;

/// Player 1's Tunnel in [`base`], on tiles (4, 8) to (7, 11): its Tier 1 area is (2, 6) to (9, 13).
const T1: ObjectId = ObjectId(1);
/// Player 1's Hideout in [`base`], on tiles (8, 12) to (9, 13), in [`T1`]'s area.
const H1: ObjectId = ObjectId(2);
/// An expansion's place in [`T1`]'s area: its north-west corner, (2, 6) to (3, 7).
const NORTH_WEST: TilePos = TilePos::new(2, 6);
/// Another, its south-west corner, (2, 12) to (3, 13).
const SOUTH_WEST: TilePos = TilePos::new(2, 12);

/// `factions/syndicate/structures/hideout.md > Produces`: "Guard: 125 Space Crystals, 120 frames (7.5 seconds)".
const GUARD_COST: u32 = 125;
const GUARD_TICKS: u32 = 120;
/// `factions/syndicate/structures/hideout.md > Produces`: "Grenadier: 150 Space Crystals, 160 frames (10 seconds),
/// placeholder" (design change 0260).
const GRENADIER_COST: u32 = 150;
const GRENADIER_TICKS: u32 = 160;

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

/// The centre of tile `(x, y)`.
fn point(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(at(x, y).center())
}

/// Both players play the Syndicate on a flat 40 × 20 map: player 1's start, as a Starting Layout has it
/// (`factions/syndicate/tunnels.md > Tunnel > Starting Conditions`): the Tier 1 Tunnel [`T1`], the Hideout [`H1`] in
/// its area, and an Agent outside it (3); Tunnel Space 1/11.
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8))
        .spawn(HIDEOUT, Some(P1), tile(8, 12))
        .spawn(AGENT, Some(P1), point(14, 10))
}

/// `scenario` started with auto-targeting off, player 1 with `crystals` Space Crystals and 10 Supplies from tick 1,
/// which has run.
fn started(scenario: ScenarioBuilder, crystals: u32) -> TestSim {
    let mut t = scenario.start();
    t.without_auto_targeting();
    t.debug_at(1, banks(P1, crystals, 10));
    t.run_until(1);
    t
}

/// Sets `player`'s Space Crystals and Supplies, their other banks to 0.
fn banks(player: PlayerId, crystals: u32, supplies: u32) -> DebugCommand {
    DebugCommand::SetResources {
        player,
        crystals,
        supplies,
        alloys: 0,
        essence: 0,
        flux: 0,
    }
}

/// What the gate says to `player`'s `command` now (`Sim::validate`).
fn gate(t: &TestSim, player: PlayerId, command: Command) -> Result<(), Reject> {
    t.sim().validate(&PlayerCommand { player, command })
}

/// `player`'s `command` in the next tick, which runs.
fn next(t: &mut TestSim, player: PlayerId, command: Command) {
    let tick = t.sim().tick() + 1;
    t.at(tick, player, command);
    t.step();
}

/// A debug command in the next tick, which runs.
fn next_debug(t: &mut TestSim, command: DebugCommand) {
    next(t, P1, Command::Debug(command));
}

/// An Expand of a `kind` at `origin`, unturned, to [`T1`] (its ExpandMenu's entry, then a click).
fn expand(kind: ObjectKind, origin: TilePos) -> Command {
    let command = TunnelCommand::Expand {
        kind,
        origin,
        orientation: BuildingOrientation::default(),
    };
    Command::Tunnel { tunnel: T1, command }
}

/// A Train of a Guard at `hideout` (its W).
fn train_guard(hideout: ObjectId) -> Command {
    Command::Production {
        objects: vec![hideout],
        command: ProductionCommand::Train(GUARD),
    }
}

/// A Train of a Grenadier at `hideout` (its E).
fn train_grenadier(hideout: ObjectId) -> Command {
    Command::Production {
        objects: vec![hideout],
        command: ProductionCommand::Train(GRENADIER),
    }
}

/// A debug removal of `id`: the removal path, as a destruction (`DebugCommand::Remove`).
fn remove(id: ObjectId) -> DebugCommand {
    DebugCommand::Remove { id }
}

fn object(t: &TestSim, id: ObjectId) -> &Object {
    t.sim().object(id).unwrap_or_else(|| panic!("{id:?} is on the map"))
}

/// Player 1's Space Crystals and Supplies.
fn bank(t: &TestSim) -> (u32, u32) {
    let player = t.sim().player(P1).expect("in the game");
    (player.crystals, player.supplies)
}

/// The site [`T1`] builds; panics if it builds none.
fn building(t: &TestSim) -> ObjectId {
    match t.sim().object(T1).and_then(|o| o.tunnel.as_ref()?.operation) {
        Some(Operation::Building(site)) => site,
        other => panic!("Tunnel {T1:?} builds nothing: {other:?}"),
    }
}

/// [`T1`]'s operation, if any.
fn operation(t: &TestSim) -> Option<Operation> {
    t.sim().object(T1)?.tunnel.as_ref()?.operation
}

/// Player 1's Tunnel Space, used and available.
fn tunnel_space(t: &TestSim) -> (u32, u32) {
    let cap = t.sim().army_cap(P1).expect("a Syndicate player");
    (cap.used, cap.available)
}

/// The Guards in player 1's Tunnel Network, where a Hideout releases its units
/// (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`).
fn guards_inside(t: &TestSim) -> usize {
    t.sim().network(P1).filter(|held| held.object.kind == GUARD).count()
}

/// The units [`H1`] produced so far: (tick, unit).
fn produced(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    let events = t.events().iter();
    let produced = |(tick, e): &(Tick, SimEvent)| match *e {
        SimEvent::UnitProduced { producer, unit } if producer == H1 => Some((*tick, unit)),
        _ => None,
    };
    events.filter_map(produced).collect()
}

/// The sites [`T1`] completed so far, in order.
fn completed(t: &TestSim) -> Vec<ObjectId> {
    let events = t.events().iter().map(|(_, e)| e);
    let completed = |e: &SimEvent| match *e {
        SimEvent::ExpansionCompleted { tunnel, site } if tunnel == T1 => Some(site),
        _ => None,
    };
    events.filter_map(completed).collect()
}

/// Design change 0229's verification, "Building an Optics Lab takes 25 s and then enables Guards at the Hideout"
/// (`factions/syndicate/structures/optics_lab.md`: "Cost - 150 Space Crystals (no Supplies)", "Build Time - 400 frames
/// (25 seconds)", "HP - 400"). Before it, the gate refuses the Hideout's Guard for want of one (`Reject::Requires`,
/// the client's "Requires an Optics Lab"). Ordered at the Tunnel in tick 2, it pays 150 Space Crystals and no Supply,
/// and stands as a site of 40 HP (the ConstructionHP Rule of design change 0254), which doesn't count
/// (`core/tech_and_research.md > Tech Prerequisites > Counting Rule`: "only counts toward a TechPrerequisite once it is
/// **fully constructed**"): the Guard is still refused at the end of tick 401. At the end of tick 402 the lab is
/// complete at 400 HP and the Tunnel free, and the Hideout takes the Guard, which it trains into the Tunnel Network in
/// its 120 frames for 125 Space Crystals.
#[test]
fn an_optics_lab_takes_400_ticks_then_the_hideout_trains_guards() {
    let mut t = started(base(), 1000);
    let refused = Err(Reject::Requires(OPTICS_LAB));
    assert_eq!(gate(&t, P1, train_guard(H1)), refused);
    next(&mut t, P1, expand(OPTICS_LAB, NORTH_WEST));
    assert_eq!((t.sim().tick(), bank(&t)), (2, (850, 10)));
    let lab = building(&t);
    let o = object(&t, lab);
    assert_eq!((o.kind, o.owner, o.is_site()), (OPTICS_LAB, Some(P1), true));
    assert_eq!((o.hp, o.max_hp()), (Some(40.0), Some(40.0)));
    t.run_until(401);
    assert!(object(&t, lab).is_site(), "a site until the end of tick 402");
    assert!(!t.sim().requirement_met(P1, OPTICS_LAB));
    assert_eq!(gate(&t, P1, train_guard(H1)), refused, "a site doesn't count");
    t.step();
    let o = object(&t, lab);
    assert_eq!((t.sim().tick(), o.is_site(), o.hp), (402, false, Some(400.0)));
    assert_eq!((operation(&t), completed(&t)), (None, vec![lab]));
    assert!(t.sim().requirement_met(P1, OPTICS_LAB));
    assert_eq!(gate(&t, P1, train_guard(H1)), Ok(()));
    next(&mut t, P1, train_guard(H1));
    assert_eq!(bank(&t), (850 - GUARD_COST, 10));
    t.run(GUARD_TICKS + 2);
    assert_eq!(guards_inside(&t), 1, "a Guard in the Tunnel Network");
}

/// `core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites` ("**Losing a Prerequisite**: units already
/// queued or in production continue and complete normally; new ones cannot be queued until the player again owns a
/// qualifying prerequisite structure"; "**Check Timing**: checked only when the unit is queued"): two Guards queued at
/// the Hideout, the first in production, and then the only Optics Lab destroyed. The Hideout refuses a third Guard,
/// and both queued ones are trained. A new Optics Lab, once complete, lets it take Guards again.
#[test]
fn guards_queued_before_the_lab_is_lost_are_trained() {
    let mut t = started(base().spawn(OPTICS_LAB, Some(P1), tile(2, 6)), 1000); // 4
    let lab = ObjectId(4);
    next(&mut t, P1, train_guard(H1));
    next(&mut t, P1, train_guard(H1));
    let queued = t.sim().tick();
    let p = object(&t, H1).production.clone().expect("a producer");
    assert_eq!((p.in_production(), p.queue.len()), (Some(GUARD), 2));
    next_debug(&mut t, remove(lab));
    assert!(!t.sim().requirement_met(P1, OPTICS_LAB));
    assert_eq!(gate(&t, P1, train_guard(H1)), Err(Reject::Requires(OPTICS_LAB)));
    t.run_until(queued + 2 * GUARD_TICKS + 2);
    assert_eq!(guards_inside(&t), 2, "both queued Guards trained");
    assert_eq!(bank(&t), (1000 - 2 * GUARD_COST, 10));
    // Again an Optics Lab of player 1's: Guards can be queued again once it is complete.
    next(&mut t, P1, expand(OPTICS_LAB, SOUTH_WEST));
    let again = building(&t);
    assert_eq!(gate(&t, P1, train_guard(H1)), Err(Reject::Requires(OPTICS_LAB)));
    while object(&t, again).is_site() {
        t.step();
    }
    assert_eq!(gate(&t, P1, train_guard(H1)), Ok(()));
}

/// The same when the Optics Lab is lost with its Tunnel (`factions/syndicate/tunnels.md > Losing Tunnels`: "every
/// expansion in its Tunnel Area becomes neutral"), a neutral structure being nobody's
/// (`core/tech_and_research.md > Tech Prerequisites > Counting Rule`): player 1's two Tunnels, the lab in the first's
/// area and the Hideout in the second's. The first Tunnel falls while a Guard is in production: the Hideout, still
/// player 1's, trains it, and refuses the next.
#[test]
fn a_lab_left_neutral_by_its_fallen_tunnel_no_longer_counts() {
    let scenario = ScenarioBuilder::new(&flat(40, 20))
        .faction(P1, Faction::Syndicate)
        .faction(P2, Faction::Syndicate)
        .spawn(TUNNEL, Some(P1), tile(4, 8)) // 1: area (2, 6) to (9, 13)
        .spawn(TUNNEL, Some(P1), tile(24, 8)) // 2: area (22, 6) to (29, 13)
        .spawn(HIDEOUT, Some(P1), tile(28, 12)) // 3
        .spawn(OPTICS_LAB, Some(P1), tile(2, 6)); // 4
    let (hideout, lab) = (ObjectId(3), ObjectId(4));
    let mut t = started(scenario, 1000);
    next(&mut t, P1, train_guard(hideout));
    let queued = t.sim().tick();
    next_debug(&mut t, remove(T1));
    assert_eq!((object(&t, lab).owner, object(&t, hideout).owner), (None, Some(P1)));
    assert_eq!(gate(&t, P1, train_guard(hideout)), Err(Reject::Requires(OPTICS_LAB)));
    t.run_until(queued + GUARD_TICKS + 2);
    assert_eq!(guards_inside(&t), 1, "the Guard in production trained");
}

/// Design change 0260 ("**Chemical Plant:** like the Optics Lab: 2x2, 150 SC, 400 frames, HP 400 [...] Groupable, no
/// commands"; `factions/syndicate/structures/chemical_plant.md`), the ExpandMenu's E: it is built as the Optics Lab
/// is, 150 Space Crystals and no Supply at the click, complete at the end of tick 402 at 400 HP, and from then on it
/// counts (for the Grenadier it enables: [`the_hideout_trains_grenadiers_once_a_chemical_plant_stands`]). Each
/// expansion takes a Tunnel Space for good (`factions/syndicate/tunnel_area_and_space.md > Tunnel Space`:
/// "Tunnel Space = floor((sum of unused tunnel-region grid units across all owned Tunnels) / 4)"; "Building an
/// expansion permanently converts part of a Tunnel's unused tunnel-region into occupied space"), from the click
/// (design change 0254): from the start's 1/11, 1/10 with the Chemical Plant, and 1/9 with an Optics Lab beside it.
#[test]
fn a_chemical_plant_is_built_alike_and_each_takes_a_tunnel_space() {
    let mut t = started(base(), 1000);
    assert_eq!(tunnel_space(&t), (1, 11));
    next(&mut t, P1, expand(CHEMICAL_PLANT, NORTH_WEST));
    assert_eq!((t.sim().tick(), bank(&t), tunnel_space(&t)), (2, (850, 10), (1, 10)));
    let plant = building(&t);
    let o = object(&t, plant);
    assert_eq!((o.kind, o.owner, o.is_site()), (CHEMICAL_PLANT, Some(P1), true));
    assert_eq!((o.hp, o.max_hp()), (Some(40.0), Some(40.0)));
    t.run_until(401);
    assert!(object(&t, plant).is_site() && !t.sim().requirement_met(P1, CHEMICAL_PLANT));
    t.step();
    let o = object(&t, plant);
    assert_eq!((o.is_site(), o.hp, completed(&t)), (false, Some(400.0), vec![plant]));
    assert!(t.sim().requirement_met(P1, CHEMICAL_PLANT));
    assert_eq!(tunnel_space(&t), (1, 10));
    next(&mut t, P1, expand(OPTICS_LAB, SOUTH_WEST));
    assert_eq!((bank(&t), tunnel_space(&t)), ((700, 10), (1, 9)));
}

/// `factions/syndicate/structures/hideout.md > ObjectInterfaceState[Hideout]`: "E: Build Grenadier: deducts 150 Space
/// Crystals (placeholder), adds Grenadier to BuildQueue. Only available if the player owns at least one Chemical
/// Plant" (`factions/syndicate/structures/hideout.md > Produces`: "Grenadier: 150 Space Crystals, 160 frames (10
/// seconds), placeholder"; design change 0260, slice M4-U). Without one the gate refuses the Grenadier for want of it (`Reject::Requires`, the client's "Requires a
/// Chemical Plant"), and while it is a site too (`core/tech_and_research.md > Tech Prerequisites > Counting Rule`).
/// Ordered at the Tunnel in tick 2, the plant is complete at the end of tick 402, and the Hideout takes the Grenadier:
/// 150 Space Crystals at the click, and 160 ticks later the Grenadier is in the Tunnel Network, never on the map
/// (`factions/syndicate/structures/README.md > Tunnel Expansions > Rally Point Behavior`), as a second one queued
/// behind it is 160 ticks after that. The plant lost meanwhile, a third is refused, and the two queued are trained
/// (`core/tech_and_research.md > Tech Prerequisites > Unit Prerequisites`: "units already queued or in production
/// continue and complete normally; new ones cannot be queued"). Each takes its TunnelSpaceCost of 1.
#[test]
fn the_hideout_trains_grenadiers_once_a_chemical_plant_stands() {
    let mut t = started(base(), 1000);
    let refused = Err(Reject::Requires(CHEMICAL_PLANT));
    assert_eq!(gate(&t, P1, train_grenadier(H1)), refused);
    next(&mut t, P1, expand(CHEMICAL_PLANT, NORTH_WEST));
    let plant = building(&t);
    assert_eq!((t.sim().tick(), bank(&t)), (2, (850, 10)));
    t.run_until(401);
    assert!(object(&t, plant).is_site());
    assert_eq!(gate(&t, P1, train_grenadier(H1)), refused, "a site doesn't count");
    t.step();
    assert!(!object(&t, plant).is_site() && t.sim().requirement_met(P1, CHEMICAL_PLANT));
    assert_eq!(gate(&t, P1, train_grenadier(H1)), Ok(()));
    next(&mut t, P1, train_grenadier(H1));
    let queued = t.sim().tick();
    assert_eq!(bank(&t), (850 - GRENADIER_COST, 10), "paid at the click");
    next(&mut t, P1, train_grenadier(H1));
    assert_eq!(bank(&t), (850 - 2 * GRENADIER_COST, 10));
    next_debug(&mut t, remove(plant));
    assert!(!t.sim().requirement_met(P1, CHEMICAL_PLANT));
    assert_eq!(gate(&t, P1, train_grenadier(H1)), refused, "lost");

    let first_done = queued + GRENADIER_TICKS - 1;
    t.run_until(first_done - 1);
    assert_eq!(produced(&t), []);
    t.step();
    let [(tick, first)] = produced(&t)[..] else {
        panic!("one Grenadier: {:?}", produced(&t));
    };
    assert_eq!(tick, first_done, "160 ticks from tick {queued} on");
    assert!(t.sim().object(first).is_none(), "not on the map");
    let held = t.sim().held_unit(first).expect("in the Tunnel Network");
    assert_eq!(
        (held.place, held.object.kind, held.object.owner),
        (Place::Network, GRENADIER, Some(P1))
    );
    assert_eq!(held.object.hp, Some(70.0), "MaxHP - 70");
    t.run_until(first_done + GRENADIER_TICKS);
    let ticks: Vec<Tick> = produced(&t).iter().map(|&(tick, _)| tick).collect();
    assert_eq!(
        ticks,
        [first_done, first_done + GRENADIER_TICKS],
        "the second 160 ticks later"
    );
    let inside = t.sim().network(P1).filter(|held| held.object.kind == GRENADIER);
    assert_eq!(inside.count(), 2);
    assert_eq!(
        tunnel_space(&t).0,
        3,
        "used: the Agent's and two Grenadiers' TunnelSpaceCost - 1"
    );
    assert_eq!(bank(&t), (850 - 2 * GRENADIER_COST, 10));
}

/// The gate's reasons in its order (`Sim::validate`): the prerequisite before the bank, so that the Hideout's W says
/// "Requires an Optics Lab" to a player who couldn't pay for a Guard either; the Optics Lab itself requires nothing
/// ("TechPrerequisite - None") but its 150 Space Crystals: 149 aren't enough. Once it is complete, the Guard's 125
/// is what's missing.
#[test]
fn the_prerequisite_comes_before_the_bank() {
    let mut t = started(base(), 100);
    assert_eq!(gate(&t, P1, train_guard(H1)), Err(Reject::Requires(OPTICS_LAB)));
    next_debug(&mut t, banks(P1, 149, 10));
    let lab = expand(OPTICS_LAB, NORTH_WEST);
    assert_eq!(gate(&t, P1, lab.clone()), Err(Reject::InsufficientCrystals));
    next_debug(&mut t, banks(P1, 150, 10));
    assert_eq!(gate(&t, P1, lab.clone()), Ok(()));
    next(&mut t, P1, lab);
    let site = building(&t);
    while object(&t, site).is_site() {
        t.step();
    }
    assert_eq!(bank(&t), (0, 10));
    assert_eq!(gate(&t, P1, train_guard(H1)), Err(Reject::InsufficientCrystals));
}

/// `core/tech_and_research.md > Tech Prerequisites > Counting Rule` and `factions/syndicate/structures/README.md >
/// Tunnel Expansions > Expansion Requirements` (a requirement "satisfied by a fully constructed structure anywhere in
/// the Tunnel Network", the player's own): player 2's complete Optics Lab, in its Tunnel's area, lets player 2's
/// Hideout train Guards and not player 1's; nor does a neutral one, an expansion whose Tunnel fell, on the map's edge.
#[test]
fn an_enemys_or_a_neutral_lab_does_not_count() {
    let scenario = base()
        .spawn(TUNNEL, Some(P2), tile(28, 8)) // 4: area (26, 6) to (33, 13)
        .spawn(HIDEOUT, Some(P2), tile(32, 12)) // 5
        .spawn(OPTICS_LAB, Some(P2), tile(26, 6)) // 6
        .spawn(OPTICS_LAB, None, tile(16, 0)); // 7
    let mut t = started(scenario, 1000);
    t.debug_at(2, banks(P2, 1000, 0));
    t.run_until(2);
    assert_eq!(gate(&t, P2, train_guard(ObjectId(5))), Ok(()));
    assert_eq!(gate(&t, P1, train_guard(H1)), Err(Reject::Requires(OPTICS_LAB)));
    assert_eq!(
        [P1, P2].map(|player| t.sim().requirement_met(player, OPTICS_LAB)),
        [false, true]
    );
}

/// `factions/syndicate/structures/README.md > Tunnel Expansions` ("Expansions are invisible to enemies without
/// detection"; plan decision T32), the Optics Lab and the Chemical Plant as the Hideout, their sites included: player
/// 2's Peacekeeper holds beside both places, which it sees, while player 1 builds an Optics Lab in one and then a
/// Chemical Plant in the other. From each click to completion and after, player 2 neither sees nor remembers either,
/// and can't name it in an order.
#[test]
fn the_enemy_sees_neither_nor_their_sites() {
    let scenario = base().spawn(ObjectKind::Peacekeeper, Some(P2), point(1, 9)); // 4
    let peacekeeper = ObjectId(4);
    let mut t = started(scenario, 1000);
    let hold = Command::Order {
        units: vec![peacekeeper],
        kind: OrderKind::Hold,
        queue: false,
    };
    next(&mut t, P2, hold);
    t.run(sc_sim::tuning::vision::BUILTIN.fog_delay_ticks + 1);
    let unseen = |t: &TestSim, id: ObjectId| {
        let o = object(t, id);
        assert!(t.sim().hidden_from(o, P2) && !t.sim().hidden_from(o, P1), "{id:?}");
        assert!(!t.sim().is_visible(P2, o), "{id:?}");
        let vision = t.sim().vision().player(P2).expect("in the game");
        assert!(vision.record(id).is_none(), "{id:?} unremembered");
        for kind in [OrderKind::Attack(id), OrderKind::Move(Target::Object(id))] {
            let order = Command::Order {
                units: vec![peacekeeper],
                kind,
                queue: false,
            };
            assert_eq!(gate(t, P2, order), Err(Reject::NotVisible(id)));
        }
    };
    let mut built = Vec::new();
    for (kind, origin) in [(OPTICS_LAB, NORTH_WEST), (CHEMICAL_PLANT, SOUTH_WEST)] {
        let corners = [origin, at(origin.x + 1, origin.y + 1)];
        assert!(
            corners
                .iter()
                .all(|&c| t.sim().visibility(P2, c) == VisibilityState::Visible)
        );
        next(&mut t, P1, expand(kind, origin));
        let site = building(&t);
        built.push(site);
        while object(&t, site).is_site() {
            built.iter().for_each(|&id| unseen(&t, id));
            t.step();
        }
        built.iter().for_each(|&id| unseen(&t, id));
    }
    t.run(20);
    built.iter().for_each(|&id| unseen(&t, id));
    assert_eq!(
        built.iter().map(|&id| object(&t, id).kind).collect::<Vec<_>>(),
        [OPTICS_LAB, CHEMICAL_PLANT]
    );
}
