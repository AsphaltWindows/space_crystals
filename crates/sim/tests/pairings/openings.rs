//! The pairings' scripted openings (`super::OPENINGS`, one hook a faction): each plays its player from their faction's
//! Starting Conditions through the lines of `sc_sim::testing` (`build_order` for the GDO, `colonists`, `syndicate`,
//! `cults`), with the player's own commands through the gate, as a player would.
//!
//! - **Any map, either slot.** A hook takes what its line needs from the game as it starts, never from a map's layout:
//!   the patch nearest the player's drop-off ([`nearest_patch`]), the Beacon sites around their Robo Portal
//!   ([`beacon_sites`]), the Optics Lab's spot in their Tunnel's area ([`expansion_site`]), the patches nearest the
//!   Cults' Storage and their Armory's spot by their Hall ([`armory_site`]).
//! - **An opening, not an attack.** Each gathers, builds and trains at home, and sends no unit out, so that a pairing
//!   ends only by the test's removals (`super::end`).

use sc_sim::economy::drop_off;
use sc_sim::nav::TileRect;
use sc_sim::placement::footprint_size;
use sc_sim::testing::build_order::{BuildOrder, Order};
use sc_sim::testing::colonists::Colonists;
use sc_sim::testing::cults::{Build, Crew, Cults};
use sc_sim::testing::syndicate::{Done, Syndicate};
use sc_sim::tunnel::area;
use sc_sim::{
    BuildingOrientation, Command, Object, ObjectId, ObjectKind, PlayerCommand, PlayerId, ProductionCommand, SU_PER_GU,
    Shape, Sim, Target, Tick, TilePos, Vec2,
};

use ObjectKind::{
    Agent, Armory, Barracks, Beacon, BioPortal, CrystalPatch, DeploymentCenter, Drone, ExtractionFacility,
    ExtractionPlate, Generator, Guard, Hideout, OpticsLab, Peacekeeper, Recruit, RecruitmentHall, RoboPortal,
    SabreLegionnaire, Storage, Tunnel,
};

/// A player's scripted opening.
pub trait Opening {
    /// The player's commands for the step to tick `sim.tick() + 1`, from the game as it is now.
    fn play(&mut self, sim: &Sim) -> Vec<PlayerCommand>;
    /// What the player's commands of the step to `tick` paid: in Space Crystals, and in Supplies.
    fn paid_in(&self, tick: Tick) -> (u32, u32);
}

/// A faction's hook: the opening of `player`, from the game as it starts.
pub type Hook = fn(&Sim, PlayerId) -> Box<dyn Opening>;

/// The GDO's Peacekeepers.
const PEACEKEEPERS: usize = 6;
/// The Colonists' Drones, trained one after the other.
const DRONES: usize = 12;
/// The Colonists' Sabre Legionnaires.
const SABRES: usize = 2;
/// The Agents the Syndicate's Hideout trains: with the starting one, three on one crystal hatch ([`syndicate`]).
const AGENTS: usize = 2;
/// The Syndicate's Guards, behind the Optics Lab.
const GUARDS: usize = 4;
/// The Beacon sites' lattice, in tiles ([`beacon_sites`]): a 2 × 2 Beacon, the column and row that a Bio Portal warped
/// in on it adds (its 3 × 3 footprint starts at the Beacon's, `sc_sim::transform::warp_in_footprint`), and a free one.
const LATTICE: i32 = 4;
/// How many lattice steps from the Robo Portal the Beacon sites reach, and from the Recruitment Hall the Armory's
/// ([`armory_site`]).
const REACH: i32 = 3;
/// The patches the Cults' Recruits are dealt to ([`cults`]), and how many a patch seats.
const SEATED: usize = 4;
const PER_PATCH: u32 = 2;
/// The Recruits that build the Cults' Armory.
const BUILDERS: usize = 3;
/// The Cults' Militants.
const MILITANTS: usize = 2;

/// The GDO (`testing::build_order`): player 1's line in the golden game `gdo_opening` (`tests/golden/gdo.rs`), the
/// fastest clean Barracks (`factions/gdo/economy.md > Starting Conditions > Opening Economy Baseline`: "Plate,
/// Generator, Generator, Barracks"), each building placed at the first spot the placement check takes
/// (`build_order::first_spot`); then [`PEACEKEEPERS`] Peacekeepers, with no rally point, so that they stay by the
/// Barracks.
pub fn gdo(_: &Sim, player: PlayerId) -> Box<dyn Opening> {
    let buildings = [
        Order::new(ExtractionFacility, ExtractionPlate),
        Order::new(DeploymentCenter, Generator),
        Order::new(DeploymentCenter, Generator),
        Order::new(DeploymentCenter, Barracks),
    ];
    let units = [Order::new(Barracks, Peacekeeper); PEACEKEEPERS];
    Box::new(BuildOrder::new(player).track(buildings.into_iter().chain(units)))
}

