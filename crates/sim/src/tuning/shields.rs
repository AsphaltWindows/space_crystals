//! Shields (`systems/combat.md > Shields`; `crate::shields`). The MaxShields are the defs'.

tunables! {
    /// The tunables of shields (module docs).
    Shields;

    /// `systems/combat.md > Shields > ShieldRegenRate`: "4 Shields per second (0.25 per frame). Constant base rate
    /// shared by all Colonists units and buildings". A power of two: Shields that start whole stay exact as they
    /// regenerate.
    shield_regen_per_tick: f32 = 0.25,
}
