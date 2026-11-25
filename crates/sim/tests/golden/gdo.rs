//! The GDO's golden game (slice M2-J1): `dev_skirmish` from its real start, both players GDO, playing by
//! script ([`Opening`]).

use super::*;

/// `dev_skirmish` from its real start, both players GDO, playing by script ([`Opening`]; slice M2-J1). Player 1 plays
/// the fastest clean Barracks (Plate, Generator, Generator, Barracks: `tests/economy_oracle/`), trains ten
/// Peacekeepers and rallies them to the edge of player 2's crystal field ([`RALLY`]), where they shoot what they find:
/// player 2's plates first. Player 2 plates its field and builds Generators. Both players' books balance every step;
/// at least one of player 2's plates is destroyed, its patch is mined no more and player 2's income falls; player 2
/// keeps its other structures, so it isn't defeated and the game goes on (`core/game_rules.md > Defeat Condition`) to
/// [`GDO_OPENING_END`].
pub(crate) fn gdo_opening() -> Game {
    let p1 = BuildOrder::new(P1).track(
        [
            Order::new(EXTRACTION_FACILITY, PLATE),
            Order::new(DEPLOYMENT_CENTER, GENERATOR),
            Order::new(DEPLOYMENT_CENTER, GENERATOR),
            Order::new(DEPLOYMENT_CENTER, BARRACKS),
        ]
        .into_iter()
        .chain([Order::new(BARRACKS, PK); 10]),
    );
    let p2 = BuildOrder::new(P2)
        .track([Order::new(EXTRACTION_FACILITY, PLATE); 6])
        .track([Order::new(DEPLOYMENT_CENTER, GENERATOR); 6]);
    Game {
        scenario: scenarios::dev_skirmish(),
        commands: Vec::new(),
        script: Some(Box::new(Opening {
            p1,
            p2,
            rallied: false,
            open: Vec::new(),
            plates: BTreeMap::new(),
            lost: BTreeMap::new(),
            destroyed: Vec::new(),
            mined: Vec::new(),
        })),
        end: GDO_OPENING_END,
        during: |_, sim| {
            assert!(sim.outcome().is_none(), "tick {}: the game ended", sim.tick());
        },
        expect: |sim| {
            let p2 = sim.player(P2).unwrap();
            assert!(!p2.is_defeated(), "player 2 isn't defeated");
            let structures = sim
                .objects()
                .filter(|o| o.owner == Some(P2) && o.def().structure.is_some())
                .count();
            assert!(structures > 0, "player 2 keeps structures");
            let peacekeepers = units_of(sim, P1).iter().filter(|o| o.kind == PK).count();
            assert!(peacekeepers > 0, "player 1's Peacekeepers");
        },
    }
}

const EXTRACTION_FACILITY: ObjectKind = ObjectKind::ExtractionFacility;
const DEPLOYMENT_CENTER: ObjectKind = ObjectKind::DeploymentCenter;
const PLATE: ObjectKind = ObjectKind::ExtractionPlate;
const GENERATOR: ObjectKind = ObjectKind::Generator;
const BARRACKS: ObjectKind = ObjectKind::Barracks;
/// [`gdo_opening`]'s last tick, 2:45: the Peacekeepers have destroyed four plates, and the Deployment Center, which
/// they shoot next, still stands.
const GDO_OPENING_END: Tick = 2640;
/// [`gdo_opening`]: player 1's rally point, on open ground at the south-east corner of player 2's crystal field
/// (`dev_skirmish`'s field of the second slot: patches (56, 45) to (59, 45), (60, 44), (61, 40) to (61, 42)).
const RALLY: TilePos = TilePos::new(60, 46);

