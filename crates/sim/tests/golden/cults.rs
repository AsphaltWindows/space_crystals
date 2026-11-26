//! The Cults' golden games (slice M5-L): `dev_skirmish` from its real start, played by script as
//! `sc_sim::testing::cults::skirmish` plays it (its module docs), through the gate:
//! - [`opening`]: player 1 alone. Its Recruits are dealt to the field, the patches a tile from the starting Storage
//!   first; two of them build a second Storage for the field's western column, three a second Recruitment Hall east
//!   of the hill, which claims its whole area, and three an Armory, into which four new Recruits go to come out as
//!   Militants and wait south-east of the base.
//! - [`versus_syndicate`]: player 1 builds the Armory first and trains eight Militants of the Recruits it has no seat
//!   for; six set off together for player 2's Tunnel, then the two others. Player 2's Syndicate gathers, trains two
//!   Agents, expands an Optics Lab and a Laser Turret and trains four Guards, which wait at its patch. The turret,
//!   popping up, kills six Militants before they destroy it; the last two destroy the Tunnel, which defeats player 2:
//!   the game is the Cults'.
//!
//! Both players' books balance every step: their Space Crystals, the workers' deliveries included
//! (`sc_sim::testing::build_order::Books`), and player 2's Supplies (`sc_sim::testing::syndicate::SupplyBooks`). Every
//! command the scripts give is one the gate takes.

use std::collections::BTreeMap;

use sc_sim::testing::build_order::Books;
use sc_sim::testing::cults::skirmish::{
    self, ARMORY_SITE, CULTS, GUARDS, HALL_SITE, MILITANTS, OPENING_MILITANTS, SYNDICATE, Skirmish,
};
use sc_sim::testing::syndicate::SupplyBooks;
use sc_sim::{ObjectKind, PlayerCommand, PlayerId, Scenario, Sim, SimEvent, Tick, TilePos};

use super::harness::{Game, Script};
use super::units_of;

use ObjectKind::{
    Agent, Armory, Guard, Hideout, LaserTurret, Militant, OpticsLab, Recruit, RecruitmentHall, Storage, Tunnel,
};

/// [`opening`]'s last tick, 2:45: the fourth Militant has come out of the Armory and reached its rally point.
const OPENING_END: Tick = 2640;
/// [`versus_syndicate`]'s last tick, 3:40.56: the game's end, as player 2's Tunnel falls and it is defeated. A replay
/// plays back in the client to where its game ended (`ARCHITECTURE.md > Replays and golden hashes`, "Playback"), so the
/// golden game ends there too, with its last checkpoint. A change that moves the end fails [`versus_syndicate`]'s
/// `expect`: set this to the tick it names.
const VERSUS_END: Tick = 3529;
/// [`opening`]'s camera starts over this tile: player 1's base, between its Halls.
const OPENING_CAMERA: TilePos = TilePos::new(14, 12);
/// [`versus_syndicate`]'s: player 2's Tunnel, where the Militants fight.
const VERSUS_CAMERA: TilePos = TilePos::new(52, 38);

/// `skirmish::scenario`, with player 1's camera over `camera`, where the game plays out, for watching it in the
/// client (and player 2's over its image: the map is symmetric).
fn scenario(versus: bool, camera: TilePos) -> Scenario {
    let mut scenario = skirmish::scenario(versus);
    scenario.map.slots[0].camera_start = Some(camera.center());
    scenario.map.regenerate();
    scenario
}

