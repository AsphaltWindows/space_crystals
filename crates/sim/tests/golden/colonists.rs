//! The Colonists' golden games (slice M3-K1): `dev_skirmish` from its real start, played by script as
//! `sc_sim::testing::colonists::skirmish` plays it (its module docs), through the gate:
//! - [`opening`]: player 1 alone, gathering, training Drones, deploying, warping in a Bio Portal, training Sabre
//!   Legionnaires and converting one;
//! - [`versus_gdo`]: the same opening against player 2's GDO, whose Peacekeepers come for the Colonists' field, and
//!   the Colonists' Legionnaires, converted to Rifles, wait for them.
//!
//! Both players' books balance every step, the Drones' deliveries included (`sc_sim::testing::build_order::Books`),
//! and every command the scripts give is one the gate takes.

use std::collections::BTreeMap;

use sc_sim::testing::build_order::Books;
use sc_sim::testing::colonists::skirmish::{self, COLONISTS, GDO, SITES, Skirmish};
use sc_sim::{
    Command, ConstructionCommand, ObjectId, ObjectKind, PlayerCommand, PlayerId, Reject, Scenario, Sim, SimEvent, Tick,
    TilePos,
};

use super::harness::{Game, Script};
use super::units_of;

use ObjectKind::{Beacon, BioPortal, Drone, RifleLegionnaire as Rifle, SabreLegionnaire as Sabre};

/// [`opening`]'s last tick, 1:50: the Bio Portal has trained two Sabres, and the first has converted.
const OPENING_END: Tick = 1760;
/// [`versus_gdo`]'s last tick, 3:00.
const VERSUS_END: Tick = 2880;
/// [`opening`]'s camera starts over this tile: player 1's base, its field and its Bio Portal.
const OPENING_CAMERA: TilePos = TilePos::new(13, 7);
/// [`versus_gdo`]'s: player 1's base and where the fight is.
const VERSUS_CAMERA: TilePos = TilePos::new(16, 10);

/// `skirmish::scenario`, with player 1's camera over `camera`, where the game plays out, for watching it in the
/// client (and player 2's over its image: the map is symmetric).
fn scenario(camera: TilePos) -> Scenario {
    let mut scenario = skirmish::scenario();
    scenario.map.slots[0].camera_start = Some(camera.center());
    scenario.map.regenerate();
    scenario
}

/// The Colonists' opening (`Skirmish::opening`): player 1 trains Drones and, from its first Beacon, a Bio Portal that
/// trains two Sabre Legionnaires; the first converts to a Rifle Legionnaire a second after it comes out. Player 2
/// does nothing. At the end, 1:50, player 1 has its Bio Portal, a Sabre and a Rifle Legionnaire, and its Drones.
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
            let own = |kind| units_of(sim, COLONISTS).iter().filter(|o| o.kind == kind).count();
            assert_eq!(own(BioPortal), 1, "the Bio Portal");
            let transforming = units_of(sim, COLONISTS).iter().any(|o| o.is_transforming());
            assert!(!transforming, "nothing of player 1's transforms");
            assert_eq!(
                (own(Sabre), own(Rifle)),
                (1, 1),
                "a Sabre, and the Rifle Legionnaire it converted to"
            );
            assert!(own(Drone) >= 10, "player 1's Drones: {}", own(Drone));
        },
    }
}

/// The Colonists against the GDO (`Skirmish::versus_gdo`): player 1 plays the opening on, with eight Sabres, each
/// converted to a Rifle Legionnaire, waiting east of its Bio Portal; player 2 trains ten Peacekeepers and sends them
/// to player 1's field in two waves of AttackMove. The Colonists' Shields take hits, the Colonists hit back, and
/// units die on either side; neither player is defeated by the end, 3:00.
pub(crate) fn versus_gdo() -> Game {
    Game {
        scenario: scenario(VERSUS_CAMERA),
        commands: Vec::new(),
        script: Some(Box::new(Players::new(Skirmish::versus_gdo()))),
        end: VERSUS_END,
        during: |_, sim| {
            assert!(sim.outcome().is_none(), "tick {}: the game ended", sim.tick());
        },
        expect: |sim| {
            for player in [COLONISTS, GDO] {
                assert!(!sim.player(player).unwrap().is_defeated(), "player {}", player.0);
            }
        },
    }
}

/// The players of a Colonists golden game, with their books and what they saw.
struct Players {
    skirmish: Skirmish,
    /// The books of the step under way, each player's, with what its orders paid.
    open: Vec<(PlayerId, Books, u32)>,
    /// What player 1's Drones delivered.
    delivered: u32,
    /// The units each player's producers made, by kind.
    made: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// The transformations completed, by the kind they made.
    completed: BTreeMap<ObjectKind, usize>,
    /// Deploys that failed.
    failed: usize,
    /// Sabres converted to Rifle Legionnaires.
    converted: usize,
    /// Hits on player 1's Shields, and hits by player 1's units.
    shielded: usize,
    struck: usize,
    /// The objects each player lost.
    died: BTreeMap<PlayerId, usize>,
}

