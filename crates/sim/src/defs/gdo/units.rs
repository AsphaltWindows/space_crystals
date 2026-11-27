//! The GDO's units — `factions/gdo/units/`, one module per design file: the Peacekeeper's two types, the Field
//! Engineer, the Rocketman, and since slice M6-GK the rest (plan decision T84): the Raptor, the Twister, the Prospector,
//! the Artillery Rig's two types, the Hornet, the Dragonfly, the Drone Operator with its drone, and the Supply Chopper.

pub(crate) mod artillery_rig;
pub(crate) mod dragonfly;
pub(crate) mod drone_operator;
pub(crate) mod field_engineer;
pub(crate) mod hornet;
pub(crate) mod peacekeeper;
pub(crate) mod prospector;
pub(crate) mod raptor;
pub(crate) mod rocketman;
pub(crate) mod supply_chopper;
pub(crate) mod twister;

/// (M6-GK) A GDO unit's stat block (`factions/gdo/units/*.md`): its name, Silhouette (w × h), UnitBase, MaxHP,
/// PointArmor and FullArmor, SightRange and UnitControlCost. It is a selectable, destructible, Groupable GDO unit
/// (every one of slice M6-GK's is), with no MaxShields and none of the other factions' costs. Its unit attributes.
#[cfg(test)]
#[expect(clippy::too_many_arguments, reason = "one argument per line of the stat block")]
fn assert_stat_block(
    kind: crate::defs::ObjectKind,
    name: &str,
    silhouette: (f32, f32),
    base: crate::defs::UnitBase,
    hp: f32,
    armor: (f32, f32),
    sight: f32,
    unit_control: u32,
) -> &'static crate::defs::UnitDef {
    use crate::defs::{Armor, Category};
    use crate::player::Faction;
    let def = kind.def();
    assert_eq!(
        (def.name, def.category, def.faction),
        (name, Category::Unit, Some(Faction::Gdo)),
        "{kind:?}"
    );
    assert_eq!(def.silhouette(), Some(glam::Vec2::from(silhouette)), "{kind:?}");
    let (point, full) = armor;
    assert_eq!(
        (def.max_hp, def.max_shields, def.armor, def.sight_range),
        (Some(hp), None, Armor { point, full }, sight),
        "{kind:?}"
    );
    assert_eq!(
        (def.selectable, def.destructible, def.groupable, def.needs_flat_ground),
        (true, true, true, false),
        "{kind:?}"
    );
    assert!(def.structure.is_none() && def.default_settings.is_none(), "{kind:?}");
    let unit = def.unit.as_ref().unwrap();
    assert_eq!(
        (
            unit.base,
            unit.unit_control_cost,
            unit.tunnel_space_cost,
            unit.beacon_capacity_cost
        ),
        (base, Some(unit_control), None, None),
        "{kind:?}"
    );
    unit
}

/// A unit's attack timings, `(AimDuration, FiringDuration, CooldownDuration, ReloadDuration)`, and the frames from one
/// attack to the next, their sum (as the Cults' `timings`).
#[cfg(test)]
fn cycle(attack: &crate::defs::AttackAttrs) -> ((u32, u32, u32, u32), u32) {
    let t = (
        attack.aim_ticks,
        attack.firing_ticks,
        attack.cooldown_ticks,
        attack.reload_ticks,
    );
    (t, t.0 + t.1 + t.2 + t.3)
}
