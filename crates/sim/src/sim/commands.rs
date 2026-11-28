//! The commands phase (`ARCHITECTURE.md > Commands, events and replays`): each player command, once the gate has
//! checked it ([`Sim::validate`]), applied; an order only fills the units' queues (`systems/controls/commands.md >
//! Unit Command`), which phase 5a then starts.

use super::Sim;
use super::gate::Checked;
use crate::augmentation::{self, Research};
use crate::command::{Command, DebugCommand, PlayerCommand};
use crate::ids::ObjectId;
use crate::orders::{Order, OrderGroup, OrderKind, Target, destinations};

impl Sim {
    /// `ARCHITECTURE.md > Commands, events and replays`: every command passes the gate ([`Sim::validate`]) first.
    /// A rejected one changes nothing; player input never panics the sim. The one rejection the sim reports is a
    /// Place the placement check refuses (plan decision T4, `Sim::report_refusal`).
    pub(super) fn apply_command(&mut self, cmd: &PlayerCommand) {
        let checked = match self.check(cmd) {
            Ok(checked) => checked,
            Err(reject) => return self.report_refusal(cmd, reject),
        };
        match checked {
            Checked::Order { units, kind, queue } => self.give_order(&units, kind, queue),
            Checked::Ability { units, ability } => self.use_ability(&units, ability),
            Checked::Spawn {
                kind,
                owner,
                pos,
                placement,
            } => {
                self.insert_object(kind, owner, pos, placement, None);
                // A debug spawn is the one way a unit comes into the game from nothing, between two grid syncs: the
                // first of its class builds its grid now (slice M5-Z2, `paths`), for the commands after it.
                self.cover_kind(kind);
            }
            Checked::Remove(id) => self.remove_objects(&[id]),
            Checked::Teleport { id, pos } => {
                if let Some(o) = self.objects.get_mut(&id) {
                    // No interpolation from the old position.
                    o.pos = pos;
                    o.prev_pos = pos;
                    // A path from the old position leads nowhere now.
                    if let Some(mover) = &mut o.mover {
                        mover.repath(self.tick);
                        // Nor does it arrive with the speed it had (`Mover::momentum`, slice M3-G).
                        mover.momentum = glam::Vec2::ZERO;
                    }
                }
            }
            Checked::AutoTargeting(on) => self.auto_targeting = on,
            Checked::SetResources { player, banks } => self.set_resources(player, banks),
            Checked::SetTier { id, tier } => self.set_tier(id, tier),
            Checked::Eject { tunnel, unit } => self.queue_eject(tunnel, unit, None),
            Checked::Expand {
                tunnel,
                kind,
                placement,
            } => self.start_expansion(cmd.player, tunnel, kind, placement),
            Checked::CancelOperation { tunnel } => self.cancel_operation(tunnel),
            Checked::Upgrade { tunnel, to, price } => self.start_upgrade(cmd.player, tunnel, to, price),
            Checked::Production { player, plan } => self.apply_production(player, plan),
            Checked::Construction { player, resolved } => self.apply_construction(player, resolved),
            Checked::Holding { buildings, command } => self.apply_holding(&buildings, command),
            Checked::Cast { caster, ability, at } => self.cast(caster, ability, at),
            Checked::SetAugmentation { id, augmentation } => {
                // (M5-N1a) Its HP follows its MaxHP (`augmentation::set`).
                let research = Research::new(&self.players);
                if let Some(o) = self.objects.get_mut(&id) {
                    augmentation::set(o, augmentation, research);
                }
            }
            // (M5-X1b) The next tick starts on the command's rules (`sim/rules.rs`).
            Checked::SetRules => {
                if let Command::Debug(DebugCommand::SetRules(rules)) = &cmd.command {
                    self.set_rules(rules);
                }
            }
        }
    }