impl Players {
    fn new(skirmish: Skirmish) -> Self {
        Self {
            skirmish,
            open: Vec::new(),
            delivered: 0,
            made: BTreeMap::new(),
            completed: BTreeMap::new(),
            failed: 0,
            converted: 0,
            shielded: 0,
            struck: 0,
            died: BTreeMap::new(),
        }
    }

    /// Closes the books of the step that just ran, which must balance, and notes what happened in it.
    fn close(&mut self, sim: &Sim) {
        for (player, books, spent) in std::mem::take(&mut self.open) {
            let entry = books.close(sim, spent);
            if player == COLONISTS {
                self.delivered += entry.delivered;
            }
        }
        let events = sim.events();
        // An object's owner, if it died in the step too.
        let owner = |id: ObjectId| -> Option<PlayerId> {
            let died = || {
                events.iter().find_map(|e| match *e {
                    SimEvent::Died { id: dead, owner, .. } if dead == id => owner,
                    _ => None,
                })
            };
            sim.object(id).and_then(|o| o.owner).or_else(died)
        };
        for e in events {
            match *e {
                SimEvent::UnitProduced { unit, .. } => {
                    if let Some(o) = sim.object(unit)
                        && let Some(player) = o.owner
                    {
                        *self.made.entry((player, o.kind)).or_default() += 1;
                    }
                }
                SimEvent::TransformCompleted { kind, .. } => *self.completed.entry(kind).or_default() += 1,
                SimEvent::DeployFailed { .. } => self.failed += 1,
                SimEvent::KindChanged {
                    from: Sabre, to: Rifle, ..
                } => self.converted += 1,
                SimEvent::Hit {
                    attacker,
                    target,
                    shield_damage,
                    ..
                } => {
                    self.shielded += usize::from(shield_damage > 0.0 && owner(target) == Some(COLONISTS));
                    self.struck += usize::from(owner(attacker) == Some(COLONISTS));
                }
                SimEvent::Died {
                    owner: Some(player), ..
                } => *self.died.entry(player).or_default() += 1,
                _ => {}
            }
        }
    }
}

impl Script for Players {
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.close(sim);
        if sim.tick() == 0 {
            let patch = sim.object(skirmish::PATCH).and_then(|o| o.placement);
            let origin = patch.map(|p| (p.origin.x, p.origin.y));
            assert_eq!(origin, Some((3, 3)), "dev_skirmish's patch on (3, 3)");
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
        self.open = [COLONISTS, GDO]
            .map(|player| (player, Books::open(sim, player), self.skirmish.spent_in(player, tick)))
            .into();
        commands
    }

    fn end(mut self: Box<Self>, sim: &Sim) {
        self.close(sim);
        let versus = self.skirmish.gdo().is_some();
        let deploys: Vec<_> = (self.skirmish.colonists().deploys().iter())
            .map(|d| (d.tick, d.origin.x, d.origin.y))
            .collect();
        println!(
            "== {} to tick {}: made {:?}; transformed {:?}; player 1's Drones delivered {}; deploys {deploys:?}; hits \
             on player 1's Shields {}, player 1's hits {}, lost {:?}",
            if versus {
                "colonists_vs_gdo"
            } else {
                "colonists_opening"
            },
            sim.tick(),
            self.made,
            self.completed,
            self.delivered,
            self.shielded,
            self.struck,
            self.died,
        );
        assert!(self.delivered > 0, "player 1's Drones delivered");
        let first = self.skirmish.colonists().deploys().first().map(|d| d.origin);
        assert_eq!(first, Some(SITES[0]), "a Drone deployed, at the first site");
        assert!(
            self.completed.get(&Beacon).is_some_and(|&n| n >= 1),
            "a Beacon deployed"
        );
        assert_eq!(self.completed.get(&BioPortal), Some(&1), "the Bio Portal warped in");
        let converted = if versus {
            self.converted >= 1
        } else {
            self.converted == 1
        };
        assert!(converted, "Sabres converted: {}", self.converted);
        let sabres = self.made.get(&(COLONISTS, Sabre)).copied().unwrap_or(0);
        assert!(sabres >= 2, "Sabres trained: {sabres}");
        if !versus {
            // Nothing stands in the Drones' way (in a fight, a Peacekeeper on a site can fail a Deploy).
            assert_eq!(self.failed, 0, "no Deploy failed");
            return;
        }
        let peacekeepers = self.made.get(&(GDO, ObjectKind::Peacekeeper)).copied().unwrap_or(0);
        assert!(peacekeepers > 0, "player 2's Peacekeepers");
        // The fight: the Colonists' Shields take hits (`systems/combat.md > Shields`), the Colonists hit back, and
        // units die on either side.
        assert!(self.shielded > 0, "the Colonists' Shields took hits");
        assert!(self.struck > 0, "the Colonists hit back");
        let lost = |player| self.died.get(&player).copied().unwrap_or(0);
        assert!(lost(COLONISTS) > 0 && lost(GDO) > 0, "units died on either side");
    }
}