impl Opening for BuildOrder {
    fn play(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.commands(sim)
    }

    fn paid_in(&self, tick: Tick) -> (u32, u32) {
        let issued = self.issued().iter().filter(|i| i.tick == tick);
        (issued.map(|i| i.cost).sum(), 0)
    }
}

/// The Colonists (`testing::colonists`): player 1's line in the golden game `colonists_opening`
/// (`testing::colonists::skirmish`), but for the Sabres' rally point and conversion. The Drones gather the patch
/// nearest the Egress Node, and the Robo Portal, rallied on it, trains [`DRONES`] more; a Drone it releases deploys a
/// Beacon at the next of [`beacon_sites`] when the room under Beacon Capacity falls to 1; a Bio Portal warps in on
/// the first Beacon and trains [`SABRES`] Sabre Legionnaires, which stay by it. The Warp In and the Sabres get the
/// money first.
pub fn colonists(sim: &Sim, player: PlayerId) -> Box<dyn Opening> {
    let line = BuildOrder::new(player)
        .track([Order::new(Beacon, BioPortal)])
        .track(vec![Order::new(BioPortal, SabreLegionnaire); SABRES])
        .track(vec![Order::new(RoboPortal, Drone); DRONES]);
    let patch = nearest_patch(sim, player);
    let mut line = Colonists::new(line).deploy_at(beacon_sites(sim, player, patch));
    if let Some(patch) = patch {
        line = line.gather(patch).rally(RoboPortal, Target::Object(patch));
    }
    Box::new(line)
}

impl Opening for Colonists {
    fn play(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.commands(sim)
    }

    fn paid_in(&self, tick: Tick) -> (u32, u32) {
        (self.spent_in(tick), 0)
    }
}

/// The Syndicate (`testing::syndicate`): player 1's line in the golden game `syndicate_vs_gdo`
/// (`testing::syndicate::skirmish`), cut down: two Agents trained rather than four, and no second Tunnel, Supplies or
/// Guards' post. The Agent gathers the patch nearest the crystal hatch, and the Hideout, rallied on it, trains
/// [`AGENTS`] more; the Tunnel expands an Optics Lab at [`expansion_site`], and the Hideout then trains [`GUARDS`]
/// Guards, which wait in the Tunnel Network ([`SyndicateOpening`]).
///
/// Three Agents on one crystal hatch, its saturation (design change 0294), and no more: as built, four or more whose
/// patch lies to the side of the hatch can gridlock beside it for good, and the income stops (Q0109, whose first step,
/// design change 0315, is built and doesn't end it; `docs/playtest/m4.md > Known issues (from the M4 QA, not fixed
/// yet)`: "Three Agents a hatch, the design's saturation, never jammed"). So this line keeps to three, as
/// `tests/syndicate_soak.rs` does.
pub fn syndicate(sim: &Sim, player: PlayerId) -> Box<dyn Opening> {
    let mut line = Syndicate::new(BuildOrder::new(player).track(vec![Order::new(Hideout, Agent); AGENTS]))
        .after(OpticsLab, vec![Order::new(Hideout, Guard); GUARDS]);
    if let Some(patch) = nearest_patch(sim, player) {
        line = line.gather(patch).rally(Target::Object(patch));
    }
    let tunnel = (sim.objects()).find(|o| o.owner == Some(player) && o.kind == Tunnel);
    if let Some(origin) = tunnel.and_then(|tunnel| expansion_site(sim, tunnel, OpticsLab)) {
        line = line.expand(0, OpticsLab, origin);
    }
    Box::new(SyndicateOpening {
        player,
        line,
        cleared: false,
    })
}

/// [`syndicate`]'s line, and the Hideout's rally point cleared as the first Guard is ordered (the producer itself as
/// the target clears it, design change 0086), so that the Guards stay in the Tunnel Network rather than join the
/// Agents at their patch, as `testing::syndicate::skirmish` keeps its Guards.
struct SyndicateOpening {
    player: PlayerId,
    line: Syndicate,
    cleared: bool,
}

impl Opening for SyndicateOpening {
    fn play(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let tick = sim.tick() + 1;
        let mut commands = self.line.commands(sim);
        let guard = (self.line.done().iter()).find_map(|&(at, done)| match done {
            Done::Train { by, order } if at == tick && order.kind == Guard => Some(by),
            _ => None,
        });
        if !self.cleared
            && let Some(hideout) = guard
        {
            commands.push(PlayerCommand {
                player: self.player,
                command: Command::Production {
                    objects: vec![hideout],
                    command: ProductionCommand::SetRally(Target::Object(hideout)),
                },
            });
            self.cleared = true;
        }
        commands
    }

