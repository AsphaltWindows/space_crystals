//! The Altar's panel in the client (slice M5-N1b; `factions/cults/augmentation.md > Altar >
//! ObjectInterfaceState[Altar]`): units right-clicked into it, then "a three-stage selection: **unit type → current
//! AugmentationState → Augmentation to apply**", each stage with its counts, its orders queued one per press, the menu
//! staying open, and the InfoPanel showing the AugmentationQueue (`> Altar > AltarInstanceState`); a fully augmented
//! unit's Release, X's refund, A's Eject All and the rally point. The panel as plain data is the unit tests'
//! (`command_ui/interface/tests/altar.rs`), the augmenting itself the sim's (`crates/sim/tests/altar`).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    AugmentOrder, Augmentation, Command, DebugCommand, Faction, HoldCommand, ObjectId, ObjectKind, OrderKind, PlayerId,
    ProductionCommand, Rally, RecruitKind, SimEvent, SpawnAt, TilePos,
};
use space_crystals::command_ui::feedback::{NO_RECRUIT_TEXT, NOTHING_INSIDE_TEXT};
use space_crystals::command_ui::{ButtonLook, InterfaceState, PanelButton, PanelCommand};

use super::armory::{buttons, counts_drawn, feedback, holdings, inside, on_map, tick, until};
use super::production::activity;
use super::*;
use KeyCode::{KeyA, KeyE, KeyQ, KeyR, KeyW, KeyX, KeyZ};
use RecruitKind::{Both, Constitution, Metabolism, Plain};

const P1: PlayerId = PlayerId(1);
const ALTAR_KIND: ObjectKind = ObjectKind::Altar;
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const MILITANT: ObjectKind = ObjectKind::Militant;
const GUNNER: ObjectKind = ObjectKind::Gunner;
const DISCIPLE: ObjectKind = ObjectKind::Disciple;

/// The objects of [`altar_game`], by id, and where they are drawn (the map is flat). The Altar, unturned, has its
/// entrance, side B, to the east and its exit, side C, to the west.
const ALTAR: u32 = 1;
const ALTAR_AT: Vec2 = Vec2::new(1120.0, 832.0); // tiles (16, 12) to (18, 13): x 1024 to 1216
const MILITANTS: [u32; 3] = [2, 3, 4];
/// A Disciple with both Augmentations (a debug command stands for an earlier augmentation).
const FULLY_AUGMENTED: u32 = 5;
/// Bare ground west of the Altar, beyond its exit, in the world view.
const RALLY_AT: Vec2 = Vec2::new(820.0, 700.0);

/// A game of player 1, the Cults, on a flat 64 × 48 map, set up as [`game`] does, with 1000 Space Crystals: an Altar,
/// three Militants and a fully augmented Disciple, which a box selects and a right-click on the Altar sends in
/// (`> Altar > Entry`: "Any Cults infantry (Recruit, Militant, Gunner, Disciple) can enter the Altar"; the Enter
/// pointer over it); returned once they are all inside.
fn altar_game() -> App {
    let point = |x, y| SpawnAt::Point(Vec2::new(x, y));
    let builder = ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Cults)
        .spawn(ALTAR_KIND, Some(P1), SpawnAt::Tile(TilePos::new(16, 12)))
        .spawn(MILITANT, Some(P1), point(1300.0, 800.0))
        .spawn(MILITANT, Some(P1), point(1300.0, 832.0))
        .spawn(MILITANT, Some(P1), point(1300.0, 864.0))
        .spawn(DISCIPLE, Some(P1), point(1340.0, 832.0));
    let mut app = game_of(builder.build());
    set_crystals(&mut app, 1000);
    let augmentation = Augmentation::of(Both);
    let id = ObjectId(FULLY_AUGMENTED);
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetAugmentation { id, augmentation }),
    );
    app.update();
    let units = [MILITANTS.as_slice(), &[FULLY_AUGMENTED]].concat();
    select_units(&mut app, &units);
    assert_eq!(pointer_over(&mut app, ALTAR_AT), Some(PointerDisplay::Enter));
    right_click(&mut app, ALTAR_AT, &[]);
    assert_eq!(
        orders(&app).last(),
        Some(&given(&units, OrderKind::Enter(ObjectId(ALTAR)), false))
    );
    until(&mut app, 300, |app| inside(app, ALTAR).len() == units.len());
    app
}

