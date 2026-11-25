//! Crowds under normal collision, within what the design says a layout holds (`factions/cults/economy.md > Storage`:
//! "The best layout leaves a 1-tile gap between patch and Storage; each 2-tile face supports roughly 4–5 Recruits at
//! full efficiency"; "an efficient crystal field take[s] more Storages (3–4 for an 8-patch field)"). Each runs for 5
//! minutes with the jam check (plan §3.3 D): no Recruit goes 10 seconds without a delivery. More Recruits than that
//! on a face: `jam`.

use super::*;

/// One Recruit alone at a gap of 1 and of 2 tiles (`trips`): Space Crystals per second.
const ALONE: [f32; 2] = [32.0 / 24.0, 32.0 / 56.0];

/// Five minutes of `t`: the income once settled, and the jam check for every one of `recruits`.
fn five_minutes(mut t: TestSim, recruits: &[u32], what: &str) -> f32 {
    t.run_until(FIVE_MINUTES);
    let (longest, who) = longest_gap(&t, recruits, SETTLED, FIVE_MINUTES);
    let income = income(&t, SETTLED, FIVE_MINUTES);
    assert!(
        longest < JAM,
        "{what}: Recruit {who} delivered nothing for {longest} ticks; {income:.2} Space Crystals per second"
    );
    income
}

/// Plan §3.3 D: "4 and 5 Recruits on one 2-tile face". Two patches face the Storage's 2-tile face; the Recruits are
/// dealt to them in turn. At a gap of 1 tile four work at the rate of one alone, 5.33 Space Crystals per second, and
/// five at 92% of it, 6.14 (the fifth shares a patch's two lanes with two others); at 2 tiles, 2.24 and 2.74. So
/// "Moving a Storage one tile closer" gains 0.77 per Recruit with four (the design's "~0.86", from its formula) and
/// 3.4 for the face with five ("~+4 SC/s per face").
#[test]
fn four_and_five_recruits_on_one_two_tile_face_work_without_a_jam() {
    let mut incomes = Vec::new();
    for gap in [1, 2] {
        for count in [4, 5] {
            let (t, recruits) = face_at_work(count, gap, 2, 0);
            let income = five_minutes(t, &recruits, &format!("{count} Recruits, {gap} tiles"));
            let full = count as f32 * ALONE[gap as usize - 1];
            println!("{count} Recruits, a gap of {gap}: {income:.2} of {full:.2} Space Crystals per second");
            let least = if count == 4 { 0.97 } else { 0.90 };
            assert!(
                income >= least * full && income <= 1.01 * full,
                "{count} Recruits, {gap} tiles: {income} of {full}"
            );
            incomes.push(income);
        }
    }
    let closer = (incomes[0] - incomes[2]) / 4.0;
    assert!(
        (0.73..=0.99).contains(&closer),
        "{closer} per Recruit, the design's ~0.86"
    );
}

/// Plan §3.3 D: "10 Recruits on an 8-patch field with three Storages", and "A 5-minute soak has no jam". The Storages
/// stand a tile from the row; the Recruits are dealt to the patches, two of which get two. 12.9 Space Crystals per
/// second, 97% of ten working alone; every Storage takes loads.
#[test]
fn ten_recruits_work_an_eight_patch_field_with_three_storages_for_five_minutes() {
    let (mut t, recruits) = field_at_work(10, 1);
    t.run_until(FIVE_MINUTES);
    let (longest, who) = longest_gap(&t, &recruits, SETTLED, FIVE_MINUTES);
    let income = income(&t, SETTLED, FIVE_MINUTES);
    println!("10 Recruits, 8 patches, 3 Storages: {income:.2} Space Crystals per second, longest gap {longest}");
    assert!(longest < JAM, "Recruit {who} delivered nothing for {longest} ticks");
    assert!(income >= 0.95 * 10.0 * ALONE[0], "{income}");
    for storage in 9..=11 {
        assert!(deliveries(&t).iter().any(|d| d.2 == storage), "Storage {storage}");
    }
    let delivered: u32 = deliveries(&t).iter().map(|d| d.3).sum();
    assert_eq!(bank(&t), delivered);
    assert!(
        recruits
            .iter()
            .all(|&n| matches!(order(&t, n), Some(OrderKind::Gather(_))))
    );
}