    fn paid_in(&self, tick: Tick) -> (u32, u32) {
        (self.line.spent_in(tick), self.line.supplies_spent_in(tick))
    }
}

/// The Cults (`testing::cults`): player 1's line in the golden game `cults_opening` (`testing::cults::skirmish`), cut
/// down to what any map gives: the Recruits dealt to the [`SEATED`] patches nearest the starting Storage, edge to
/// edge, [`PER_PATCH`] to a patch (within the design's 4–5 Recruits a 2-tile face, design change 0320); an Armory at
/// [`armory_site`], built by the [`BUILDERS`] Recruits nearest it, which go back to their patches after; and
/// [`MILITANTS`] Militants of the Recruits without a seat, which stay by the Armory (no rally point).
pub fn cults(sim: &Sim, player: PlayerId) -> Box<dyn Opening> {
    cults_training(sim, player, MILITANTS)
}

/// [`cults`], training `militants` Militants rather than [`MILITANTS`]: (slice M5-Mb) the soak's, whose Armory holds
/// Recruits long enough for the soak to destroy it with Recruits inside (`super::soak`).
pub fn cults_training(sim: &Sim, player: PlayerId, militants: usize) -> Box<dyn Opening> {
    let mut line = Cults::new(player);
    let storage = (sim.objects()).find(|o| o.owner == Some(player) && o.kind == Storage);
    if let Some(storage) = storage {
        let from = Shape::of(storage);
        let mut patches: Vec<(f32, ObjectId)> = (sim.objects())
            .filter(|o| o.kind == CrystalPatch)
            .map(|o| (from.gap(&Shape::of(o)), o.id))
            .collect();
        patches.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        line = line.deal(patches.iter().take(SEATED).map(|&(_, patch)| (patch, PER_PATCH)));
    }
    if let Some(origin) = armory_site(sim, player) {
        line = line.build(Build::new(Armory, origin, Crew::Nearest(BUILDERS)));
    }
    Box::new(line.militants(militants, None))
}

impl Opening for Cults {
    fn play(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.commands(sim)
    }

    fn paid_in(&self, tick: Tick) -> (u32, u32) {
        (self.spent_in(tick), 0)
    }
}

/// Where `player`'s Armory goes ([`cults`]): of the origins within [`REACH`] lattice steps of their Recruitment
/// Hall's, a tile at a time, those whose footprint has a free tile all round it on the map as the game starts (clear
/// of every footprint, patches included) and lies beyond the Hall from the patch nearest it, out of the Recruits' way
/// between the Hall, its Storage and the field; the nearest the Hall first, ties row by row; the first the build check
/// takes for a Recruit of theirs (`Sim::build_check`: the gate's, but for the price, which the line waits for). `None`
/// without a Hall, a Recruit or such a spot.
fn armory_site(sim: &Sim, player: PlayerId) -> Option<TilePos> {
    let own = |kind| (sim.objects()).find(move |o: &&Object| o.owner == Some(player) && o.kind == kind);
    let (hall, recruit) = (own(RecruitmentHall)?, own(Recruit)?);
    let placement = hall.placement?;
    let orientation = BuildingOrientation::default();
    let (w, h) = footprint_size(Armory, orientation)?;
    let patches = sim.objects().filter(|o| o.kind == CrystalPatch);
    let patch = patches.min_by(|a, b| (a.pos.distance_squared(hall.pos)).total_cmp(&b.pos.distance_squared(hall.pos)));
    let away = patch.map(|patch| hall.pos - patch.pos);
    let taken: Vec<TileRect> = sim.objects().filter_map(footprint).collect();
    let map = TileRect::new(TilePos::new(0, 0), sim.map().width(), sim.map().height());
    let span = REACH * LATTICE;
    let mut sites: Vec<(f32, TilePos)> = ((-span..=span).flat_map(|j| (-span..=span).map(move |i| (i, j))))
        .map(|(i, j)| TilePos::new(placement.origin.x + i, placement.origin.y + j))
        .filter(|&origin| {
            let margin = TileRect::new(TilePos::new(origin.x - 1, origin.y - 1), w + 2, h + 2);
            area::encloses(map, margin) && !taken.iter().any(|&t| area::overlaps(t, margin))
        })
        .map(|origin| {
            (
                origin.corner() + Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0),
                origin,
            )
        })
        .filter(|&(centre, _)| away.is_none_or(|away| (centre - hall.pos).dot(away) > 0.0))
        .map(|(centre, origin)| (centre.distance_squared(hall.pos), origin))
        .collect();
    sites.sort_by(|a, b| (a.0.total_cmp(&b.0)).then((a.1.y, a.1.x).cmp(&(b.1.y, b.1.x))));
    let takes = |origin| {
        let check = sim.build_check(recruit.id, Armory, origin, orientation, false);
        check.is_some_and(|check| check.is_valid())
    };
    sites
        .into_iter()
        .map(|(_, origin)| origin)
        .find(|&origin| takes(origin))
}

