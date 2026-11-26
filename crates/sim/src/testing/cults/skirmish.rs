//! `dev_skirmish` played by script from its real start (slice M5-L): the golden games `cults_opening` and
//! `cults_vs_syndicate` (`tests/golden/cults.rs`). Player 1 plays the Cults from slot 1's Cults Starting Layout (a
//! Recruitment Hall, a Storage, six Recruits and 0 Space Crystals, `factions/cults/economy.md > Starting Conditions`)
//! as a [`Cults`] line; player 2 plays the Syndicate from slot 2's Syndicate Starting Layout as a [`Syndicate`] line,
//! or does nothing. Both give their own commands, which the gate takes, as players would:
//! - [`Skirmish::opening`]: player 1's Recruits are dealt to the field, first to the patches a tile from the starting
//!   Storage ([`NEAR`]), then to its western column ([`WEST`]), for which the line builds a second Storage
//!   ([`STORAGE_SITE`]) as soon as it can; then a second Recruitment Hall ([`HALL_SITE`]), an Armory
//!   ([`ARMORY_SITE`]) and [`OPENING_MILITANTS`] Militants, rallied at [`STAGING`]. Player 2 does nothing.
//! - [`Skirmish::versus_syndicate`]: player 1's Recruits are dealt to [`NEAR`] only, the Armory comes first, then a
//!   Gunworks ([`GUNWORKS_SITE`], slice M5-N1a's first step), and every Recruit without a seat goes into the Armory to
//!   become one of [`MILITANTS`] Militants or [`GUNNERS`] Gunners, the Gunners trained first once the Gunworks stands.
//!   Once [`WAVE`] of them wait at [`STAGING`] they attack-move to player 2's Tunnel, fighting what they meet on the
//!   way; one trained later goes on its own. Player 2's Agents gather [`PATCH`], 3 grid units from its crystal hatch,
//!   and its Hideout, rallied on the patch, trains [`AGENTS`] more; its Tunnel expands an Optics Lab
//!   ([`OPTICS_LAB_SITE`]) and then a Laser Turret ([`TURRET_SITE`]), ordered from its ExpandMenu as slice M5-Y made
//!   it, and the Hideout trains [`GUARDS`] Guards once the Optics Lab stands, which come out to the patch. The script
//!   knows where player 2's Tunnel is, as a player who scouted would.
//!
//! Nothing here assumes the game goes as the goldens' do: the players stop once the game is over or they are
//! defeated, and a Militant or Gunner whose target has fallen goes on to the next Tunnel.

use std::collections::BTreeMap;

