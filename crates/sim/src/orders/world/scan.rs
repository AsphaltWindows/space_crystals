//! The behaviours' scans for targets (`systems/controls/behaviors.md > BaseAutoTargeting`,
//! `systems/controls/behaviors.md > TurretAutonomousScanning`, `systems/combat.md > ValidTarget`) on the snapshot of
//! phase 5a ([`World`]): which enemies a scan reaches ([`Scan`]), and the best of them by the selection priority
//! ([`World::pick`]).

use glam::Vec2;

use super::{Unit, View, World};
use crate::combat::{domain_allows, inside_min_range, turret, within_range};
use crate::defs::{AttackAttrs, ObjectKind, TurretAttrs};
use crate::elevation::{self, Elevation};
use crate::ids::ObjectId;
use crate::math;
use crate::ruleset::rules;
use crate::units::gu;

/// Which enemies a scan takes. Never one inside the unit's MinRange (`systems/controls/behaviors.md >
/// BaseAutoTargeting > Targets inside MinRange`, design change 0182: auto-targeting "skips enemies inside the weapon's
/// MinRange and picks one it can hit").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Scan {
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`,
    /// `systems/controls/command_behaviors.md > AttackMovingToLocation` 2: ValidTargets within the unit's SightRange,
    /// beyond its MinRange.
    Sight,
    /// `systems/controls/command_behaviors.md > HoldingPosition` 3: ValidTargets within weapon range (Range, beyond
    /// MinRange).
    Weapon,
}

/// A ValidTarget within the reach of a scan, as the scan's own filter sees it ([`World::pick`]).
pub(crate) struct Candidate<'v> {
    pub view: &'v View,
    /// The unit's attack reaches it from where the unit stands: within Range, beyond MinRange
    /// (`combat::within_range`).
    pub in_range: bool,
}

/// The 45° step (`Orders::rotation_step`) of a rotation of `rotation` radians, 0 to π (either direction): 0 for 0° to
/// 45°, 1 for over 45° to 90°, 2 for over 90° to 135°, 3 for over 135° to 180°. Targets in one step count as needing
/// equal rotation. **The bounds**: the design's steps ("0–45°, 45–90°, 90–135°, 135–180°") share their bounds; a
/// placeholder reading takes each step up to its upper bound, so a target exactly 45° off is in the first step, with
/// those dead ahead.
pub(crate) fn rotation_step(rotation: f32) -> u8 {
    let bounds = [
        rules().tuning.orders.rotation_step,
        2.0 * rules().tuning.orders.rotation_step,
        3.0 * rules().tuning.orders.rotation_step,
    ];
    bounds.iter().filter(|&&bound| rotation > bound).count() as u8
}

impl World<'_> {
    /// Whether `view` is within the reach of `scan` from `unit`, the gap between their shapes
    /// (`systems/combat.md > Range Measurement`).
    /// - [`Scan::Sight`]: within the unit's SightRange, with the ElevationModifier (`core/vision.md >
    ///   ElevationModifier`), and beyond its MinRange (design change 0182). The design doesn't say how "within
    ///   SightRange" is measured for a scan; a placeholder reading: edge to edge, like Range.
    /// - [`Scan::Weapon`]: within Range and beyond MinRange (`combat::within_range`).
    pub(crate) fn within(&self, unit: &Unit, attrs: &AttackAttrs, scan: Scan, view: &View) -> bool {
        let gap = unit.shape().gap(&view.shape);
        Self::gap_within(unit.kind, attrs, scan, gap, self.elevation(unit), view)
    }

    /// [`World::within`] for a unit of `kind` on `source`, `gap` away from `view`.
    fn gap_within(kind: ObjectKind, attrs: &AttackAttrs, scan: Scan, gap: f32, source: Elevation, view: &View) -> bool {
        match scan {
            Scan::Sight => {
                !inside_min_range(attrs, gap)
                    && gap <= gu(elevation::sight_range(kind.def().sight_range, source, view.elevation))
            }
            Scan::Weapon => within_range(attrs, gap, source, view.elevation),
        }
    }

    /// The angle a unit at `pos` whose attack faces `facing` ([`Unit::aim_facing`]: its own facing, or its turret's)
    /// would turn to face `view`, radians, 0 to π (design change 0179: "unit facing for infantry, turret angle for
    /// turret units").
    fn rotation(pos: Vec2, facing: f32, view: &View) -> f32 {
        let to = view.pos - pos;
        if to == Vec2::ZERO {
            return 0.0;
        }
        math::wrap_angle(math::vector_angle(to) - facing).abs()
    }

    /// `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`
    /// (`systems/controls/behaviors.md > TurretAutonomousScanning`): the best ValidTarget within the reach of `scan`,
    /// if any, among those `admit` lets through:
    /// 1. **threatening** targets first, those whose own attack can target this unit's domain, and (M5-S2b) the
    ///    buildings with a unit inside whose attack, as fired from there, can (design change 0377, answering Q0139:
    ///    "**Buildings that fire through their occupants** [...] count as threatening while **any occupant's weapon**
    ///    [...] could attack this unit's domain"; [`World::with_occupants`]);
    /// 2. then the **least rotation** from the unit's facing, or its turret's for a unit with a turret (slice M5-U1,
    ///    [`Unit::aim_facing`]), in 45° steps (design change 0179, [`rotation_step`]): targets in one step count as
    ///    needing equal rotation;
    /// 3. then the **closest** (the gap between the shapes); then the lowest id, the fixed order of the remaining
    ///    ties.
    ///
    /// Enemy structures are ValidTargets too (`systems/combat.md > ValidTarget`, design change 0103), so units
    /// attack enemy buildings on their own; those that can't attack (all but a complete Laser Turret, slice M4-S1:
    /// `combat::threat`, and a manned Garrison, 0377) are non-threatening, and come after the targets that can shoot
    /// back. An enemy inside the
    /// unit's MinRange is never picked (design change 0182, [`Scan`]).
    ///
    /// Nothing while auto-targeting is off (`DebugCommand::SetAutoTargeting`).
    ///
    /// `facing_only`: a non-turret unit that can't turn in place engages only what it already faces
    /// (`systems/controls/command_behaviors.md > HoldingPosition` 4, within `tuning::combat::Combat::facing_tolerance`,
    /// the exact angle).
    // Inlined into each caller, whose `admit` it specializes on: the behaviours' hottest scan. Without the hint, how
    // the release build splits the crate into codegen units can leave a call to it out of line (slice M4-S1: the one
    // in `travel::march`, 3% of `behaviours_perf`'s battle, though nothing on the path had changed).
    #[inline]
    pub(crate) fn pick(
        &self,
        unit: &Unit,
        scan: Scan,
        facing_only: bool,
        admit: impl Fn(&Candidate) -> bool,
    ) -> Option<ObjectId> {
        let attrs = unit.attrs().filter(|_| self.auto_targeting)?;
        unit.owner?;
        let my_domain = unit.unit_def()?.base_def().domain;
        // Everything a scan can reach lies within the unit's longest range, plus the modifier, of the unit's shape:
        // its centre lies within that and the two half shapes along each axis, in the cells this box touches.
        let longest = unit.kind.def().sight_range.max(attrs.range) + rules().tuning.vision.elevation_modifier;
        let own_half = unit.kind.def().half_extents().unwrap_or(Vec2::ZERO).max_element();
        let radius = gu(longest) + own_half + self.candidate_half;
        let (x0, x1) = (
            self.cell_coord(unit.pos.x - radius, self.cells_wide),
            self.cell_coord(unit.pos.x + radius, self.cells_wide),
        );
        let (y0, y1) = (
            self.cell_coord(unit.pos.y - radius, self.cells_high),
            self.cell_coord(unit.pos.y + radius, self.cells_high),
        );
        let (shape, source, facing) = (unit.shape(), self.elevation(unit), unit.aim_facing());
        let mut best: Option<(bool, u8, f32, ObjectId)> = None;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let cell = (y * self.cells_wide + x) as usize;
                let range = self.cell_starts[cell] as usize..self.cell_starts[cell + 1] as usize;
                for &index in &self.candidates[range] {
                    let view = &self.views[index as usize];
                    // The cheap tests first: most candidates are the unit's own side.
                    if view.owner == unit.owner {
                        continue;
                    }
                    let gap = shape.gap(&view.shape);
                    if !Self::gap_within(unit.kind, attrs, scan, gap, source, view)
                        || !self.valid_target(unit, attrs, view)
                    {
                        continue;
                    }
                    let rotation = Self::rotation(unit.pos, facing, view);
                    if facing_only && rotation > rules().tuning.combat.facing_tolerance {
                        continue;
                    }
                    let in_range = within_range(attrs, gap, source, view.elevation);
                    if !admit(&Candidate { view, in_range }) {
                        continue;
                    }
                    let harmless = !view.threat.is_some_and(|threat| domain_allows(threat, my_domain));
                    let key = (harmless, rotation_step(rotation), gap, view.id);
                    let better = best.is_none_or(|b| {
                        key.0
                            .cmp(&b.0)
                            .then(key.1.cmp(&b.1))
                            .then(key.2.total_cmp(&b.2))
                            .then(key.3.cmp(&b.3))
                            .is_lt()
                    });
                    if better {
                        best = Some(key);
                    }
                }
            }
        }
        best.map(|(.., id)| id)
    }

    /// (M5-U1) `systems/controls/behaviors.md > TurretAutonomousScanning`: the target the turret `turret` of `unit`
    /// locks when it has none ("selects the best available target from ValidTarget enemies within weapon range and
    /// turret arc", Algorithm steps 1 to 3): the best by [`World::pick`]'s priority (threatening first, then the
    /// least rotation from the turret's facing in 45° steps, then the closest) among the enemies within Range and
    /// within the arc (`combat::turret::in_arc`), never one inside MinRange ([`Scan::Weapon`];
    /// `systems/controls/behaviors.md > BaseAutoTargeting > Targets inside MinRange`: auto-targeting "skips enemies
    /// inside the weapon's MinRange and picks one it can hit; with none, it does nothing"). None: the turret stays
    /// inactive (step 5).
    pub(crate) fn turret_pick(&self, unit: &Unit, turret: &TurretAttrs) -> Option<ObjectId> {
        let (pos, facing) = (unit.pos, unit.facing);
        self.pick(unit, Scan::Weapon, false, |c| {
            turret::in_arc(turret, facing, pos, c.view.pos)
        })
    }
}