/// Whether the Altar's rally point is [`RALLY_AT`], give or take the cursor's rounding.
fn rallied(app: &App) -> bool {
    let sim = app.world().resource::<SimRes>();
    let rally = sim.object(ObjectId(ALTAR)).and_then(|o| o.production.as_ref()?.rally);
    matches!(rally, Some(Rally::Point(at)) if near(at, RALLY_AT))
}

/// Unit `id` on the map: its kind, its Augmentations and its HP and MaxHP, those of its Augmentations.
fn out(app: &App, id: u32) -> (ObjectKind, Augmentation, Option<f32>, Option<f32>) {
    let sim = app.world().resource::<SimRes>();
    let o = sim.object(ObjectId(id)).expect("on the map");
    (o.kind, o.augmentation, o.hp, o.max_hp_with(sim.research()))
}

/// The acceptance's one augmentation (`> Altar > Augmentation Orders`: "Cost and time per order type (placeholder,
/// designer 2026-11-26): **Enhanced Constitution** and **Improved Metabolism**: 25 Space Crystals, 160 frames (10 s)
/// each"), by keys: the DefaultState shows each type with its count, the Recruit's and the Gunner's greyed, saying so,
/// and X greyed with nothing to cancel; W opens AugmentMenu\[Militant\], its Unaugmented tile 3 and the others 0; Q
/// opens AugmentMenu\[Militant, Unaugmented\]: Q Enhanced Constitution, W Improved Metabolism, E Both, A Release, Z
/// Back. Q queues one order for 25 Space Crystals, the menu staying open, and the InfoPanel shows it under way and the
/// queue; the Militant that went in first comes out of side C 160 frames later with Enhanced Constitution, MaxHP 60
/// (`factions/cults/augmentation.md > Constitution and Metabolism`), and walks to the rally point the right-click set.
/// Z goes back a stage at a time, the counts following.
#[test]
fn w_q_q_augments_a_militant_which_walks_to_the_rally_point() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{Augment, AugmentMenu, AugmentState, CancelProduction, EjectAll, ReleaseOf, SetRallyPoint};
    let mut app = altar_game();
    let first_in = inside(&app, ALTAR)[0];
    assert!(MILITANTS.contains(&first_in));
    click(&mut app, ALTAR_AT, &[]);
    assert_eq!(selected(&app), [ALTAR]);
    let c = PanelButton::Command;
    let (x, rally_slot) = (CommandSlot::CANCEL_PRODUCTION, CommandSlot::RALLY_POINT);
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(AugmentMenu(RECRUIT)), Disabled, Some(0)),
            (slot(0, 1), c(AugmentMenu(MILITANT)), Common, Some(3)),
            (slot(0, 2), c(AugmentMenu(GUNNER)), Disabled, Some(0)),
            (slot(0, 3), c(AugmentMenu(DISCIPLE)), Common, Some(1)),
            (slot(1, 0), c(EjectAll), Common, None),
            (x, c(CancelProduction), Disabled, None),
            (rally_slot, c(SetRallyPoint), Common, None),
        ]
    );
    let drawn = ["0", "3", "0", "1"].map(String::from);
    assert_eq!(
        counts_drawn(&mut app),
        [0, 1, 2, 3].map(|column| (slot(0, column), drawn[column as usize].clone()))
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(feedback(&app), Some(NO_RECRUIT_TEXT));
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyX);
    assert_eq!(feedback(&app), Some("Nothing to cancel"));
    right_click(&mut app, RALLY_AT, &[]);
    assert!(rallied(&app));

    tap(&mut app, &[], KeyW);
    assert_eq!(interface(&app), InterfaceState::AugmentMenu(MILITANT));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(0, 0), c(AugmentState(MILITANT, Plain)), Common, Some(3)),
            (slot(0, 1), c(AugmentState(MILITANT, Constitution)), Disabled, Some(0)),
            (slot(0, 2), c(AugmentState(MILITANT, Metabolism)), Disabled, Some(0)),
            (slot(0, 3), c(AugmentState(MILITANT, Both)), Disabled, Some(0)),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );
    tap(&mut app, &[], KeyQ);
    let menu = InterfaceState::AugmentStateMenu(MILITANT, Plain);
    assert_eq!(interface(&app), menu);
    assert_eq!(
        buttons(&mut app),
        [
            (
                slot(0, 0),
                c(Augment(MILITANT, Plain, AugmentOrder::EnhancedConstitution)),
                Common,
                None
            ),
            (
                slot(0, 1),
                c(Augment(MILITANT, Plain, AugmentOrder::ImprovedMetabolism)),
                Common,
                None
            ),
            (
                slot(0, 2),
                c(Augment(MILITANT, Plain, AugmentOrder::Both)),
                Common,
                None
            ),
            (slot(1, 0), c(ReleaseOf(MILITANT, Plain)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ]
    );

    // Q: one order, 25 Space Crystals; the frames from the press until the Militant is out.
    input(&mut app).tap_key(KeyQ);
    let mut events = Vec::new();
    let mut frames = 0;
    while !on_map(&app, first_in) {
        assert!(frames < 200, "out within 200 frames");
        app.update();
        frames += 1;
        let sim = app.world().resource::<SimRes>();
        events.extend(sim.events().iter().cloned().map(|e| (tick(&app), e)));
        if frames == 2 {
            assert_eq!(bank(&app), 1000 - 25, "paid");
            assert_eq!(interface(&app), menu, "the menu stays open");
            let lines = [
                "Inside: 4 / 10",
                "Free: Recruit 0, Militant 2, Gunner 0, Disciple 1",
                "Augmenting Militant (Enhanced Constitution): 0%",
                "Queue: Enhanced Constitution (1 / 10)",
                "Rally point: ground",
            ];
            assert_eq!(activity(&mut app), Some(lines.join("\n")));
        }
    }
    let order = ProductionCommand::Augment {
        kind: MILITANT,
        from: Plain,
        order: AugmentOrder::EnhancedConstitution,
    };
    assert_eq!(
        productions(&app)[1..],
        [(P1, vec![ALTAR], order)],
        "after the rally point's"
    );
    let queued = (events.iter()).find_map(|(at, e)| matches!(e, SimEvent::AugmentQueued { .. }).then_some(*at));
    let produced = SimEvent::UnitProduced {
        producer: ObjectId(ALTAR),
        unit: ObjectId(first_in),
    };
    let produced = (events.iter()).find_map(|(at, e)| (*e == produced).then_some(*at));
    assert_eq!(
        produced.zip(queued).map(|(out, queued)| out - queued),
        Some(159),
        "out in the 160th tick of its order: 10 s"
    );
    assert_eq!(
        frames,
        1 + 160,
        "the press's frame, which sends it, then the order's 160 frames"
    );
    let augmented = Augmentation::of(Constitution);
    assert_eq!(out(&app, first_in), (MILITANT, augmented, Some(60.0), Some(60.0)));
    assert!(
        pos(&app, first_in).x <= 1024.0,
        "out of side C: {}",
        pos(&app, first_in)
    );
    until(&mut app, 200, |app| pos(app, first_in).distance(RALLY_AT) < 40.0);

    assert_eq!(interface(&app), menu, "still open");
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::AugmentMenu(MILITANT));
    assert_eq!(buttons(&mut app)[0].3, Some(2), "two Unaugmented Militants left");
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(buttons(&mut app)[1].3, Some(2));
}

