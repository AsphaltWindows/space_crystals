//! Construction times (`docs/architecture/economy.md > Economy > Recipe timing`): at full power, and under the power
//! penalty, which slows the Extraction Facility and never the Deployment Center (design change 0089).

use super::*;

/// The plan's acceptance, times at full power (`docs/architecture/economy.md > Economy > Recipe timing`): ordered in tick 1, a
/// Barracks is ready at the end of tick 160, a Generator of tick 240, an Extraction Facility of 320, a plate of 400
/// (160, 240, 320 and 400 frames, the command's own tick the first), and each can be placed from the next tick on.
/// Each cost is paid when the command applies.
#[test]
fn constructions_take_their_frames_at_full_power() {
    let mut t = ScenarioBuilder::new(&flat(40, 12))
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 1
        .spawn(DeploymentCenter, Some(P1), tile(6, 1)) // 2
        .spawn(DeploymentCenter, Some(P1), tile(11, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(16, 1)) // 4
        .start();
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[1], Construct(Generator)))
        .at(1, P1, construction(&[2], Construct(Barracks)))
        .at(1, P1, construction(&[3], Construct(ExtractionFacility)))
        .at(1, P1, construction(&[4], Construct(ExtractionPlate)))
        .run_until(1);
    assert_eq!(bank(&t, P1), 1000 - 100 - 200 - 200 - 75);
    assert_eq!(state(&t, 1).current(), Some(Generator));
    assert_eq!(
        state(&t, 1).frames_done(),
        Some(1),
        "the command's own tick is the first frame"
    );
    t.run_until(120);
    assert_eq!(state(&t, 1).frames_done(), Some(120));
    // The Generator is ready at the end of tick 240: a Place can't apply in tick 240, and can in tick 241.
    let place = PlayerCommand {
        player: P1,
        command: construction(&[1], place(1, 6)),
    };
    t.run_until(239);
    assert_eq!(t.sim().validate(&place), Err(Reject::NothingReady));
    t.run_until(240);
    assert_eq!(t.sim().validate(&place), Ok(()));
    t.run_until(500);
    assert_eq!(
        completions(&t),
        [
            (160, 2, Barracks),
            (240, 1, Generator),
            (320, 3, ExtractionFacility),
            (400, 4, ExtractionPlate)
        ]
    );
    for (n, kind) in [
        (1, Generator),
        (2, Barracks),
        (3, ExtractionFacility),
        (4, ExtractionPlate),
    ] {
        assert_eq!(state(&t, n), Constructor::Ready { kind }, "held until placed");
    }
    assert_eq!(bank(&t, P1), 425, "nothing more paid");
}

/// The plan's acceptance under the penalty and design change 0089's verification: at 30 generated and 51 consumed
/// (the all-in Barracks), the Extraction Facility builds a plate at 30/51, in exactly 680 ticks (400 × 51/30), while
/// the Deployment Center builds a Generator at full speed, in 240: ordered in tick 1, ready at the end of ticks 680
/// and 240.
#[test]
fn negative_power_slows_the_facility_and_not_the_center() {
    let mut t = ScenarioBuilder::new(&flat(16, 10))
        .spawn(CrystalPatch, None, tile(1, 7)) // 1
        .spawn(CrystalPatch, None, tile(3, 7)) // 2
        .spawn(DeploymentCenter, Some(P1), tile(1, 1)) // 3
        .spawn(ExtractionFacility, Some(P1), tile(6, 1)) // 4
        .spawn(ExtractionPlate, Some(P1), tile(1, 7)) // 5
        .spawn(ExtractionPlate, Some(P1), tile(3, 7)) // 6
        .spawn(Barracks, Some(P1), tile(10, 1)) // 7
        .start();
    let power = t.sim().power(P1);
    assert_eq!((power.generated, power.consumed), (30, 51));
    assert_eq!(power.speed(ExtractionFacility), 38_551, "≈ 59%");
    assert_eq!(power.speed(DeploymentCenter), RATIO_ONE);
    t.debug_at(1, fund(P1, 1000))
        .at(1, P1, construction(&[4], Construct(ExtractionPlate)))
        .at(1, P1, construction(&[3], Construct(Generator)))
        .run_until(1000);
    assert_eq!(completions(&t), [(240, 3, Generator), (680, 4, ExtractionPlate)]);
}