/// The crystal patch nearest where `player`'s first drop-off (by id) takes loads: its crystal hatch's square for a
/// Tunnel (`economy::drop_off::hatch_square`), else its centre; ties by id. `None` without a drop-off or a patch.
fn nearest_patch(sim: &Sim, player: PlayerId) -> Option<ObjectId> {
    let own = (sim.objects()).find(|o| o.owner == Some(player) && drop_off::rule(o.kind).is_some())?;
    let hatch = drop_off::hatch_square(own, drop_off::Resource::Crystals);
    let at = hatch.map_or(own.pos, TilePos::center);
    let patches = sim.objects().filter(|o| o.kind == CrystalPatch);
    let nearest = patches.min_by(|a, b| (a.pos.distance_squared(at)).total_cmp(&b.pos.distance_squared(at)));
    nearest.map(|o| o.id)
}

/// Where `player`'s Beacons deploy, in order (`Colonists::deploy_at`: each at the first of these the deploy check
/// takes then): the sites on a lattice of [`LATTICE`] tiles from their Robo Portal's top-left tile, within [`REACH`]
/// steps of it, each on the map with a free tile all round it as the game starts (clear of every footprint, patches
/// included), and beyond the portal from `patch`, the one their Drones gather, out of the Drones' way; the nearest the
/// portal first, ties row by row. Empty without a Robo Portal.
fn beacon_sites(sim: &Sim, player: PlayerId, patch: Option<ObjectId>) -> Vec<TilePos> {
    let Some(portal) = (sim.objects()).find(|o| o.owner == Some(player) && o.kind == RoboPortal) else {
        return Vec::new();
    };
    let (Some(placement), Some((w, h))) = (portal.placement, footprint_size(Beacon, BuildingOrientation::default()))
    else {
        return Vec::new();
    };
    let away = (patch.and_then(|patch| sim.object(patch))).map(|patch| portal.pos - patch.pos);
    let taken: Vec<TileRect> = sim.objects().filter_map(footprint).collect();
    let map = TileRect::new(TilePos::new(0, 0), sim.map().width(), sim.map().height());
    let lattice = (-REACH..=REACH).flat_map(|j| (-REACH..=REACH).map(move |i| (i, j)));
    let mut sites: Vec<(f32, TilePos)> = lattice
        .map(|(i, j)| TilePos::new(placement.origin.x + LATTICE * i, placement.origin.y + LATTICE * j))
        .filter(|&origin| {
            let margin = TileRect::new(TilePos::new(origin.x - 1, origin.y - 1), w + 2, h + 2);
            area::encloses(map, margin) && !taken.iter().any(|&t| area::overlaps(t, margin))
        })
        .map(|origin| {
            (
                origin.corner() + Vec2::new(w as f32, h as f32) * (SU_PER_GU / 2.0),
                origin,
            )
        })
        .filter(|&(centre, _)| away.is_none_or(|away| (centre - portal.pos).dot(away) > 0.0))
        .map(|(centre, origin)| (centre.distance_squared(portal.pos), origin))
        .collect();
    sites.sort_by(|a, b| (a.0.total_cmp(&b.0)).then((a.1.y, a.1.x).cmp(&(b.1.y, b.1.x))));
    sites.into_iter().map(|(_, origin)| origin).collect()
}

/// The tiles an object's footprint covers, if it has one.
fn footprint(o: &Object) -> Option<TileRect> {
    let p = o.placement?;
    Some(TileRect::new(p.origin, p.w, p.h))
}

/// Where `tunnel` expands a `kind` (`Syndicate::expand`): the first origin of its area, row by row
/// (`tunnel::area::cells`), that the expansion check takes as the game starts. An underground expansion blocks no
/// movement, and no unit blocks it (design change 0290), so any spot of the area serves.
fn expansion_site(sim: &Sim, tunnel: &Object, kind: ObjectKind) -> Option<TilePos> {
    let orientation = BuildingOrientation::default();
    let check = |origin| sim.expansion_check(tunnel.id, kind, origin, orientation);
    area::cells(area::of(tunnel)?).find(|&origin| check(origin).is_some_and(|check| check.is_valid()))
}
