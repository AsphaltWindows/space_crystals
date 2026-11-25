//! The Syndicate's golden games (slice M4-L): `dev_skirmish` from its real start, played by script as
//! `sc_sim::testing::syndicate::skirmish` plays it (its module docs), through the gate:
//! - [`opening`]: player 1 alone. Its Agents gather the patch 3 grid units from the crystal hatch, and its Hideout,
//!   rallied on the patch, trains four more. The third of them builds a second Tunnel with the starting Supply, comes
//!   out of it once it is built and gathers Supplies; the first Tunnel expands an Optics Lab.
//! - [`versus_gdo`]: the same, with six Guards behind the Optics Lab: four come out at the second Tunnel and hold the
//!   top of the plateau's west ramp, two wait in the Tunnel Network. Player 2's GDO sends its Peacekeepers in waves:
//!   they kill the Guards and lose some of theirs, destroy the second Tunnel, then the first, which leaves the
//!   Hideout and the Optics Lab neutral and the Guards in the network stranded, and defeats player 1.
//!
//! Both players' books balance every step: their Space Crystals, the Agents' deliveries included
//! (`sc_sim::testing::build_order::Books`), and player 1's Supplies (`sc_sim::testing::syndicate::SupplyBooks`). Every
//! command the scripts give is one the gate takes.

use std::collections::BTreeMap;

use sc_sim::testing::build_order::Books;
use sc_sim::testing::syndicate::skirmish::{
    self, AGENTS, GDO, GUARDS, GUARDS_OUT, PATCH, STATION, SYNDICATE, Skirmish, TUNNEL_SITE,
};
use sc_sim::testing::syndicate::{Done, SupplyBooks};
use sc_sim::tunnel::Resource;
use sc_sim::{
    Command, ConstructionCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Reject, Scenario, Sim, SimEvent, Tick,
    TilePos,
};

use super::harness::{Game, Script};
use super::units_of;

use ObjectKind::{Agent, Guard, Hideout, OpticsLab, Peacekeeper, Tunnel};

/// [`opening`]'s last tick, 2:15: the builder of the second Tunnel has delivered its second Supply.
const OPENING_END: Tick = 2160;
/// [`versus_gdo`]'s last tick, 3:12: some seconds after player 1's defeat, the neutral expansions decaying.
const VERSUS_END: Tick = 3072;
/// [`opening`]'s camera starts over this tile: player 1's base and its second Tunnel.
const OPENING_CAMERA: TilePos = TilePos::new(13, 11);
/// [`versus_gdo`]'s: player 1's two Tunnels and the plateau's west ramp, where the Guards fight.
const VERSUS_CAMERA: TilePos = TilePos::new(18, 15);

/// `skirmish::scenario`, with player 1's camera over `camera`, where the game plays out, for watching it in the
/// client (and player 2's over its image: the map is symmetric).
fn scenario(camera: TilePos) -> Scenario {
    let mut scenario = skirmish::scenario();
    scenario.map.slots[0].camera_start = Some(camera.center());
    scenario.map.regenerate();
    scenario
}

/// The Syndicate's opening (`Skirmish::opening`). Player 2 does nothing. At the end, 2:15, player 1 has its two
/// Tunnels, the Hideout, the Optics Lab and five Agents, one of them carrying Supplies.
pub(crate) fn opening() -> Game {
    Game {
        scenario: scenario(OPENING_CAMERA),
        commands: Vec::new(),
        script: Some(Box::new(Players::new(Skirmish::opening()))),
        end: OPENING_END,
        during: |_, sim| {
            assert!(sim.outcome().is_none(), "tick {}: the game ended", sim.tick());
        },
        expect: |sim| {
            let own = |kind| units_of(sim, SYNDICATE).iter().filter(|o| o.kind == kind).count();
            assert_eq!(
                (own(Tunnel), own(Hideout), own(OpticsLab), own(Agent)),
                (2, 1, 1, 1 + AGENTS),
                "player 1's Tunnels, Hideout, Optics Lab and Agents"
            );
            assert!(
                units_of(sim, SYNDICATE).iter().all(|o| o.is_operational()),
                "nothing of player 1's is unfinished"
            );
        },
    }
}