use crate::command::{Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Object;
use crate::orders::{OrderKind, Target};
use crate::player::Faction;
use crate::scenario::Scenario;
use crate::scenarios;
use crate::sim::Sim;
use crate::testing::build_order::{BuildOrder, Order};
use crate::testing::syndicate::Syndicate;
use crate::units::{Tick, TilePos};

use super::{Build, Crew, Cults};

use ObjectKind::{
    Agent, Armory, Guard, Gunner, Gunworks, Hideout, LaserTurret, Militant, OpticsLab, RecruitmentHall, Storage, Tunnel,
};

/// The Cults' player, in slot 1.
pub const CULTS: PlayerId = PlayerId(1);
/// The Syndicate's, in slot 2.
pub const SYNDICATE: PlayerId = PlayerId(2);
/// Seats on slot 1's field (`dev_skirmish`'s first spawns, in its order) a tile from the starting Storage on (5, 4)
/// to (7, 5), edge to edge: the northern row's (4, 2) to (7, 2), patches 5 to 8, two each on patches 6 and 7 and one
/// on patch 8, the three straight above the Storage's long north side, and one on patch 5 at its corner; and (3, 3),
/// patch 4. Seven seats, as slice M5-L set them for the 2x2 Storage of the time (design change 0320's 4–5 Recruits a
/// 2-tile face, for its north face and its corner); kept with the 3x2 (design change 0380), whose 3-tile side holds
/// "about 2 Recruits per tile of face length".
pub const NEAR: [(ObjectId, u32); 5] = [
    (ObjectId(6), 2),
    (ObjectId(7), 2),
    (ObjectId(5), 1),
    (ObjectId(8), 1),
    (ObjectId(4), 1),
];
/// [`Skirmish::opening`]: seats on the field's western column, (2, 5) to (2, 7), patches 1 to 3, 2 and 3 tiles from
/// the starting Storage and a tile from the second ([`STORAGE_SITE`]): two each.
pub const WEST: [(ObjectId, u32); 3] = [(ObjectId(1), 2), (ObjectId(2), 2), (ObjectId(3), 2)];
/// [`Skirmish::opening`]: the second Storage, unturned on (4, 6) to (6, 7) (design change 0380: 3x2), a 1-tile gap
/// east of the western column's three patches (its west side facing (2, 6) and (2, 7), its corner (2, 5)), and
/// south-west of the starting Storage, whose south side it touches along two tiles.
pub const STORAGE_SITE: TilePos = TilePos::new(4, 6);
/// [`Skirmish::opening`]: the second Recruitment Hall, on (20, 17) to (23, 20), unturned (its exit side, B, west):
/// on flat Plane east of the hill and west of the plateau. Its Recruitment Area, (17, 14) to (26, 23), is all on the
/// map, off the lake and clear of the starting Hall's (5, 5) to (14, 14), so it claims all of it.
pub const HALL_SITE: TilePos = TilePos::new(20, 17);
/// The Armory, on (13, 5) to (15, 6), unturned: its entrance, B, east; its exit, C, west, toward the field. North-east
/// of the starting Hall, off the Recruits' ways between the Hall, the field and the Storages.
pub const ARMORY_SITE: TilePos = TilePos::new(13, 5);
/// [`Skirmish::versus_syndicate`]: the Gunworks, on (13, 2) to (14, 3), unturned: north of the Armory, a tile from it,
/// east of the field's northern row and off the Recruits' ways between the Hall, the field, the Storage and the
/// Armory's entrance, B, east.
pub const GUNWORKS_SITE: TilePos = TilePos::new(13, 2);
/// Where the Armory's Militants wait: open ground south-east of the starting Hall.
pub const STAGING: TilePos = TilePos::new(16, 12);
/// [`Skirmish::opening`]: the Militants the Armory trains.
pub const OPENING_MILITANTS: usize = 4;
/// [`Skirmish::versus_syndicate`]: the Militants the Armory trains, and how many wait at [`STAGING`] before they set
/// off.
pub const MILITANTS: usize = 8;
pub const WAVE: usize = 6;
/// [`Skirmish::versus_syndicate`]: the Gunners the Armory trains once the Gunworks stands
/// (`factions/cults/structures/armory.md > Training`: "**Gunner** (SpaceCrystalsCost: 40, TrainingTime: 32 frames).
/// TechPrerequisite: player owns at least one Gunworks").
pub const GUNNERS: usize = 3;
/// [`Skirmish::versus_syndicate`]: slot 2's patch on tile (61, 40), the image of slot 1's (2, 7), 3 grid units from
/// slot 2's crystal hatch: the third of slot 2's field, whose spawns follow the station's. Player 2's starting Agent is
/// sent to it, and its Hideout is rallied on it.
pub const PATCH: ObjectId = ObjectId(12);
/// How many Agents player 2's Hideout trains.
pub const AGENTS: usize = 2;
/// Player 2's Optics Lab, on (52, 42) to (53, 43): the image of the Syndicate goldens' (10, 4), in its Tunnel's area,
/// (52, 36) to (59, 43).
pub const OPTICS_LAB_SITE: TilePos = TilePos::new(52, 42);
/// Player 2's Laser Turret, on (52, 39): in its Tunnel's area, west of the Tunnel, facing the Cults' way in.
pub const TURRET_SITE: TilePos = TilePos::new(52, 39);
/// The Guards player 2's Hideout trains once the Optics Lab stands.
pub const GUARDS: usize = 4;

/// `dev_skirmish` with player 1 playing the Cults (slot 1), and player 2 the Syndicate (slot 2) if `versus`.
pub fn scenario(versus: bool) -> Scenario {
    let mut scenario = scenarios::dev_skirmish();
    scenario.setup.players[0].faction = Faction::Cults;
    if versus {
        scenario.setup.players[1].faction = Faction::Syndicate;
    }
    scenario
}

/// The players of a [`scenario`] game (module docs).
#[derive(Clone, Debug)]
pub struct Skirmish {
    cults: Cults,
    syndicate: Option<Syndicate>,
    /// Player 1's Militants and Gunners sent to attack, each with the Tunnel it was sent to.
    sent: BTreeMap<ObjectId, ObjectId>,
}

impl Skirmish {
    /// The Cults' opening (module docs): the field dealt, a second Storage, a second Hall, an Armory and
    /// [`OPENING_MILITANTS`] Militants.
    pub fn opening() -> Self {
        let cults = Cults::new(CULTS)
            .deal(NEAR)
            .deal(WEST)
            .build(Build::new(Storage, STORAGE_SITE, Crew::Nearest(2)))
            .build(Build::new(RecruitmentHall, HALL_SITE, Crew::Nearest(3)))
            .build(Build::new(Armory, ARMORY_SITE, Crew::Nearest(3)))
            .militants(OPENING_MILITANTS, Some(Target::Point(STAGING.center())));
        Self::new(cults, None)
    }

    /// The Cults against the Syndicate (module docs): [`MILITANTS`] Militants and [`GUNNERS`] Gunners in a wave
    /// against player 2's Tunnel, its Optics Lab, Laser Turret and [`GUARDS`] Guards.
    pub fn versus_syndicate() -> Self {
        let cults = Cults::new(CULTS)
            .deal(NEAR)
            .build(Build::new(Armory, ARMORY_SITE, Crew::Nearest(3)))
            .build(Build::new(Gunworks, GUNWORKS_SITE, Crew::Nearest(3)))
            .militants(MILITANTS, Some(Target::Point(STAGING.center())))
            .gunners(GUNNERS);
        let agents = BuildOrder::new(SYNDICATE).track(vec![Order::new(Hideout, Agent); AGENTS]);
        let syndicate = Syndicate::new(agents)
            .gather(PATCH)
            .rally(Target::Object(PATCH))
            .expand(0, OpticsLab, OPTICS_LAB_SITE)
            .expand(0, LaserTurret, TURRET_SITE)
            .after(OpticsLab, vec![Order::new(Hideout, Guard); GUARDS]);
        Self::new(cults, Some(syndicate))
    }

    fn new(cults: Cults, syndicate: Option<Syndicate>) -> Self {
        Self {
            cults,
            syndicate,
            sent: BTreeMap::new(),
        }
    }

    /// Player 1's line.
    pub fn cults(&self) -> &Cults {
        &self.cults
    }

    /// Player 2's line, if player 2 plays.
    pub fn syndicate(&self) -> Option<&Syndicate> {
        self.syndicate.as_ref()
    }

    /// What `player`'s orders of the step to tick `tick` paid in Space Crystals.
    pub fn spent_in(&self, player: PlayerId, tick: Tick) -> u32 {
        if player == CULTS {
            return self.cults.spent_in(tick);
        }
        self.syndicate.as_ref().map_or(0, |line| line.spent_in(tick))
    }

    /// What player 2's orders of the step to tick `tick` paid in Supplies.
    pub fn supplies_spent_in(&self, tick: Tick) -> u32 {
        self.syndicate.as_ref().map_or(0, |line| line.supplies_spent_in(tick))
    }

    /// Both players' commands for the step to tick `sim.tick() + 1`, from the game as it is now (module docs).
    pub fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        let mut commands = self.cults.commands(sim);
        if let Some(line) = &mut self.syndicate {
            commands.extend(line.commands(sim));
        }
        let plays = sim.outcome().is_none() && sim.player(CULTS).is_some_and(|p| !p.is_defeated());
        if self.syndicate.is_some() && plays {
            self.attack(sim, &mut commands);
        }
        commands
    }

    /// Player 1's Militants and Gunners (module docs): a wave of the idle ones that never set off once there are
    /// [`WAVE`] of them, and at once each one sent that is idle and whose Tunnel has fallen, attack-moving to the
    /// centre of player 2's Tunnel nearest [`STAGING`].
    fn attack(&mut self, sim: &Sim, commands: &mut Vec<PlayerCommand>) {
        let staging = STAGING.center();
        let Some(target) = (sim.objects())
            .filter(|o| o.owner == Some(SYNDICATE) && o.kind == Tunnel)
            .min_by(|a, b| (a.pos.distance_squared(staging)).total_cmp(&b.pos.distance_squared(staging)))
        else {
            return;
        };
        let own = || sim.objects().filter(|o| o.owner == Some(CULTS));
        let idle = |o: &&Object| {
            matches!(o.kind, Militant | Gunner) && o.orders.as_ref().is_none_or(|orders| orders.is_idle())
        };
        let standing = |tunnel: &ObjectId| sim.object(*tunnel).is_some_and(|o| o.owner == Some(SYNDICATE));
        let again = own()
            .filter(idle)
            .filter(|o| self.sent.get(&o.id).is_some_and(|tunnel| !standing(tunnel)));
        let waiting: Vec<ObjectId> = own()
            .filter(idle)
            .filter(|o| !self.sent.contains_key(&o.id))
            .map(|o| o.id)
            .collect();
        let mut going: Vec<ObjectId> = again.map(|o| o.id).collect();
        let first = self.sent.is_empty();
        if waiting.len() >= WAVE || !first {
            going.extend(waiting);
        }
        if going.is_empty() {
            return;
        }
        going.sort_unstable();
        for &unit in &going {
            self.sent.insert(unit, target.id);
        }
        commands.push(PlayerCommand {
            player: CULTS,
            command: Command::Order {
                units: going,
                kind: OrderKind::AttackMove(target.pos),
                queue: false,
            },
        });
    }
}
