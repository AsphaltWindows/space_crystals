//! (M5-X1b) The rules a game plays by (plan decision T66, `crate::ruleset`): its generation ([`Sim::rules`]), the
//! current one while it steps, and `DebugCommand::SetRules`, which replaces it at a tick boundary.
//!
//! - **A tick reads one generation.** As each tick starts, the game's rules become the current ones
//!   (`ruleset::select`), whatever another game stepped on before, so that every def and tunable read in the tick is
//!   theirs; a game starts on the built-in rules (`Sim::empty`), and a loaded one on its save's (`sim/load.rs`).
//! - **A SetRules applies as the next tick starts**, in command order: the gate passes rules of the game's structure
//!   only (`Ruleset::structural_diff`), their numbers finite (`Ruleset::non_finite`) and (M5-X2) within what the game
//!   is made for (`Ruleset::check`), the command installs them as a
//!   generation ([`Sim::set_rules`]; the last of a tick's wins), and the next tick starts on them
//!   ([`Sim::start_rules`]), with a [`SimEvent::RulesChanged`]. So the tick of the command plays out on the rules it
//!   started on, and its vision records the objects as those rules left them; the next one sees them follow.
//! - **Objects keep their state** but for what stays within a maximum that the rules give: HP and Shields follow
//!   their maxima the way a transformation's do, the damage taken staying (`transform::follow_max`, design change
//!   0110: never a rescale, and never a kill), energy stays within its MaxEnergy and (M6-GD) a stock within its max
//!   (`crate::stock`). What is under way goes on by the new numbers, which it reads as it goes (a production's
//!   remaining time, a refund at the new price); what was fixed as it started stays (a projectile's damage, a path
//!   planned, (M6-GD) a purchase's frames and a drone's battery).

use super::Sim;
use crate::augmentation::Research;
use crate::events::SimEvent;
use crate::object::Object;
use crate::ruleset::{self, Ruleset};
use crate::transform;
use crate::{energy, stock};

impl Sim {
    /// The rules the game plays by (module docs): the built-in ones ([`Ruleset::BUILTIN`]) unless a
    /// `DebugCommand::SetRules` replaced them, or a save of a game on others was loaded. The current ones while it
    /// steps, and after its latest tick until another game steps or starts.
    pub fn rules(&self) -> &'static Ruleset {
        self.rules.rules()
    }

    /// A `DebugCommand::SetRules` that passed the gate (module docs): its rules, installed as a generation, are those
    /// the next tick starts on; a later one of the tick replaces them.
    pub(super) fn set_rules(&mut self, rules: &Ruleset) {
        // The gate found a generation for them (`ruleset::installable`), and nothing installs one in between.
        if let Some(generation) = ruleset::install(rules) {
            self.next_rules = Some(generation);
        }
    }

    /// As a tick starts (module docs): the rules of the last tick's SetRules, if it had one that changes them, become
    /// the game's, every object on the map and held off it following its new maxima ([`follow`]), and the game's rules
    /// become the current ones.
    pub(super) fn start_rules(&mut self) {
        let Some(to) = self.next_rules.take().filter(|&to| to != self.rules) else {
            return ruleset::select(self.rules);
        };
        ruleset::select(self.rules);
        let research = Research::new(&self.players);
        let objects = self.objects.values().chain(self.held.values().map(|held| &held.object));
        let before: Vec<Maxima> = objects.map(|o| Maxima::of(o, research)).collect();
        self.rules = to;
        ruleset::select(to);
        let objects = (self.objects.values_mut()).chain(self.held.values_mut().map(|held| &mut held.object));
        for (o, before) in objects.zip(before) {
            follow(o, before, research);
        }
        self.events.push(SimEvent::RulesChanged);
    }
}

/// The maxima of an object that the rules give and its state stays within, as the current rules give them.
#[derive(Copy, Clone)]
struct Maxima {
    /// Its MaxHP with its Augmentations (`Object::max_hp_with`).
    hp: Option<f32>,
    /// Its MaxShields (`Object::max_shields`).
    shields: Option<f32>,
}

impl Maxima {
    fn of(o: &Object, research: Research) -> Maxima {
        Maxima {
            hp: o.max_hp_with(research),
            shields: o.max_shields(),
        }
    }
}

/// `o`, whose maxima were `before`, on the current rules (module docs): its HP and its Shields go as far as their
/// maxima do, the damage staying (`transform::follow_max`: never to 0), its energy stays within its MaxEnergy, and
/// (M6-GD) its stock within its max.
fn follow(o: &mut Object, before: Maxima, research: Research) {
    let after = Maxima::of(o, research);
    if let (Some(hp), Some(before), Some(after)) = (o.hp, before.hp, after.hp) {
        o.hp = Some(transform::follow_max(hp, before, after));
    }
    if let (Some(shields), Some(before), Some(after)) = (o.shields, before.shields, after.shields) {
        o.shields = Some(transform::follow_max(shields, before, after));
    }
    if let (Some(energy), Some(max)) = (o.energy, energy::max(o.kind)) {
        o.energy = Some(energy.min(max));
    }
    // (M6-GD) A stock holds at most its max (`crate::stock`).
    if let (Some(stock), Some(def)) = (&mut o.stock, stock::def_of(o.kind)) {
        stock.count = stock.count.min(def.max);
    }
}