/// The Syndicate against the GDO (`Skirmish::versus_gdo`). Player 1 is defeated when its first Tunnel, its last,
/// falls; the game is over, the GDO's, and the Hideout and the Optics Lab it leaves neutral decay.
pub(crate) fn versus_gdo() -> Game {
    Game {
        scenario: scenario(VERSUS_CAMERA),
        commands: Vec::new(),
        script: Some(Box::new(Players::new(Skirmish::versus_gdo()))),
        end: VERSUS_END,
        during: |_, sim| {
            assert!(
                !sim.player(GDO).unwrap().is_defeated(),
                "tick {}: player 2 is defeated",
                sim.tick()
            );
            if let Some(outcome) = sim.outcome() {
                assert_eq!(outcome.winners, [GDO], "the game is the GDO's");
            }
        },
        expect: |sim| {
            assert!(sim.player(SYNDICATE).unwrap().is_defeated(), "player 1 is defeated");
            assert!(
                units_of(sim, SYNDICATE).is_empty(),
                "player 1 owns nothing: its units are neutral"
            );
            assert_eq!(
                sim.network(SYNDICATE).count(),
                0,
                "nothing of player 1's is in the Tunnel Network"
            );
            // `factions/syndicate/tunnels.md > Losing Tunnels`: "every expansion in its Tunnel Area becomes neutral",
            // and "They slowly lose HP until destroyed".
            for kind in [Hideout, OpticsLab] {
                let expansion = sim
                    .objects()
                    .find(|o| o.kind == kind)
                    .unwrap_or_else(|| panic!("the {kind:?}"));
                assert_eq!(expansion.owner, None, "the {kind:?} is neutral");
                assert!(
                    expansion.hp < expansion.max_hp(),
                    "the {kind:?} decays: {:?}",
                    expansion.hp
                );
            }
        },
    }
}

/// The players of a Syndicate golden game, with their books and what they saw.
struct Players {
    skirmish: Skirmish,
    /// The books of the step under way, each player's, with what its orders paid; player 1's Supplies too.
    open: Vec<(PlayerId, Books, u32)>,
    supplies: Option<(SupplyBooks, u32)>,
    /// Player 1's starting Tunnel.
    first: Option<ObjectId>,
    /// Player 1's units in the Tunnel Network as the step under way started.
    network: Vec<ObjectId>,
    /// What player 1's Agents delivered: Space Crystals, and Supplies.
    delivered: u32,
    supplied: u32,
    /// The units each player's producers made, by kind.
    made: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// Player 1's units out of the network, by the Tunnel and their kind.
    out: BTreeMap<(ObjectId, ObjectKind), usize>,
    /// Player 1's structures finished: its sites and its expansions, by kind.
    built: BTreeMap<ObjectKind, usize>,
    /// The objects each player lost, by kind.
    died: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// Player 1's Tunnels destroyed, in order, each with its tick.
    lost: Vec<(Tick, ObjectId)>,
    /// Player 1's units removed from the network, each with its tick: stranded.
    stranded: Vec<(Tick, ObjectId)>,
    /// The tick player 1 was defeated in.
    defeated: Option<Tick>,
}

impl Players {
    fn new(skirmish: Skirmish) -> Self {
        Self {
            skirmish,
            open: Vec::new(),
            supplies: None,
            first: None,
            network: Vec::new(),
            delivered: 0,
            supplied: 0,
            made: BTreeMap::new(),
            out: BTreeMap::new(),
            built: BTreeMap::new(),
            died: BTreeMap::new(),
            lost: Vec::new(),
            stranded: Vec::new(),
            defeated: None,
        }
    }