/// Design change 0337 (`> ObjectInterfaceState[Altar] > AugmentMenu[UnitType]`: "**R: Fully augmented** — enabled when
/// there are any (only Release is possible)"): R, R shows A Release alone, which brings the Disciple out unchanged,
/// free, in the fourth tick of its Release (4 frames). E and Q in AugmentMenu\[Militant, Unaugmented\] queue Both and
/// Enhanced Constitution, 75 Space Crystals, two Militants reserved; X, in DefaultState, takes back the last, refunding
/// its 25 (`> Augmentation Orders`: "Cancel Production (X) removes the last queued order and refunds it"), its Militant
/// free again; and A, Eject All, sends every free unit out, the one under Both staying inside, greyed once none is
/// left.
#[test]
fn r_releases_the_fully_augmented_x_refunds_and_a_ejects_the_free_units() {
    use ButtonLook::{Back, Common, Disabled};
    use PanelCommand::{AugmentState, ReleaseOf};
    let mut app = altar_game();
    click(&mut app, ALTAR_AT, &[]);
    tap(&mut app, &[], KeyR);
    assert_eq!(interface(&app), InterfaceState::AugmentMenu(DISCIPLE));
    let c = PanelButton::Command;
    assert_eq!(
        buttons(&mut app)[3],
        (slot(0, 3), c(AugmentState(DISCIPLE, Both)), Common, Some(1))
    );
    tap(&mut app, &[], KeyR);
    assert_eq!(interface(&app), InterfaceState::AugmentStateMenu(DISCIPLE, Both));
    assert_eq!(
        buttons(&mut app),
        [
            (slot(1, 0), c(ReleaseOf(DISCIPLE, Both)), Common, None),
            (CommandSlot::BACK, PanelButton::Back, Back, None),
        ],
        "only Release is possible"
    );
    input(&mut app).tap_key(KeyA);
    let mut frames = 0;
    while !on_map(&app, FULLY_AUGMENTED) {
        assert!(frames < 16, "out within a second");
        app.update();
        frames += 1;
    }
    assert_eq!(frames, 1 + 4, "the press's frame, then the Release's 4 frames: ¼ s");
    assert_eq!(bank(&app), 1000, "free");
    let (kind, augmentation, ..) = out(&app, FULLY_AUGMENTED);
    assert_eq!((kind, augmentation), (DISCIPLE, Augmentation::of(Both)), "unchanged");
    let release = ProductionCommand::ReleaseOf {
        kind: DISCIPLE,
        from: Both,
    };
    assert_eq!(productions(&app), [(P1, vec![ALTAR], release)]);
    assert_eq!(buttons(&mut app)[0].2, Disabled, "none left to release");

    // Back to the Militants: E and Q queue Both and Enhanced Constitution.
    tap(&mut app, &[], KeyZ);
    tap(&mut app, &[], KeyZ);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    tap(&mut app, &[], KeyW);
    tap(&mut app, &[], KeyQ);
    tap(&mut app, &[], KeyE);
    tap(&mut app, &[], KeyQ);
    assert_eq!(bank(&app), 1000 - 50 - 25);
    let lines = activity(&mut app).expect("the activity area shows");
    assert!(
        lines.ends_with("\nQueue: Both, Enhanced Constitution (2 / 10)"),
        "{lines}"
    );
    tap(&mut app, &[], KeyZ);
    tap(&mut app, &[], KeyZ);
    assert_eq!(buttons(&mut app)[1].3, Some(1), "two Militants reserved");
    tap(&mut app, &[], KeyX);
    assert_eq!(bank(&app), 1000 - 50, "the last order refunded");
    assert_eq!(buttons(&mut app)[1].3, Some(2), "its Militant free again");

    // A: the two free Militants come out; the one under Both stays.
    tap(&mut app, &[], KeyA);
    assert_eq!(holdings(&app), [(P1, vec![ALTAR], HoldCommand::EjectAll)]);
    until(&mut app, 100, |app| inside(app, ALTAR).len() == 1);
    let reserved = inside(&app, ALTAR)[0];
    assert!(MILITANTS.contains(&reserved));
    assert_eq!(buttons(&mut app)[4].2, Disabled, "nothing free to eject");
    tap(&mut app, &[], KeyA);
    assert_eq!(feedback(&app), Some(NOTHING_INSIDE_TEXT));
    assert_eq!(holdings(&app).len(), 1);
}