/// The Cults' opening (`Skirmish::opening`). Player 2 does nothing. At the end, 2:45, player 1 has its two Halls, the
/// second at 100% Effectiveness, its two Storages, the Armory and four Militants.
pub(crate) fn opening() -> Game {
    Game {
        scenario: scenario(false, OPENING_CAMERA),
        commands: Vec::new(),
        script: Some(Box::new(Players::new(Skirmish::opening()))),
        end: OPENING_END,
        during: |_, sim| {
            assert!(sim.outcome().is_none(), "tick {}: the game ended", sim.tick());
        },
        expect: |sim| {
            let own = |kind| units_of(sim, CULTS).iter().filter(|o| o.kind == kind).count();
            assert_eq!(
                (own(RecruitmentHall), own(Storage), own(Armory), own(Militant)),
                (2, 2, 1, OPENING_MILITANTS),
                "player 1's Halls, Storages, Armory and Militants"
            );
            assert!(
                units_of(sim, CULTS).iter().all(|o| o.is_operational()),
                "nothing of player 1's is unfinished"
            );
            let second = (units_of(sim, CULTS).into_iter())
                .find(|o| o.kind == RecruitmentHall && o.placement.is_some_and(|p| p.origin == HALL_SITE))
                .expect("the second Hall on its site");
            let state = sim.hall_state(second.id).expect("a Hall");
            assert_eq!(
                state.effectiveness_percent(),
                100,
                "the second Hall claims its whole area"
            );
        },
    }
}

/// The Cults against the Syndicate (`Skirmish::versus_syndicate`). Player 2 is defeated when its Tunnel, its only
/// one, falls: the game is over, the Cults', in its last tick.
pub(crate) fn versus_syndicate() -> Game {
    Game {
        scenario: scenario(true, VERSUS_CAMERA),
        commands: Vec::new(),
        script: Some(Box::new(Players::new(Skirmish::versus_syndicate()))),
        end: VERSUS_END,
        during: |_, sim| {
            assert!(
                !sim.player(CULTS).unwrap().is_defeated(),
                "tick {}: player 1 is defeated",
                sim.tick()
            );
            if let Some(outcome) = sim.outcome() {
                assert_eq!(outcome.winners, [CULTS], "the game is the Cults'");
            }
        },
        expect: |sim| {
            let ended = sim.outcome().map(|outcome| outcome.tick);
            assert_eq!(ended, Some(VERSUS_END), "the game ends in the golden's last tick");
            assert!(sim.player(SYNDICATE).unwrap().is_defeated(), "player 2 is defeated");
            assert!(units_of(sim, SYNDICATE).is_empty(), "player 2 owns nothing");
            // `factions/syndicate/tunnels.md > Losing Tunnels`: "every expansion in its Tunnel Area becomes neutral".
            for kind in [Hideout, OpticsLab] {
                let expansion = (sim.objects().find(|o| o.kind == kind)).unwrap_or_else(|| panic!("the {kind:?}"));
                assert_eq!(expansion.owner, None, "the {kind:?} is neutral");
            }
            let own = |kind| units_of(sim, CULTS).iter().filter(|o| o.kind == kind).count();
            assert!(own(Militant) > 0, "Militants of player 1's survive");
        },
    }
}

/// The players of a Cults golden game, with their books and what they saw.
struct Players {
    skirmish: Skirmish,
    /// The books of the step under way, each player's, with what its orders paid; player 2's Supplies too.
    open: Vec<(PlayerId, Books, u32)>,
    supplies: Option<(SupplyBooks, u32)>,
    /// What each player's workers delivered.
    delivered: BTreeMap<PlayerId, u32>,
    /// The units each player's producers made, by kind.
    made: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// The structures each player finished, by kind: sites and expansions.
    built: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// The objects each player lost, by kind.
    died: BTreeMap<(PlayerId, ObjectKind), usize>,
    /// Each player defeated, with its tick.
    defeated: Vec<(Tick, PlayerId)>,
}

impl Players {
    fn new(skirmish: Skirmish) -> Self {
        Self {
            skirmish,
            open: Vec::new(),
            supplies: None,
            delivered: BTreeMap::new(),
            made: BTreeMap::new(),
            built: BTreeMap::new(),
            died: BTreeMap::new(),
            defeated: Vec::new(),
        }
    }