    /// Closes the books of the step that just ran, which must balance, and notes what happened in it.
    fn close(&mut self, sim: &Sim) {
        for (player, books, spent) in std::mem::take(&mut self.open) {
            let entry = books.close(sim, spent);
            if player == SYNDICATE {
                self.delivered += entry.delivered;
            }
        }
        if let Some((books, spent)) = self.supplies.take() {
            books.close(sim, spent);
        }
        let tick = sim.tick();
        let owner = |id: ObjectId| -> Option<PlayerId> {
            let held = sim.held_unit(id).map(|held| &held.object);
            sim.object(id).or(held).and_then(|o| o.owner)
        };
        for e in sim.events() {
            match *e {
                SimEvent::UnitProduced { unit, .. } => {
                    let held = sim.held_unit(unit).map(|held| &held.object);
                    if let Some(o) = sim.object(unit).or(held)
                        && let Some(player) = o.owner
                    {
                        *self.made.entry((player, o.kind)).or_default() += 1;
                    }
                }
                SimEvent::LeftNetwork { unit, tunnel } => {
                    if let Some(o) = sim.object(unit).filter(|o| o.owner == Some(SYNDICATE)) {
                        *self.out.entry((tunnel, o.kind)).or_default() += 1;
                    }
                }
                SimEvent::SiteCompleted { site, .. } | SimEvent::ExpansionCompleted { site, .. } => {
                    if let Some(o) = sim.object(site).filter(|o| o.owner == Some(SYNDICATE)) {
                        *self.built.entry(o.kind).or_default() += 1;
                    }
                }
                SimEvent::ResourcesDelivered {
                    worker,
                    amount,
                    resource: Resource::Supplies,
                    drop_off,
                } if owner(worker) == Some(SYNDICATE) => {
                    assert!(
                        sim.object(drop_off).is_some_and(|o| o.kind == Tunnel),
                        "Supplies go to a Tunnel"
                    );
                    self.supplied += amount;
                }
                SimEvent::Died {
                    id,
                    kind,
                    owner: Some(player),
                    ..
                } => {
                    *self.died.entry((player, kind)).or_default() += 1;
                    if player == SYNDICATE && kind == Tunnel {
                        self.lost.push((tick, id));
                    }
                }
                SimEvent::Removed(id) if self.network.contains(&id) => self.stranded.push((tick, id)),
                SimEvent::PlayerDefeated { player } if player == SYNDICATE => self.defeated = Some(tick),
                _ => {}
            }
        }
    }
}

