use space_crystals::game::types::factions::{
    GdoPlayerResources, SyndicatePlayerResources,
    CultsPlayerResources, ColonistsPlayerResources,
};

/// QA Step 7 [auto]: For GDO, place generators and consumers and verify Power current/total
/// reflects the net sum correctly.
/// Tests the GdoPlayerResources power calculation logic.
#[test]
fn step_7_gdo_power_net_sum() {
    let mut res = GdoPlayerResources::default();

    // Default: 100 generated, 0 consumed
    assert_eq!(res.power_generated, 100, "Default power generated should be 100");
    assert_eq!(res.power_consumed, 0, "Default power consumed should be 0");
    assert_eq!(res.current_power(), 100, "Net power should be 100");

    // Simulate adding consumers (buildings)
    res.power_consumed = 60;
    assert_eq!(res.current_power(), 40, "After consuming 60, net power should be 40");

    // Simulate adding more generators
    res.power_generated = 200;
    assert_eq!(res.current_power(), 140, "200 generated - 60 consumed = 140");

    // Deficit scenario
    res.power_generated = 30;
    res.power_consumed = 100;
    assert_eq!(res.current_power(), -70, "30 generated - 100 consumed = -70 (deficit)");

    // Power ratio in deficit
    let ratio = res.power_ratio();
    assert!((ratio - 0.3).abs() < f32::EPSILON, "Power ratio should be 0.3 in deficit, got {}", ratio);
}

/// QA Step 8 [auto]: Verify no faction shows resources belonging to another faction.
/// Each faction resource struct has only its own resource fields — enforced by Rust type system.
#[test]
fn step_8_no_cross_faction_resources() {
    // GDO has: space_crystals, supplies, power_generated, power_consumed, unit_control
    let gdo = GdoPlayerResources::default();
    assert_eq!(gdo.space_crystals, 500);
    assert_eq!(gdo.supplies, 100);
    assert!(gdo.power_generated > 0, "GDO should have power_generated");

    // Syndicate has: space_crystals, supplies, tunnel_space — NO power fields
    let syn = SyndicatePlayerResources::default();
    assert_eq!(syn.space_crystals, 500);

    // Cults has: space_crystals, unit_control — NO supplies, NO power
    let cults = CultsPlayerResources::default();
    assert_eq!(cults.space_crystals, 500);

    // Colonists has: space_crystals, alloys, essence, conduits, beacon_capacity
    let col = ColonistsPlayerResources::default();
    assert_eq!(col.space_crystals, 500);

    // All factions share space_crystals but have unique faction-specific resources.
    // Type system prevents accessing fields that don't belong to a faction.
    // E.g., syn.power_generated would be a compile error.
}