/// [`gdo_opening`]'s players.
struct Opening {
    /// Player 1's line: the fastest clean Barracks, then ten Peacekeepers.
    p1: BuildOrder,
    /// Player 2's: its field plated, and Generators.
    p2: BuildOrder,
    /// Whether player 1 has set its Barracks' rally point.
    rallied: bool,
    /// The books of the step under way, each player's, with what its orders paid.
    open: Vec<(PlayerId, Books, u32)>,
    /// Player 2's plates, with the patch each covers.
    plates: BTreeMap<ObjectId, ObjectId>,
    /// The patches of player 2's destroyed plates that no plate covers again, with the tick their plate was
    /// destroyed in and the crystals they held then.
    lost: BTreeMap<ObjectId, (Tick, u32)>,
    /// Each of player 2's plates destroyed: the tick, and its patch.
    destroyed: Vec<(Tick, ObjectId)>,
    /// What player 2's plates mined, step by step: `mined[i]` in tick `i + 1`.
    mined: Vec<u32>,
}

impl Opening {
    /// Closes the books of the step that just ran, which must balance.
    fn close(&mut self, sim: &Sim) {
        for (player, books, spent) in std::mem::take(&mut self.open) {
            let entry = books.close(sim, spent);
            if player == P2 {
                self.mined.push(entry.mined);
            }
        }
    }
}

/// The crystals patch `id` holds, if it is one.
fn remaining(sim: &Sim, id: ObjectId) -> Option<u32> {
    match sim.object(id)?.resource.as_ref()? {
        sc_sim::ResourceState::CrystalPatch { remaining } => Some(*remaining),
        _ => None,
    }
}

impl Script for Opening {
    fn commands(&mut self, sim: &Sim) -> Vec<PlayerCommand> {
        self.close(sim);
        // A destroyed plate's patch stays, uncovered, with what it held (`factions/gdo/economy.md > ExtractionPlate >
        // On destruction`), and nothing mines it until a plate covers it again.
        for (&patch, &(since, left)) in &self.lost {
            assert_eq!(
                remaining(sim, patch),
                Some(left),
                "tick {}: patch #{}, whose plate was destroyed in tick {since}",
                sim.tick(),
                patch.0
            );
        }
        for e in sim.events() {
            if let SimEvent::Died {
                id,
                kind: PLATE,
                owner: Some(P2),
                ..
            } = e
            {
                let patch = self.plates[id];
                let left = remaining(sim, patch)
                    .filter(|&n| n > 0)
                    .expect("the patch stays, uncovered");
                self.lost.insert(patch, (sim.tick(), left));
                self.destroyed.push((sim.tick(), patch));
            }
        }
        self.plates = sim
            .objects()
            .filter(|o| o.owner == Some(P2))
            .filter_map(|o| Some((o.id, o.extractor?.patch)))
            .collect();
        let covered: Vec<ObjectId> = self.plates.values().copied().collect();
        self.lost.retain(|patch, _| !covered.contains(patch));

        let tick = sim.tick() + 1;
        let mut commands = self.p1.commands(sim);
        if !self.rallied
            && let Some(barracks) = sim.objects().find(|o| o.owner == Some(P1) && o.kind == BARRACKS)
        {
            commands.push(PlayerCommand {
                player: P1,
                command: Command::Production {
                    objects: vec![barracks.id],
                    command: ProductionCommand::SetRally(Target::Point(RALLY.center())),
                },
            });
            self.rallied = true;
        }
        commands.extend(self.p2.commands(sim));
        let paid = |line: &BuildOrder| line.issued().iter().filter(|i| i.tick == tick).map(|i| i.cost).sum();
        self.open = vec![
            (P1, Books::open(sim, P1), paid(&self.p1)),
            (P2, Books::open(sim, P2), paid(&self.p2)),
        ];
        commands
    }

    fn end(mut self: Box<Self>, sim: &Sim) {
        self.close(sim);
        let trained = self.p1.issued().iter().filter(|i| i.order.kind == PK).count();
        assert_eq!(trained, 10, "player 1 trained ten Peacekeepers");
        let Some(&(first, _)) = self.destroyed.first() else {
            panic!("player 1 destroyed none of player 2's plates");
        };
        // Player 2's income: the 9 seconds (3 cycles) before the first plate was lost, and the last 9.
        let window = |end: Tick| -> u32 { self.mined[(end - 144) as usize..end as usize].iter().sum() };
        let (before, last) = (window(first - 1), window(sim.tick()));
        assert!(
            last < before,
            "player 2 mined {last} in the last 9 s, {before} before it lost a plate"
        );
    }
}