impl Script for Players {
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.close(sim);
        if sim.tick() == 0 {
            let at = |id| sim.object(id).and_then(|o| Some((o.kind, o.placement?.origin)));
            let patch = (ObjectKind::CrystalPatch, TilePos::new(2, 7));
            assert_eq!(at(PATCH), Some(patch), "dev_skirmish's patch on (2, 7)");
            let station = (ObjectKind::SupplyDeliveryStation, TilePos::new(31, 23));
            assert_eq!(at(STATION), Some(station), "dev_skirmish's station on (31, 23)");
            self.first = units_of(sim, SYNDICATE).iter().find(|o| o.kind == Tunnel).map(|o| o.id);
        }
        let tick = sim.tick() + 1;
        let commands = self.skirmish.commands(sim);
        // The gate takes each command as the ones before it in the step left the game, and the check here sees the
        // game before the step: a constructor that places its ready building takes its next Construct in the same
        // step (`BuildOrder`, its module docs), which the check can't see.
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
            match sim.validate(c) {
                Ok(()) => {}
                Err(Reject::ConstructorBusy(id)) if placing.contains(&id) => {}
                Err(reject) => panic!("tick {tick}: the gate refuses {c:?}: {reject:?}"),
            }
        }
        self.open = [SYNDICATE, GDO]
            .map(|player| (player, Books::open(sim, player), self.skirmish.spent_in(player, tick)))
            .into();
        self.supplies = Some((SupplyBooks::open(sim, SYNDICATE), self.skirmish.supplies_spent_in(tick)));
        self.network = sim.network(SYNDICATE).map(|held| held.object.id).collect();
        commands
    }

    fn end(mut self: Box<Self>, sim: &Sim) {
        self.close(sim);
        let versus = self.skirmish.gdo().is_some();
        let made = |player, kind| self.made.get(&(player, kind)).copied().unwrap_or(0);
        let died = |player, kind| self.died.get(&(player, kind)).copied().unwrap_or(0);
        let forward = (self.skirmish.syndicate().done().iter())
            .find_map(|&(_, done)| match done {
                Done::Eject { tunnel, .. } => Some(tunnel),
                _ => None,
            })
            .expect("the second Tunnel's builder was ejected by it");
        let out_forward = |kind| self.out.get(&(forward, kind)).copied().unwrap_or(0);
        println!(
            "== {} to tick {}: made {:?}; built {:?}; out {:?}; player 1's Agents delivered {} Space Crystals and {} \
             Supplies; died {:?}; Tunnels lost {:?}; stranded {:?}; player 1 defeated in tick {:?}",
            if versus {
                "syndicate_vs_gdo"
            } else {
                "syndicate_opening"
            },
            sim.tick(),
            self.made,
            self.built,
            self.out,
            self.delivered,
            self.supplied,
            self.died,
            self.lost,
            self.stranded,
            self.defeated,
        );
        assert_eq!(made(SYNDICATE, Agent), AGENTS, "the Hideout trained the Agents");
        assert!(self.delivered > 0, "player 1's Agents delivered Space Crystals");
        assert_eq!(
            (self.built.get(&Tunnel), self.built.get(&OpticsLab)),
            (Some(&1), Some(&1)),
            "the second Tunnel and the Optics Lab"
        );
        let site = sim.object(forward).and_then(|o| o.placement).map(|p| p.origin);
        assert!(
            site.is_none_or(|site| site == TUNNEL_SITE),
            "the second Tunnel is on its site"
        );
        assert_eq!(out_forward(Agent), 1, "its builder came out of it");
        assert!(self.supplied >= 1, "and gathered Supplies");
        if !versus {
            assert!(self.died.is_empty(), "nothing died: {:?}", self.died);
            return;
        }
        // The Guards: trained behind the Optics Lab, some out at the second Tunnel, the rest kept in the network.
        assert_eq!(made(SYNDICATE, Guard), GUARDS, "the Hideout trained the Guards");
        assert_eq!(out_forward(Guard), GUARDS_OUT, "Guards came out at the second Tunnel");
        assert!(made(GDO, Peacekeeper) > 0, "player 2's Peacekeepers");
        // The fight: units die on either side.
        assert!(
            died(SYNDICATE, Guard) > 0 && died(GDO, Peacekeeper) > 0,
            "Guards and Peacekeepers died"
        );
        // The second Tunnel falls first, then the first, the last: player 1 is defeated in that tick
        // (`factions/syndicate/tunnels.md > Losing Tunnels`, "Defeat").
        let lost: Vec<Option<ObjectId>> = self.lost.iter().map(|&(_, tunnel)| Some(tunnel)).collect();
        assert_eq!(
            lost,
            [Some(forward), self.first],
            "the Tunnels lost: the second, then the first"
        );
        let last = self.lost[1].0;
        assert_eq!(
            self.defeated,
            Some(last),
            "player 1 is defeated as its last Tunnel falls"
        );
        // `factions/syndicate/tunnels.md > Losing Tunnels`, "Stranded units": "any unit inside the Tunnel Network that
        // can no longer exit through any remaining Tunnel [...] is destroyed": the Guards that stayed in the network go
        // with the last Tunnel.
        let stranded: Vec<Tick> = self.stranded.iter().map(|&(tick, _)| tick).collect();
        assert_eq!(
            stranded,
            vec![last; GUARDS - GUARDS_OUT],
            "the Guards in the network are stranded"
        );
    }
}