    /// Closes the books of the step that just ran, which must balance, and notes what happened in it.
    fn close(&mut self, sim: &Sim) {
        for (player, books, spent) in std::mem::take(&mut self.open) {
            let entry = books.close(sim, spent);
            *self.delivered.entry(player).or_default() += entry.delivered;
        }
        if let Some((books, spent)) = self.supplies.take() {
            books.close(sim, spent);
        }
        let tick = sim.tick();
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
                SimEvent::SiteCompleted { site, .. }
                | SimEvent::SiteBuilt { site }
                | SimEvent::ExpansionCompleted { site, .. } => {
                    if let Some(o) = sim.object(site)
                        && let Some(player) = o.owner
                    {
                        *self.built.entry((player, o.kind)).or_default() += 1;
                    }
                }
                SimEvent::Died {
                    kind,
                    owner: Some(player),
                    ..
                } => *self.died.entry((player, kind)).or_default() += 1,
                SimEvent::PlayerDefeated { player } => self.defeated.push((tick, player)),
                _ => {}
            }
        }
    }
}

impl Script for Players {
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.close(sim);
        let tick = sim.tick() + 1;
        let commands = self.skirmish.commands(sim);
        for c in &commands {
            if let Err(reject) = sim.validate(c) {
                panic!("tick {tick}: the gate refuses {c:?}: {reject:?}");
            }
        }
        self.open = [CULTS, SYNDICATE]
            .map(|player| (player, Books::open(sim, player), self.skirmish.spent_in(player, tick)))
            .into();
        if self.skirmish.syndicate().is_some() {
            self.supplies = Some((SupplyBooks::open(sim, SYNDICATE), self.skirmish.supplies_spent_in(tick)));
        }
        commands
    }

    fn end(mut self: Box<Self>, sim: &Sim) {
        self.close(sim);
        let versus = self.skirmish.syndicate().is_some();
        let made = |player, kind| self.made.get(&(player, kind)).copied().unwrap_or(0);
        let built = |player, kind| self.built.get(&(player, kind)).copied().unwrap_or(0);
        let died = |player, kind| self.died.get(&(player, kind)).copied().unwrap_or(0);
        println!(
            "== {} to tick {}: made {:?}; built {:?}; delivered {:?}; died {:?}; defeated {:?}",
            if versus { "cults_vs_syndicate" } else { "cults_opening" },
            sim.tick(),
            self.made,
            self.built,
            self.delivered,
            self.died,
            self.defeated,
        );
        assert!(
            self.delivered.get(&CULTS).is_some_and(|&d| d > 0),
            "player 1's Recruits delivered"
        );
        assert!(made(CULTS, Recruit) > 0, "the Halls made Recruits");
        let armory = (sim.objects()).find(|o| o.kind == Armory && o.placement.is_some_and(|p| p.origin == ARMORY_SITE));
        assert!(armory.is_some(), "the Armory on its site");
        assert_eq!(built(CULTS, Armory), 1, "player 1 built the Armory");
        if !versus {
            assert_eq!(
                (built(CULTS, Storage), built(CULTS, RecruitmentHall)),
                (1, 1),
                "the second Storage and the second Hall"
            );
            assert_eq!(
                made(CULTS, Militant),
                OPENING_MILITANTS,
                "the Armory trained the Militants"
            );
            assert_eq!(self.died, BTreeMap::new(), "nothing died");
            return;
        }
        assert_eq!(made(CULTS, Militant), MILITANTS, "the Armory trained the Militants");
        assert_eq!(
            (made(SYNDICATE, Agent), made(SYNDICATE, Guard)),
            (skirmish::AGENTS, GUARDS),
            "player 2's Hideout trained its Agents and Guards"
        );
        assert_eq!(
            (built(SYNDICATE, OpticsLab), built(SYNDICATE, LaserTurret)),
            (1, 1),
            "player 2's Tunnel expanded the Optics Lab and the Laser Turret"
        );
        assert!(died(CULTS, Militant) > 0, "the turret fought back");
        assert_eq!(
            (died(SYNDICATE, LaserTurret), died(SYNDICATE, Tunnel)),
            (1, 1),
            "the Militants destroyed the turret and the Tunnel"
        );
        assert_eq!(
            self.defeated,
            [(VERSUS_END, SYNDICATE)],
            "player 2 is defeated at the end"
        );
    }
}
