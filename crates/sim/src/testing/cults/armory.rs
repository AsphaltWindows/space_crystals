//! The Armory's part of a [`Cults`] line (`testing::cults`'s module docs, 3): the Recruits it takes, its rally point,
//! and the Militants and Gunners it trains (`factions/cults/structures/armory.md > Training`), with the player's own
//! commands through the gate.

use crate::augmentation::RecruitKind;
use crate::command::{Command, PlayerCommand};
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::production::ProductionCommand;
use crate::sim::Sim;
use crate::testing::build_order::Order;
use crate::units::Tick;

use super::{Cults, Done};

impl Cults {
    /// Whether the player's Armory takes one more idle Recruit: one for each Militant and Gunner the line trains.
    pub(super) fn armory_takes_more(&self) -> bool {
        self.entered < self.militants + self.gunners
    }

    /// What the Armory trains next: a Gunner while the line has Gunners left and the player owns a complete Gunworks
    /// (the recipe's TechPrerequisite, `factions/cults/structures/armory.md > Training`: "player owns at least one
    /// Gunworks"), else a Militant while it has Militants left; nothing once it has trained them all.
    fn next_trained(&self, sim: &Sim) -> Option<ObjectKind> {
        if self.gunners_trained < self.gunners && sim.requirement_met(self.player, ObjectKind::Gunworks) {
            return Some(ObjectKind::Gunner);
        }
        (self.trained < self.militants).then_some(ObjectKind::Militant)
    }

    /// The commands of the player's complete `armory` for tick `tick`, with `crystals` left in the bank: the line's
    /// rally point once, and one unit of a free Recruit inside ([`Self::next_trained`]) when the bank covers it and
    /// the gate takes it (`ProductionCommand::TrainFrom`, KindMenu\[Plain\]'s Q or W).
    pub(super) fn armory_commands(
        &mut self,
        sim: &Sim,
        tick: Tick,
        armory: ObjectId,
        crystals: u32,
        commands: &mut Vec<PlayerCommand>,
    ) {
        let player = self.player;
        if let Some(target) = self.armory_rally
            && self.rallied.insert(armory)
        {
            commands.push(PlayerCommand {
                player,
                command: Command::Production {
                    objects: vec![armory],
                    command: ProductionCommand::SetRally(target),
                },
            });
        }
        let Some(kind) = self.next_trained(sim) else {
            return;
        };
        let price = Order::new(ObjectKind::Armory, kind).recipe().cost;
        let train = PlayerCommand {
            player,
            command: Command::Production {
                objects: vec![armory],
                command: ProductionCommand::TrainFrom {
                    kind,
                    from: RecruitKind::Plain,
                },
            },
        };
        if price <= crystals && !sim.free_units(armory, RecruitKind::Plain).is_empty() && sim.validate(&train).is_ok() {
            commands.push(train);
            self.paid.push((tick, price));
            self.done.push((tick, Done::Trained { armory, kind, price }));
            match kind {
                ObjectKind::Gunner => self.gunners_trained += 1,
                _ => self.trained += 1,
            }
        }
    }
}