    /// `systems/controls/commands.md > Unit Command`: one order for each of `units` (checked: unique existing
    /// units), sharing a new order group. A location order (Move to a point, AttackMove, Patrol) gives each unit of a
    /// group its own destination by the magic box (`> Group Destinations (Magic Box)`, `docs/movement.md > 6.4`),
    /// resolved here, at the click, and kept in the unit's order (design change 0030); a Shift-queued one builds the
    /// box where the units' earlier orders leave them (design change 0029). Orders only fill queues here; phase 5a
    /// starts them. An order given without Shift replaces the unit's, a Build among them refunded, and a Build is paid
    /// for as it is given (`sim/builds.rs`). (M5-E) A group's Build is one order, paid once (`sim/builds/shared.rs`).
    /// (M6-GH1) A Rebase reassigns each aircraft that takes it as it is given (`Sim::rebase`, plan decision T88).
    pub(super) fn give_order(&mut self, units: &[ObjectId], kind: OrderKind, queue: bool) {
        let group = OrderGroup(self.next_group);
        self.next_group = self.next_group.wrapping_add(1);
        let slots = match kind {
            // (M5-V2) A Reverse is a location-targeted movement order too (`OrderKind::Reverse`).
            OrderKind::Move(Target::Point(click))
            | OrderKind::AttackMove(click)
            | OrderKind::Patrol(click)
            | OrderKind::Reverse(click) => {
                // One player's units (the gate), sent by what that player knows of the ground.
                let owner = units.first().and_then(|id| self.objects.get(id)).and_then(|o| o.owner);
                if let Some(player) = owner {
                    self.nav.ensure_known(player);
                }
                destinations::assign(self.nav.planning(owner), &self.map, &self.objects, units, click, queue)
            }
            OrderKind::Move(Target::Object(_)) | OrderKind::Attack(_) | OrderKind::Hold | OrderKind::Stop => {
                vec![None; units.len()]
            }
            OrderKind::Gather(_) | OrderKind::DropOff(_) | OrderKind::Deploy { .. } => vec![None; units.len()],
            OrderKind::Enter(_) | OrderKind::Build { .. } => vec![None; units.len()],
            // Each unit fires at the point clicked (plan decision T47: no magic box).
            OrderKind::AttackGround(_) => vec![None; units.len()],
            // (M5-E) Each walks to its own spot beside the site (`crate::spot_work`); (M5-N8) a Repair's beside its
            // building, or after its vehicle.
            OrderKind::Assist(_) | OrderKind::Repair(_) => vec![None; units.len()],
            // (M5-N4b) Each channels where it stands; (M5-K) each locks from where it stands; (M6-GD) each launches
            // over itself.
            OrderKind::Shroud | OrderKind::TargetLock(_) | OrderKind::Launch => vec![None; units.len()],
            // (M6-GH1) Each flies to its Airfield (`orders::air_base`).
            OrderKind::Rebase(_) | OrderKind::ReturnToBase => vec![None; units.len()],
            // (M6-GA) Each deploys where it stands (`crate::vehicle_deploy`); (M6-ST1) each unloads round itself
            // (`crate::carry`).
            OrderKind::VehicleDeploy { .. } | OrderKind::UnloadAll => vec![None; units.len()],
        };
        if self.gives_shared_build(units, kind) {
            return self.give_shared_build(units, kind, queue, group);
        }
        for (id, slot) in units.iter().zip(slots) {
            // (M6-GH1) A Rebase assigns the aircraft as it is given (`Sim::rebase`): one the Airfield has no room for
            // any more gets no order.
            if let OrderKind::Rebase(airfield) = kind
                && !self.rebase(*id, airfield)
            {
                continue;
            }
            if !queue {
                self.drop_orders(*id);
            }
            let price = self.price_of_order(*id, kind);
            // A full queue drops the order for this unit only.
            let given = (self.objects.get_mut(id).and_then(|o| o.orders.as_mut()))
                .is_some_and(|orders| orders.give(Order { kind, group, slot }, queue));
            if let (true, Some((player, price))) = (given, price) {
                self.pay_build(player, price);
            }
        }
    }
}
